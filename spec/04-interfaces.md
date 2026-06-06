# 04 Interfaces

## HTTP Endpoints

All endpoints use JSON for request and response bodies. Every request to a
versioned endpoint carries `Authorization: Bearer <api_key>`.

### Endpoint Summary

| Operation | Method | Path | Success body | Error |
|-----------|--------|------|--------------|-------|
| Create response | POST | `/v1/responses` | response | APIError |
| Get response | GET | `/v1/responses/{id}` | response | APIError |
| Delete response | DELETE | `/v1/responses/{id}` | delete-response | APIError |
| Health check | GET | `/health` | (status-only) | NetworkError |

### Path Parameter

```abnf
response-id = "resp_" 1*CHAR
CHAR        = ALPHA / DIGIT
```

The `{id}` path segment **MUST** be a valid response-ID.

### Authentication Header

```abnf
auth-header = "Authorization" ":" OWS "Bearer" SP api-key OWS
api-key     = 1*VCHAR
```

## Request Body (Create Response)

```abnf
create-request-body = "{"
  %x22 "input" %x22 ":" ( text-input / messages-input )
  [ "," instructions-field ]
  [ "," previous-id-field ]
  [ "," conversation-field ]
  [ "," store-field ]
  [ "," model-field ]
  [ "," history-field ]
  "}"

text-input     = DQUOTE utf8-text DQUOTE
messages-input = "[" 1*( input-message ) "]"

instructions-field  = %x22 "instructions" %x22 ":" DQUOTE utf8-text DQUOTE
previous-id-field   = %x22 "previous_response_id" %x22 ":" DQUOTE response-id DQUOTE
conversation-field  = %x22 "conversation" %x22 ":" DQUOTE conversation-name DQUOTE
store-field         = %x22 "store" %x22 ":" ( "true" / "false" )
model-field         = %x22 "model" %x22 ":" DQUOTE model-name DQUOTE
history-field       = %x22 "conversation_history" %x22 ":" "[" *(input-message) "]"

input-message = "{" %x22 "role" %x22 ":" DQUOTE role DQUOTE ","
                   %x22 "content" %x22 ":" DQUOTE utf8-text DQUOTE "}"

role = "user" / "assistant" / "system"

conversation-name = 1*VCHAR
model-name        = 1*VCHAR
utf8-text         = *( Any valid UTF-8 codepoint except DQUOTE and backslash
                       unless escaped per JSON )
```

Fields not present in the JSON body are treated as absent (optional fields use
omit-if-null serialization).

## Builder Interface

The builder collects fields fluently and validates at build time.

```abnf
builder-call = builder-method *( builder-method ) ".build()"

builder-method = ".input("        text-value  ")"
               / ".messages("     msg-list    ")"
               / ".instructions(" text-value  ")"
               / ".previous_response_id(" text-value ")"
               / ".conversation(" text-value ")"
               / ".store("        boolean     ")"
               / ".model("        text-value  ")"
               / ".conversation_history(" msg-list ")"
```

### Builder Validation Rules

| Rule | Trigger | Result |
|------|---------|--------|
| V1 | `input` is not set | ConfigError: "input is required" |
| V2 | Both `conversation` and `previous_response_id` are set | ConfigError: "conversation and previous_response_id are mutually exclusive" |

## Response JSON Shape (Wire Format)

### response

```json
{
  "id":          "resp_...",
  "object":      "response",
  "status":      "completed",
  "created_at":  1700000000,
  "model":       "...",
  "output":      [ output-item, ... ],
  "usage":       { "input_tokens": N, "output_tokens": N, "total_tokens": N }
}
```

### output-item (tagged union on "type")

```json
{ "type": "function_call",         "name": "...", "arguments": "...", "call_id": "..." }
{ "type": "function_call_output",  "call_id": "...", "output": "..." }
{ "type": "message",               "role": "...", "content": [ content-part, ... ] }
```

### content-part

```json
{ "type": "output_text", "text": "..." }
```

### delete-response

```json
{ "id": "resp_...", "object": "response", "deleted": true }
```

### error response (from gateway)

```json
{
  "error": {
    "message": "...",
    "type":   "...",
    "param":  "...",
    "code":   "..."
  }
}
```

Fields `param` and `code` are optional in the error object.

## Accessor Methods

| Accessor | Applicable to | Returns |
|----------|--------------|---------|
| `response.text()` | response | text of first `Message` output-item, or nothing |
| `output-item.as_text()` | output-item (Message variant) | text of first `OutputText` content-part, or nothing |
| `output-item.as_function_call()` | output-item (FunctionCall variant) | `(name, arguments)`, or nothing |

## Extension Points

- **Model selection**: the `model` field allows the consumer to specify which
  model the gateway should use; the SDK does not validate model names.
- **Conversation history**: the `conversation_history` field allows injecting
  arbitrary prior message history; the gateway determines precedence.
- **Tool definitions**: not part of the SDK request -- tool availability is
  configured gateway-side; the SDK merely relays tool-call and tool-result
  output-items.
