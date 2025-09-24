// Legacy modules (to be deprecated)
pub mod ids;
pub mod msg;
pub mod codec;
pub mod error;
pub mod promise;
#[cfg(feature = "validation")]
pub mod validate;
pub mod il;

// New Cap'n Web protocol implementation
pub mod protocol;

// Re-export legacy types for backward compatibility
pub use ids::{CallId, PromiseId, CapId};
pub use msg::{Message, Target, Outcome};
pub use error::{RpcError, ErrorCode};
pub use codec::{encode_message, decode_message};
pub use il::{Source, Op, Plan};
pub use promise::{ArgValue, ExtendedTarget, PromiseDependencyGraph, PendingPromise};

// Re-export new protocol types
pub use protocol::{
    message::Message as CapnWebMessage,
    expression::Expression,
    ids::{ImportId, ExportId},
    tables::{ImportTable, ExportTable, Value},
};

// RPC Target trait for capability implementations
use async_trait::async_trait;

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