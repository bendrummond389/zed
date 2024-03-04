use crate::providers::ollama::model::OllamaModel;
use crate::providers::open_ai::model::OpenAiModel;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub enum TruncationDirection {
    Start,
    End,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum AiModelVariant {
    OpenAI(OpenAiModel),
    Ollama(OllamaModel),
}

pub trait AiModel {
    fn full_name(&self) -> &'static str;
    fn short_name(&self) -> &'static str;
    fn cycle(&self) -> Self;
}

impl AiModel for AiModelVariant {
    fn full_name(&self) -> &'static str {
        match self {
            AiModelVariant::OpenAI(model) => model.full_name(),
            AiModelVariant::Ollama(model) => model.full_name(),
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            AiModelVariant::OpenAI(model) => model.short_name(),
            AiModelVariant::Ollama(model) => model.short_name(),
        }
    }

    fn cycle(&self) -> Self {
        match self {
            AiModelVariant::OpenAI(model) => AiModelVariant::OpenAI(model.cycle()),
            AiModelVariant::Ollama(model) => AiModelVariant::Ollama(model.cycle()),
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
