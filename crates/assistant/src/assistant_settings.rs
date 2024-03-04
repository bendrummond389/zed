use ai::{
    models::AiModelVariant,
    providers::{
        ollama::{model::OllamaModel, OLLAMA_API_URL},
        open_ai::{model::OpenAiModel, OPEN_AI_API_URL},
    },
};
use anyhow;
use gpui::Pixels;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::Settings;

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

    pub fn default_model(&self) -> AiModelVariant {
        match self {
            AiProvider::OpenAI => AiModelVariant::OpenAI(OpenAiModel::ThreePointFiveTurbo),
            AiProvider::Ollama => AiModelVariant::Ollama(OllamaModel::CodeLlamaSevenBillion),
        }
    }

    pub fn api_url(&self) -> &'static str {
        match self {
            AiProvider::OpenAI => OPEN_AI_API_URL,
            AiProvider::Ollama => OLLAMA_API_URL,
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
    pub default_ai_model: AiModelVariant,
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
    pub default_ai_model: Option<AiModelVariant>,
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
