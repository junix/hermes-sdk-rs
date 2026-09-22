use super::*;

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Read one HTTP request (request line + headers + blank line), ignoring any body.
async fn read_request_head(conn: &mut tokio::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let n = conn.read(&mut byte).await.expect("read");
        if n == 0 {
            break;
        }
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    String::from_utf8(buf).expect("utf8 request head")
}

/// A single canned response the mock server will emit.
struct Canned {
    status_line: &'static str,
    body: String,
}

/// Spawn a mock that accepts one connection, asserts the request line matches,
/// then replies with the canned status + JSON body.
async fn spawn_mock(expected_request_line: &'static str, canned: Canned) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        let mut conn = listener.accept().await.expect("accept").0;
        let head = read_request_head(&mut conn).await;
        let first_line = head.lines().next().unwrap_or("");
        assert!(
            first_line.starts_with(expected_request_line),
            "unexpected request line: {first_line}"
        );
        // Assert Authorization header is set with the bearer token (case-insensitive).
        let head_lower = head.to_ascii_lowercase();
        assert!(
            head_lower.contains("authorization: bearer secret-key"),
            "missing/incorrect auth header in request:\n{head}"
        );

        let body_bytes = canned.body.as_bytes();
        let resp = format!(
            "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            canned.status_line,
            body_bytes.len(),
            canned.body
        );
        conn.write_all(resp.as_bytes()).await.expect("write");
        conn.flush().await.expect("flush");
        // Drain until the client closes so hyper sees a clean EOF.
        let mut sink = [0u8; 64];
        loop {
            match tokio::time::timeout(Duration::from_secs(1), conn.read(&mut sink)).await {
                Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break,
                Ok(Ok(_)) => continue,
            }
        }
    });

    url
}

fn client_at(url: &str) -> HermesClient {
    HermesClient::new("secret-key", url)
}

#[tokio::test]
async fn delete_response_success_returns_deleted_flag() {
    let url = spawn_mock(
        "DELETE /v1/responses/resp_99",
        Canned {
            status_line: "HTTP/1.1 200 OK",
            body: r#"{"id":"resp_99","object":"response.deleted","deleted":true}"#.to_string(),
        },
    )
    .await;

    let out = client_at(&url)
        .delete_response("resp_99")
        .await
        .expect("ok");
    assert_eq!(out.id, "resp_99");
    assert_eq!(out.object_type, "response.deleted");
    assert!(out.deleted);
}

