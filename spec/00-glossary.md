# Glossary

Canonical terms used throughout this specification. All chapters MUST use the
*Canonical surface form* column exclusively; any synonym MUST be replaced or
annotated `(synonymous with X)`.

| Term | Canonical surface form | Citation | Used in |
|------|------------------------|----------|---------|
| client | client | e2e: `client()` helper, doc:README `HermesClient` | 01, 03, 04 |
| create-response request | create-response request | e2e: `CreateResponseRequest::builder()` section, doc:README `Request types` | 02, 04 |
| response | response | e2e: "returns response" assertion, doc:README `Response` | 02, 03, 04 |
| input | input | e2e: "simple text input" + "structured messages" sections, doc:README `Input` | 02, 04 |
| input-message | input-message | e2e: `InputMessage::user()` calls, doc:README `InputMessage` | 02, 04 |
| output-item | output-item | e2e: "output items (tool usage)" section, doc:README `OutputItem` | 02, 04 |
| content-part | content-part | e2e: pattern match on `ContentPart`, doc:README `ContentPart` | 02 |
| token-usage | token-usage | e2e: "usage.total_tokens > 0" assertion, doc:README `Usage` | 02 |
| conversation-name | conversation-name | e2e: "multi-turn conversation" section, doc:README `conversation()` | 02, 04 |
| delete-response | delete-response | e2e: "delete_response" section, doc:README `DeleteResponse` | 02, 04 |
| API error | API error | e2e: "bad API key returns 401" + "non-existent id returns API error", doc:README `HermesError` | 03 |
| builder | builder | e2e: "builder validation" section, doc:README `CreateResponseRequest::builder()` | 04 |
| gateway | gateway | doc:README `Hermes Agent API gateway`, e2e: `BASE_URL` constant | 01, 03 |
