# 01 Concept

## Scope

This specification covers the client library (SDK) for the Hermes Agent API
gateway. The SDK provides an async, builder-pattern-based interface for
creating agent responses, managing multi-turn conversations, retrieving and
deleting stored responses, and checking gateway health. The scope is limited to
the SDK's external contract as observed from its public API surface; the
gateway's internal behavior is out of scope.

## Problem Statement

Consumers need a typed, validated, async-capable client to interact with the
Hermes gateway's OpenAI-compatible Responses API without manually constructing
HTTP requests, managing authentication headers, or parsing response bodies.

## Goals

- Provide a client that authenticates via Bearer token and communicates over
  HTTP with JSON payloads.
- Support two input modes: free-form text and structured multi-role messages.
- Support multi-turn conversations via two mechanisms: named conversations and
  response-ID chaining.
- Validate request construction locally before sending, rejecting invalid
  combinations at build time.
- Expose structured error information (HTTP status, error type, error code,
  message) on failure.
- Provide convenience accessors for common response fields (assistant text,
  function-call details, token-usage totals).

## Non-Goals

- The SDK does not manage authentication key lifecycle (rotation, refresh).
- The SDK does not implement retry, backoff, timeout, or cancellation logic.
- The SDK does not validate response payloads beyond what the JSON
  deserialization layer enforces.
- The SDK does not provide streaming or server-sent-events support.
- The SDK does not define the gateway's tool-execution semantics; it merely
  relays tool-call and tool-result output-items.

## Notational Conventions

This specification uses the following notations:

1. **RFC 2119 / BCP 14** --规范性陈述使用大写关键词
   **MUST / MUST NOT / SHOULD / SHOULD NOT / MAY**。
2. **ABNF (RFC 5234 + RFC 7405)** -- 输入语法用 ABNF 定义。
3. **Algorithm blocks** -- 关键流程用 paper-style 伪代码（见 03-runtime-model）。

Any term in *italics* referring to a noun defined in `00-glossary.md` uses the
canonical surface form from that glossary.

## Design Principles

- **Build-time validation over run-time errors**: mutually exclusive fields and
  missing required fields are rejected during request construction, not after
  the request reaches the gateway.
- **Typed output over raw JSON**: the client exposes structured records
  (response, output-item, token-usage) rather than untyped maps.
- **Bearer-token authentication**: every request to a versioned endpoint carries
  an `Authorization: Bearer <key>` header; the health endpoint also requires
  this header.

## Dependencies

| Capability | Abstract interface | Notes |
|------------|-------------------|-------|
| Async HTTP transport | HTTP/1.1 client with JSON serialization and TLS | Used for all gateway communication |
| JSON serialization | Bidirectional JSON codec (serialize request, deserialize response) | Untagged union discrimination for polymorphic fields |
| Async runtime | Task scheduler supporting async/await | Consumer's responsibility to provide |
| Structured error derivation | Error type with source-chain support | For propagating transport-level errors |
