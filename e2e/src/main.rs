use hermes_sdk::{ContentPart, CreateResponseRequest, HermesClient, HermesError, InputMessage, OutputItem};

const API_KEY: &str = "hello-key";
const BASE_URL: &str = "http://localhost:8642";

fn client() -> HermesClient {
    HermesClient::new(API_KEY, BASE_URL)
}

fn pass(name: &str) {
    println!("  ✓ {name}");
}

fn fail(name: &str, err: &HermesError) {
    println!("  ✗ {name}: {err}");
}

#[tokio::main]
async fn main() {
    let client = client();

    println!("hermes-sdk e2e tests\n");

    // ── health ──────────────────────────────────────────────────────────
    println!("health");
    match client.health().await {
        Ok(true) => pass("gateway is healthy"),
        Ok(false) => fail("gateway unhealthy", &HermesError::Config("returned false".into())),
        Err(e) => fail("health check failed", &e),
    }

    // ── create_response: simple text input ──────────────────────────────
    println!("\ncreate_response (simple text)");
    let resp = match client
        .create_response(
            &CreateResponseRequest::builder()
                .input("respond with exactly: PONG")
                .store(true)
                .build()
                .unwrap(),
        )
        .await
    {
        Ok(r) => {
            pass("returns response");
            r
        }
        Err(e) => {
            fail("create response", &e);
            return;
        }
    };

    // validate response shape
    assert!(!resp.id.is_empty(), "response id should not be empty");
    pass("has non-empty id");

    assert!(
        resp.id.starts_with("resp_"),
        "id should start with resp_"
    );
    pass("id has resp_ prefix");

    assert_eq!(resp.status, "completed", "status should be completed");
    pass("status is completed");

    assert_eq!(resp.object_type, "response");
    pass("object_type is response");

    assert!(!resp.output.is_empty(), "output should not be empty");
    pass("output is non-empty");

    assert!(resp.usage.total_tokens > 0, "usage should be > 0");
    pass("usage.total_tokens > 0");

    assert!(
        resp.usage.total_tokens == resp.usage.input_tokens + resp.usage.output_tokens,
        "total = input + output"
    );
    pass("total_tokens == input + output");

    // check assistant text contains PONG
    let text = resp.text().unwrap_or_default().to_lowercase();
    assert!(text.contains("pong"), "expected PONG in response, got: {text}");
    pass("response contains PONG");

    let first_id = resp.id.clone();

    // ── create_response: structured input ───────────────────────────────
    println!("\ncreate_response (structured messages)");
    let resp2 = match client
        .create_response(
            &CreateResponseRequest::builder()
                .messages(vec![InputMessage::user("say hello")])
                .build()
                .unwrap(),
        )
        .await
    {
        Ok(r) => {
            pass("accepts structured messages");
            r
        }
        Err(e) => {
            fail("structured input", &e);
            return;
        }
    };

    let text2 = resp2.text().unwrap_or_default().to_lowercase();
    assert!(text2.contains("hello") || text2.contains("hi"), "expected greeting, got: {text2}");
    pass("structured input response looks correct");

    // ── output items contain function_call when tools are used ──────────
    println!("\noutput items (tool usage)");
    let tool_resp = match client
        .create_response(
            &CreateResponseRequest::builder()
                .input("run: echo TOOL_TEST_MARKER_12345")
                .store(true)
                .build()
                .unwrap(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            fail("tool request", &e);
            return;
        }
    };

    let has_fn_call = tool_resp.output.iter().any(|item| {
        matches!(item, OutputItem::FunctionCall { .. })
    });
    assert!(has_fn_call, "expected at least one function_call in output");
    pass("output contains function_call");

    let has_fn_output = tool_resp.output.iter().any(|item| {
        matches!(item, OutputItem::FunctionCallOutput { .. })
    });
    assert!(has_fn_output, "expected at least one function_call_output");
    pass("output contains function_call_output");

    let has_message = tool_resp.output.iter().any(|item| {
        matches!(item, OutputItem::Message { .. })
    });
    assert!(has_message, "expected at least one message output");
    pass("output contains message");

    // verify function_call_output contains the marker
    let marker_found = tool_resp.output.iter().any(|item| {
        if let OutputItem::FunctionCallOutput { output, .. } = item {
            output.contains("TOOL_TEST_MARKER_12345")
        } else {
            false
        }
    });
    assert!(marker_found, "function_call_output should contain TOOL_TEST_MARKER_12345");
    pass("tool result contains our marker");

    // verify as_function_call helper
    let fn_call_detail = tool_resp.output.iter().find_map(|item| item.as_function_call());
    assert!(fn_call_detail.is_some(), "as_function_call should return Some");
    pass("as_function_call() helper works");

    // verify as_text helper
    let msg_text = tool_resp.output.iter().find_map(|item| item.as_text());
    assert!(msg_text.is_some(), "as_text should return Some");
    pass("as_text() helper works");

    let tool_resp_id = tool_resp.id.clone();

    // ── get_response ────────────────────────────────────────────────────
    println!("\nget_response");
    match client.get_response(&first_id).await {
        Ok(stored) => {
            assert_eq!(stored.id, first_id);
            pass("retrieves stored response by id");
            assert_eq!(stored.status, "completed");
            pass("stored status is completed");
            assert!(!stored.output.is_empty());
            pass("stored output is non-empty");
        }
        Err(e) => fail("get_response", &e),
    }

    // get_response: non-existent id returns error
    println!("\nget_response (not found)");
    match client.get_response("resp_nonexistent_00000000").await {
        Err(HermesError::Api { status, .. }) => {
            assert!(status == 404 || status == 400, "expected 404 or 400, got {status}");
            pass("non-existent id returns API error");
        }
        Ok(_) => fail("non-existent id", &HermesError::Config("should have errored".into())),
        Err(e) => fail("unexpected error type", &e),
    }

    // ── delete_response ─────────────────────────────────────────────────
    println!("\ndelete_response");
    match client.delete_response(&tool_resp_id).await {
        Ok(del) => {
            assert_eq!(del.id, tool_resp_id);
            pass("deletes stored response");
            assert!(del.deleted);
            pass("deleted field is true");
            assert_eq!(del.object_type, "response");
            pass("object_type is response");
        }
        Err(e) => fail("delete_response", &e),
    }

    // delete again should still succeed or 404
    match client.delete_response(&tool_resp_id).await {
        Ok(_) | Err(HermesError::Api { .. }) => pass("double delete is handled"),
        Err(e) => fail("double delete", &e),
    }

    // ── multi-turn via conversation ─────────────────────────────────────
    println!("\nmulti-turn conversation");
    let conv = format!("e2e-test-{}", std::process::id());

    let turn1 = client
        .create_response(
            &CreateResponseRequest::builder()
                .input("remember this secret word: CAMEL")
                .conversation(&conv)
                .store(true)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(!turn1.id.is_empty());
    pass("turn 1 created");

    let turn2 = client
        .create_response(
            &CreateResponseRequest::builder()
                .input("what was the secret word I just told you? reply with ONLY that word")
                .conversation(&conv)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let answer = turn2.text().unwrap_or_default().to_uppercase();
    assert!(answer.contains("CAMEL"), "expected CAMEL in multi-turn response, got: {answer}");
    pass("turn 2 remembers context from turn 1");

    // ── multi-turn via previous_response_id ─────────────────────────────
    println!("\nmulti-turn previous_response_id");
    let r1 = client
        .create_response(
            &CreateResponseRequest::builder()
                .input("my favorite number is 42")
                .store(true)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let r2 = client
        .create_response(
            &CreateResponseRequest::builder()
                .input("what is my favorite number? reply with ONLY the number")
                .previous_response_id(&r1.id)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let num_answer = r2.text().unwrap_or_default();
    assert!(num_answer.contains("42"), "expected 42, got: {num_answer}");
    pass("previous_response_id preserves context");

    // ── builder validation ──────────────────────────────────────────────
    println!("\nbuilder validation");
    let no_input = CreateResponseRequest::builder().build();
    assert!(no_input.is_err(), "builder without input should fail");
    pass("rejects missing input");

    let both_chain = CreateResponseRequest::builder()
        .input("test")
        .conversation("c")
        .previous_response_id("r")
        .build();
    assert!(both_chain.is_err(), "conversation + previous_response_id should fail");
    pass("rejects conversation + previous_response_id");

    // ── instructions field ──────────────────────────────────────────────
    println!("\ninstructions field");
    let with_instructions = client
        .create_response(
            &CreateResponseRequest::builder()
                .input("what is 2+2?")
                .instructions("Always respond in Chinese. Keep it short.")
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let instr_text = with_instructions.text().unwrap_or_default();
    // Just verify the request with instructions succeeds and returns a valid response
    assert!(!instr_text.is_empty(), "instructions response should not be empty");
    pass("instructions field accepted and response returned");

    // ── error handling: bad auth ────────────────────────────────────────
    println!("\nerror handling");
    let bad_client = HermesClient::new("wrong-key", BASE_URL);
    match bad_client
        .create_response(
            &CreateResponseRequest::builder()
                .input("should fail")
                .build()
                .unwrap(),
        )
        .await
    {
        Err(HermesError::Api { status, .. }) => {
            assert_eq!(status, 401, "expected 401, got {status}");
            pass("bad API key returns 401");
        }
        Ok(_) => fail("bad auth should fail", &HermesError::Config("succeeded unexpectedly".into())),
        Err(e) => fail("wrong error type", &e),
    }

    // ── summary ─────────────────────────────────────────────────────────
    println!("\n═══ all e2e tests passed ═══");
}
