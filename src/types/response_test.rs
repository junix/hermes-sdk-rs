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
