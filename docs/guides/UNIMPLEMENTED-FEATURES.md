# Cap'n Web Rust - Unimplemented Features

## Overview

This document tracks features from the official Cap'n Web protocol that are not yet implemented in the Rust version. Based on the official repository and protocol specification, these are the key missing features.

## ✅ Implemented Features

### Core Protocol (Milestone 1 - COMPLETE)
- ✅ Wire protocol (newline-delimited JSON arrays)
- ✅ Basic message types: Push, Pull, Resolve, Reject, Release
- ✅ Import/Export ID management
- ✅ HTTP Batch transport
- ✅ Basic capability invocation
- ✅ Error handling

## ❌ Not Yet Implemented

### 1. Promise Pipelining (Milestone 2)
**Priority: HIGH**

The ability to chain operations on promises before they resolve, enabling single round-trip for complex operations.

**Example from official Cap'n Web:**
```typescript
// This should work in a single round trip:
let user = api.authenticate(cookie);
let notifications = await user.getNotifications();
```

**What needs implementation:**
- Promise types in wire protocol
- Deferred execution of pipelined calls
- Promise resolution tracking
- Export of unresolved promises

### 2. .map() Record-Replay (Milestone 3)
**Priority: HIGH**

Transform remote values without pulling them locally, executing callbacks on remote promises with zero additional network trips.

**Example from official Cap'n Web:**
```typescript
let idsPromise = api.listUserIds();
let names = await idsPromise.map(id => [id, api.getUserName(id)]);
```

**What needs implementation:**
- IL (Intermediate Language) expressions
- Plan execution engine
- Variable bindings in expressions
- MapOperation wire format

### 3. IL (Intermediate Language) Support (Milestone 3)
**Priority: HIGH**

Complex operation plans that can be executed server-side.

**Wire format expressions needed:**
- `["var", index]` - Variable references
- `["plan", operations...]` - Execution plans
- `["bind", ...]` - Variable binding
- `["if", condition, then, else]` - Conditional execution

### 4. WebSocket Transport (Milestone 4)
**Priority: HIGH**

Persistent bidirectional connections with session state.

**What needs implementation:**
- Complete WebSocket wire protocol handler (started in `ws_wire.rs`)
- Session state management across messages
- Connection lifecycle management
- Reconnection support

### 5. WebTransport Support (Milestone 5)
**Priority: MEDIUM**

Modern HTTP/3 transport with multiplexed streams.

**What needs implementation:**
- Quinn integration for HTTP/3
- Stream multiplexing
- WebTransport protocol adapter
- Compatibility with h3 crate (currently blocked by version conflicts)

### 6. Stub/Proxy Capabilities (Advanced)
**Priority: MEDIUM**

Pass functions and objects by reference, enabling bidirectional calling.

**Example from official Cap'n Web:**
```typescript
// Client can pass a function to server
api.onEvent((event) => {
    console.log('Event received:', event);
});
```

**What needs implementation:**
- Stub export/import in wire protocol
- Bidirectional capability references
- Callback registration and invocation
- Cross-connection capability passing

### 7. Resume Tokens (Advanced)
**Priority: LOW**

Allow sessions to be resumed after disconnection.

**What needs implementation:**
- Resume token generation
- Session state serialization
- Token validation and session restoration
- Expiration handling

### 8. postMessage Transport
**Priority: LOW**

Browser iframe/worker communication transport.

**What needs implementation:**
- JavaScript bridge for browser environments
- postMessage protocol adapter
- Cross-origin handling

### 9. Recorder Macros (Milestone 6)
**Priority: LOW**

Client-side ergonomics for TypeScript-like API generation.

**What needs implementation:**
- Rust macro for generating client stubs
- Type-safe API wrappers
- Automatic proxy generation

## Implementation Roadmap

### Phase 1: Core Protocol Extensions (Q1 2025)
1. **Promise Pipelining** - Enable chained operations
2. **IL and .map()** - Server-side execution plans
3. **WebSocket completion** - Finish persistent connections

### Phase 2: Advanced Transports (Q2 2025)
1. **WebTransport** - HTTP/3 with stream multiplexing
2. **Stub capabilities** - Bidirectional function passing
3. **Resume tokens** - Session persistence

### Phase 3: Developer Experience (Q3 2025)
1. **Recorder macros** - Ergonomic Rust client API
2. **postMessage** - Browser integration
3. **Performance optimizations**

## Testing Requirements

Each feature needs:
1. Unit tests in Rust
2. Integration tests with TypeScript client
3. Protocol compliance verification
4. Performance benchmarks

## Dependencies

### External Crates Needed
- `quinn` (0.11+) - For WebTransport
- `h3` (compatible version) - HTTP/3 support
- Updated `tokio-tungstenite` - WebSocket improvements

### Protocol Documentation Needed
- Complete IL expression specification
- Stub capability lifecycle details
- Resume token format specification

## Notes

The Rust implementation currently covers the core protocol well but lacks the advanced features that make Cap'n Web particularly powerful:
- Promise pipelining for reduced round trips
- .map() for server-side data transformation
- Stub capabilities for true bidirectional RPC

These features are what differentiate Cap'n Web from simpler RPC protocols and should be prioritized for implementation.

## References

- [Cap'n Web Repository](https://github.com/cloudflare/capnweb)
- [Blog Post](https://blog.cloudflare.com/capnweb-javascript-rpc-library/)
- Internal CLAUDE.md milestones

---

*Last updated: 2025-01-25*
*Next review: After Milestone 2 completion*