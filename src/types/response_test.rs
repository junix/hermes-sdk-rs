use super::*;

fn text_part(text: &str) -> ContentPart {
    ContentPart::OutputText {
        text: text.to_string(),
    }
}

#[test]
fn as_text_returns_first_text_from_message() {
    let item = OutputItem::Message {
        role: "assistant".into(),
        content: vec![text_part("first"), text_part("second")],
    };
    assert_eq!(item.as_text(), Some("first"));
}

#[test]
fn as_text_returns_none_for_message_with_empty_content() {
    let item = OutputItem::Message {
        role: "assistant".into(),
        content: vec![],
    };
    assert_eq!(item.as_text(), None);
}

#[test]
fn as_text_returns_none_for_non_message_variants() {
    let fc = OutputItem::FunctionCall {
        name: "n".into(),
        arguments: "{}".into(),
        call_id: "c".into(),
    };
    let fco = OutputItem::FunctionCallOutput {
        call_id: "c".into(),
        output: "o".into(),
    };
    assert_eq!(fc.as_text(), None);
    assert_eq!(fco.as_text(), None);
}

#[test]
fn as_function_call_returns_name_and_args() {
    let item = OutputItem::FunctionCall {
        name: "run".into(),
        arguments: "{\"x\":1}".into(),
        call_id: "call_1".into(),
    };
    let (name, args) = item.as_function_call().expect("function call extracts");
    assert_eq!(name, "run");
    assert_eq!(args, "{\"x\":1}");
}

#[test]
fn as_function_call_returns_none_for_non_function_variants() {
    let msg = OutputItem::Message {
        role: "assistant".into(),
        content: vec![text_part("hi")],
    };
    let fco = OutputItem::FunctionCallOutput {
        call_id: "c".into(),
        output: "o".into(),
    };
    assert_eq!(msg.as_function_call(), None);
    assert_eq!(fco.as_function_call(), None);
}

#[test]
fn response_text_aggregates_first_message_text() {
    let resp = Response {
        id: "resp_1".into(),
        object_type: "response".into(),
        status: "completed".into(),
        created_at: 0,
        model: "m".into(),
        output: vec![
            OutputItem::FunctionCall {
                name: "n".into(),
                arguments: "{}".into(),
                call_id: "c".into(),
            },
            OutputItem::Message {
                role: "assistant".into(),
                content: vec![text_part("final answer")],
            },
        ],
        usage: Usage {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        },
    };
    assert_eq!(resp.text(), Some("final answer"));
}

// ── text() edge cases: find_map semantics over the output list ───────────

fn sample_response(output: Vec<OutputItem>) -> Response {
    Response {
        id: "resp_x".into(),
        object_type: "response".into(),
        status: "completed".into(),
        created_at: 0,
        model: "m".into(),
        output,
        usage: Usage {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
        },
    }
}

#[test]
fn response_text_is_none_for_empty_output() {
    let resp = sample_response(vec![]);
    assert_eq!(resp.text(), None);
}

#[test]
fn response_text_is_none_when_no_output_item_is_a_message() {
    let resp = sample_response(vec![
        OutputItem::FunctionCall {
            name: "n".into(),
            arguments: "{}".into(),
            call_id: "c".into(),
        },
        OutputItem::FunctionCallOutput {
            call_id: "c".into(),
            output: "o".into(),
        },
    ]);
    assert_eq!(resp.text(), None);
}

#[test]
fn response_text_skips_message_with_no_text_part_and_takes_next_message() {
    // A Message whose content has no OutputText yields None from as_text();
    // text() must continue to the next output item instead of giving up.
    // (ContentPart currently has only the OutputText variant, so an empty
    // content vec is the way to make a Message yield None.)
    let resp = sample_response(vec![
        OutputItem::Message {
            role: "assistant".into(),
            content: vec![],
        },
        OutputItem::Message {
            role: "assistant".into(),
            content: vec![text_part("real answer")],
        },
    ]);
    assert_eq!(resp.text(), Some("real answer"));
}

