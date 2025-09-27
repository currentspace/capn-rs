# Cap'n Web Rust Implementation - Engineering Task List

## Pre-requisites & Setup

### Workspace Initialization
1. Create workspace `Cargo.toml` with 5 member crates
2. Set up shared dependencies in workspace `[dependencies]`
3. Configure shared features and optimization profiles
4. Set up CI/CD pipeline skeleton (GitHub Actions/GitLab CI)

### Development Dependencies
```toml
[workspace.dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"

# Async runtime
tokio = { version = "1.40", features = ["full"] }
futures = "0.3"
async-trait = "0.1"

# HTTP stack
axum = "0.7"
hyper = { version = "1.5", features = ["http1", "http2", "server", "client"] }
tower = "0.4"
tower-http = "0.6"

# HTTP/3 & QUIC
quinn = "0.11"
h3 = "0.0.6"
h3-quinn = "0.0.6"
rustls = "0.23"

# WebSocket
tokio-tungstenite = "0.26"
tungstenite = "0.26"

# Utilities
dashmap = "6.1"
indexmap = "2.2"
governor = "0.7"
nonzero_ext = "0.3"

# Validation
jsonschema = "0.25"
schemars = "0.8"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Testing
criterion = "0.5"
proptest = "1.0"
mockito = "1.0"
insta = "1.0"
```

---

## Milestone 1: Core Wire Protocol & HTTP Batch Server

### 1.1 Create `capnweb-core` crate

#### Setup
1. Create `capnweb-core/Cargo.toml` with minimal dependencies
2. Set up module structure: `ids.rs`, `msg.rs`, `codec.rs`, `error.rs`
3. Configure feature flags: `simd-json`, `validation`

#### Implementation Tasks

