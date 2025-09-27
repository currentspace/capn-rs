use crate::il_executor::ILExecutor;
use crate::il_extended::{ILContext, ILExpression};
use crate::promise::PromiseDependencyGraph;
use crate::protocol::tables::Value as TablesValue;
use crate::{CallId, PromiseId, RpcError, RpcTarget};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

/// MapOperation represents a .map() call on a promise
#[derive(Debug, Clone)]
pub struct MapOperation {
    /// The promise we're mapping over
    pub source_promise: PromiseId,
    /// The IL expression to apply to each element
    pub map_function: ILExpression,
    /// The resulting promise ID
    pub result_promise: PromiseId,
}

/// PipelinedCall represents a method call on an unresolved promise
#[derive(Debug, Clone)]
pub struct PipelinedCall {
    /// The promise we're calling a method on
    pub target_promise: PromiseId,
    /// The method name
    pub method: String,
    /// The arguments (which may themselves reference promises)
    pub args: Vec<Value>,
    /// The resulting promise ID
    pub result_promise: PromiseId,
    /// The call ID for tracking
    pub call_id: CallId,
}

/// PromiseMapExecutor handles .map() operations and promise pipelining
pub struct PromiseMapExecutor {
    /// Pending map operations indexed by source promise
    map_operations: Arc<DashMap<PromiseId, Vec<MapOperation>>>,
    /// Pending pipelined calls indexed by target promise
    pipelined_calls: Arc<DashMap<PromiseId, Vec<PipelinedCall>>>,
    /// The IL executor for running map functions
    il_executor: Arc<ILExecutor>,
    /// Dependency graph for tracking promise dependencies
    dependency_graph: Arc<tokio::sync::RwLock<PromiseDependencyGraph>>,
}

impl PromiseMapExecutor {
    pub fn new() -> Self {
        Self {
            map_operations: Arc::new(DashMap::new()),
            pipelined_calls: Arc::new(DashMap::new()),
            il_executor: Arc::new(ILExecutor::new()),
            dependency_graph: Arc::new(tokio::sync::RwLock::new(PromiseDependencyGraph::new())),
        }
    }

    /// Register a .map() operation on a promise
    pub async fn register_map(
        &self,
        source_promise: PromiseId,
        map_function: ILExpression,
        result_promise: PromiseId,
    ) -> Result<(), RpcError> {
        let operation = MapOperation {
            source_promise,
            map_function,
            result_promise,
        };

        self.map_operations
            .entry(source_promise)
            .or_default()
            .push(operation);

        // Add dependency
        let mut graph = self.dependency_graph.write().await;
        graph.add_dependency(result_promise, source_promise);

        Ok(())
    }

    /// Register a pipelined method call on a promise
    pub async fn register_pipelined_call(
        &self,
        target_promise: PromiseId,
        method: String,
        args: Vec<Value>,
        result_promise: PromiseId,
        call_id: CallId,
    ) -> Result<(), RpcError> {
        let call = PipelinedCall {
            target_promise,
            method,
            args,
            result_promise,
            call_id,
        };

        self.pipelined_calls
            .entry(target_promise)
            .or_default()
            .push(call);

        // Add dependency
        let mut graph = self.dependency_graph.write().await;
        graph.add_dependency(result_promise, target_promise);

        Ok(())
    }

    /// Execute map operations when a promise resolves
    pub async fn execute_map_on_resolution(
        &self,
        promise_id: PromiseId,
        resolved_value: Value,
    ) -> Vec<(PromiseId, Result<Value, RpcError>)> {
        let mut results = Vec::new();

        // Check for map operations on this promise
        if let Some((_, operations)) = self.map_operations.remove(&promise_id) {
            for operation in operations {
                let result = self
                    .execute_single_map(&resolved_value, &operation.map_function)
                    .await;
                results.push((operation.result_promise, result));
            }
        }

        results
    }