// ── Wire-format deserialization (the SDK's reason for existing) ──────────
// These pin the #[serde(tag = "type")] / #[serde(rename = ...)] / field-name
// attributes against the gateway's actual JSON shape. Without them, a renamed
// tag or field would compile green while failing on real responses.

#[test]
fn deserializes_full_response_with_all_output_variants_from_wire_json() {
    let json = r#"{
        "id":"resp_abc",
        "object":"response",
        "status":"completed",
        "created_at":1700000000,
        "model":"hermes-agent",
        "output":[
            {"type":"function_call","name":"echo","arguments":"{\"k\":\"v\"}","call_id":"call_7"},
            {"type":"function_call_output","call_id":"call_7","output":"done"},
            {"type":"message","role":"assistant","content":[{"type":"output_text","text":"hi there"}]}
        ],
        "usage":{"input_tokens":3,"output_tokens":5,"total_tokens":8}
    }"#;
    let resp: Response = serde_json::from_str(json).expect("parse gateway response");

    assert_eq!(resp.id, "resp_abc");
    assert_eq!(resp.object_type, "response");
    assert_eq!(resp.status, "completed");
    assert_eq!(resp.created_at, 1700000000);
    assert_eq!(resp.model, "hermes-agent");
    assert_eq!(resp.usage.input_tokens, 3);
    assert_eq!(resp.usage.output_tokens, 5);
    assert_eq!(resp.usage.total_tokens, 8);

    // Ordering is significant (spec: function calls + outputs precede message).
    assert_eq!(resp.output.len(), 3);
    assert!(matches!(resp.output[0], OutputItem::FunctionCall { .. }));
    assert!(matches!(resp.output[1], OutputItem::FunctionCallOutput { .. }));
    assert!(matches!(resp.output[2], OutputItem::Message { .. }));

    match &resp.output[0] {
        OutputItem::FunctionCall {
            name,
            arguments,
            call_id,
        } => {
            assert_eq!(name, "echo");
            assert_eq!(arguments, "{\"k\":\"v\"}");
            assert_eq!(call_id, "call_7");
        }
        _ => unreachable!("output[0] must be FunctionCall"),
    }
    match &resp.output[1] {
        OutputItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "call_7");
            assert_eq!(output, "done");
        }
        _ => unreachable!("output[1] must be FunctionCallOutput"),
    }
    // text() walks past the two non-message items to the message's text.
    assert_eq!(resp.text(), Some("hi there"));
}

#[test]
fn deserializes_message_with_multiple_output_text_parts_from_wire_json() {
    let json = r#"{"type":"message","role":"assistant","content":[
        {"type":"output_text","text":"alpha"},
        {"type":"output_text","text":"beta"}
    ]}"#;
    let item: OutputItem = serde_json::from_str(json).expect("parse message");
    // as_text returns the FIRST OutputText part (find_map semantics).
    assert_eq!(item.as_text(), Some("alpha"));
}

#[test]
fn deserializes_delete_response_with_object_rename_from_wire_json() {
    let json = r#"{"id":"resp_9","object":"response","deleted":true}"#;
    let d: DeleteResponse = serde_json::from_str(json).expect("parse delete response");
    assert_eq!(d.id, "resp_9");
    assert_eq!(d.object_type, "response");
    assert!(d.deleted);
}

#[test]
fn output_item_with_unknown_type_tag_fails_to_deserialize() {
    // The SDK currently rejects unknown output-item types rather than silently
    // dropping them. Pin this so a future move to lenient parsing is intentional.
    let json = r#"{"type":"reasoning","summary":"..."}"#;
    let err = serde_json::from_str::<OutputItem>(json);
    assert!(err.is_err(), "unknown type tag must not parse, got: {:?}", err);
}

#[test]
fn response_with_missing_object_field_fails_to_deserialize() {
    // `object` is required (not Option); omitting it must fail rather than
    // silently producing an empty object_type.
    let json = r#"{"id":"x","status":"completed","created_at":0,"model":"m","output":[],"usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}}"#;
    let err = serde_json::from_str::<Response>(json);
    assert!(err.is_err(), "missing `object` must fail, got: {:?}", err);
}
