use anyhow;
use gpui::Pixels;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::Settings;

pub trait AiModelTrait {
    fn full_name(&self) -> &'static str;
    fn short_name(&self) -> &'static str;
    fn cycle(&self) -> Self;
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum AiProvider {
    OpenAI,
    Ollama,
}

impl AiProvider {
    pub fn cycle(&self) -> Self {
        match self {
            AiProvider::OpenAI => AiProvider::Ollama,
            AiProvider::Ollama => AiProvider::OpenAI,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => "Open AI",
            AiProvider::Ollama => "Ollama",
        }
    }

    pub fn default_model(&self) -> AiModel {
        match self {
            AiProvider::OpenAI => AiModel::OpenAI(OpenAiModel::ThreePointFiveTurbo),
            AiProvider::Ollama => AiModel::Ollama(OllamaModel::CodeLlamaSevenBillion),
        }
    }

    pub fn api_url(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => "https://api.openai.com/v1",
            AiProvider::Ollama => "http://localhost:11434/v1",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum AiModel {
    OpenAI(OpenAiModel),
    Ollama(OllamaModel),
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum OpenAiModel {
    #[serde(rename = "gpt-3.5-turbo-0613")]
    ThreePointFiveTurbo,
    #[serde(rename = "gpt-4-0613")]
    Four,
    #[serde(rename = "gpt-4-1106-preview")]
    FourTurbo,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum OllamaModel {
    #[serde(rename = "codellama:7b")]
    CodeLlamaSevenBillion,
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

impl AiModelTrait for OllamaModel {
    fn full_name(&self) -> &'static str {
        match self {
            OllamaModel::CodeLlamaSevenBillion => "codellama:7b",
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            OllamaModel::CodeLlamaSevenBillion => "codellama",
        }
    }

    fn cycle(&self) -> Self {
        match self {
            OllamaModel::CodeLlamaSevenBillion => OllamaModel::CodeLlamaSevenBillion,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssistantDockPosition {
    Left,
    Right,
    Bottom,
}

#[derive(Deserialize, Debug)]
pub struct AssistantSettings {
    pub button: bool,
    pub dock: AssistantDockPosition,
    pub default_width: Pixels,
    pub default_height: Pixels,
    pub default_ai_model: AiModel,
    pub open_ai_api_url: String,
    pub ollama_api_url: String,
    pub default_provider: AiProvider,
}

/// Assistant panel settings
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, Debug)]
pub struct AssistantSettingsContent {
    /// Whether to show the assistant panel button in the status bar.
    ///
    /// Default: true
    pub button: Option<bool>,
    /// Where to dock the assistant.
    ///
    /// Default: right
    pub dock: Option<AssistantDockPosition>,
    /// Default width in pixels when the assistant is docked to the left or right.
    ///
    /// Default: 640
    pub default_width: Option<f32>,
    /// Default height in pixels when the assistant is docked to the bottom.
    ///
    /// Default: 320
    pub default_height: Option<f32>,
    /// The default AI model to use when starting new conversations.
    ///
    /// Default: gpt-4-1106-preview
    pub default_ai_model: Option<AiModel>,
    /// OpenAi API base URL to use when starting new conversations.
    ///
    /// Default: http://localhost:11434/v1
    pub open_ai_api_url: Option<String>,
    /// Ollama API base URL to use when starting new conversations.
    ///
    /// Default: http://localhost:11434/v1
    pub ollama_api_url: Option<String>,
    /// The default AI provider to use when starting new conversations.
    /// This setting determines which AI model and API URL to use by default.
    /// It can be switched dynamically in the application to alternate between using OpenAI and Ollama models and endpoints.
    ///
    /// Default: OpenAI
    pub default_provider: Option<AiProvider>,
}

impl Settings for AssistantSettings {
    const KEY: Option<&'static str> = Some("assistant");

    type FileContent = AssistantSettingsContent;

    fn load(
        default_value: &Self::FileContent,
        user_values: &[&Self::FileContent],
        _: &mut gpui::AppContext,
    ) -> anyhow::Result<Self> {
        Self::load_via_json_merge(default_value, user_values)
    }
}
