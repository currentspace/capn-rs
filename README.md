# Cap'n Web Rust Implementation

A high-performance Rust implementation of the Cap'n Web RPC protocol, providing capability-based security, promise pipelining, and multi-transport support.

## Features

- ✅ **Core Wire Protocol** - Complete implementation of Cap'n Web message types
- ✅ **HTTP Batch Transport** - Efficient batch RPC over HTTP
- ✅ **Capability-Based Security** - Unforgeable object references
- 🚧 **Promise Pipelining** - Reduced round-trips (in progress)
- 🚧 **WebSocket Support** - Real-time bidirectional communication (planned)
- 🚧 **WebTransport** - Next-gen transport over HTTP/3 (planned)
- 🚧 **IL Plan Execution** - Complex operation batching (planned)

## Quick Start

### Running the Example Server

```bash
# Clone the repository
git clone https://github.com/currentspace/capn-rs.git
cd capn-rs

# Run the example server
cargo run --example basic_server -p capnweb-server
```

The server will start on `http://127.0.0.1:8080` with two demo capabilities:
- **Calculator** (ID: 1) - Methods: add, subtract, multiply, divide
- **EchoService** (ID: 2) - Methods: echo, reverse

### Making RPC Calls

Send a batch request to the server:

```bash
curl -X POST http://127.0.0.1:8080/rpc/batch \
  -H "Content-Type: application/json" \
  -d '[
    {"type":"call","id":1,"target":1,"member":"add","args":[5,3]},
    {"type":"call","id":2,"target":1,"member":"multiply","args":[4,7]}
  ]'
```

Response:
```json
[
  {"type":"result","id":1,"value":8},
  {"type":"result","id":2,"value":28}
]
```

## Library Usage

### Creating a Server

```rust
use async_trait::async_trait;
use capnweb_server::{Server, ServerConfig, RpcTarget};
use capnweb_core::{CapId, RpcError};
use serde_json::{json, Value};
use std::sync::Arc;

// Define a capability
struct MyService;

#[async_trait]
impl RpcTarget for MyService {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "greet" => {
                let name = args[0].as_str()
                    .ok_or_else(|| RpcError::bad_request("Name must be a string"))?;
                Ok(json!(format!("Hello, {}!", name)))
            }
            _ => Err(RpcError::not_found("Method not found"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = Server::new(config);

    // Register capabilities
    server.register_capability(CapId::new(1), Arc::new(MyService));

    // Start server
    server.run().await?;
    Ok(())
}
```

## Architecture

The implementation is organized into modular crates:

- **capnweb-core** - Core protocol types and message definitions
- **capnweb-transport** - Transport layer abstractions
- **capnweb-server** - Server implementation with capability management
- **capnweb-client** - Client implementation (in development)
- **capnweb-interop-tests** - Cross-implementation compatibility tests

## Protocol Overview

Cap'n Web is an object-capability RPC protocol that provides:

1. **Capability-Based Security**: Objects are referenced by unforgeable capability IDs
2. **Promise Pipelining**: Chain operations on future results without waiting
3. **Explicit Resource Management**: Dispose of capabilities when done
4. **Multi-Transport Support**: Works over HTTP, WebSocket, and WebTransport

### Message Types

- **Call**: Invoke a method on a capability
- **Result**: Response to a call (success or error)
- **CapRef**: Reference to a capability
- **Dispose**: Release capabilities

## Development Status

| Component | Status | Tests |
|-----------|--------|-------|
| Core Protocol | ✅ Complete | 36 passing |
| HTTP Batch | ✅ Complete | 3 passing |
| Server | ✅ Complete | 8 passing |
| Promise Pipelining | 🚧 In Progress | - |
| IL Execution | 📋 Planned | - |
| WebSocket | 📋 Planned | - |
| WebTransport | 📋 Planned | - |
| Client Library | 🚧 In Progress | - |
| TypeScript Interop | 📋 Planned | - |

## Testing

Run the test suite:

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p capnweb-core
cargo test -p capnweb-server

# Run with output
cargo test -- --nocapture
```

## Performance

The Rust implementation provides:
- Zero-copy message parsing where possible
- Concurrent capability table using DashMap
- Efficient batch processing
- Minimal allocations in hot paths

## Compatibility

This implementation aims for full compatibility with the [TypeScript reference implementation](https://github.com/cloudflare/capnweb). Interoperability tests ensure protocol compliance.

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass: `cargo test --workspace`
2. Code follows Rust idioms and conventions
3. New features include tests
4. Documentation is updated

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines.

## License

MIT OR Apache-2.0 (dual licensed)

## Acknowledgments

Based on the Cap'n Web protocol designed by Cloudflare. Special thanks to the original TypeScript implementation team.