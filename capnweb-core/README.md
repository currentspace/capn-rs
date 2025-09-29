# capnweb-core

Core protocol implementation for Cap'n Web RPC - capability-based security with promise pipelining.

## Overview

`capnweb-core` provides the foundational types and protocol implementation for the Cap'n Web RPC system. It includes:

- Wire protocol types (Messages, Calls, Results, Errors)
- Capability ID management
- Session and connection handling
- Protocol validation and error handling
- Intermediate Language (IL) for complex operations

## Features

- **Capability-based security**: Unforgeable references with explicit lifetime management
- **Promise pipelining**: Chain operations on results before they complete
- **Type-safe IDs**: Strongly-typed identifiers for capabilities, calls, and sessions
- **Structured errors**: Rich error model with proper context propagation
- **Optional validation**: Schema validation support via feature flag

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
capnweb-core = "0.1.0"
```

Basic usage:

```rust
use capnweb_core::{CapId, Message, RpcError};

// Create a capability ID
let cap_id = CapId::new(1);

// Handle protocol messages
match message {
    Message::Call { target, method, args, .. } => {
        // Process RPC call
    }
    Message::Result { value, .. } => {
        // Handle result
    }
    Message::Error { error, .. } => {
        // Handle error
    }
}
```

## Feature Flags

- `validation` (default): Enable JSON schema validation
- `simd`: Use SIMD-accelerated JSON parsing

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.