##### A. IDs Module (`ids.rs`)
1. Define ID types with newtype pattern:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub struct CallId(u64);

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub struct PromiseId(u64);

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub struct CapId(u64);
   ```
2. Implement monotonic ID allocators (thread-safe using AtomicU64)
3. Add display traits and conversion methods
4. **Tests**: Unit tests for ID generation, uniqueness, thread safety

##### B. Messages Module (`msg.rs`)
1. Define message enums and structs:
   ```rust
   #[derive(Serialize, Deserialize)]
   #[serde(tag = "type", rename_all = "camelCase")]
   pub enum Message {
       Call { id: CallId, target: Target, member: String, args: Vec<Value> },
       Result { id: CallId, #[serde(flatten)] outcome: Outcome },
       CapRef { id: CapId },
       Dispose { caps: Vec<CapId> },
   }
   ```
2. Implement Target enum (CapId or special targets)
3. Define Outcome enum (success Value or Error)
4. **Tests**: Serialization/deserialization roundtrip tests

##### C. Error Module (`error.rs`)
1. Define error codes enum:
   ```rust
   #[derive(Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum ErrorCode {
       BadRequest,
       NotFound,
       CapRevoked,
       PermissionDenied,
       Canceled,
       Internal,
   }
   ```
2. Implement RpcError with code, message, optional data
3. Add conversion traits from common error types
4. **Tests**: Error construction, serialization tests

##### D. Codec Module (`codec.rs`)
1. Implement encode/decode functions for Messages
2. Add frame format support (length-prefixed or newline-delimited)
3. Optional simd-json feature integration
4. **Tests**: Encode/decode with various message types, malformed input handling

### 1.2 Create `capnweb-transport` crate (minimal for HTTP batch)

#### Setup
1. Create `capnweb-transport/Cargo.toml`
2. Define transport trait skeleton
3. Implement HTTP batch adapter

#### Implementation Tasks

##### A. Transport Trait (`transport.rs`)
```rust
#[async_trait]
pub trait RpcTransport: Send + Sync {
    async fn send(&mut self, msg: Message) -> Result<(), TransportError>;
    async fn recv(&mut self) -> Result<Option<Message>, TransportError>;
    async fn close(&mut self) -> Result<(), TransportError>;
}
```

##### B. HTTP Batch Module (`http_batch.rs`)
1. Implement BatchTransport that collects messages
2. Add execute method to send batch as HTTP POST
3. Parse response and distribute results
4. **Tests**: Batch collection, HTTP request/response simulation

### 1.3 Create `capnweb-server` crate (HTTP batch endpoint)

#### Setup
1. Create `capnweb-server/Cargo.toml` with axum, hyper dependencies
2. Set up basic axum application structure

#### Implementation Tasks

##### A. Server Module (`server.rs`)
1. Define RpcTarget trait:
   ```rust
   #[async_trait]
   pub trait RpcTarget: Send + Sync {
       async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
   }
   ```
2. Implement basic server struct with axum Router
3. Add configuration options (port, TLS, limits)

##### B. Capability Table (`cap_table.rs`)
1. Implement CapTable using DashMap<CapId, Arc<dyn RpcTarget>>
2. Add methods: insert, lookup, remove, clear
3. Track ownership and reference counting
4. **Tests**: Concurrent access, lifecycle management

##### C. HTTP Batch Handler
1. Create `/rpc/batch` POST endpoint
2. Parse batch request, execute calls sequentially
3. Collect results and return batch response
4. Add basic rate limiting with governor
5. **Tests**: End-to-end batch processing, error handling

### 1.4 Integration Tests for Milestone 1
1. Create test server with mock RpcTargets
2. Test batch requests with multiple calls
3. Test error propagation and partial failures
4. Benchmark batch processing performance

---

## Milestone 2: Pipelining and Disposal

### 2.1 Enhance `capnweb-core`

#### A. Promise Support
1. Add Promise type to message args/results
2. Implement PromiseId allocation and tracking
3. Update codec to handle promise references
4. **Tests**: Promise serialization, reference resolution

#### B. Capability References
1. Enhance CapRef message with ownership info
2. Add capability passing in call arguments
3. Implement capability return in results
4. **Tests**: Cap passing scenarios, ownership transfer

### 2.2 Enhance `capnweb-server`

#### A. Promise Resolution (`runner.rs`)
1. Create PromiseTable for pending promises
2. Implement promise pipelining logic
3. Add topological sorting for dependent calls
4. Handle promise cancellation
5. **Tests**: Pipeline execution, dependency resolution

#### B. Disposal Management
1. Process Dispose messages
2. Track capability lifecycle
3. Implement cleanup on session end
4. Add disposal callbacks to RpcTarget
5. **Tests**: Explicit disposal, implicit cleanup

### 2.3 Create Basic Client (`capnweb-client`)

#### Setup
1. Create `capnweb-client/Cargo.toml`
2. Basic client structure with transport abstraction

#### A. Client Core (`client.rs`)
1. Implement Client struct with transport
2. Add call method returning futures
3. Promise table for tracking pending calls
4. **Tests**: Basic client calls, promise resolution

#### B. Capability Stubs (`stubs.rs`)
1. Define Capability<T> wrapper
2. Implement Drop for automatic disposal
3. Add method call forwarding
4. **Tests**: Stub lifecycle, disposal on drop

### 2.4 Integration Tests for Milestone 2
1. Test pipelined calls with dependencies
2. Test capability passing between calls
3. Test disposal patterns (explicit and implicit)
4. Performance tests for pipeline execution

---

## Milestone 3: IL, Plan Runner, and .map()

### 3.1 Implement IL in `capnweb-core` (`il.rs`)

#### A. IL Data Structures
```rust
#[derive(Serialize, Deserialize)]
pub enum Source {
    Capture(u32),
    Result(u32),
    Param(Vec<String>),
    ByValue(Value),
}

#[derive(Serialize, Deserialize)]
pub enum Op {
    Call { target: Source, member: String, args: Vec<Source>, result: u32 },
    Object { fields: BTreeMap<String, Source>, result: u32 },
    Array { items: Vec<Source>, result: u32 },
}

