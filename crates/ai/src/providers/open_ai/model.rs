use crate::models::AiModelTrait;
use anyhow::anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tiktoken_rs::CoreBPE;

use crate::models::{LanguageModel, TruncationDirection};

use super::OPEN_AI_BPE_TOKENIZER;

#[derive(Clone)]
pub struct OpenAiLanguageModel {
    name: String,
    bpe: Option<CoreBPE>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum OpenAiModel {
    #[serde(rename = "gpt-3.5-turbo-0613")]
    ThreePointFiveTurbo,
    #[serde(rename = "gpt-4-0613")]
    Four,
    #[serde(rename = "gpt-4-1106-preview")]
    FourTurbo,
}

impl OpenAiLanguageModel {
    pub fn load(model_name: &str) -> Self {
        let bpe =
            tiktoken_rs::get_bpe_from_model(model_name).unwrap_or(OPEN_AI_BPE_TOKENIZER.to_owned());
        OpenAiLanguageModel {
            name: model_name.to_string(),
            bpe: Some(bpe),
        }
    }
}

impl AiModelTrait for OpenAiModel {
    fn full_name(&self) -> &'static str {
        match self {
            OpenAiModel::ThreePointFiveTurbo => "gpt-3.5-turbo-0613",
            OpenAiModel::Four => "gpt-4-0613",
            OpenAiModel::FourTurbo => "gpt-4-1106-preview",
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            OpenAiModel::ThreePointFiveTurbo => "gpt-3.5-turbo",
            OpenAiModel::Four => "gpt-4",
            OpenAiModel::FourTurbo => "gpt-4-turbo",
        }
    }

    fn cycle(&self) -> Self {
        match self {
            OpenAiModel::ThreePointFiveTurbo => OpenAiModel::Four,
            OpenAiModel::Four => OpenAiModel::FourTurbo,
            OpenAiModel::FourTurbo => OpenAiModel::ThreePointFiveTurbo,
        }
    }
}

impl LanguageModel for OpenAiLanguageModel {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn count_tokens(&self, content: &str) -> anyhow::Result<usize> {
        if let Some(bpe) = &self.bpe {
            anyhow::Ok(bpe.encode_with_special_tokens(content).len())
        } else {
            Err(anyhow!("bpe for open ai model was not retrieved"))
        }
    }
    fn truncate(
        &self,
        content: &str,
        length: usize,
        direction: TruncationDirection,
    ) -> anyhow::Result<String> {
        if let Some(bpe) = &self.bpe {
            let tokens = bpe.encode_with_special_tokens(content);
            if tokens.len() > length {
                match direction {
                    TruncationDirection::End => bpe.decode(tokens[..length].to_vec()),
                    TruncationDirection::Start => bpe.decode(tokens[length..].to_vec()),
                }
            } else {
                bpe.decode(tokens)
            }
        } else {
            Err(anyhow!("bpe for open ai model was not retrieved"))
        }
    }
    fn capacity(&self) -> anyhow::Result<usize> {
        anyhow::Ok(tiktoken_rs::model::get_context_size(&self.name))
    }
}