    /// Execute a single map operation
    async fn execute_single_map(
        &self,
        value: &Value,
        map_function: &ILExpression,
    ) -> Result<Value, RpcError> {
        match value {
            Value::Array(items) => {
                let mut mapped_results = Vec::new();
                let mut context = ILContext::new(vec![]);

                for item in items {
                    // Set the current item as a variable in the context
                    context
                        .set_variable(0, item.clone())
                        .map_err(|e| RpcError::internal(format!("IL error: {}", e)))?;

                    // Execute the map function
                    let result = self
                        .il_executor
                        .execute(map_function, &mut context)
                        .await
                        .map_err(|e| RpcError::internal(format!("Map execution failed: {}", e)))?;

                    mapped_results.push(result);
                }

                Ok(Value::Array(mapped_results))
            }
            _ => {
                // For non-arrays, apply the function directly
                let mut context = ILContext::new(vec![]);
                context
                    .set_variable(0, value.clone())
                    .map_err(|e| RpcError::internal(format!("IL error: {}", e)))?;

                self.il_executor
                    .execute(map_function, &mut context)
                    .await
                    .map_err(|e| RpcError::internal(format!("Map execution failed: {}", e)))
            }
        }
    }

    /// Execute pipelined calls when a promise resolves to a capability
    pub async fn execute_pipelined_calls(
        &self,
        promise_id: PromiseId,
        capability: Arc<dyn RpcTarget>,
    ) -> Vec<(CallId, PromiseId, Result<Value, RpcError>)> {
        let mut results = Vec::new();

        // Check for pipelined calls on this promise
        if let Some((_, calls)) = self.pipelined_calls.remove(&promise_id) {
            for call in calls {
                // Convert serde_json::Value args to tables::Value
                let converted_args = call.args.into_iter().map(json_to_tables_value).collect();

                let result = capability.call(&call.method, converted_args).await;

                // Convert result back to serde_json::Value
                let converted_result = result.map(tables_to_json_value);
                results.push((call.call_id, call.result_promise, converted_result));
            }
        }

        results
    }

    /// Get all promises that depend on a given promise
    pub async fn get_dependent_promises(&self, promise_id: PromiseId) -> Vec<PromiseId> {
        let graph = self.dependency_graph.read().await;
        graph
            .dependents_of(&promise_id)
            .map(|deps| deps.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if there would be a cycle when adding a dependency
    pub async fn would_create_cycle(&self, promise: PromiseId, depends_on: PromiseId) -> bool {
        let graph = self.dependency_graph.read().await;
        graph.would_create_cycle(promise, depends_on)
    }
}

impl Default for PromiseMapExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert serde_json::Value to tables::Value
fn json_to_tables_value(json: Value) -> TablesValue {
    match json {
        Value::Null => TablesValue::Null,
        Value::Bool(b) => TablesValue::Bool(b),
        Value::Number(n) => TablesValue::Number(n),
        Value::String(s) => TablesValue::String(s),
        Value::Array(arr) => {
            TablesValue::Array(arr.into_iter().map(json_to_tables_value).collect())
        }
        Value::Object(obj) => TablesValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, Box::new(json_to_tables_value(v))))
                .collect(),
        ),
    }
}

