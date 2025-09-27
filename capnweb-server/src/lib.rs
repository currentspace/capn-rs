//! # Cap'n Web Server Library
//!
//! Server implementation for the Cap'n Web RPC protocol.
//!
//! This crate provides a production-ready server with:
//! - Capability registration and lifecycle management
//! - Automatic batching and pipelining support
//! - Multiple transport protocols (HTTP, WebSocket, WebTransport)
//! - Rate limiting and connection management
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use capnweb_server::{Server, ServerConfig};
//! use capnweb_core::{CapId, RpcError, Value};
//! use capnweb_core::RpcTarget;  // Use RpcTarget from core, not server
//! use async_trait::async_trait;
//! use std::sync::Arc;
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct HelloService;
//!
//! #[async_trait]
//! impl RpcTarget for HelloService {
//!     async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
//!         match method {
//!             "greet" => {
//!                 // Note: Value is from capnweb_core, not serde_json
//!                 Ok(Value::String(format!("Hello, World!")))
//!             }
//!             _ => Err(RpcError::not_found("method not found"))
//!         }
//!     }
//!
//!     async fn get_property(&self, _property: &str) -> Result<Value, RpcError> {
//!         Err(RpcError::not_found("property access not implemented"))
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create server configuration
//! let config = ServerConfig {
//!     port: 8080,
//!     host: "127.0.0.1".to_string(),
//!     max_batch_size: 100,
//! };
//!
//! // Create and configure server
//! let server = Server::new(config);
//! server.register_capability(
//!     CapId::new(1),
//!     Arc::new(HelloService)
//! );
//!
//! // Run the server
//! server.run().await;
//! # Ok(())
//! # }
//! ```
//!
//! ## Capability Registration
//!
//! Register multiple capabilities with different IDs:
//!
//! ```rust,ignore
//! # use capnweb_server::Server;
//! # use capnweb_core::CapId;
//! # use std::sync::Arc;
//! # struct AuthService;
//! # struct DataService;
//! # struct AdminService;
//! # let server = Server::new(Default::default()).unwrap();
//! server.register_capability(CapId::new(1), Arc::new(AuthService));
//! server.register_capability(CapId::new(2), Arc::new(DataService));
//! server.register_capability(CapId::new(3), Arc::new(AdminService));
//! ```
//!
//! ## Transport Configuration
//!
//! The server supports multiple transport protocols:
//!
//! - **HTTP Batch**: Default transport at `/rpc/batch`
//! - **WebSocket**: Real-time bidirectional communication (with feature flag)
//! - **WebTransport**: HTTP/3-based transport (with feature flag)

// Official Cap'n Web wire protocol server
pub mod server_wire_handler;
pub mod wire_server;

// Legacy servers (TO BE REMOVED - only wire protocol should be used)
pub mod advanced_capability;
pub mod cap_table;
pub mod capnweb_server;
#[cfg(feature = "h3-server")]
pub mod h3_server;
pub mod lifecycle;
pub mod limits;
pub mod logging;
pub mod promise_table;
pub mod runner;
pub mod server;
#[cfg(feature = "all-transports")]
pub mod ws_h1;
#[cfg(feature = "all-transports")]
pub mod ws_wire;

// Primary exports: Official Cap'n Web wire protocol
pub use wire_server::{RpcTargetAdapter, WireCapability, WireServer, WireServerConfig};

// Legacy exports
pub use advanced_capability::{
    AdvancedCapability, AdvancedCapabilityBuilder, AdvancedCapabilityConfig,
};
pub use cap_table::CapTable;
pub use capnweb_server::{CapnWebServer as NewCapnWebServer, CapnWebServerConfig};
pub use lifecycle::{CapabilityLifecycle, Disposable, LifecycleStats};
pub use limits::RateLimits;
pub use logging::{init_logging, init_test_logging};
pub use promise_table::{PromiseTable, PromiseTableStats};
pub use runner::PlanRunner;
pub use server::{RpcTarget, Server, ServerConfig};
