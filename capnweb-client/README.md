# capnweb-client

High-performance Rust client for Cap'n Web RPC protocol with batching and pipelining.

## Overview

`capnweb-client` provides a feature-rich client implementation for the Cap'n Web RPC protocol. It supports automatic batching, promise pipelining, capability management, and multiple transport options.

## Features

- **Automatic batching**: Efficiently batch multiple RPC calls
- **Promise pipelining**: Chain operations without waiting for intermediate results
- **Multiple transports**: HTTP, WebSocket, and WebTransport support
- **Connection pooling**: Reuse connections for better performance
- **Retry logic**: Automatic retry with exponential backoff
- **Type-safe API**: Strongly-typed client interface

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
capnweb-client = "0.1.0"
```

Basic client usage:

```rust
use capnweb_client::{Client, ClientConfig};
use serde_json::json;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with configuration
    let config = ClientConfig {
        url: "http://localhost:8080/rpc/batch".to_string(),
        batch_size: 10,
        batch_timeout_ms: 100,
        ..Default::default()
    };

    let client = Client::new(config)?;

    // Make RPC calls
    let result = client
        .call("calculator", "add", vec![json!(5), json!(3)])
        .await?;

    println!("Result: {}", result);

    // Batch multiple calls
    let batch = client.batch();
    let future1 = batch.call("service1", "method1", vec![]);
    let future2 = batch.call("service2", "method2", vec![]);
    batch.execute().await?;

    let result1 = future1.await?;
    let result2 = future2.await?;

    Ok(())
}
```

## Advanced Features

### Promise Pipelining

```rust
// Chain operations on promises
let promise = client.call("service", "getUser", vec![json!(123)]);
let email_promise = promise.pipeline("getEmail", vec![]);
let email = email_promise.await?;
```

### Connection Management

```rust
// Use WebSocket for real-time communication
let ws_client = Client::websocket("ws://localhost:8080/rpc/ws").await?;

// Subscribe to events
ws_client.subscribe("events", |event| {
    println!("Received event: {:?}", event);
}).await?;
```

## Configuration Options

- `url`: Server endpoint URL
- `batch_size`: Maximum batch size
- `batch_timeout_ms`: Batch window timeout
- `connection_timeout_ms`: Connection timeout
- `retry_count`: Number of retries on failure
- `retry_delay_ms`: Initial retry delay

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](https://github.com/currentspace/capn-rs/blob/main/LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](https://github.com/currentspace/capn-rs/blob/main/LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.