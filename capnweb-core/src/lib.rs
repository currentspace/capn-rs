// Legacy modules (to be deprecated)
pub mod ids;
pub mod msg;
pub mod codec;
pub mod error;
pub mod promise;
#[cfg(feature = "validation")]
pub mod validate;
pub mod il;
pub mod il_extended;
pub mod il_executor;
pub mod promise_map;

// New Cap'n Web protocol implementation
pub mod protocol;

// Re-export legacy types for backward compatibility
pub use ids::{CallId, PromiseId, CapId};
pub use msg::{Message, Target, Outcome};
pub use error::{RpcError, ErrorCode};
pub use codec::{encode_message, decode_message};
pub use il::{Source, Op, Plan};
pub use il_extended::{ILExpression, ILContext, ILPlan, ILOperation, ILError};
pub use il_executor::ILExecutor;
pub use promise_map::{PromiseMapExecutor, MapOperation, PipelinedCall};
pub use promise::{ArgValue, ExtendedTarget, PromiseDependencyGraph, PendingPromise};

// Re-export official Cap'n Web wire protocol (primary)
pub use protocol::{
    wire::{WireMessage, WireExpression, PropertyKey, parse_wire_batch, serialize_wire_batch},
    ids::{ImportId, ExportId},
    tables::{ImportTable, ExportTable, Value},
    capability_registry::{CapabilityRegistry, RegistrableCapability},
};

// Legacy protocol types
pub use protocol::{
    message::Message as LegacyMessage,
    expression::Expression,
};

// RPC Target trait for capability implementations
use async_trait::async_trait;
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
pub struct MockRpcTarget {
}

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
        Ok(Value::String(format!("Mock call to {} with {} args", method, args.len())))
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        Ok(Value::String(format!("Mock property {}", property)))
    }
}