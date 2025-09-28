# Rust Coding Standards for Cap'n Web

## Version 2.0 - Modern Rust 1.85+ Best Practices

This document defines the comprehensive coding standards for the Cap'n Web Rust implementation, incorporating modern Rust 1.85+ best practices including async traits, const evaluation, zero-copy patterns, and advanced type system features. These standards represent the state-of-the-art in Rust development as of 2025.

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
11. [Modern Async Patterns](#modern-async-patterns)
12. [Zero-Copy Operations](#zero-copy-operations)
13. [Compile-Time Optimization](#compile-time-optimization)
14. [Advanced Type Patterns](#advanced-type-patterns)
15. [Observability](#observability)
16. [Feature Flags](#feature-flags)
17. [Macro Patterns](#macro-patterns)
18. [Common Anti-Patterns to Avoid](#common-anti-patterns-to-avoid)
19. [Tooling & Lints](#tooling--lints)
20. [Performance Benchmarking](#performance-benchmarking)
21. [Security Considerations](#security-considerations)
22. [Migration Guide](#migration-guide)

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

// ✅ GOOD - Use let-else for cleaner error handling (stabilized 1.65)
let Some(user) = find_user(id) else {
    tracing::warn!("User {} not found", id);
    return Err(Error::NotFound);
};
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

// ✅ GOOD - Custom Try trait for validation (experimental, requires nightly)
#[cfg(feature = "try_trait")]
use std::ops::Try;

pub enum Validated<T> {
    Valid(T),
    Invalid(ValidationError),
}

#[cfg(feature = "try_trait")]
impl<T> Try for Validated<T> {
    type Output = T;
    type Residual = ValidationError;

    fn from_output(output: Self::Output) -> Self {
        Validated::Valid(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Validated::Valid(v) => ControlFlow::Continue(v),
            Validated::Invalid(e) => ControlFlow::Break(e),
        }
    }
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

### Modern Testing Patterns

```rust
// ✅ GOOD - Snapshot testing for complex outputs
#[test]
fn test_render_output() {
    let output = render_complex_structure();
    insta::assert_snapshot!(output);
}

// ✅ GOOD - Parameterized tests with rstest
use rstest::rstest;

#[rstest]
#[case(0, false)]
#[case(1, true)]
#[case(100, true)]
fn test_is_valid(#[case] input: u32, #[case] expected: bool) {
    assert_eq!(is_valid(input), expected);
}

// ✅ GOOD - Test fixtures
#[fixture]
fn test_db() -> TestDatabase {
    TestDatabase::new()
}

#[rstest]
fn test_with_db(test_db: TestDatabase) {
    // Use the test database
}
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

## Modern Async Patterns

### Structured Concurrency

```rust
// ✅ GOOD - Use async drop guards for cleanup
use tokio_util::sync::CancellationToken;

pub struct Server {
    cancellation: CancellationToken,
}

impl Server {
    pub async fn graceful_shutdown(self) {
        self.cancellation.cancel();
        // Await all spawned tasks
        tokio::time::timeout(
            Duration::from_secs(30),
            self.await_termination()
        ).await.ok();
    }
}

// ✅ GOOD - Structured task spawning with tracing
pub fn spawn_scoped<F>(task: F) -> JoinHandle<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let span = tracing::Span::current();
    tokio::spawn(
        task.instrument(span)
            .catch_unwind()  // Prevent task panics from propagating
    )
}
```

### Async Traits (Stabilized 1.75)

```rust
// ✅ GOOD - Use async traits directly
pub trait AsyncProcessor: Send + Sync {
    async fn process(&self, input: Input) -> Result<Output>;
}

// ✅ GOOD - Generic Associated Types (GATs) - stabilized 1.65
pub trait Database {
    type Transaction<'a>: Transaction where Self: 'a;

    async fn begin_transaction(&mut self) -> Result<Self::Transaction<'_>>;
}
```

### Pin and Async Safety

```rust
// ✅ GOOD - Document pin requirements
use std::pin::Pin;

/// This future captures self-referential data and must be pinned.
pub struct Connection {
    _pin: PhantomPinned,
}

impl Connection {
    pub fn poll_read(self: Pin<&mut Self>) -> Poll<Result<Bytes>> {
        // Safe because we're pinned
        // ...
    }
}
```

## Zero-Copy Operations

### Use Bytes for Efficient Memory Management

```rust
// ✅ GOOD - Use bytes for zero-copy operations
use bytes::{Bytes, BytesMut};

pub struct Message {
    // Cheap to clone, reference-counted
    payload: Bytes,
}

impl Message {
    pub fn parse(mut buffer: BytesMut) -> Result<Self> {
        // Zero-copy: split buffer without allocation
        let payload = buffer.split_to(header.len).freeze();
        Ok(Message { payload })
    }
}

// ✅ GOOD - Use zerocopy for parsing
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[repr(C)]
#[derive(AsBytes, FromBytes, FromZeroes)]
pub struct Header {
    version: u32,
    flags: u32,
    length: u32,
}
```

### Efficient String Formatting

```rust
// ✅ GOOD - Reuse buffer for formatting
use std::fmt::Write;

thread_local! {
    static FMT_BUFFER: RefCell<String> = RefCell::new(String::with_capacity(256));
}

// ✅ GOOD - Use format_args! for zero-allocation formatting
fn log_message(level: Level, args: fmt::Arguments<'_>) {
    // No allocation if not actually logging
    if level >= current_level() {
        println!("{}: {}", level, args);
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        log_message(Level::Info, format_args!($($arg)*))
    };
}
```

## Compile-Time Optimization

### Const Context Evaluation

```rust
// ✅ GOOD - Leverage const evaluation for compile-time validation
pub const fn validate_at_compile_time(value: u32) -> u32 {
    assert!(value > 0, "Value must be positive");
    value
}

// ✅ GOOD - Use const generics for compile-time guarantees
pub struct BoundedVec<T, const MAX_SIZE: usize> {
    inner: Vec<T>,
}

impl<T, const MAX_SIZE: usize> BoundedVec<T, MAX_SIZE> {
    pub fn push(&mut self, item: T) -> Result<()> {
        if self.inner.len() >= MAX_SIZE {
            return Err(Error::CapacityExceeded);
        }
        self.inner.push(item);
        Ok(())
    }
}
```

### NonZero Types for Space Optimization

```rust
// ✅ GOOD - Use NonZero types for Option optimization
use std::num::NonZeroU64;

pub struct SessionId(NonZeroU64);  // Option<SessionId> is same size as SessionId

impl SessionId {
    /// Creates a new SessionId.
    /// Returns None for invalid (zero) input.
    pub fn new(id: u64) -> Option<Self> {
        NonZeroU64::new(id).map(Self)
    }
}
```

### Inline and Optimization Hints

```rust
// ✅ GOOD - Strategic inlining
#[inline]  // Small, frequently called
pub fn is_valid(&self) -> bool {
    self.state == State::Valid
}

#[inline(never)]  // Large, cold path
fn handle_rare_error_case() {
    // ... complex error handling
}

#[cold]  // Hint that function is unlikely to be called
fn panic_handler(msg: &str) -> ! {
    panic!("{}", msg)
}
```

## Advanced Type Patterns

### Sealed Traits for API Stability

```rust
// ✅ GOOD - Sealed traits prevent external implementation
mod sealed {
    pub trait Sealed {}
}

pub trait PublicTrait: sealed::Sealed {
    // Users can use but not implement this trait
}

impl sealed::Sealed for MyType {}
impl PublicTrait for MyType {}
```

### Type-Safe Dependency Injection

```rust
// ✅ GOOD - Type-safe dependency injection
pub struct AppContext {
    db: Arc<dyn Database>,
    cache: Arc<dyn Cache>,
    config: Arc<Config>,
}

// ✅ GOOD - Use extension traits for testing
#[cfg_attr(test, mockall::automock)]
pub trait TimeProvider: Send + Sync {
    fn now(&self) -> Instant;
}

pub struct Handler<T: TimeProvider = SystemTime> {
    time: T,
}
```

### Modern Iterator Patterns

```rust
// ✅ GOOD - Use array_chunks (stabilized 1.77)
let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
for chunk in data.array_chunks::<4>() {
    process_chunk(chunk);  // chunk is &[u8; 4]
}

// ✅ GOOD - Use try_fold for fallible iteration
let sum = numbers.iter()
    .try_fold(0u32, |acc, &x| {
        acc.checked_add(x).ok_or(Error::Overflow)
    })?;

// ✅ GOOD - Use intersperse (stabilized 1.79)
let formatted = items.iter()
    .map(|i| i.to_string())
    .intersperse(", ".to_string())
    .collect::<String>();
```

## Observability

### Structured Logging with Tracing

```rust
// ✅ GOOD - Structured logging with sensitive data handling
use tracing::{instrument, span, Level};

#[instrument(
    skip(password),  // Don't log sensitive data
    fields(
        user_id = %user.id,
        request_id = %Uuid::new_v4(),
    )
)]
pub async fn authenticate(user: User, password: String) -> Result<Session> {
    let span = span!(Level::DEBUG, "auth_check");
    let _guard = span.enter();

    // Metrics integration
    metrics::counter!("auth_attempts").increment(1);

    // ... authentication logic
}
```

### OpenTelemetry Integration

```rust
// ✅ GOOD - OpenTelemetry for distributed tracing
use opentelemetry::trace::Tracer;

pub fn init_telemetry() -> Result<()> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("capnweb")
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(())
}
```

## Feature Flags

### Best Practices

```toml
# Cargo.toml
[features]
default = ["std", "async"]
std = []
async = ["tokio", "futures"]
# Use additive features only
testing = ["mockall", "quickcheck"]

# Document feature requirements
#! Features:
#! - `std`: Enables standard library (enabled by default)
#! - `async`: Enables async runtime support
#! - `testing`: Enables test utilities (not for production)
```

```rust
// ✅ GOOD - Conditional compilation
#[cfg(feature = "async")]
pub async fn async_process() { }

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;  // no_std support

// ✅ GOOD - Feature-gated test utilities
#[cfg(feature = "testing")]
pub mod test_helpers {
    pub fn create_mock_client() -> MockClient { }
}
```

### RAII and Drop Patterns

```rust
// ✅ GOOD - RAII for resource management
pub struct TempFile {
    path: PathBuf,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);  // Best effort cleanup
    }
}

// ✅ GOOD - Guard patterns
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
    _phantom: PhantomData<*const ()>,  // !Send + !Sync
}
```

## Macro Patterns

### Hygienic Macro Design

```rust
// ✅ GOOD - Hygienic macros with proper scoping
macro_rules! define_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name($crate::types::InternalId);

        impl $name {
            pub const fn new(id: u64) -> Self {
                Self($crate::types::InternalId::new(id))
            }
        }
    };
}

// ✅ GOOD - Use proc macros for complex derivations
use proc_macro::TokenStream;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    // ... for complex compile-time code generation
}

// ✅ GOOD - Declarative macro with clear error messages
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}
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

## Common Anti-Patterns to Avoid

### Collection Anti-Patterns

```rust
// ❌ BAD - Unnecessary intermediate collection
let names: Vec<_> = users.iter().map(|u| u.name.clone()).collect();
let count = names.len();

// ✅ GOOD - Direct count without collection
let count = users.iter().count();

// ❌ BAD - Collecting just to iterate again
let items: Vec<_> = data.iter().filter(|x| x.is_valid()).collect();
for item in items { process(item); }

// ✅ GOOD - Direct iteration
for item in data.iter().filter(|x| x.is_valid()) {
    process(item);
}
```

### Async Anti-Patterns

```rust
// ❌ BAD - Blocking in async context
async fn bad_async() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks thread!
}

// ✅ GOOD - Use async-aware sleep
async fn good_async() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// ❌ BAD - Large futures on stack
async fn huge_future() -> [u8; 1_000_000] {
    [0; 1_000_000] // Too large for stack
}

// ✅ GOOD - Box large futures
async fn boxed_future() -> Box<[u8; 1_000_000]> {
    Box::new([0; 1_000_000])
}
```

### Error Handling Anti-Patterns

```rust
// ❌ BAD - Silently ignoring errors
let _ = important_operation();

// ✅ GOOD - Explicitly handle or log
if let Err(e) = important_operation() {
    tracing::warn!("Operation failed: {}", e);
}

// ❌ BAD - String errors without context
return Err("operation failed".to_string());

// ✅ GOOD - Typed errors with context
return Err(Error::OperationFailed {
    reason: "timeout",
    context: format!("after {} retries", retries),
});
```

## Migration Guide

When updating existing code to meet these standards:

1. **Phase 1**: Eliminate all `unwrap()` calls
2. **Phase 2**: Add error context with `anyhow`
3. **Phase 3**: Replace performance anti-patterns
4. **Phase 4**: Add comprehensive documentation
5. **Phase 5**: Add property-based tests

## Version History

- **2.0** (2025-01-28): Added modern Rust 1.85+ patterns:
  - Async traits and structured concurrency
  - Zero-copy operations with bytes crate
  - Const context optimization and NonZero types
  - Generic Associated Types (GATs)
  - Sealed traits and dependency injection
  - Modern iterator patterns (array_chunks, try_fold, intersperse)
  - Observability with tracing and OpenTelemetry
  - Feature flag best practices
  - RAII and macro patterns
  - Enhanced testing with rstest and snapshot testing
- **1.0** (2025-01-24): Initial version based on code analysis and modern Rust best practices

---

*This document is a living standard and should be updated as the project evolves and new patterns emerge.*