#[tokio::test]
async fn delete_response_maps_non_2xx_to_api_error() {
    let url = spawn_mock(
        "DELETE /v1/responses/missing",
        Canned {
            status_line: "HTTP/1.1 404 Not Found",
            body: r#"{"error":{"message":"not found","type":"not_found","code":"404"}}"#
                .to_string(),
        },
    )
    .await;

    let err = client_at(&url)
        .delete_response("missing")
        .await
        .expect_err("404 must error");
    match err {
        HermesError::Api {
            status,
            message,
            error_type,
            code,
        } => {
            assert_eq!(status, 404);
            assert_eq!(message, "not found");
            assert_eq!(error_type.as_deref(), Some("not_found"));
            assert_eq!(code.as_deref(), Some("404"));
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[test]
fn new_uses_default_base_url_when_empty() {
    // An empty base_url must fall back to the documented default
    // (http://127.0.0.1:8642). The field is private, so observe it through the
    // derived Debug representation, which renders it as a quoted string.
    let client = HermesClient::new("k", "");
    let debug = format!("{:?}", client);
    assert!(
        debug.contains("base_url: \"http://127.0.0.1:8642\""),
        "empty base_url must default to http://127.0.0.1:8642, got: {debug}"
    );
}

#[tokio::test]
async fn ensure_success_passes_through_2xx_response() {
    // Drive ensure_success directly: issue a raw request so we can hand the
    // resulting Response to the helper instead of going through an endpoint.
    let url = spawn_mock(
        "GET /v1/responses/ok",
        Canned {
            status_line: "HTTP/1.1 200 OK",
            body: r#"{"ok":true}"#.to_string(),
        },
    )
    .await;
    let resp = reqwest::Client::new()
        .get(format!("{}/v1/responses/ok", url))
        .header("Authorization", "Bearer secret-key")
        .send()
        .await
        .expect("send");
    let resp = client_at(&url)
        .ensure_success(resp)
        .await
        .expect("2xx must pass through");
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn ensure_success_maps_non_2xx_to_api_error() {
    let url = spawn_mock(
        "GET /v1/responses/bad",
        Canned {
            status_line: "HTTP/1.1 422 Unprocessable Entity",
            body: r#"{"error":{"message":"nope","type":"invalid_request","code":"422"}}"#
                .to_string(),
        },
    )
    .await;
    let resp = reqwest::Client::new()
        .get(format!("{}/v1/responses/bad", url))
        .header("Authorization", "Bearer secret-key")
        .send()
        .await
        .expect("send");
    let err = client_at(&url)
        .ensure_success(resp)
        .await
        .expect_err("non-2xx must error");
    match err {
        HermesError::Api {
            status,
            message,
            error_type,
            code,
        } => {
            assert_eq!(status, 422);
            assert_eq!(message, "nope");
            assert_eq!(error_type.as_deref(), Some("invalid_request"));
            assert_eq!(code.as_deref(), Some("422"));
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn from_response_falls_back_to_raw_body_when_error_body_is_not_structured_json() {
    // from_response's other branch: a body that is not `{"error":{...}}` JSON
    // must land in the fallback arm — raw body becomes the message, and
    // error_type/code are None (spec: APIError fallback).
    let url = spawn_mock(
        "GET /v1/responses/crash",
        Canned {
            status_line: "HTTP/1.1 500 Internal Server Error",
            body: "upstream boom".to_string(),
        },
    )
    .await;
    let resp = reqwest::Client::new()
        .get(format!("{}/v1/responses/crash", url))
        .header("Authorization", "Bearer secret-key")
        .send()
        .await
        .expect("send");
    let err = client_at(&url)
        .ensure_success(resp)
        .await
        .expect_err("500 must error");
    match err {
        HermesError::Api {
            status,
            message,
            error_type,
            code,
        } => {
            assert_eq!(status, 500);
            assert_eq!(message, "upstream boom");
            assert_eq!(error_type, None, "error_type must be absent for raw body");
            assert_eq!(code, None, "code must be absent for raw body");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn health_returns_true_for_2xx() {
    let url = spawn_mock(
        "GET /health",
        Canned {
            status_line: "HTTP/1.1 200 OK",
            body: "{}".to_string(),
        },
    )
    .await;
    let ok = client_at(&url)
        .health()
        .await
        .expect("health must not error on 2xx");
    assert!(ok);
}

#[tokio::test]
async fn health_returns_false_for_non_2xx_without_erroring() {
    // Distinct contract from the CRUD endpoints: health surfaces a failing
    // gateway as Ok(false), not Err, so callers can use it as a probe.
    let url = spawn_mock(
        "GET /health",
        Canned {
            status_line: "HTTP/1.1 503 Service Unavailable",
            body: "{}".to_string(),
        },
    )
    .await;
    let ok = client_at(&url)
        .health()
        .await
        .expect("health must not error on non-2xx");
    assert!(!ok);
}

#[tokio::test]
async fn create_response_posts_and_decodes_full_response() {
    // Flagship endpoint: POST /v1/responses with the JSON request body, then
    // decode the gateway Response (validates the full happy-path + parse).
    let body = r#"{
        "id":"resp_1",
        "object":"response",
        "status":"completed",
        "created_at":7,
        "model":"hermes-agent",
        "output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"PONG"}]}],
        "usage":{"input_tokens":1,"output_tokens":2,"total_tokens":3}
    }"#;
    let url = spawn_mock(
        "POST /v1/responses",
        Canned {
            status_line: "HTTP/1.1 200 OK",
            body: body.to_string(),
        },
    )
    .await;
    let req = CreateResponseRequest::builder()
        .input("ping")
        .store(true)
        .build()
        .expect("build");
    let resp = client_at(&url)
        .create_response(&req)
        .await
        .expect("create must succeed");
    assert_eq!(resp.id, "resp_1");
    assert_eq!(resp.status, "completed");
    assert_eq!(resp.object_type, "response");
    assert_eq!(resp.created_at, 7);
    assert_eq!(resp.usage.input_tokens, 1);
    assert_eq!(resp.usage.output_tokens, 2);
    assert_eq!(resp.usage.total_tokens, 3);
    assert_eq!(resp.text(), Some("PONG"));
}

#[tokio::test]
async fn get_response_gets_and_decodes_response() {
    let body = r#"{
        "id":"resp_42",
        "object":"response",
        "status":"completed",
        "created_at":9,
        "model":"hermes-agent",
        "output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hi"}]}],
        "usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}
    }"#;
    let url = spawn_mock(
        "GET /v1/responses/resp_42",
        Canned {
            status_line: "HTTP/1.1 200 OK",
            body: body.to_string(),
        },
    )
    .await;
    let resp = client_at(&url)
        .get_response("resp_42")
        .await
        .expect("get must succeed");
    assert_eq!(resp.id, "resp_42");
    assert_eq!(resp.status, "completed");
    assert_eq!(resp.text(), Some("hi"));
}
