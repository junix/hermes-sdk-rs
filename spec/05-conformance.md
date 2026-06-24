# 05 Conformance

## Examples

### E.1 Simple text input produces a valid response

Given a client with valid credentials, when a create-response request is built
with `.input("respond with exactly: PONG").store(true)` and sent, then:

- `response.id` starts with `"resp_"`.
- `response.status` equals `"completed"`.
- `response.object_type` equals `"response"`.
- `response.output` is non-empty.
- `response.text()` contains `"PONG"` (case-insensitive).
- `response.usage.total_tokens > 0`.

### E.2 Structured messages are accepted

Given a create-response request built with `.messages([InputMessage.user("say hello")])`,
the gateway returns a response whose text contains a greeting.

### E.3 Tool usage produces function-call output-items

Given a create-response request whose input triggers tool execution, the
response's `output` list contains at least one `FunctionCall` item, at least
one `FunctionCallOutput` item, and at least one `Message` item. The
`FunctionCallOutput` includes the executed command's output.

### E.4 Multi-turn via conversation-name preserves context

Given two sequential create-response requests using the same conversation-name:

1. First request: `.input("remember this secret word: CAMEL").conversation("s1")`
2. Second request: `.input("what was the secret word?").conversation("s1")`

Then the second response's text contains `"CAMEL"`.

### E.5 Multi-turn via previous_response_id preserves context

Given a first response R1, and a second create-response request built with
`.input("follow-up").previous_response_id(R1.id)`, then the second response
has access to context from R1.

### E.6 Retrieve a stored response

Given a previously created response with `store(true)`, when `get_response` is
called with that response's ID, then the returned response has the same ID,
status `"completed"`, and non-empty output.

### E.7 Delete a stored response

Given a previously created and stored response, when `delete_response` is
called with that response's ID, then the returned delete-response has
`deleted = true`, matching ID, and `object_type = "response"`.

### E.8 Duplicate deletion is tolerated

Given a previously deleted response, when `delete_response` is called again
with the same ID, the operation either succeeds or returns an APIError (e.g.,
404).

### E.9 Authentication failure returns 401

Given a client constructed with an invalid API key, when `create_response` is
called, the result is an APIError with `status = 401`.

### E.10 Non-existent response returns error

Given a `get_response` call with a non-existent response-ID
(e.g., `"resp_nonexistent_00000000"`), the result is an APIError with status
404 or 400.

### E.11 Builder rejects missing input

Calling `.build()` on a builder without setting `.input()` or `.messages()`
returns a ConfigError.

### E.12 Builder rejects conversation + previous_response_id

Calling `.input("x").conversation("c").previous_response_id("r").build()`
returns a ConfigError.

### E.13 Health check returns boolean

Given a reachable gateway, `health()` returns `true`. Given an unreachable
gateway, the operation returns a NetworkError.

### E.14 Token-usage invariant holds

For any successful response, `usage.total_tokens = usage.input_tokens +
usage.output_tokens`.

---

## Definition of Done

### A. Client Construction

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| A.1 | Empty base_url resolves to default | Client sends requests to `http://127.0.0.1:8642` | [U] 当前未覆盖 |
| A.2 | Non-empty base_url is used as-is | Client sends requests to the given URL | [U] 当前未覆盖 |

### B. Builder Validation

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| B.1 | Builder without input fails | ConfigError returned; message contains "input is required" | [T] |
| B.2 | Builder with both conversation and previous_response_id fails | ConfigError returned; message contains "mutually exclusive" | [T] |
| B.3 | Builder with only input succeeds | Valid request returned; optional fields are absent | [T] |
| B.4 | Builder with messages input plus all optional fields succeeds | Input is Messages; instructions/previous_response_id/store/model/conversation_history carried through | [T] |
| B.5 | Builder with conversation but no previous_response_id succeeds | conversation set, previous_response_id absent | [T] |
| B.6 | Builder setters overwrite (last value wins) | Repeated setter call for the same field keeps only the last value | [T] |

### C. Create Response

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| C.1 | Simple text input returns response | response.id starts with "resp_", status is "completed" | [T] |
| C.2 | Structured messages input returns response | Response text is non-empty | [T] |
| C.3 | Tool-triggering input produces function-call items | output contains FunctionCall, FunctionCallOutput, Message | [T] |
| C.4 | Response includes token-usage | total_tokens > 0, total = input + output | [T] |
| C.5 | Response object_type is "response" | object_type equals "response" | [T] |
| C.6 | Instructions field is accepted | Response returned successfully | [T] |

### D. Multi-Turn

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| D.1 | Conversation-name chaining preserves context | Second turn references first turn's content | [T] |
| D.2 | previous_response_id chaining preserves context | Second turn references first turn's content | [T] |

### E. Retrieve and Delete

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| E.1 | get_response returns stored response | Same ID, status, non-empty output | [T] |
| E.2 | delete_response marks response as deleted | deleted = true, matching ID | [T] |
| E.3 | get_response with non-existent ID returns error | APIError with status 404 or 400 | [T] |
| E.4 | Double delete is tolerated | Either succeeds or returns APIError | [T] |
| E.5 | delete-response object_type is "response" | object_type = "response" | [T] |

### F. Error Handling

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| F.1 | Invalid API key returns 401 | APIError with status 401 | [T] |
| F.2 | Non-2xx produces APIError with status | status, message populated; error_type and code present when gateway returns structured error | [T] |
| F.3 | Network failure produces NetworkError | NetworkError variant returned | [U] 当前未覆盖 |
| F.4 | Non-JSON error body still produces APIError | message = raw body, error_type and code absent | [U] 当前未覆盖 |

### G. Health Check

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| G.1 | Healthy gateway returns true | health() returns true | [T] |
| G.2 | Unreachable gateway returns error | NetworkError returned | [U] 当前未覆盖 |

### H. Accessors

| ID | Behavior | Observable result | Status |
|----|----------|-------------------|--------|
| H.1 | response.text() returns first assistant text, skipping non-Message items | First Message output-item's first text content-part | [T] |
| H.2 | as_function_call() returns name and arguments | (name, arguments) for FunctionCall variant | [T] |
| H.3 | as_text() returns the first text of a Message item | First content-part's text when Message has content | [T] |
| H.4 | as_text() returns nothing for non-Message items | Returns nothing for FunctionCall / FunctionCallOutput | [T] |
| H.5 | as_text() returns nothing for a Message with empty content | Nothing returned when content list is empty | [T] |
| H.6 | as_function_call() returns nothing for non-FunctionCall items | Returns nothing for Message / FunctionCallOutput | [T] |

---

## Revision History

- 2026-06-24 — code_stronger_than_code: commit ecedfed added unit tests (`builder_test.rs`, `response_test.rs`, `client_test.rs`) locking previously-uncovered builder/accessor/delete behaviors; upgraded H.3/H.4 from [U] to [T], added B.4/B.5/B.6 and H.5/H.6, and clarified H.1 to reflect that `response.text()` skips non-Message items.
- 2026-06-24 — aligned: builder validation ConfigError messages ("input is required", "mutually exclusive") confirmed by `build_fails_when_input_missing` and `build_rejects_conversation_and_previous_response_id_together`; B.1/B.2 observable-result column tightened with the substring.
