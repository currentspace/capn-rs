# Cap'n Web Rust Implementation

A complete, production-ready implementation of the [Cap'n Web](https://github.com/cloudflare/capnweb) protocol in Rust, providing capability-based RPC with promise pipelining and multi-transport support.

[![CI](https://github.com/currentspace/capn-rs/workflows/CI/badge.svg)](https://github.com/currentspace/capn-rs/actions/workflows/ci.yml)
[![TypeScript Interop](https://github.com/currentspace/capn-rs/workflows/TypeScript%20Interop%20Tests/badge.svg)](https://github.com/currentspace/capn-rs/actions/workflows/interop.yml)
[![Documentation](https://docs.rs/capnweb-core/badge.svg)](https://docs.rs/capnweb-core)
[![Crates.io](https://img.shields.io/crates/v/capnweb-core.svg)](https://crates.io/crates/capnweb-core)
[![License](https://img.shields.io/crates/l/capnweb-core.svg)](https://github.com/currentspace/capn-rs#license)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

## Features

✅ **Full Protocol Compliance** - Implements the complete Cap'n Web wire protocol
🔒 **Capability-Based Security** - Unforgeable capability references with automatic lifecycle management
🚀 **Promise Pipelining** - Reduced round-trips through dependency resolution
🌐 **Multi-Transport** - HTTP batch, WebSocket, and WebTransport support
🛡️ **Production-Ready** - Zero-panic code, comprehensive error handling with context
✅ **IL Expression Evaluation** - Complete intermediate language support with array notation
🌍 **JavaScript Interop** - Validated against official TypeScript implementation

## Quick Start

### Add to Cargo.toml

```toml
[dependencies]
capnweb-server = { git = "https://github.com/currentspace/capn-rs" }
capnweb-client = { git = "https://github.com/currentspace/capn-rs" }
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
    let server = Server::new(config);

    // Register capabilities
    server.register_capability(CapId::new(1), Arc::new(Calculator));

    // Run server with HTTP batch endpoint
    server.run().await?;
    Ok(())
}
```

### Client Example

```rust
use capnweb_client::{Client, ClientConfig};
use capnweb_core::CapId;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ClientConfig {
        url: "http://localhost:8080/rpc/batch".to_string(),
        ..Default::default()
    };
    let client = Client::new(config)?;

    // Make RPC calls
    let result = client.call(CapId::new(1), "add", vec![json!(10), json!(20)]).await?;
    println!("Result: {}", result);

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
use capnweb_client::{Client, ClientConfig};

let config = ClientConfig {
    url: "http://localhost:8080/rpc/batch".to_string(),
    ..Default::default()
};
let client = Client::new(config)?;
```

### WebSocket Transport
```rust
// WebSocket transport is implemented and available
// Usage requires creating a WebSocketTransport from an established WebSocket connection
use capnweb_transport::WebSocketTransport;
use tokio_tungstenite::connect_async;

let (ws_stream, _) = connect_async("ws://localhost:8080/ws").await?;
let transport = WebSocketTransport::new(ws_stream);
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
// Promise pipelining is handled internally by the protocol
// Multiple calls in a batch are automatically optimized
let batch = vec![
    Message::Call { /* ... */ },
    Message::Call { /* ... */ },
];
// The server processes these with dependency resolution
```

### Complex Data Structures
```rust
use serde_json::json;

// Build complex JSON structures for RPC calls
let request_data = json!({
    "users": [user1, user2, user3],
    "statistics": {
        "total_count": total,
        "active_count": active,
    },
    "metadata": {
        "generated_at": timestamp,
        "version": version_info,
    },
});
```

### Error Handling
```rust
match client.call(cap_id, "method", args).await {
    Ok(result) => println!("Success: {}", result),
    Err(e) => {
        // RpcError contains code, message, and optional data
        println!("Error {}: {}", e.code, e.message);
        if let Some(data) = &e.data {
            println!("Additional data: {}", data);
        }
    }
}
```

## Examples

Run the included examples to see the implementation in action:

```bash
# Run client examples
cargo run --example basic_client
cargo run --example calculator_client
cargo run --example error_handling
cargo run --example batch_pipelining

# Start the server (using bin/capnweb-server)
cargo run --bin capnweb-server
```

## Performance

Run benchmarks to measure performance:

```bash
cargo bench --bench protocol_benchmarks
```

The implementation includes optimizations for:
- Concurrent capability execution
- Efficient promise dependency resolution
- Connection pooling and reuse
- Minimal memory allocations

## Testing

Comprehensive test suite with tests across all core modules:

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p capnweb-core
cargo test -p capnweb-server
cargo test -p capnweb-client

# Run with output for debugging
cargo test -- --nocapture
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
    port: 8080,
    host: "0.0.0.0".to_string(),
    max_batch_size: 100,
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
- **Zero panics** - No `unwrap()` in production code, all errors handled explicitly
- All code must pass `cargo test` and `cargo clippy`
- Use latest crate versions unless compatibility requires older versions
- Research errors before attempting fixes
- Comprehensive documentation for all public APIs
- See [RUST_CODING_STANDARDS.md](RUST_CODING_STANDARDS.md) for complete guidelines

### Automated Code Review
This repository uses [CodeRabbit](https://coderabbit.ai) for automated PR reviews. The bot will:
- Check for compliance with our coding standards
- Suggest improvements for error handling and performance
- Verify protocol implementation correctness
- Ensure no `unwrap()` or `panic!` in production code

Configuration is in [`.coderabbit.yaml`](.coderabbit.yaml). The bot's suggestions are educational but not mandatory - maintainers make final decisions.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Documentation

- [API Documentation](https://docs.rs/capnweb-core)
- [Cap'n Web Protocol Specification](spec.md)
- [Development Guide](CLAUDE.md)
- [Coding Standards](RUST_CODING_STANDARDS.md)
- Additional docs in [`docs/`](docs/) directory

## Roadmap

- [ ] Certificate-based authentication
- [ ] Capability attestation and verification
- [ ] Message compression for large payloads
- [ ] Streaming capabilities for real-time data
- [ ] Protocol versioning and evolution
- [ ] Performance optimizations and caching

---

**Built with ❤️ in Rust. Ready for production use.**