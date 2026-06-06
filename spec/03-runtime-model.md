# 03 Runtime Model

## Client Construction

Algorithm 1 ConstructClient

Require: api_key (non-empty secret text), base_url (UTF-8 text)
Ensure: returns a client with resolved base_url

```
1: if base_url is empty then
2:   resolved ← "http://127.0.0.1:8642"
3: else
4:   resolved ← base_url
5: end if
6: return Client{ base_url = resolved, api_key = api_key }
```

## Request Building

Algorithm 2 BuildCreateResponseRequest

Require: builder has input set
Require: builder does NOT have both conversation AND previous_response_id set
Ensure: returns a valid create-response request

```
1: if builder.input is not set then
2:   return Error("input is required")
3: end if
4: if builder.conversation is set AND builder.previous_response_id is set then
5:   return Error("conversation and previous_response_id are mutually exclusive")
6: end if
7: return CreateResponseRequest constructed from builder fields
```

Validation errors are returned as ConfigError variants (local, not from the
gateway).

## Gateway Interaction Sequence

All gateway operations follow a common request/response pattern:

```
   Client                              Gateway
     |                                    |
     |------ POST /v1/responses -------->|
     |        Authorization: Bearer K     |
     |        Body: JSON                  |
     |                                    |--+ validate auth + process
     |                                    |<-+
     |<----- 200 JSON (response) ---------|    ▷ success
     |                                    |
     |  or                                |
     |                                    |
     |<----- 401/4xx/5xx JSON (error) ----|    ▷ failure
     |                                    |
```

The same pattern applies to GET and DELETE, differing only in HTTP method and
path. The health endpoint (`GET /health`) also requires the Authorization
header and returns success (2xx) or failure (non-2xx).

## Gateway Operations

### Create Response

Algorithm 3 CreateResponse

Require: client is initialized, request passes Algorithm 2
Ensure: returns response on success, or APIError / NetworkError on failure

```
 1: url ← client.base_url + "/v1/responses"
 2: http_response ← HTTP POST to url
 3:   with header "Authorization: Bearer " + client.api_key
 4:   with JSON-serialized request as body
 5: if transport failure then
 6:   return NetworkError
 7: end if
 8: if http_response status is not 2xx then
 9:   body ← read http_response body as UTF-8
10:   return ParseAPIError(status, body)           ▷ see Error Handling
11: end if
12: return response deserialized from http_response body
```

Failure modes:

| Trigger | Error | HTTP status |
|---------|-------|-------------|
| Transport failure (DNS, TLS, connection refused) | NetworkError | -- |
| Invalid or missing API key | APIError | 401 |
| Malformed request body | APIError | 400 |
| Gateway internal error | APIError | 5xx |
| Non-JSON response body | APIError (raw body as message) | any non-2xx |

### Get Response

Algorithm 4 GetResponse

Require: client is initialized, response-ID is a non-empty string
Ensure: returns stored response on success, or APIError / NetworkError on failure

```
1: url ← client.base_url + "/v1/responses/" + response-ID
2: http_response ← HTTP GET to url
3:   with header "Authorization: Bearer " + client.api_key
4: if transport failure then
5:   return NetworkError
6: end if
7: if http_response status is not 2xx then
8:   body ← read http_response body as UTF-8
9:   return ParseAPIError(status, body)
10: end if
11: return response deserialized from http_response body
```

Failure modes:

| Trigger | Error | HTTP status |
|---------|-------|-------------|
| Transport failure (DNS, TLS, connection refused) | NetworkError | -- |
| Invalid or missing API key | APIError | 401 |
| Response-ID not found | APIError | 404 or 400 |
| Gateway internal error | APIError | 5xx |
| Non-JSON response body | APIError (raw body as message) | any non-2xx |

### Delete Response

Algorithm 5 DeleteResponse

Require: client is initialized, response-ID is a non-empty string
Ensure: returns delete-response on success, or APIError / NetworkError on failure

```
1: url ← client.base_url + "/v1/responses/" + response-ID
2: http_response ← HTTP DELETE to url
3:   with header "Authorization: Bearer " + client.api_key
4: if transport failure then
5:   return NetworkError
6: end if
7: if http_response status is not 2xx then
8:   body ← read http_response body as UTF-8
9:   return ParseAPIError(status, body)
10: end if
11: return delete-response deserialized from http_response body
```

Failure modes:

| Trigger | Error | HTTP status |
|---------|-------|-------------|
| Transport failure (DNS, TLS, connection refused) | NetworkError | -- |
| Invalid or missing API key | APIError | 401 |
| Response-ID not found | APIError | 404 or 400 |
| Already deleted | APIError or success (gateway-dependent) | 404 or 200 |
| Gateway internal error | APIError | 5xx |
| Non-JSON response body | APIError (raw body as message) | any non-2xx |

### Health Check

Algorithm 6 HealthCheck

Require: client is initialized
Ensure: returns true if gateway responded with 2xx, false otherwise

```
1: url ← client.base_url + "/health"
2: http_response ← HTTP GET to url
3:   with header "Authorization: Bearer " + client.api_key
4: if transport failure then
5:   return NetworkError
6: end if
7: return http_response status is 2xx
```

Failure modes:

| Trigger | Error | Result |
|---------|-------|--------|
| Transport failure (DNS, TLS, connection refused) | NetworkError | -- |
| Non-2xx response | -- | returns `false` |

## Error Handling

**ParseAPIError(status, body)** -- referenced by Algorithms 3-5:

1. Read `body` as UTF-8 text.
2. Attempt to parse as structured error JSON:
   `{ "error": { "message": ..., "type": ..., "param": ..., "code": ... } }`.
3. If parsing succeeds, return APIError with `status` from the HTTP status
   code, and `message`, `error_type`, `code` from the parsed body.
4. If parsing fails, return APIError with `status` from the HTTP status code,
   `message` set to the raw body, and `error_type` / `code` absent.

Transport-level failures (connection refused, DNS resolution failure, TLS
handshake failure, etc.) are returned as NetworkError.

## Multi-Turn Semantics

Two mechanisms for chaining create-response requests into a conversation:

1. **conversation-name**: the client sends a `conversation` field with a
   stable name. The gateway is responsible for resolving the conversation to
   the latest response in that named sequence. Multiple requests using the
   same conversation-name form a single logical conversation.

2. **previous_response_id**: the client sends a `previous_response_id` field
   referencing a specific response. The gateway chains the new request to that
   exact response.

These two mechanisms **MUST NOT** be used simultaneously in the same request
(enforced by Algorithm 2).

Cross-operation invariant:

```
conversation-name set ⊕ previous_response_id set → request is valid
conversation-name set ∧ previous_response_id set → ConfigError
```

## State Considerations

The client itself holds no mutable state between operations. The client is
safe to use concurrently. All conversation state is managed server-side by the
gateway.

The SDK does not implement retry, timeout, or cancellation. If the underlying
transport times out, the error surfaces as a NetworkError. The consumer MAY
wrap operations with their own retry/timeout logic.
