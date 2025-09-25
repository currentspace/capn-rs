// Promise pipelining support for Cap'n Web protocol
// Implements the official pipelining feature per specification

use super::ids::ImportId;
use super::tables::Value;
use super::expression::{Expression, PropertyKey};
use dashmap::DashMap;
use std::collections::VecDeque;

/// Pipeline manager for promise pipelining as per Cap'n Web spec
pub struct PipelineManager {
    /// Pending pipeline operations for unresolved promises
    pipelines: DashMap<ImportId, VecDeque<PipelineOperation>>,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new() -> Self {
        Self {
            pipelines: DashMap::new(),
        }
    }

    /// Register a pipelined operation on a promise (spec-compliant)
    pub fn add_pipeline_operation(
        &self,
        promise_id: ImportId,
        operation: PipelineOperation,
    ) -> ImportId {
        let result_id = operation.result_id;

        self.pipelines
            .entry(promise_id)
            .or_insert_with(VecDeque::new)
            .push_back(operation);

        result_id
    }

    /// Execute all pipelined operations when a promise resolves (spec-compliant)
    pub async fn resolve_promise(&self, promise_id: ImportId, value: Value) -> Result<Vec<(ImportId, Result<Value, PipelineError>)>, PipelineError> {
        let mut results = Vec::new();

        if let Some((_, operations)) = self.pipelines.remove(&promise_id) {
            for operation in operations {
                let result = self.execute_pipeline_operation(&value, &operation).await;
                results.push((operation.result_id, result));
            }
        }

        Ok(results)
    }

    /// Execute a single pipeline operation (property access or method call)
    async fn execute_pipeline_operation(
        &self,
        value: &Value,
        operation: &PipelineOperation,
    ) -> Result<Value, PipelineError> {
        match operation.operation_type {
            PipelineOperationType::PropertyAccess { ref path } => {
                self.access_property_path(value, path).await
            }
            PipelineOperationType::MethodCall { ref method, ref args } => {
                self.call_method(value, method, args).await
            }
        }
    }

    /// Access a property path on a value (supports chained property access)
    async fn access_property_path(
        &self,
        mut current_value: &Value,
        path: &[PropertyKey],
    ) -> Result<Value, PipelineError> {
        let mut owned_value = None;

        for key in path {
            current_value = match current_value {
                Value::Object(obj) => {
                    match key {
                        PropertyKey::String(key_str) => {
                            if let Some(boxed_val) = obj.get(key_str) {
                                boxed_val.as_ref()
                            } else {
                                return Err(PipelineError::PropertyNotFound(key_str.clone()));
                            }
                        }
                        PropertyKey::Number(_) => {
                            return Err(PipelineError::InvalidPropertyType);
                        }
                    }
                }
                Value::Array(arr) => {
                    match key {
                        PropertyKey::Number(index) => {
                            let idx = *index as usize;
                            if let Some(val) = arr.get(idx) {
                                owned_value = Some(val.clone());
                                owned_value.as_ref().unwrap()
                            } else {
                                return Err(PipelineError::IndexOutOfBounds(idx));
                            }
                        }
                        PropertyKey::String(_) => {
                            return Err(PipelineError::InvalidPropertyType);
                        }
                    }
                }
                _ => {
                    return Err(PipelineError::CannotAccessProperty);
                }
            };
        }

        Ok(current_value.clone())
    }

    /// Call a method on a value (basic implementation)
    async fn call_method(
        &self,
        _value: &Value,
        _method: &str,
        _args: &Expression,
    ) -> Result<Value, PipelineError> {
        // Method calls on values require RPC target resolution
        // This would need integration with the capability system
        Err(PipelineError::MethodCallNotImplemented)
    }
}

impl Default for PipelineManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A pipeline operation as per Cap'n Web specification
#[derive(Debug, Clone)]
pub struct PipelineOperation {
    pub operation_type: PipelineOperationType,
    pub result_id: ImportId,
}

/// Types of pipeline operations supported by Cap'n Web
#[derive(Debug, Clone)]
pub enum PipelineOperationType {
    /// Property access with path (e.g., obj.foo.bar)
    PropertyAccess {
        path: Vec<PropertyKey>,
    },
    /// Method call with arguments
    MethodCall {
        method: String,
        args: Expression,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("Invalid property type for access")]
    InvalidPropertyType,

    #[error("Cannot access property on this value type")]
    CannotAccessProperty,

    #[error("Method call not implemented")]
    MethodCallNotImplemented,

    #[error("Pipeline execution failed")]
    ExecutionFailed,
}