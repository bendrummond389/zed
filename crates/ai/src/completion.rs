use anyhow::Result;
use futures::{future::BoxFuture, stream::BoxStream};

use crate::{auth::CredentialProvider, models::LanguageModel};

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

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

pub trait CompletionRequest: Send + Sync {
    fn data(&self) -> serde_json::Result<String>;
}

pub trait CompletionProvider: CredentialProvider {
    fn base_model(&self) -> Box<dyn LanguageModel>;
    fn complete(
        &self,
        prompt: Box<dyn CompletionRequest>,
    ) -> BoxFuture<'static, Result<BoxStream<'static, Result<String>>>>;
    fn box_clone(&self) -> Box<dyn CompletionProvider>;
}

impl Clone for Box<dyn CompletionProvider> {
    fn clone(&self) -> Box<dyn CompletionProvider> {
        self.box_clone()
    }
}
