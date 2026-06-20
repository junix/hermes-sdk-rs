# hermes-sdk-e2e

> End-to-end test binary for the `hermes-sdk` Rust client — exercises the full Responses API contract against a live Hermes gateway.

`hermes-sdk` 客户端的 e2e 验证程序。它会把 `HermesClient` 拉起来，针对一个
正在运行的 Hermes Agent API 网关跑一整套请求/响应断言，把 SDK 的
boundary 行为（成功路径、错误码、builder 校验、多轮对话）一次性打实。

测试覆盖：`health`、`create_response` 简单文本输入与结构化 messages、
`output` 中的 `function_call` / `function_call_output` / `Message` 三种
item、`get_response` 存在/不存在两种情况、`delete_response` 幂等性、
基于 `conversation` 和 `previous_response_id` 两种方式的多轮对话、
`CreateResponseRequest` builder 的输入校验、以及错误凭据下应返回 401。

## Build

```bash
cargo build --release
```

## Test / Run

```bash
# 假设 Hermes 网关跑在默认地址
cargo run --release
```

Prereq：本地或可达的 Hermes gateway，监听 `http://localhost:8642`，API
key 写死在 `src/main.rs` 的 `API_KEY` 常量里（与参考实现一致，便于回归）。
要换地址/key 直接改 `BASE_URL` / `API_KEY` 即可。

Exit code 行为：单个断言失败会让 `cargo run` 的进程非 0 退出（依赖
`assert!` 与 `unwrap`）；打印形如 `✓ <name>` / `✗ <name>: <err>` 的
分项结果，最后一行 `═══ all e2e tests passed ═══` 表示全部通过。

## Notes

这是一个 e2e 二进制，不是 `cargo test` 单元测试——它依赖一个真实的
Hermes 网关，断言的是端到端行为而非模块内部逻辑。每次 `just install`
发布 `hermes-sdk` SDK 改动后，建议跑一次以确认 wire contract 没漂。

## Workspace / parent

SDK 自身的 API、依赖与基础用法见 [`../README.md`](../README.md)。
