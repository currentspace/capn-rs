# capnweb-server

Production-ready server for Cap'n Web RPC protocol with capability management.

## Overview

`capnweb-server` provides a high-performance, production-ready server implementation of the Cap'n Web RPC protocol. It supports multiple transport layers and includes built-in capability management, rate limiting, and observability.

## Features

- **Multiple transports**: HTTP batch, WebSocket, and WebTransport support
- **Capability management**: Register and manage RPC capabilities
- **Rate limiting**: Built-in rate limiting and DOS protection
- **Session management**: Automatic session tracking and cleanup
- **Observability**: Structured logging and metrics
- **Type-safe handlers**: Async trait-based RPC target implementation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
capnweb-server = "0.1.0"
```

Create a simple server:

```rust
use capnweb_server::{Server, ServerConfig, RpcTarget};
use capnweb_core::{CapId, RpcError};
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug)]
struct MyService;

#[async_trait]
impl RpcTarget for MyService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "echo" => Ok(json!({ "echoed": args })),
            _ => Err(RpcError::not_found("Method not found")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    let server = Server::new(config);
    server.register_capability(CapId::new(0), Arc::new(MyService));

    server.run().await?;
    Ok(())
}
```

## Binary

The crate includes a `capnweb-server` binary for quick testing:

```bash
cargo install capnweb-server
capnweb-server
```

## Configuration

The server can be configured via `ServerConfig`:

- `port`: Server port (default: 8080)
- `host`: Bind address (default: "127.0.0.1")
- `max_batch_size`: Maximum batch request size
- `rate_limit`: Requests per second limit
- `session_timeout`: Session inactivity timeout

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.