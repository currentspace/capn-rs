# Cap’n Web Rust Implementation Spec (Absolute Mode)

## Scope

Implement Cap’n Web protocol in Rust. Deliver server and client. Support transports: HTTP/3 (H3), WebTransport, WebSocket (H1/H2/H3), HTTP batch. Implement: capability passing, promise pipelining, record–replay `.map()`, explicit disposal, error model, validation, rate limiting. Ensure compatibility with TypeScript reference implementation.

## References

- Cap’n Web repository: [https://github.com/cloudflare/capnweb](https://github.com/cloudflare/capnweb)
- Protocol specification: [https://github.com/cloudflare/capnweb/blob/main/protocol.md](https://github.com/cloudflare/capnweb/blob/main/protocol.md)
- Launch blog: [https://blog.cloudflare.com/capnweb-javascript-rpc-library/](https://blog.cloudflare.com/capnweb-javascript-rpc-library/)
- RFC 9220 (WebSocket over HTTP/3): [https://www.rfc-editor.org/rfc/rfc9220](https://www.rfc-editor.org/rfc/rfc9220)
- WebTransport API docs: [https://www.w3.org/TR/webtransport/](https://www.w3.org/TR/webtransport/)
- `axum`: [https://github.com/tokio-rs/axum](https://github.com/tokio-rs/axum)
- `hyper`: [https://github.com/hyperium/hyper](https://github.com/hyperium/hyper)
- `quinn`: [https://github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn)
- `h3`: [https://github.com/hyperium/h3](https://github.com/hyperium/h3)
- `tokio-tungstenite`: [https://github.com/snapview/tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- `governor` (rate limiting): [https://github.com/antifuchs/governor](https://github.com/antifuchs/governor)
- `jsonschema`: [https://github.com/Stranger6667/jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs)
- `schemars`: [https://github.com/GREsau/schemars](https://github.com/GREsau/schemars)

## Stack Decisions

- **HTTP/1.1 and HTTP/2**: use **axum** on top of **hyper**.
- **WebSocket (H1/H2)**: use **tokio-tungstenite** integrated into axum routes.
- **HTTP/3, WebTransport, WebSocket-over-H3**: use **quinn + h3 (h3-quinn)**.
- **TLS**: use **rustls** (integrates with quinn, pure-Rust, no OpenSSL dependency).
- **Rate limiting**: use **governor**.
- **JSON codec**: use **serde_json**, optional `simd-json`.
- **Concurrent data structures**: `dashmap`, `indexmap`.
- **Error handling**: `thiserror`, `anyhow`.
- **Observability**: `tracing`, `tracing-subscriber`.
- WebTransport implementation: `web-transport-quinn` (native QUIC binding)

## Crate Layout

```
capnweb-rs/
  capnweb-core/
    src/{ids.rs,msg.rs,codec.rs,error.rs,il.rs,validate.rs}
  capnweb-transport/
    src/{transport.rs,webtransport.rs,websocket.rs,http_batch.rs,negotiate.rs}
  capnweb-server/
    src/{server.rs,ws_h1.rs,h3_server.rs,cap_table.rs,runner.rs,limits.rs}
  capnweb-client/
    src/{client.rs,recorder.rs,stubs.rs,macros/*}
  capnweb-interop-tests/
    src/{fixtures.rs,js_client_tests.rs,js_server_tests.rs}
```

## capnweb-core

- **IDs**: CallId, PromiseId, CapId. Allocate monotonically. Do not reuse within session.
- **Messages**:

  - Call {id, target, member, args}
  - Result {id, value | error}
  - CapRef {id}
  - Dispose {caps\[]}

- **Codec**: JSON via `serde_json`. Optional `simd-json` behind feature flag.
- **Error envelope**: {code, message, data?}. Codes: `bad_request`, `not_found`, `cap_revoked`, `permission_denied`, `canceled`, `internal`.
- **IL (Plan)**:

  - Source {Capture(u32), Result(u32), Param(Vec<String>), ByValue(Value)}
  - Op {Call{target, member, args, result}, Object{fields, result}, Array{items, result}}
  - Plan {captures, ops, result}

- **Validation**: `jsonschema` or `schemars`. Optional.

## capnweb-transport

- **Trait RpcTransport**: connect, send, recv, close.
- **Events**: Frame(Bytes), Closed(Option<Error>), Heartbeat.
- **Adapters**:

  - WebTransport: one control bidi stream. Extra streams reserved. Datagrams optional. Use `quinn+h3`.
  - WebSocket: WS over H1/H2 via `tokio-tungstenite`. WS over H3 via Extended CONNECT RFC 9220.
  - HTTP batch: POST request. Wrap to look like transport.

- **Negotiation**: attempt WT → WS → HTTP batch.

## capnweb-server

- **Runtime**: axum + hyper for H1/H2. Quinn + h3 for H3.
- **RpcTarget trait**:

  ```rust
  async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
  ```

- **CapTable**: DashMap\<CapId, Arc<dyn RpcTarget>>. Ownership tracked.
- **HTTP batch**: `/rpc/batch`. Execute pipeline. Implicit dispose at batch end.
- **WS session**: multiplex calls, explicit dispose, heartbeats.
- **WT session**: control stream, explicit dispose, optional resume token.
- **Plan Runner**: evaluate IL. Resolve captures. Execute ops topologically. Return result.
- **Limits**: enforce max ops per plan, max inflight calls, max message size. Use `governor`.
- **Tracing**: attach spans per call and plan. Include IDs.

## capnweb-client

- **Transport**: negotiate best available.
- **Promise table**: CallId → oneshot sender.
- **Capability stubs**: Capability<T>. Remote CapId. Drop sends Dispose.
- **Recorder**: build Plan from placeholder calls.
- **Macros**: `#[interface]` generates stub + placeholder. `record_map!` macro generates Plan IL.
- **Pipelining**: allow multiple chained ops to be encoded into one Plan.

## capnweb-interop-tests

- **Golden transcripts**: fixture JSON from TS impl.
- **Cross-interop**:

  - JS client → Rust server
  - Rust client → JS server

- **Coverage**: basic calls, cap returns, pipelined calls, `.map()`, disposal, injected errors.

## Milestones

1. Core wire: IDs, messages, error envelope, codec. Batch server.
2. Pipelining and disposal.
3. IL, Plan runner, `.map()`.
4. WS (H1/H2, H3).
5. WebTransport (H3).
6. Recorder macros.
7. Interop tests.

## Constraints

- No conditional logic in `.map()` on placeholders. Detect and reject.
- No side effects in record closures. Detect and reject.
- On record error: abort plan. Do not send partial.
- Stubs invalid after session close unless resume token used.
- Enforce budgets: max ops, max inflight, max frame size.
- Structured error responses only.
- No conversational or engagement features.
- No tone adaptation.

## Remaining Decisions

- Frame format: length-prefixed JSON vs JSON-seq. research what the typescript implemention does and follow that.
- Resume token implementation: optional or mandatory for WT reconnect. research what the typescript implemention does and follow that.
