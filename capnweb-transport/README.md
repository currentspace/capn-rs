# capnweb-transport

Transport layer implementations for Cap'n Web protocol (HTTP, WebSocket, WebTransport).

## Overview

`capnweb-transport` provides multiple transport implementations for the Cap'n Web RPC protocol:

- **HTTP Batch**: Traditional request/response with batching support
- **WebSocket**: Full-duplex streaming with multiplexing
- **WebTransport**: Modern HTTP/3-based transport with stream multiplexing

## Features

- Pluggable transport abstraction
- Automatic reconnection and error recovery
- Message framing and buffering
- Backpressure handling
- Connection pooling (HTTP)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
capnweb-transport = "0.1.0"
```

Using different transports:

```rust
use capnweb_transport::{Transport, HttpBatchTransport, WebSocketTransport};
use capnweb_core::Message;

// HTTP Batch transport
let http_transport = HttpBatchTransport::new("http://localhost:8080/rpc/batch");

// WebSocket transport
let ws_transport = WebSocketTransport::connect("ws://localhost:8080/rpc/ws").await?;

// Send messages
transport.send(message).await?;
let response = transport.receive().await?;
```

## Feature Flags

- `websocket` (default): WebSocket transport support
- `http-batch` (default): HTTP batch transport support
- `webtransport`: WebTransport/HTTP3 support (experimental)

## Transport Selection

Choose your transport based on your needs:

- **HTTP Batch**: Simple, firewall-friendly, works everywhere
- **WebSocket**: Real-time updates, lower latency, persistent connection
- **WebTransport**: Best performance, multiplexing, requires HTTP/3

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.