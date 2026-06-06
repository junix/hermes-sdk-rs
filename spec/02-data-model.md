# 02 Data Model

## Client

```
Client {
  base_url    : URI          ▷ gateway endpoint, defaults when empty
  api_key     : secret-text  ▷ Bearer token for Authorization header
  http_handle : opaque       ▷ implementation-defined HTTP connection pool
}
```

- The client is cloneable and MAY be shared across concurrent tasks.
- `base_url` **MUST** be used as the prefix for all endpoint paths.

## Create-Response Request

```
CreateResponseRequest {
  input                 : Input,
  instructions          : optional UTF-8 text,
  previous_response_id  : optional response-ID,
  conversation          : optional conversation-name,
  store                 : optional boolean,
  model                 : optional model-identifier,
  conversation_history  : optional ordered list of input-message,
}
```

Constraints (enforced at build time):

- `input` **MUST** be present.
- `conversation` and `previous_response_id` **MUST NOT** both be present.

## Input

Discriminated union:

```
Input =
  | Text(UTF-8 string)
  | Messages(ordered list of input-message)
```

Serialization: untagged -- a bare string is serialized as a JSON string; a
list of messages is serialized as a JSON array.

## Input-Message

```
InputMessage {
  role    : UTF-8 string   ▷ one of: "user", "assistant", "system"
  content : UTF-8 string
}
```

Factory methods exist for the three known roles; any string value is accepted
for `role`.

## Response

```
Response {
  id          : response-ID,
  object_type : literal "response",
  status      : UTF-8 string,
  created_at  : UNIX timestamp (non-negative integer),
  model       : model-identifier,
  output      : ordered list of output-item,
  usage       : token-usage,
}
```

Invariant: `usage.total_tokens = usage.input_tokens + usage.output_tokens`.

The `text()` accessor returns the concatenated text of the first output-item
of type `Message` found in the `output` list, or nothing if none exists.

### Response-ID

Begins with the prefix `"resp_"`.

## Output-Item

Discriminated union on a `"type"` tag:

```
OutputItem =
  | FunctionCall {
      name      : UTF-8 string,
      arguments : UTF-8 string,
      call_id   : UTF-8 string,
    }
  | FunctionCallOutput {
      call_id : UTF-8 string,
      output  : UTF-8 string,
    }
  | Message {
      role    : UTF-8 string,
      content : ordered list of content-part,
    }
```

Accessor methods:

- `as_text()` -- returns the text of the first `OutputText` content-part if
  this item is a `Message`, otherwise nothing.
- `as_function_call()` -- returns `(name, arguments)` if this item is a
  `FunctionCall`, otherwise nothing.

## Content-Part

Discriminated union on a `"type"` tag:

```
ContentPart =
  | OutputText { text : UTF-8 string }
```

## Token-Usage

```
TokenUsage {
  input_tokens  : non-negative integer,
  output_tokens : non-negative integer,
  total_tokens  : non-negative integer,
}
```

## Delete-Response

```
DeleteResponse {
  id          : response-ID,
  object_type : literal "response",
  deleted     : boolean,
}
```

## API Error

```
APIError {
  status     : HTTP status code (integer),
  message    : UTF-8 string,
  error_type : optional UTF-8 string,
  code       : optional UTF-8 string,
}
```

When the gateway returns a structured error body (`{ "error": { ... } }`),
`error_type` and `code` are populated from that body. When the body is not
parseable as structured error JSON, `error_type` and `code` are absent and
`message` contains the raw response body.

## Error Hierarchy

```
SDKError =
  | APIError      ▷ non-2xx HTTP response
  | NetworkError  ▷ transport-level failure (connection, DNS, TLS, etc.)
  | ConfigError   ▷ local validation failure (missing input, mutually exclusive fields)
```

## Relationships

- A create-response request MAY reference a previous response via
  `previous_response_id` or `conversation`, but not both.
- A response contains zero or more output-items; ordering is significant
  (function calls and their outputs precede the final message).
- A function-call-output references a function-call by `call_id`.