#[derive(Serialize, Deserialize)]
pub struct Plan {
    pub captures: Vec<CapId>,
    pub ops: Vec<Op>,
    pub result: Source,
}
```

#### B. IL Validation
1. Validate op result indices are unique
2. Check topological ordering
3. Verify source references are valid
4. **Tests**: Valid/invalid plan structures

### 3.2 Plan Runner in `capnweb-server` (`runner.rs`)

#### A. Plan Executor
1. Parse and validate incoming plans
2. Resolve captures from CapTable
3. Execute ops in dependency order
4. Build result values incrementally
5. Handle errors and rollback
6. **Tests**: Complex plan execution, error cases

#### B. Limits Enforcement
1. Max ops per plan
2. Max execution time
3. Memory usage limits
4. **Tests**: Limit enforcement, resource cleanup

### 3.3 Recorder in `capnweb-client` (`recorder.rs`)

#### A. Recording Infrastructure
1. Create Recorder struct to build Plans
2. Implement placeholder values for results
3. Track operations and dependencies
4. Generate IL from recorded operations
5. **Tests**: Recording scenarios, IL generation

#### B. .map() Implementation
1. Add record_map! macro for closure recording
2. Detect and prevent conditional logic
3. Validate no side effects in closures
4. Convert closure to IL operations
5. **Tests**: Valid/invalid .map() usage

### 3.4 Integration Tests for Milestone 3
1. Test complex plans with multiple operations
2. Test .map() with various transformations
3. Test error handling in plan execution
4. Benchmark plan execution performance

---

## Milestone 4: WebSocket Support (H1/H2/H3)

### 4.1 WebSocket Transport in `capnweb-transport`

#### A. WebSocket H1/H2 (`websocket.rs`)
1. Implement WebSocketTransport using tokio-tungstenite
2. Add connection management
3. Implement heartbeat/ping-pong
4. Handle reconnection logic
5. **Tests**: Connection lifecycle, message flow

#### B. WebSocket H3 Support
1. Use Extended CONNECT (RFC 9220)
2. Integrate with h3 crate
3. Handle H3 specific features
4. **Tests**: H3 WebSocket connection

### 4.2 WebSocket Server Support

#### A. Axum WebSocket Handler
1. Add WebSocket upgrade endpoint
2. Manage session state per connection
3. Multiplex concurrent calls
4. Implement heartbeat timer
5. **Tests**: Multiple concurrent clients

#### B. Session Management
1. Track active sessions
2. Capability ownership per session
3. Cleanup on disconnect
4. **Tests**: Session lifecycle, cleanup

### 4.3 Client WebSocket Support

#### A. WebSocket Client Transport
1. Add WebSocket connection option
2. Automatic reconnection with backoff
3. Message queuing during reconnect
4. **Tests**: Reconnection scenarios

### 4.4 Integration Tests for Milestone 4
1. Test WebSocket with multiple concurrent calls
2. Test connection interruption and recovery
3. Test H1/H2/H3 WebSocket variants
4. Load testing with many connections

---

## Milestone 5: WebTransport Support

### 5.1 WebTransport in `capnweb-transport` (`webtransport.rs`)

#### A. Quinn + H3 Integration
1. Set up QUIC endpoint with quinn
2. Implement h3-quinn WebTransport
3. Manage control bidirectional stream
4. Handle datagram support (optional)
5. **Tests**: QUIC connection, stream management

#### B. WebTransport Protocol
1. Implement control stream protocol
2. Add stream multiplexing support
3. Resume token generation/validation
4. **Tests**: Protocol compliance

### 5.2 H3 Server in `capnweb-server` (`h3_server.rs`)

#### A. H3 Server Setup
1. Create quinn endpoint with rustls
2. Accept WebTransport connections
3. Route to RPC handler
4. **Tests**: H3 server basics

#### B. WebTransport Sessions
1. Manage WT session state
2. Handle stream lifecycle
3. Implement resume tokens
4. **Tests**: Session management

### 5.3 Client WebTransport Support

#### A. WebTransport Client
1. Add WT connection option
2. Implement negotiation fallback
3. Resume token handling
4. **Tests**: Connection scenarios

### 5.4 Integration Tests for Milestone 5
1. Test WebTransport with heavy load
2. Test stream multiplexing
3. Test resume functionality
4. Compare performance vs WebSocket

---

## Milestone 6: Recorder Macros

### 6.1 Macro Development (`macros/`)

#### A. Interface Macro
```rust
#[interface]
pub trait Calculator {
    async fn add(a: f64, b: f64) -> f64;
    async fn multiply(a: f64, b: f64) -> f64;
}
```
1. Parse trait definition
2. Generate stub implementation
3. Generate placeholder types
4. **Tests**: Various trait shapes

#### B. record_map! Macro
1. Parse closure syntax
2. Generate IL operations
3. Validate no conditionals
4. **Tests**: Complex transformations

### 6.2 Enhanced Recorder

#### A. Macro Integration
1. Connect macros to recorder
2. Type-safe stub generation
3. Compile-time validation
4. **Tests**: End-to-end macro usage

### 6.3 Integration Tests for Milestone 6
1. Test generated stubs with server
2. Test complex .map() scenarios
3. Test error detection in macros

---

## Milestone 7: Interop Tests

### 7.1 Create `capnweb-interop-tests` crate

#### A. Test Fixtures (`fixtures.rs`)
1. Load golden transcripts from TypeScript
2. Parse and validate fixtures
3. Create test scenarios
4. **Tests**: Fixture loading

#### B. JS Client Tests (`js_client_tests.rs`)
1. Start Rust server
2. Run JS client test suite
3. Validate protocol compliance
4. **Tests**: All JS client scenarios

#### C. JS Server Tests (`js_server_tests.rs`)
1. Run JS server
2. Execute Rust client tests
3. Validate responses
4. **Tests**: All Rust client scenarios

### 7.2 Comprehensive Test Coverage
1. Basic calls
2. Capability returns
3. Pipelined calls
4. .map() operations
5. Disposal patterns
6. Error injection
7. Transport negotiation
8. Performance benchmarks

---

## Testing Strategy

### Unit Tests (per component)
- Test in isolation with mocks
- Property-based testing with proptest
- Fuzz testing for parsers/codecs
- Use insta for snapshot testing

### Integration Tests (per milestone)
- Real transport connections
- Multi-crate interactions
- Load and stress testing
- Use criterion for benchmarks

### Interop Tests (final milestone)
- Cross-language validation
- Protocol compliance
- Performance comparison
- Regression test suite

### Test Infrastructure
1. Docker containers for JS runtime
2. Test harness for fixture execution
3. CI pipeline with test matrix
4. Coverage reporting (tarpaulin/llvm-cov)
5. Benchmark tracking over time

---

## Crate Wiring Instructions

### Dependencies Graph
```
capnweb-core (standalone)
    ↑
