# Rust Coding Standards for Cap'n Web

## Version 1.0 - 2025 Best Practices

This document defines the coding standards for the Cap'n Web Rust implementation, incorporating modern Rust best practices and lessons learned from code analysis.

## Table of Contents
1. [Core Principles](#core-principles)
2. [Error Handling](#error-handling)
3. [Performance Guidelines](#performance-guidelines)
4. [Memory Management](#memory-management)
5. [Type Safety](#type-safety)
6. [Concurrency](#concurrency)
7. [API Design](#api-design)
8. [Testing](#testing)
9. [Documentation](#documentation)
10. [Project Structure](#project-structure)
11. [Tooling & Lints](#tooling--lints)

## Core Principles

### 1. Zero-Panic Production Code
**Production code must NEVER panic.** Every potential failure must be handled gracefully.

```rust
// ❌ BAD - Can panic in production
let value = some_option.unwrap();
let number = Number::from_f64(n).unwrap();

// ✅ GOOD - Proper error handling
let value = some_option.ok_or(Error::MissingValue)?;
let number = Number::from_f64(n)
    .ok_or_else(|| Error::InvalidNumber(n))?;
```

### 2. Explicit Over Implicit
Make intentions clear through types and naming.

```rust
// ❌ BAD - Unclear what the String represents
fn process(id: String) -> String

// ✅ GOOD - Self-documenting types
type SessionId = String;
type CapabilityToken = String;
fn process(id: SessionId) -> CapabilityToken
```

### 3. Performance by Default
Write efficient code from the start, don't rely on future optimization.

## Error Handling

### Never Use `unwrap()` in Production Code

```rust
// ❌ NEVER do this in non-test code
let config = fs::read_to_string("config.json").unwrap();

// ✅ GOOD - Return Result
let config = fs::read_to_string("config.json")
    .context("Failed to read config.json")?;

// ✅ GOOD - Provide default for Options
let port = config.port.unwrap_or(8080);

// ✅ GOOD - Use expect() only with impossible-to-fail invariants
let mutex = STATIC_MUTEX.lock()
    .expect("mutex poisoned - this is a bug");
```

### Error Context Pattern

Always add context to errors for debugging:

```rust
use anyhow::{Context, Result};

// ✅ GOOD - Rich error context
fn load_capability(id: CapId) -> Result<Capability> {
    db.get(id)
        .with_context(|| format!("Failed to load capability {}", id))?
        .parse()
        .with_context(|| format!("Invalid capability format for {}", id))
}
```

### Custom Error Types

Define domain-specific errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    #[error("Capability {id} not found")]
    NotFound { id: CapId },

    #[error("Permission denied for capability {id}")]
    PermissionDenied { id: CapId },

    #[error("Capability {id} has been revoked")]
    Revoked { id: CapId, revoked_at: Instant },
}
```

## Performance Guidelines

### Avoid Allocations in Hot Paths

```rust
// ❌ BAD - Allocates on every request
fn log_request(req: &Request) {
    let preview = req.body.chars().take(100).collect::<String>();
    debug!("Request preview: {}", preview);
}

// ✅ GOOD - Only allocate when actually logging
fn log_request(req: &Request) {
    // The closure is only evaluated if debug logging is enabled
    debug!("Request preview: {}",
        req.body.chars().take(100).collect::<String>());
}

// ✅ BETTER - No allocation at all
fn log_request(req: &Request) {
    if log::log_enabled!(log::Level::Debug) {
        let preview: String = req.body.chars().take(100).collect();
        debug!("Request preview: {}", preview);
    }
}
```

### String Operations

```rust
// ❌ BAD - Unnecessary allocations
fn get_message(code: u32) -> String {
    format!("Error code: {}", code)  // Allocates
}

// ✅ GOOD - Use static strings when possible
fn get_message(code: u32) -> Cow<'static, str> {
    match code {
        404 => Cow::Borrowed("Not found"),
        500 => Cow::Borrowed("Internal error"),
        _ => Cow::Owned(format!("Error code: {}", code)),
    }
}

// ✅ GOOD - Implement Display instead of to_string()
impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {}", self.0)
    }
}
```

### Use Capacity Hints

```rust
// ❌ BAD - Vector reallocates as it grows
let mut results = Vec::new();
for item in items {
    results.push(process(item));
}

// ✅ GOOD - Pre-allocate capacity
let mut results = Vec::with_capacity(items.len());
for item in items {
    results.push(process(item));
}

// ✅ BETTER - Use iterator when possible
let results: Vec<_> = items.iter()
    .map(|item| process(item))
    .collect();
```

## Memory Management

### Clone vs Reference

```rust
// ❌ BAD - Unnecessary clone
fn process(data: Data) {
    let copy = data.clone();
    read_only_operation(&copy);
}

// ✅ GOOD - Use reference
fn process(data: &Data) {
    read_only_operation(data);
}

// ✅ GOOD - Clone only when necessary
fn process(data: &Data) -> ProcessedData {
    ProcessedData {
        // Only clone the field we need to own
        name: data.name.clone(),
        // Reference for read-only access
        metadata: compute_metadata(data),
    }
}
```

### Smart Pointer Usage

```rust
// ✅ GOOD - Use Arc for shared ownership
type SharedCapability = Arc<Capability>;

// ✅ GOOD - Use Arc<RwLock> for shared mutable state
type SessionStore = Arc<RwLock<HashMap<SessionId, Session>>>;

// ✅ GOOD - Be explicit about Arc cloning
let shared_ref = Arc::clone(&original);  // Not original.clone()
```

### Avoid Collect When Possible

```rust
// ❌ BAD - Collects into Vec unnecessarily
let names: Vec<String> = items.iter()
    .map(|i| i.name.clone())
    .collect();
for name in names {
    println!("{}", name);
}

// ✅ GOOD - Use iterator directly
for name in items.iter().map(|i| &i.name) {
    println!("{}", name);
}
```

## Type Safety

### Use NewType Pattern for Domain Types

```rust
// ❌ BAD - Easy to mix up parameters
fn transfer(from: u64, to: u64, amount: u64);

// ✅ GOOD - Type-safe domain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount(u64);

fn transfer(from: AccountId, to: AccountId, amount: Amount);
```

### Avoid Unsafe Type Casts

```rust
// ❌ BAD - Lossy and unsafe
let size = (bytes.len() as u32);

// ✅ GOOD - Safe conversion with error handling
let size = u32::try_from(bytes.len())
    .map_err(|_| Error::PayloadTooLarge)?;

// ✅ GOOD - Use saturating operations when appropriate
let clamped_size = bytes.len().min(u32::MAX as usize) as u32;
```

### Use Enums for State Machines

```rust
// ✅ GOOD - Type-safe state transitions
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: SessionId },
    Disconnecting,
}

impl ConnectionState {
    fn connect(self) -> Result<Self> {
        match self {
            Self::Disconnected => Ok(Self::Connecting { attempt: 1 }),
            _ => Err(Error::InvalidStateTransition),
        }
    }
}
```

## Concurrency

### Prefer Message Passing

```rust
// ✅ GOOD - Use channels for communication
use tokio::sync::mpsc;

async fn worker(mut rx: mpsc::Receiver<Task>) {
    while let Some(task) = rx.recv().await {
        process_task(task).await;
    }
}
```

### Lock Ordering and Duration

```rust
// ❌ BAD - Holding lock during async operation
let data = shared_state.write().await;
let result = expensive_async_operation(&data).await; // Lock held!

// ✅ GOOD - Minimize lock scope
let data_copy = {
    let data = shared_state.read().await;
    data.clone() // Clone only what we need
}; // Lock released here
let result = expensive_async_operation(&data_copy).await;
```

### Avoid Deadlocks

```rust
// ✅ GOOD - Always acquire locks in the same order
// Document lock ordering in module docs
/// Lock ordering: config -> sessions -> capabilities
async fn update_all(
    config: &RwLock<Config>,
    sessions: &RwLock<Sessions>,
    capabilities: &RwLock<Capabilities>,
) {
    let _config = config.write().await;
    let _sessions = sessions.write().await;
    let _capabilities = capabilities.write().await;
    // ... update all
}
```

## API Design

### Builder Pattern for Complex Types

```rust
// ✅ GOOD - Flexible initialization
pub struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    // ... many fields
}

impl ServerConfig {
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<usize>,
}

impl ServerConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn build(self) -> Result<ServerConfig> {
        Ok(ServerConfig {
            host: self.host.ok_or(Error::MissingHost)?,
            port: self.port.unwrap_or(8080),
            max_connections: self.max_connections.unwrap_or(1000),
        })
    }
}
```

### Return Types

```rust
// ✅ GOOD - Consistent Result types
pub type Result<T> = std::result::Result<T, Error>;

// ✅ GOOD - Use Option for nullable returns
fn find_capability(id: CapId) -> Option<Capability>;

// ✅ GOOD - Use Result for fallible operations
fn create_capability(config: Config) -> Result<Capability>;

// ✅ GOOD - Use impl Trait for iterators
fn list_capabilities(&self) -> impl Iterator<Item = &Capability>;
```

## Testing

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests in same file as implementation
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = TestData::default();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value());
    }

    // Property-based tests
    #[quickcheck]
    fn prop_serialization_roundtrip(data: Data) -> bool {
        let serialized = serialize(&data);
        let deserialized = deserialize(&serialized).unwrap();
        data == deserialized
    }
}

// Integration tests in tests/ directory
// Benchmarks in benches/ directory
```

### Test Utilities

```rust
// ✅ GOOD - Test helpers in test modules
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_test_capability() -> Capability {
        Capability::builder()
            .id(CapId::new(1))
            .name("test")
            .build()
            .expect("valid test capability")
    }
}
```

## Documentation

### Module Documentation

```rust
//! # Capability Management
//!
//! This module provides the core capability management functionality
//! for the Cap'n Web protocol.
//!
//! ## Lock Ordering
//!
//! To prevent deadlocks, always acquire locks in this order:
//! 1. Configuration
//! 2. Sessions
//! 3. Capabilities
//!
//! ## Example
//!
//! ```rust
//! use capnweb_core::capability::Capability;
//!
//! let cap = Capability::new(config)?;
//! ```
```

### Function Documentation

```rust
/// Creates a new capability with the given configuration.
///
/// # Arguments
///
/// * `config` - The capability configuration
///
/// # Returns
///
/// Returns `Ok(Capability)` on success, or an error if:
/// - The configuration is invalid
/// - The system is at capacity
/// - The requestor lacks permission
///
/// # Example
///
/// ```rust
/// let cap = create_capability(config)?;
/// ```
pub fn create_capability(config: Config) -> Result<Capability> {
    // ...
}
```

### Invariant Documentation

```rust
/// Represents a capability ID.
///
/// # Invariants
///
/// - IDs are unique within a session
/// - IDs are never reused
/// - ID 0 is reserved for the root capability
#[derive(Debug, Clone, Copy)]
pub struct CapId(u64);

impl CapId {
    /// Creates a new capability ID.
    ///
    /// # Panics
    ///
    /// Panics if `id` is 0 (reserved value).
    pub fn new(id: u64) -> Self {
        assert!(id != 0, "CapId 0 is reserved");
        Self(id)
    }
}
```

## Project Structure

### Module Organization

```
capnweb-core/
├── src/
│   ├── lib.rs           # Public API exports
│   ├── error.rs         # Error types
│   ├── types.rs         # Common types
│   ├── protocol/        # Protocol implementation
│   │   ├── mod.rs       # Module exports
│   │   ├── codec.rs     # Wire format
│   │   └── session.rs   # Session management
│   └── tests/           # Unit tests
├── tests/               # Integration tests
├── benches/            # Benchmarks
└── examples/           # Usage examples
```

### Dependency Management

```toml
# Cargo.toml
[dependencies]
# Use exact versions for critical dependencies
serde = "=1.0.193"
tokio = { version = "1.35", features = ["full"] }

# Use caret requirements for non-critical deps
anyhow = "^1.0"
thiserror = "^1.0"

[dev-dependencies]
# Test dependencies
quickcheck = "1.0"
proptest = "1.4"
criterion = "0.5"
```

## Tooling & Lints

### Required Clippy Lints

Add to `Cargo.toml`:

```toml
[workspace.lints.clippy]
# Correctness
unwrap_used = "deny"           # No unwrap in production
expect_used = "warn"            # Require message for expect
panic = "deny"                  # No explicit panics
todo = "warn"                   # Track TODOs
unimplemented = "warn"          # Track unimplemented

# Performance
redundant_clone = "warn"        # Avoid unnecessary clones
inefficient_to_string = "warn"  # Use Display instead
unnecessary_to_owned = "warn"   # Avoid unnecessary ownership
large_enum_variant = "warn"     # Box large variants
large_stack_arrays = "warn"     # Avoid stack overflow

# Style
missing_docs = "warn"           # Document public API
missing_debug_implementations = "warn"
missing_copy_implementations = "warn"

# Async
async_yields_async = "deny"     # Avoid async fn returning Future
future_not_send = "warn"        # Futures should be Send

[workspace.lints.rust]
unsafe_code = "deny"            # No unsafe without review
missing_docs = "warn"            # Document everything public
```

### Pre-commit Checks

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

# Format check
cargo fmt -- --check

# Clippy with all targets
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Doc tests
cargo test --doc

# Security audit
cargo audit

echo "✅ All pre-commit checks passed!"
```

### Continuous Integration

```yaml
# .github/workflows/ci.yml
- name: Check
  run: |
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features
    cargo doc --no-deps --all-features
```

## Performance Benchmarking

### Benchmark Organization

```rust
// benches/capability_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_capability_creation(c: &mut Criterion) {
    c.bench_function("create_capability", |b| {
        b.iter(|| {
            let cap = create_capability(black_box(test_config()));
            black_box(cap);
        });
    });
}

criterion_group!(benches, benchmark_capability_creation);
criterion_main!(benches);
```

### Performance Tracking

- Run benchmarks before and after changes
- Track performance in CI
- Set performance budgets for critical paths

## Security Considerations

### Input Validation

```rust
// ✅ GOOD - Validate and limit inputs
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

fn parse_message(data: &[u8]) -> Result<Message> {
    if data.len() > MAX_MESSAGE_SIZE {
        return Err(Error::MessageTooLarge);
    }

    // Additional validation...
    serde_json::from_slice(data)
        .map_err(|e| Error::InvalidMessage(e.to_string()))
}
```

### Resource Limits

```rust
// ✅ GOOD - Prevent resource exhaustion
pub struct RateLimiter {
    max_requests_per_second: u32,
    max_concurrent_operations: usize,
}

impl RateLimiter {
    pub async fn check_limit(&self, client_id: ClientId) -> Result<()> {
        // Implementation
    }
}
```

## Migration Guide

When updating existing code to meet these standards:

1. **Phase 1**: Eliminate all `unwrap()` calls
2. **Phase 2**: Add error context with `anyhow`
3. **Phase 3**: Replace performance anti-patterns
4. **Phase 4**: Add comprehensive documentation
5. **Phase 5**: Add property-based tests

## Version History

- **1.0** (2025-01-24): Initial version based on code analysis and modern Rust best practices

---

*This document is a living standard and should be updated as the project evolves and new patterns emerge.*