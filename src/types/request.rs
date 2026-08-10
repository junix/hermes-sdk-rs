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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Input: untagged wire shape (spec: bare string | JSON array) ─────────

    #[test]
    fn input_text_serializes_as_bare_json_string() {
        let input = Input::Text("hello".to_string());
        let json = serde_json::to_value(&input).expect("serialize");
        assert_eq!(json, serde_json::json!("hello"));
    }

    #[test]
    fn input_messages_serializes_as_json_array() {
        let input = Input::Messages(vec![
            InputMessage::user("hi"),
            InputMessage::assistant("hey"),
        ]);
        let json = serde_json::to_value(&input).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!([
                {"role":"user","content":"hi"},
                {"role":"assistant","content":"hey"}
            ])
        );
    }

    #[test]
    fn input_from_str_and_string_yield_text_variant() {
        let from_str: Input = "hi".into();
        let from_string: Input = "hi".to_string().into();
        assert!(matches!(from_str, Input::Text(ref s) if s == "hi"));
        assert!(matches!(from_string, Input::Text(ref s) if s == "hi"));
    }

    #[test]
    fn input_from_vec_messages_yields_messages_variant() {
        let v: Input = vec![InputMessage::assistant("x")].into();
        match v {
            Input::Messages(m) => {
                assert_eq!(m.len(), 1);
                assert_eq!(m[0].role, "assistant");
                assert_eq!(m[0].content, "x");
            }
            _ => panic!("expected Messages variant"),
        }
    }

    #[test]
    fn input_text_deserializes_from_bare_json_string() {
        let input: Input = serde_json::from_str("\"raw\"").expect("parse bare string");
        assert!(matches!(input, Input::Text(ref s) if s == "raw"));
    }

    #[test]
    fn input_messages_deserializes_from_json_array() {
        let input: Input =
            serde_json::from_str(r#"[{"role":"system","content":"s"}]"#).expect("parse array");
        match input {
            Input::Messages(m) => {
                assert_eq!(m.len(), 1);
                assert_eq!(m[0].role, "system");
            }
            _ => panic!("expected Messages variant"),
        }
    }

    // ── InputMessage factory roles ──────────────────────────────────────────

    #[test]
    fn input_message_factories_set_correct_roles_and_content() {
        let u = InputMessage::user("u-payload");
        let a = InputMessage::assistant("a-payload");
        let s = InputMessage::system("s-payload");
        assert_eq!(u.role, "user");
        assert_eq!(a.role, "assistant");
        assert_eq!(s.role, "system");
        assert_eq!(u.content, "u-payload");
        assert_eq!(a.content, "a-payload");
        assert_eq!(s.content, "s-payload");
    }

    // ── CreateResponseRequest: skip_serializing_if + field names ────────────

    #[test]
    fn request_omits_all_none_optional_fields() {
        let req = CreateResponseRequest {
            input: Input::Text("p".into()),
            instructions: None,
            previous_response_id: None,
            conversation: None,
            store: None,
            model: None,
            conversation_history: None,
        };
        let map = serde_json::to_value(&req)
            .expect("serialize")
            .as_object()
            .expect("object")
            .clone();
        assert_eq!(
            map.len(),
            1,
            "only `input` should serialize when all optionals are None, got: {map:?}"
        );
        assert!(map.contains_key("input"));
    }

    #[test]
    fn request_serializes_all_set_fields_with_documented_names() {
        let req = CreateResponseRequest {
            input: Input::Text("p".into()),
            instructions: Some("be brief".into()),
            previous_response_id: Some("resp_1".into()),
            conversation: Some("conv".into()),
            store: Some(false),
            model: Some("hermes-x".into()),
            conversation_history: Some(vec![InputMessage::system("sys")]),
        };
        let json = serde_json::to_value(&req).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "input": "p",
                "instructions": "be brief",
                "previous_response_id": "resp_1",
                "conversation": "conv",
                "store": false,
                "model": "hermes-x",
                "conversation_history": [{"role": "system", "content": "sys"}]
            })
        );
    }

    #[test]
    fn request_round_trips_through_json() {
        let req = CreateResponseRequest {
            input: Input::Messages(vec![InputMessage::user("hi")]),
            instructions: Some("instr".into()),
            previous_response_id: None,
            conversation: None,
            store: Some(true),
            model: Some("m".into()),
            conversation_history: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: CreateResponseRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            back.input,
            Input::Messages(ref m) if m.len() == 1 && m[0].role == "user" && m[0].content == "hi"
        ));
        assert_eq!(back.instructions.as_deref(), Some("instr"));
        assert_eq!(back.store, Some(true));
        assert_eq!(back.model.as_deref(), Some("m"));
        // None fields stay None across the round trip.
        assert!(back.previous_response_id.is_none());
        assert!(back.conversation.is_none());
        assert!(back.conversation_history.is_none());
    }
}
