use crate::providers::ollama::model::OllamaModel;
use crate::providers::open_ai::model::OpenAiModel;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub enum TruncationDirection {
    Start,
    End,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum AiModel {
    OpenAI(OpenAiModel),
    Ollama(OllamaModel),
}

pub trait AiModelTrait {
    fn full_name(&self) -> &'static str;
    fn short_name(&self) -> &'static str;
    fn cycle(&self) -> Self;
}

impl AiModelTrait for AiModel {
    fn full_name(&self) -> &'static str {
        match self {
            AiModel::OpenAI(model) => model.full_name(),
            AiModel::Ollama(model) => model.full_name(),
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            AiModel::OpenAI(model) => model.short_name(),
            AiModel::Ollama(model) => model.short_name(),
        }
    }

    fn cycle(&self) -> Self {
        match self {
            AiModel::OpenAI(model) => AiModel::OpenAI(model.cycle()),
            AiModel::Ollama(model) => AiModel::Ollama(model.cycle()),
        }
    }
}

pub trait LanguageModel {
    fn name(&self) -> String;
    fn count_tokens(&self, content: &str) -> anyhow::Result<usize>;
    fn truncate(
        &self,
        content: &str,
        length: usize,
        direction: TruncationDirection,
    ) -> anyhow::Result<String>;
    fn capacity(&self) -> anyhow::Result<usize>;
}
