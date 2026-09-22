use super::*;

use crate::types::{Input, InputMessage};

#[test]
fn build_fails_when_input_missing() {
    let err = CreateResponseRequestBuilder::default()
        .build()
        .expect_err("missing input must error");
    match err {
        crate::types::HermesError::Config(msg) => {
            assert!(
                msg.contains("input is required"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected Config error, got {other:?}"),
    }
}

#[test]
fn build_rejects_conversation_and_previous_response_id_together() {
    let err = CreateResponseRequestBuilder::default()
        .input("hi")
        .previous_response_id("resp_1")
        .conversation("conv")
        .build()
        .expect_err("mutually exclusive options must error");
    match err {
        crate::types::HermesError::Config(msg) => {
            assert!(
                msg.contains("mutually exclusive"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected Config error, got {other:?}"),
    }
}

#[test]
fn build_ok_with_text_input_only() {
    let req = CreateResponseRequestBuilder::default()
        .input("hello")
        .build()
        .expect("text input alone must build");
    match req.input {
        Input::Text(t) => assert_eq!(t, "hello"),
        other => panic!("expected Text input, got {other:?}"),
    }
    assert_eq!(req.instructions, None);
    assert_eq!(req.previous_response_id, None);
    assert_eq!(req.conversation, None);
    assert_eq!(req.store, None);
    assert_eq!(req.model, None);
    assert!(req.conversation_history.is_none());
}

#[test]
fn build_ok_with_messages_input() {
    let msgs = vec![InputMessage::user("a"), InputMessage::assistant("b")];
    let req = CreateResponseRequestBuilder::default()
        .messages(msgs.clone())
        .instructions("be brief")
        .previous_response_id("resp_42")
        .store(false)
        .model("hermes-x")
        .conversation_history(vec![InputMessage::system("sys")])
        .build()
        .expect("messages input with all fields must build");
    match req.input {
        Input::Messages(m) => {
            assert_eq!(m.len(), 2);
            assert_eq!(m[0].role, "user");
        }
        other => panic!("expected Messages input, got {other:?}"),
    }
    assert_eq!(req.instructions.as_deref(), Some("be brief"));
    assert_eq!(req.previous_response_id.as_deref(), Some("resp_42"));
    assert_eq!(req.conversation, None);
    assert_eq!(req.store, Some(false));
    assert_eq!(req.model.as_deref(), Some("hermes-x"));
    assert_eq!(req.conversation_history.as_ref().map(|v| v.len()), Some(1));
}

#[test]
fn build_ok_with_conversation_only() {
    let req = CreateResponseRequestBuilder::default()
        .input("hi")
        .conversation("my-conv")
        .build()
        .expect("conversation alone (no previous_response_id) must build");
    assert_eq!(req.conversation.as_deref(), Some("my-conv"));
    assert_eq!(req.previous_response_id, None);
}

#[test]
fn conversation_history_builder_chain_overwrites() {
    // builder setters overwrite; verify latest wins for a Vec field.
    let req = CreateResponseRequestBuilder::default()
        .input("hi")
        .conversation_history(vec![InputMessage::user("first")])
        .conversation_history(vec![InputMessage::user("second")])
        .build()
        .unwrap();
    let hist = req.conversation_history.expect("history set");
    assert_eq!(hist.len(), 1);
    assert_eq!(hist[0].content, "second");
}
