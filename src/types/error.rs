use serde::Deserialize;

/// Error detail returned by the Hermes API (OpenAI-compatible format).
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(default)]
    pub param: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

/// All errors that can occur when using the Hermes SDK.
#[derive(Debug, thiserror::Error)]
pub enum HermesError {
    /// The API returned a non-2xx response.
    #[error("API error (status {status}): {message}")]
    Api {
        status: u16,
        message: String,
        error_type: Option<String>,
        code: Option<String>,
    },

    /// A network or connection error occurred.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// A configuration or validation error.
    #[error("config error: {0}")]
    Config(String),
}

impl HermesError {
    pub(crate) fn from_response(status: u16, body: &str) -> Self {
        if let Ok(detail) = serde_json::from_str::<ApiErrorResponse>(body) {
            Self::Api {
                status,
                message: detail.error.message,
                error_type: Some(detail.error.error_type),
                code: detail.error.code,
            }
        } else {
            Self::Api {
                status,
                message: body.to_string(),
                error_type: None,
                code: None,
            }
        }
    }
}

#[derive(Deserialize)]
struct ApiErrorResponse {
    error: ApiErrorDetail,
}
