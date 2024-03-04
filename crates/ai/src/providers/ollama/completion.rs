use crate::{
    auth::{CredentialProvider, ProviderCredential},
    completion::{CompletionProvider, CompletionRequest},
    models::LanguageModel,
};
use anyhow::{anyhow, Result};
use futures::{
    future::BoxFuture, io::BufReader, stream::BoxStream, AsyncBufReadExt, AsyncReadExt, FutureExt,
    Stream, StreamExt,
};
use gpui::{AppContext, BackgroundExecutor};
use isahc::{http::StatusCode, Request, RequestExt};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    io,
};

use crate::providers::ollama::OllamaLanguageModel;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Role {
    pub fn cycle(&mut self) {
        *self = match self {
            Role::User => Role::Assistant,
            Role::Assistant => Role::System,
            Role::System => Role::User,
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Assistant => write!(f, "Assistant"),
            Role::System => write!(f, "System"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct RequestMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Default, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub messages: Vec<RequestMessage>,
    pub stream: bool,
    pub stop: Vec<String>,
    pub temperature: f32,
}

impl CompletionRequest for OllamaRequest {
    fn data(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ResponseMessage {
    pub role: Option<Role>,
    pub content: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug)]
pub struct ChatChoiceDelta {
    pub index: u32,
    pub delta: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaResponseStreamEvent {
    pub id: Option<String>,
    pub object: String,
    pub created: u32,
    pub model: String,
    pub choices: Vec<ChatChoiceDelta>,
    pub usage: Option<OllamaUsage>,
}

pub async fn stream_completion(
    api_url: String,
    executor: BackgroundExecutor,
    request: Box<dyn CompletionRequest>,
) -> Result<impl Stream<Item = Result<OllamaResponseStreamEvent>>> {
    let (tx, rx) = futures::channel::mpsc::unbounded::<Result<OllamaResponseStreamEvent>>();

    let json_data = request.data()?;
    let mut response = Request::post(format!("{api_url}/chat/completions"))
        .header("Content-Type", "application/json")
        .body(json_data)?
        .send_async()
        .await?;

    let status = response.status();

    if status == StatusCode::OK {
        executor
            .spawn(async move {
                let mut lines = BufReader::new(response.body_mut()).lines();

                fn parse_line(
                    line: Result<String, io::Error>,
                ) -> Result<Option<OllamaResponseStreamEvent>> {
                    if let Some(data) = line?.strip_prefix("data: ") {
                        let event = serde_json::from_str(data)?;
                        Ok(Some(event))
                    } else {
                        Ok(None)
                    }
                }

                while let Some(line) = lines.next().await {
                    if let Some(event) = parse_line(line).transpose() {
                        let done = event.as_ref().map_or(false, |event| {
                            event
                                .choices
                                .last()
                                .map_or(false, |choice| choice.finish_reason.is_some())
                        });
                        if tx.unbounded_send(event).is_err() {
                            break;
                        }

                        if done {
                            break;
                        }
                    }
                }

                anyhow::Ok(())
            })
            .detach();

        Ok(rx)
    } else {
        let mut body = String::new();
        response.body_mut().read_to_string(&mut body).await?;

        #[derive(Deserialize)]
        struct OllamaResponse {
            error: OllamaError,
        }

        #[derive(Deserialize)]
        struct OllamaError {
            message: String,
        }

        match serde_json::from_str::<OllamaResponse>(&body) {
            Ok(response) if !response.error.message.is_empty() => Err(anyhow!(
                "Failed to connect to OpenAI API: {}",
                response.error.message,
            )),

            _ => Err(anyhow!(
                "Failed to connect to OpenAI API: {} {}",
                response.status(),
                body,
            )),
        }
    }
}

#[derive(Clone)]
pub struct OllamaCompletionProvider {
    api_url: String,
    model: OllamaLanguageModel,
    executor: BackgroundExecutor,
}

impl OllamaCompletionProvider {
    pub async fn new(api_url: String, model_name: String, executor: BackgroundExecutor) -> Self {
        let model = executor
            .spawn(async move { OllamaLanguageModel::load(&model_name) })
            .await;
        Self {
            api_url,
            model,
            executor,
        }
    }
}

impl CredentialProvider for OllamaCompletionProvider {
    fn has_credentials(&self) -> bool {
        true
    }

    fn retrieve_credentials(&self, _cx: &mut AppContext) -> BoxFuture<ProviderCredential> {
        async { ProviderCredential::NotNeeded }.boxed()
    }

    fn save_credentials(
        &self,
        _cx: &mut AppContext,
        _credential: ProviderCredential,
    ) -> BoxFuture<()> {
        async {}.boxed()
    }

    fn delete_credentials(&self, _cx: &mut AppContext) -> BoxFuture<()> {
        async {}.boxed()
    }
}

impl CompletionProvider for OllamaCompletionProvider {
    fn base_model(&self) -> Box<dyn LanguageModel> {
        let model: Box<dyn LanguageModel> = Box::new(self.model.clone());
        model
    }

    fn complete(
        &self,
        prompt: Box<dyn CompletionRequest>,
    ) -> BoxFuture<'static, Result<BoxStream<'static, Result<String>>>> {
        let api_url = self.api_url.clone();
        let executor = self.executor.clone();

        async move {
            let response = stream_completion(api_url, executor, prompt).await?;
            let stream = response
                .filter_map(|response| async move {
                    match response {
                        Ok(mut response) => Some(Ok(response
                            .choices
                            .pop()?
                            .delta
                            .content
                            .unwrap_or_default())),
                        Err(error) => Some(Err(error)),
                    }
                })
                .boxed();
            Ok(stream)
        }
        .boxed()
    }

    fn box_clone(&self) -> Box<dyn CompletionProvider> {
        Box::new((*self).clone())
    }
}
