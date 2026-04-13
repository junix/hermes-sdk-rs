use serde::{Deserialize, Serialize};

/// A response from the Hermes Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Unique response ID (e.g. "resp_abc123").
    pub id: String,

    /// Object type, always "response".
    #[serde(rename = "object")]
    pub object_type: String,

    /// Status: "completed", "in_progress", etc.
    pub status: String,

    /// Unix timestamp of creation.
    pub created_at: u64,

    /// Model used.
    pub model: String,

    /// Ordered list of output items (tool calls, results, final message).
    pub output: Vec<OutputItem>,

    /// Token usage statistics.
    pub usage: Usage,
}

/// A single output item in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputItem {
    /// A tool call made by the agent.
    #[serde(rename = "function_call")]
    FunctionCall {
        name: String,
        arguments: String,
        call_id: String,
    },

    /// The result of a tool call.
    #[serde(rename = "function_call_output")]
    FunctionCallOutput { call_id: String, output: String },

    /// The final assistant message.
    #[serde(rename = "message")]
    Message {
        role: String,
        content: Vec<ContentPart>,
    },
}

impl OutputItem {
    /// Extract the assistant text if this is a message output.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            OutputItem::Message { content, .. } => content.iter().find_map(|p| match p {
                ContentPart::OutputText { text } => Some(text.as_str()),
            }),
            _ => None,
        }
    }

    /// Get the function name if this is a function_call.
    pub fn as_function_call(&self) -> Option<(&str, &str)> {
        match self {
            OutputItem::FunctionCall {
                name, arguments, ..
            } => Some((name, arguments)),
            _ => None,
        }
    }
}

/// A content part within a message output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "output_text")]
    OutputText { text: String },
}

/// Token usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// Response from `DELETE /v1/responses/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub id: String,

    #[serde(rename = "object")]
    pub object_type: String,

    pub deleted: bool,
}

impl Response {
    /// Convenience: get the assistant text from the first message output.
    pub fn text(&self) -> Option<&str> {
        self.output.iter().find_map(|item| item.as_text())
    }
}