capnweb-transport (depends on core)
    ↑
capnweb-server (depends on core, transport)
capnweb-client (depends on core, transport)
    ↑
capnweb-interop-tests (depends on all)
```

### Feature Flags
```toml
# capnweb-core
[features]
default = ["validation"]
simd-json = ["simd-json"]
validation = ["jsonschema", "schemars"]

# capnweb-transport
[features]
default = ["websocket", "webtransport", "http-batch"]
websocket = ["tokio-tungstenite"]
webtransport = ["quinn", "h3", "h3-quinn"]
http-batch = ["hyper", "reqwest"]

# capnweb-server
[features]
default = ["all-transports"]
all-transports = ["capnweb-transport/default"]

# capnweb-client
[features]
default = ["all-transports", "macros"]
all-transports = ["capnweb-transport/default"]
macros = ["capnweb-macros"]
```

### Version Management
- Use workspace version inheritance
- Keep all crates at same version
- Semantic versioning from 0.1.0
- Tag releases with git tags

---

## Development Workflow

### Branch Strategy
1. `main`: stable releases
2. `develop`: integration branch
3. `feature/*`: individual features
4. `milestone/*`: per-milestone work

### PR Requirements
1. All tests passing
2. No clippy warnings
3. Formatted with rustfmt
4. Documentation for public APIs
5. CHANGELOG entry
6. Benchmark comparison

### Release Process
1. Version bump in all Cargo.toml
2. Update CHANGELOG
3. Run full test suite
4. Create git tag
5. Publish to crates.io in dependency order

---

## Research Tasks

### Pre-implementation Research
1. Analyze TypeScript implementation for:
   - Frame format (length-prefixed vs JSON-seq)
   - Resume token implementation
   - Protocol edge cases
2. Review WebTransport browser compatibility
3. Benchmark transport options
4. Evaluate validation library performance

### Ongoing Research
1. Monitor h3/quinn updates
2. Track WebTransport spec changes
3. Profile memory usage patterns
4. Optimize hot paths based on profiles