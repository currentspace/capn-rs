//! # Cap'n Web Core Protocol
//!
//! Core implementation of the [Cap'n Web protocol](https://github.com/cloudflare/capnweb),
//! providing capability-based RPC with promise pipelining.
//!
//! ## Features
//!
//! - **Capability-based security**: Unforgeable object references with fine-grained access control
//! - **Promise pipelining**: Chain dependent calls without waiting for intermediate results
//! - **IL expression evaluation**: Execute complex operations with the Intermediate Language
//! - **Wire protocol compliance**: Full compatibility with the official TypeScript implementation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use capnweb_core::{CapId, RpcTarget, RpcError, Value};
//! use async_trait::async_trait;
//! use serde_json::json;
//!
//! #[derive(Debug)]
//! struct Calculator;
//!
//! #[async_trait]
//! impl RpcTarget for Calculator {
//!     async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
//!         match method {
//!             "add" => {
//!                 // Note: Value is from capnweb_core, not serde_json
//!                 // In a real implementation, you'd need to convert between types
//!                 Ok(Value::String("result".to_string()))
//!             }
//!             _ => Err(RpcError::not_found("method not found")),
//!         }
//!     }
//!
//!     async fn get_property(&self, _property: &str) -> Result<Value, RpcError> {
//!         Err(RpcError::not_found("property access not implemented"))
//!     }
//! }
//! ```
//!
//! ## Protocol Concepts
//!
//! ### Capabilities
//! Capabilities are unforgeable references to remote objects. They provide secure,
//! fine-grained access control without ambient authority.
//!
//! ### Promise Pipelining
//! Calls can be chained on promises before they resolve, reducing round-trips:
//!
//! ```ignore
//! let user = client.call(cap, "getUser", vec![user_id]);
//! let profile = client.pipeline(&user, vec!["profile"], "load", vec![]);
//! ```
//!
//! ### IL (Intermediate Language)
//! The IL allows expressing complex operations that execute on the server:
//!
//! ```ignore
//! let expr = ILExpression::if_expr(
//!     ILExpression::var(0),
//!     ILExpression::literal(json!("authenticated")),
//!     ILExpression::literal(json!("anonymous"))
//! );
//! ```

// Legacy modules (to be deprecated)
pub mod codec;
pub mod error;
pub mod ids;
pub mod il;
pub mod il_executor;
pub mod il_extended;
pub mod msg;
pub mod promise;
pub mod promise_map;
#[cfg(feature = "validation")]
pub mod validate;

// New Cap'n Web protocol implementation
pub mod protocol;

// Re-export legacy types for backward compatibility
pub use codec::{decode_message, encode_message};
pub use error::{ErrorCode, RpcError};
pub use ids::{CallId, CapId, PromiseId};
pub use il::{Op, Plan, Source};
pub use il_executor::ILExecutor;
pub use il_extended::{ILContext, ILError, ILExpression, ILOperation, ILPlan};
pub use msg::{Message, Outcome, Target};
pub use promise::{ArgValue, ExtendedTarget, PendingPromise, PromiseDependencyGraph};
pub use promise_map::{MapOperation, PipelinedCall, PromiseMapExecutor};

// Re-export official Cap'n Web wire protocol (primary)
pub use protocol::{
    capability_registry::{CapabilityRegistry, RegistrableCapability},
    ids::{ExportId, ImportId},
    tables::{ExportTable, ImportTable, Value},
    wire::{parse_wire_batch, serialize_wire_batch, PropertyKey, WireExpression, WireMessage},
};

// Legacy protocol types
pub use protocol::{expression::Expression, message::Message as LegacyMessage};

// RPC Target trait for capability implementations
pub use async_trait::async_trait;

#[async_trait]
pub trait RpcTarget: Send + Sync + std::fmt::Debug {
    /// Call a method on this capability
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError>;

    /// Get a property from this capability
    async fn get_property(&self, property: &str) -> Result<Value, RpcError>;
}

// Mock RPC target for testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockRpcTarget {}

#[cfg(test)]
impl MockRpcTarget {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
impl Default for MockRpcTarget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[async_trait]
impl RpcTarget for MockRpcTarget {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        Ok(Value::String(format!(
            "Mock call to {} with {} args",
            method,
            args.len()
        )))
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        Ok(Value::String(format!("Mock property {}", property)))
    }
}
