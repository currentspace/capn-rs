# Cap'n Web Rust Implementation - Development Guide

## Project Overview

This is a Rust implementation of the Cap'n Web protocol, delivering both server and client with support for HTTP/3 (H3), WebTransport, WebSocket (H1/H2/H3), and HTTP batch transports. The implementation includes capability passing, promise pipelining, record-replay `.map()`, explicit disposal, structured error model, validation, and rate limiting.

## Repository Information

- **Remote**: https://github.com/currentspace/capn-rs.git
- **Main Branch**: main
- **Language**: Rust (edition 2021, rust-version 1.75+)

## Current Progress

### ✅ Completed Milestones

1. **Milestone 1: Core Wire Protocol & HTTP Batch Server**
   - Core types (IDs, Messages, Errors, Codec, IL)
   - Transport abstraction trait
   - HTTP batch transport
   - Basic server with batch endpoint
   - **47 tests passing**

### 🚧 Pending Milestones

2. **Milestone 2: Pipelining and Disposal** - Promise support, capability lifecycle
3. **Milestone 3: IL, Plan Runner, .map()** - Plan execution engine
4. **Milestone 4: WebSocket Support** - H1/H2/H3 WebSocket
5. **Milestone 5: WebTransport Support** - H3 WebTransport with quinn
6. **Milestone 6: Recorder Macros** - Client-side ergonomics
7. **Milestone 7: Interop Tests** - TypeScript compatibility tests

## Development Standards

### 📚 Coding Standards
**IMPORTANT**: All code must follow the standards defined in **[RUST_CODING_STANDARDS.md](./RUST_CODING_STANDARDS.md)**

Key requirements:
- **NO `unwrap()` in production code** - Use proper error handling
- **NO allocations in hot paths** - Lazy evaluate debug strings
- **NO unnecessary `clone()`** - Use references when possible
- **Document all public APIs**
- **Add error context with `anyhow`**

See RUST_CODING_STANDARDS.md for complete guidelines.
Quick reference: CODING_QUICK_REFERENCE.md for instant lookup.

### Git Workflow
```bash
# ALWAYS initialize git first
git init

# Commit after EVERY implementation step
git add -A
git commit -m "Descriptive message explaining changes"

# Push regularly
git push
```

### Testing Requirements
- **MANDATORY**: Write tests for all new code
- **MANDATORY**: All tests must pass before committing
- **Test levels**: Unit tests, integration tests, doc tests
- **Coverage goal**: Comprehensive coverage for all public APIs
- **Test execution**:
  ```bash
  # Run tests for specific crate
  cargo test -p capnweb-core

  # Run all tests
  cargo test --workspace

  # Verify before commit
  cargo test && git commit
  ```

### Dependency Management
- **ALWAYS** use latest available crate versions when adding dependencies
- **EXCEPTION**: Use older versions only to maintain compatibility with critical dependencies in spec
- **Research incompatibilities** before downgrading
- **Update process**:
  ```bash
  # Check outdated dependencies
  cargo outdated

  # Update workspace dependencies
  cargo update

  # When adding new dependency, check latest version
  cargo search <crate_name> --limit 1
  ```

### Error Handling Process
1. **Research errors** before attempting fixes
2. **Check documentation** for API changes
3. **Verify version compatibility**
4. **Test fixes** incrementally
5. **Document workarounds** if needed

### Code Organization

#### Crate Structure
```
capnweb-rs/
  capnweb-core/        # Protocol types, wire format
  capnweb-transport/   # Transport abstractions
  capnweb-server/      # Server implementation
  capnweb-client/      # Client implementation
  capnweb-interop-tests/ # Cross-language tests
```

#### Module Patterns
- Use **newtype pattern** for type-safe IDs
- Implement **Display** and **From** traits for conversions
- Use **async_trait** for async trait definitions
- Feature flags for optional components
- Separation of concerns (one module per concept)

### Coding Conventions

#### Rust Patterns
```rust
// Newtype pattern for IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CallId(u64);

// Async trait pattern
#[async_trait]
pub trait RpcTarget: Send + Sync {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
}

// Error handling with thiserror
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection closed")]
    ConnectionClosed,
}

// Feature gating
#[cfg(feature = "validation")]
pub mod validate;
```

#### Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = prepare_input();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
    }
}
```

## Critical Implementation Notes

### Known Issues
1. **h3-quinn compatibility**: Version 0.0.7 incompatible with quinn 0.11+
   - **Workaround**: WebTransport temporarily disabled in default features
   - **Fix planned**: Milestone 5 will address with compatible versions

### Frame Format Decision
- **Current**: Supporting both length-prefixed and newline-delimited
- **TODO**: Research TypeScript implementation and follow their choice

### Resume Token Implementation
- **Status**: Not yet implemented
- **TODO**: Research TypeScript implementation for compatibility

## Build & Run Commands

### Development
```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p capnweb-core

# Run example server
cargo run --example basic_server -p capnweb-server

# Run with specific features
cargo build --features webtransport
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p capnweb-server

# Run with output for debugging
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Benchmarking
```bash
# Run benchmarks (when implemented)
cargo bench -p capnweb-core
```

## API Documentation

### Server Usage
```rust
use capnweb_server::{Server, ServerConfig, RpcTarget};

// Configure server
let config = ServerConfig {
    port: 8080,
    host: "127.0.0.1".to_string(),
    max_batch_size: 100,
};

// Create and run server
let server = Server::new(config);
server.register_capability(CapId::new(1), Arc::new(MyCapability));
server.run().await?;
```

### Client Usage (Future)
```rust
use capnweb_client::{Client, ClientConfig};

// Connect to server
let client = Client::connect("http://localhost:8080/rpc/batch").await?;

// Make RPC call
let result = client.call(cap_id, "method", args).await?;
```

## TypeScript Compatibility

### Wire Format
- JSON encoding for all messages
- Message types match TypeScript: Call, Result, CapRef, Dispose
- Error codes: bad_request, not_found, cap_revoked, permission_denied, canceled, internal

### Endpoints
- HTTP Batch: `/rpc/batch`
- WebSocket: `/rpc/ws` (future)
- WebTransport: `/rpc/wt` (future)

### Testing Interop
- Golden transcripts from TypeScript implementation
- Cross-implementation test scenarios
- Protocol compliance validation

## Performance Considerations

### Optimization Targets
- Minimize allocations in hot paths
- Use Arc for shared ownership
- DashMap for concurrent capability table
- Buffer pooling for message encoding/decoding

### Benchmarking Focus
- Message encoding/decoding throughput
- Capability table lookup performance
- Batch processing latency
- Memory usage under load

## Security Considerations

### Input Validation
- Maximum batch size enforcement
- Message size limits
- Rate limiting per connection
- Schema validation for structured data

### Capability Security
- Capabilities are unforgeable references
- Explicit disposal prevents leaks
- Session isolation
- No ambient authority

## Future Enhancements

### Planned Features
1. Promise pipelining for reduced round trips
2. IL plan execution for complex operations
3. WebSocket multiplexing
4. WebTransport stream management
5. Recorder macros for ergonomic client API
6. Comprehensive TypeScript interop tests

### Research Items
1. Frame format alignment with TypeScript
2. Resume token implementation
3. Optimal buffer sizes for transports
4. Connection pooling strategies
5. Error recovery mechanisms

## Contributing Guidelines

### Before Starting
1. Check existing issues and PRs
2. Ensure tests pass: `cargo test --workspace`
3. Update dependencies: `cargo update`

### Development Cycle
1. Create feature branch
2. Implement with tests
3. Ensure all tests pass
4. Update documentation
5. Commit with descriptive message
6. Push and create PR

### Code Review Checklist
- [ ] Tests included and passing
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] Follows coding conventions
- [ ] Performance implications considered
- [ ] Security implications reviewed

## Maintenance Notes

### Regular Tasks
- Update dependencies monthly
- Review and update TypeScript compatibility
- Profile performance characteristics
- Security audit dependencies
- Update documentation

### Release Process
1. Version bump all crates together
2. Update CHANGELOG
3. Run full test suite
4. Create git tag
5. Publish to crates.io in dependency order

## Contact & Resources

### References
- Cap'n Web Repository: https://github.com/cloudflare/capnweb
- Protocol Specification: https://github.com/cloudflare/capnweb/blob/main/protocol.md
- Blog Post: https://blog.cloudflare.com/capnweb-javascript-rpc-library/

### Issue Tracking
- GitHub Issues: https://github.com/currentspace/capn-rs/issues

---

*This document should be updated as the implementation progresses and new patterns emerge.*