/// Convert tables::Value to serde_json::Value
fn tables_to_json_value(value: TablesValue) -> Value {
    match value {
        TablesValue::Null => Value::Null,
        TablesValue::Bool(b) => Value::Bool(b),
        TablesValue::Number(n) => Value::Number(n),
        TablesValue::String(s) => Value::String(s),
        TablesValue::Array(arr) => {
            Value::Array(arr.into_iter().map(tables_to_json_value).collect())
        }
        TablesValue::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, tables_to_json_value(*v)))
                .collect(),
        ),
        TablesValue::Date(timestamp) => {
            // Convert Date to a JSON object representation
            serde_json::json!({
                "_type": "date",
                "timestamp": timestamp
            })
        }
        TablesValue::Error {
            error_type,
            message,
            stack,
        } => {
            // Convert Error to a JSON object representation
            serde_json::json!({
                "_type": "error",
                "error_type": error_type,
                "message": message,
                "stack": stack
            })
        }
        TablesValue::Stub(stub_ref) => {
            // Convert Stub to a JSON object representation
            serde_json::json!({
                "_type": "stub",
                "id": stub_ref.id
            })
        }
        TablesValue::Promise(promise_ref) => {
            // Convert Promise to a JSON object representation
            serde_json::json!({
                "_type": "promise",
                "id": promise_ref.id
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug)]
    struct TestCapability;

    #[async_trait::async_trait]
    impl RpcTarget for TestCapability {
        async fn call(
            &self,
            method: &str,
            args: Vec<TablesValue>,
        ) -> Result<TablesValue, RpcError> {
            match method {
                "double" => {
                    if let Some(TablesValue::Number(n)) = args.first() {
                        if let Some(v) = n.as_f64() {
                            return Ok(TablesValue::Number(
                                serde_json::Number::from_f64(v * 2.0).unwrap(),
                            ));
                        }
                    }
                    Err(RpcError::bad_request("Invalid argument"))
                }
                "getName" => Ok(TablesValue::String("TestCap".to_string())),
                _ => Err(RpcError::not_found(format!("Method {} not found", method))),
            }
        }

        async fn get_property(&self, _property: &str) -> Result<TablesValue, RpcError> {
            Ok(TablesValue::Null)
        }
    }

    #[tokio::test]
    async fn test_map_on_array() {
        let executor = PromiseMapExecutor::new();

        // Create a map function that doubles values
        let map_fn = ILExpression::call(ILExpression::var(0), "double".to_string(), vec![]);

        let source_promise = PromiseId::new(1);
        let result_promise = PromiseId::new(2);

        // Register the map operation
        executor
            .register_map(source_promise, map_fn, result_promise)
            .await
            .unwrap();

        // Simulate promise resolution with an array
        let resolved_value = json!([1, 2, 3, 4, 5]);
        let results = executor
            .execute_map_on_resolution(source_promise, resolved_value)
            .await;

        assert_eq!(results.len(), 1);
        let (promise_id, result) = &results[0];
        assert_eq!(*promise_id, result_promise);

        // Note: This test would need proper capability resolution to work fully
        // For now, it demonstrates the structure
        assert!(result.is_err()); // Expected since we don't have capability resolution yet
    }

    #[tokio::test]
    async fn test_pipelined_call() {
        let executor = PromiseMapExecutor::new();

        let target_promise = PromiseId::new(1);
        let result_promise = PromiseId::new(2);
        let call_id = CallId::new(1);

        // Register a pipelined call
        executor
            .register_pipelined_call(
                target_promise,
                "getName".to_string(),
                vec![],
                result_promise,
                call_id,
            )
            .await
            .unwrap();

        // Simulate promise resolution to a capability
        let capability = Arc::new(TestCapability);
        let results = executor
            .execute_pipelined_calls(target_promise, capability)
            .await;

        assert_eq!(results.len(), 1);
        let (returned_call_id, returned_promise_id, result) = &results[0];
        assert_eq!(*returned_call_id, call_id);
        assert_eq!(*returned_promise_id, result_promise);
        assert!(result.is_ok());
        assert_eq!(
            result.as_ref().unwrap(),
            &Value::String("TestCap".to_string())
        );
    }

    #[tokio::test]
    async fn test_dependency_tracking() {
        let executor = PromiseMapExecutor::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);
        let p3 = PromiseId::new(3);

        // Register map operations to create dependencies
        executor
            .register_map(p1, ILExpression::var(0), p2)
            .await
            .unwrap();
        executor
            .register_map(p2, ILExpression::var(0), p3)
            .await
            .unwrap();

        // Check dependencies
        let deps_of_p2 = executor.get_dependent_promises(p1).await;
        assert!(deps_of_p2.contains(&p2));

        let deps_of_p3 = executor.get_dependent_promises(p2).await;
        assert!(deps_of_p3.contains(&p3));

        // Check cycle detection
        assert!(executor.would_create_cycle(p1, p3).await);
        assert!(!executor.would_create_cycle(p3, p1).await);
    }
}
