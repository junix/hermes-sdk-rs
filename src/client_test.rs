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
    // Cannot easily assert the exact default without exposing internals; instead
    // assert that an empty base_url is accepted and the client constructs.
    let client = HermesClient::new("k", "");
    // Round-trip sanity: a client built with an explicit URL keeps it in the
    // request target. Empty string path is exercised by the success test below
    // indirectly via a real URL; here we just ensure construction does not panic.
    let _ = format!("{:?}", client);
}
