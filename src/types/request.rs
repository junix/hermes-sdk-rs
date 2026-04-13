use serde::{Deserialize, Serialize};

/// Input for a response creation request.
///
/// Can be a simple string or a structured list of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    /// A single text prompt.
    Text(String),
    /// A list of messages or content parts.
    Messages(Vec<InputMessage>),
}

impl From<&str> for Input {
    fn from(s: &str) -> Self {
        Input::Text(s.to_string())
    }
}

impl From<String> for Input {
    fn from(s: String) -> Self {
        Input::Text(s)
    }
}

impl From<Vec<InputMessage>> for Input {
    fn from(msgs: Vec<InputMessage>) -> Self {
        Input::Messages(msgs)
    }
}

/// A single message within structured input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub role: String,
    pub content: String,
}

impl InputMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// Request body for `POST /v1/responses`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    /// The user input — a string or structured messages.
    pub input: Input,

    /// System-level instructions for this turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Chain to a previous response by ID.
    /// Mutually exclusive with `conversation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,

    /// Named conversation to continue.
    /// Mutually exclusive with `previous_response_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,

    /// Whether to store the response for later retrieval (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// Model name (default: "hermes-agent").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Explicit conversation history (takes precedence over `previous_response_id`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_history: Option<Vec<InputMessage>>,
}

impl CreateResponseRequest {
    /// Create a builder for this request.
    pub fn builder() -> super::super::builder::CreateResponseRequestBuilder {
        super::super::builder::CreateResponseRequestBuilder::default()
    }
}
