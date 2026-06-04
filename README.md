# hermes-sdk

Rust SDK for the Hermes Agent API server (Responses API).

## Overview

hermes-sdk provides an async Rust client for the Hermes Agent API gateway, which exposes an OpenAI-compatible Responses API. It supports creating agent responses (with tool calls), multi-turn conversations via chaining or named conversations, retrieving and deleting stored responses, and health checks. Built on `reqwest` with `rustls-tls` for use within Tokio runtimes.

## Usage

```rust
use hermes_sdk::{HermesClient, CreateResponseRequest, InputMessage, OutputItem, ContentPart};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HermesClient::new("your-api-key", "http://localhost:8642");

    // Health check
    assert!(client.health().await?);

    // Simple text input
    let resp = client.create_response(
        &CreateResponseRequest::builder()
            .input("list files in the current directory")
            .store(true)
            .build()?
    ).await?;
    println!("{}: {}", resp.id, resp.text().unwrap_or_default());

    // Structured messages with instructions
    let req = CreateResponseRequest::builder()
        .messages(vec![
            InputMessage::system("Be concise."),
            InputMessage::user("Explain recursion."),
        ])
        .instructions("Under 20 words.")
        .build()?;

    // Multi-turn via named conversation (gateway auto-chains to latest)
    let t1 = CreateResponseRequest::builder()
        .input("remember: CAMEL").conversation("s1").store(true).build()?;
    let t2 = CreateResponseRequest::builder()
        .input("what was the secret word?").conversation("s1").build()?;

    // Multi-turn via previous_response_id
    let r1 = client.create_response(&t1).await?;
    let r2 = CreateResponseRequest::builder()
        .input("follow-up").previous_response_id(&r1.id).build()?;

    // Retrieve and delete
    let stored = client.get_response(&resp.id).await?;
    let deleted = client.delete_response(&resp.id).await?;
    assert!(deleted.deleted);

    Ok(())
}
```

## API

### `HermesClient`

Construct with `HermesClient::new(api_key, base_url)`. The `base_url` defaults to `http://127.0.0.1:8642` when empty.

| Method | Endpoint | Returns |
|--------|----------|---------|
| `health()` | `GET /health` | `bool` |
| `create_response(&request)` | `POST /v1/responses` | `Response` |
| `get_response(id)` | `GET /v1/responses/{id}` | `Response` |
| `delete_response(id)` | `DELETE /v1/responses/{id}` | `DeleteResponse` |

### Request types

- **`CreateResponseRequest`** -- body for `POST /v1/responses`; construct via `CreateResponseRequest::builder()`.
  Builder methods: `input()`, `messages()`, `instructions()`, `previous_response_id()`, `conversation()`, `store()`, `model()`, `conversation_history()`, `build()`.
  Validation: rejects missing input; rejects `conversation` + `previous_response_id` together.
- **`Input`** -- `Text(String)` or `Messages(Vec<InputMessage>)`.
- **`InputMessage`** -- `{ role, content }` with helpers `user()`, `assistant()`, `system()`.

### Response types

- **`Response`** -- `id`, `status`, `model`, `output: Vec<OutputItem>`, `usage: Usage`; helper `text()` extracts first assistant message.
- **`OutputItem`** -- tagged enum: `FunctionCall { name, arguments, call_id }`, `FunctionCallOutput { call_id, output }`, `Message { role, content }`; helpers `as_text()`, `as_function_call()`.
- **`ContentPart`** -- `OutputText { text }`.
- **`Usage`** -- `input_tokens`, `output_tokens`, `total_tokens`.
- **`DeleteResponse`** -- `id`, `deleted`.
- **`HermesError`** -- `Api { status, message, ... }`, `Network(reqwest::Error)`, `Config(String)`.

## Development

```bash
just build          # cargo build
just test           # cargo test
```

End-to-end tests against a live Hermes gateway:

```bash
cd e2e && cargo run
```

See `examples/basic.rs` for a complete walkthrough.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` 0.12 | Async HTTP client (JSON, rustls-tls) |
| `serde` / `serde_json` | Request/response serialization |
| `thiserror` 2 | Error derive for `HermesError` |
| `tokio` 1 | Async runtime |
