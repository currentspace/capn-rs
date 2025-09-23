# Cap'n Web Rust Implementation

A complete, production-ready implementation of the [Cap'n Web](https://capnproto.org/capnweb) protocol in Rust, providing capability-based RPC with promise pipelining and multi-transport support.

[![Rust](https://github.com/currentspace/capn-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/currentspace/capn-rs/actions/workflows/rust.yml)
[![Documentation](https://docs.rs/capnweb-core/badge.svg)](https://docs.rs/capnweb-core)
[![Crates.io](https://img.shields.io/crates/v/capnweb-core.svg)](https://crates.io/crates/capnweb-core)

## Features

🔒 **Capability-Based Security** - Unforgeable capability references with automatic lifecycle management
🚀 **Promise Pipelining** - Reduced round-trips through dependency resolution
🌐 **Multi-Transport** - HTTP batch, WebSocket, and WebTransport/HTTP3 support
🛡️ **Production-Ready** - Comprehensive error handling, rate limiting, and observability
🔧 **Ergonomic API** - Fluent plan construction with procedural macros
🌍 **JavaScript Interop** - Full compatibility with JavaScript implementations

## Quick Start

### Add to Cargo.toml

```toml
[dependencies]
capnweb-server = "0.1.0"
capnweb-client = "0.1.0"
```

### Server Example

```rust
use capnweb_server::{Server, ServerConfig, RpcTarget};
use capnweb_core::{CapId, RpcError};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug)]
struct Calculator;

impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "add" => {
                let a = args[0].as_f64().unwrap();
                let b = args[1].as_f64().unwrap();
                Ok(json!(a + b))
            }
            _ => Err(RpcError::not_found("method not found")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = Arc::new(Server::new(config));

    // Register capabilities
    server.register_capability(CapId::new(1), Arc::new(Calculator))?;

    // Start server with WebSocket and HTTP endpoints
    server.start().await?;
    Ok(())
}
```

### Client Example

```rust
use capnweb_client::{Client, Recorder, params, record_object};
use capnweb_transport::WebSocketTransport;
use capnweb_core::CapId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect via WebSocket
    let transport = WebSocketTransport::connect("ws://localhost:8080/ws").await?;
    let client = Client::new(transport, Default::default());

    // Build a plan using the ergonomic recorder API
    let recorder = Recorder::new();
    let calc = recorder.capture("calculator", CapId::new(1));

    let sum = calc.call("add", params![15.5, 24.3]);
    let product = calc.call("multiply", params![7, 8]);

    let result = record_object!(recorder; {
        "sum" => sum,
        "product" => product,
    });

    let plan = recorder.build(result.as_source());

    // Execute with promise pipelining
    let response = client.execute_plan(&plan, None).await?;
    println!("Results: {}", response);

    Ok(())
}
```

## Architecture

The implementation is organized into focused crates:

- **`capnweb-core`** - Core protocol implementation (messages, IL, validation)
- **`capnweb-transport`** - Transport layer implementations (HTTP, WebSocket, WebTransport)
- **`capnweb-server`** - Server implementation with capability management
- **`capnweb-client`** - Client implementation with ergonomic recorder API
- **`capnweb-interop-tests`** - JavaScript interoperability verification

## Transport Support

### HTTP Batch Transport
```rust
use capnweb_transport::HttpBatchTransport;

let transport = HttpBatchTransport::new("http://localhost:8080/batch".to_string());
let client = Client::new(transport, Default::default());
```

### WebSocket Transport
```rust
use capnweb_transport::WebSocketTransport;

let transport = WebSocketTransport::connect("ws://localhost:8080/ws").await?;
let client = Client::new(transport, Default::default());
```

### WebTransport/HTTP3
```rust
use capnweb_server::H3Server;

let mut h3_server = H3Server::new(server);
h3_server.listen("0.0.0.0:8443".parse()?).await?;
```

## Advanced Features

### Promise Pipelining
```rust
let recorder = Recorder::new();
let api = recorder.capture("api", CapId::new(1));

// These calls are automatically pipelined
let user = api.call("getUser", params![123]);
let profile = user.call("getProfile", vec![]);  // Depends on user result
let preferences = profile.call("getPreferences", vec![]);  // Depends on profile

let plan = recorder.build(preferences.as_source());
```

### Complex Data Structures
```rust
use capnweb_client::{record_object, record_array};

let summary = record_object!(recorder; {
    "users" => record_array!(recorder; [user1, user2, user3]),
    "statistics" => record_object!(recorder; {
        "total_count" => total,
        "active_count" => active,
    }),
    "metadata" => record_object!(recorder; {
        "generated_at" => timestamp,
        "version" => version_info,
    }),
});
```

### Error Handling
```rust
match client.execute_plan(&plan, None).await {
    Ok(result) => println!("Success: {}", result),
    Err(RpcError::Network(e)) => println!("Network error: {}", e),
    Err(RpcError::Protocol(e)) => println!("Protocol error: {}", e),
    Err(RpcError::User { code, message, .. }) => {
        println!("Application error {}: {}", code, message);
    }
}
```

## Examples

Run the included examples to see the implementation in action:

```bash
# Start the calculator server
cargo run --example calculator_server

# In another terminal, run the client
cargo run --example calculator_client

# WebTransport/HTTP3 server
cargo run --example webtransport_server
```

## Performance

Run benchmarks to measure performance:

```bash
cargo bench
```

The implementation includes optimizations for:
- Concurrent capability execution
- Efficient promise dependency resolution
- Connection pooling and reuse
- Minimal memory allocations

## Testing

Comprehensive test suite with 80+ tests:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test integration_tests
cargo test --package capnweb-core
cargo test --package capnweb-interop-tests
```

## JavaScript Interoperability

The Rust implementation is fully compatible with JavaScript Cap'n Web implementations:

- ✅ Identical message formats and serialization
- ✅ Compatible IL plan structures
- ✅ Matching error handling patterns
- ✅ Shared protocol semantics

Test interoperability:
```bash
cargo test --package capnweb-interop-tests
```

## Production Deployment

### Configuration
```rust
use capnweb_server::ServerConfig;

let config = ServerConfig {
    http_bind_addr: "0.0.0.0:8080".to_string(),
    max_connections: 10000,
    rate_limit_requests_per_second: Some(1000),
    enable_cors: true,
    request_timeout_ms: 30000,
    ..Default::default()
};
```

### Monitoring
```rust
use tracing::{info, warn, error};
use tracing_subscriber;

// Enable structured logging
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .json()
    .init();
```

### Security
- Use proper TLS certificates for WebTransport
- Implement authentication for capability access
- Configure appropriate rate limiting
- Enable audit logging for capability calls

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Run benchmarks: `cargo bench`
6. Submit a pull request

### Development Standards
- All code must pass `cargo test` and `cargo clippy`
- Use latest crate versions unless compatibility requires older versions
- Research errors before attempting fixes
- Comprehensive documentation for all public APIs

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Documentation

- [API Documentation](https://docs.rs/capnweb-core)
- [Cap'n Web Specification](https://capnproto.org/capnweb)
- [Project Summary](PROJECT_SUMMARY.md)
- [Development Guide](CLAUDE.md)

## Roadmap

- [ ] Certificate-based authentication
- [ ] Capability attestation and verification
- [ ] Message compression for large payloads
- [ ] Streaming capabilities for real-time data
- [ ] Protocol versioning and evolution
- [ ] Performance optimizations and caching

---

**Built with ❤️ in Rust. Ready for production use.**