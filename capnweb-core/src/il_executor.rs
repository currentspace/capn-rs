use crate::il_extended::{ILContext, ILError, ILExpression, ILOperation, ILPlan};
use crate::RpcTarget;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Executor for IL expressions with async support
pub struct ILExecutor {
    #[allow(dead_code)] // Will be used when capability operations are implemented
    capabilities: Vec<Arc<dyn RpcTarget>>,
}

impl ILExecutor {
    pub fn new() -> Self {
        Self {
            capabilities: Vec::new(),
        }
    }

    pub fn with_capabilities(capabilities: Vec<Arc<dyn RpcTarget>>) -> Self {
        Self { capabilities }
    }

    /// Execute an IL expression
    pub fn execute<'a>(
        &'a self,
        expr: &'a ILExpression,
        context: &'a mut ILContext,
    ) -> Pin<Box<dyn Future<Output = Result<Value, ILError>> + 'a>> {
        Box::pin(self.execute_impl(expr, context))
    }

    /// Internal implementation of execute
    async fn execute_impl(
        &self,
        expr: &ILExpression,
        context: &mut ILContext,
    ) -> Result<Value, ILError> {
        match expr {
            ILExpression::Literal(value) => Ok(value.clone()),

            ILExpression::Variable { var_ref } => context
                .get_variable(*var_ref)
                .cloned()
                .ok_or(ILError::VariableNotFound(*var_ref)),

            ILExpression::Plan { plan } => self.execute_plan(plan, context).await,

            ILExpression::Bind { bind } => {
                let value = self.execute(&bind.value, context).await?;
                let _var_index = context.push_variable(value);
                let result = self.execute(&bind.body, context).await?;
                // Clean up the variable (optional, for stack-like behavior)
                Ok(result)
            }

            ILExpression::If { if_expr } => {
                let condition = self.execute(&if_expr.condition, context).await?;
                if self.is_truthy(&condition) {
                    self.execute(&if_expr.then_branch, context).await
                } else {
                    self.execute(&if_expr.else_branch, context).await
                }
            }

            ILExpression::Get { get } => {
                let object = self.execute(&get.object, context).await?;
                self.get_property(&object, &get.property)
            }

            ILExpression::Call { call } => {
                let target = self.execute(&call.target, context).await?;
                let mut args = Vec::new();
                for arg_expr in &call.args {
                    args.push(self.execute(arg_expr, context).await?);
                }
                self.call_method(target, &call.method, args).await
            }

            ILExpression::MapOp { map } => {
                let array = self.execute(&map.array, context).await?;
                self.execute_map(array, &map.function, context).await
            }

            ILExpression::FilterOp { filter } => {
                let array = self.execute(&filter.array, context).await?;
                self.execute_filter(array, &filter.predicate, context).await
            }

            ILExpression::ReduceOp { reduce } => {
                let array = self.execute(&reduce.array, context).await?;
                let initial = self.execute(&reduce.initial, context).await?;
                self.execute_reduce(array, &reduce.function, initial, context)
                    .await
            }
        }
    }

    /// Execute an IL plan
    async fn execute_plan(&self, plan: &ILPlan, context: &mut ILContext) -> Result<Value, ILError> {
        // Execute all operations in sequence
        for operation in &plan.operations {
            match operation {
                ILOperation::Store { store } => {
                    let value = self.execute(&store.value, context).await?;
                    context.set_variable(store.slot, value)?;
                }
                ILOperation::Execute { execute } => {
                    // Execute but discard result
                    self.execute(execute, context).await?;
                }
            }
        }

        // Return the final result
        self.execute(&plan.result, context).await
    }

    /// Execute a map operation on an array
    async fn execute_map(
        &self,
        array: Value,
        function: &ILExpression,
        context: &mut ILContext,
    ) -> Result<Value, ILError> {
        match array {
            Value::Array(items) => {
                let mut results = Vec::new();
                for item in items {
                    // Create a new binding for the current item
                    let var_index = context.push_variable(item);

                    // Apply the function with the item bound as a variable
                    let result = if let ILExpression::Bind { .. } = function {
                        // If the function is already a bind, execute it directly
                        self.execute(function, context).await?
                    } else {
                        // Otherwise wrap it in a bind with the current item
                        let bind_expr =
                            ILExpression::bind(ILExpression::var(var_index), function.clone());
                        self.execute(&bind_expr, context).await?
                    };

                    results.push(result);
                }
                Ok(Value::Array(results))
            }
            _ => Err(ILError::TypeError {
                expected: "array".to_string(),
                actual: self.value_type_name(&array),
            }),
        }
    }

    /// Execute a filter operation on an array
    async fn execute_filter(
        &self,
        array: Value,
        predicate: &ILExpression,
        context: &mut ILContext,
    ) -> Result<Value, ILError> {
        match array {
            Value::Array(items) => {
                let mut results = Vec::new();
                for item in items {
                    let var_index = context.push_variable(item.clone());

                    let condition = if let ILExpression::Bind { .. } = predicate {
                        self.execute(predicate, context).await?
                    } else {
                        let bind_expr =
                            ILExpression::bind(ILExpression::var(var_index), predicate.clone());
                        self.execute(&bind_expr, context).await?
                    };

                    if self.is_truthy(&condition) {
                        results.push(item);
                    }
                }
                Ok(Value::Array(results))
            }
            _ => Err(ILError::TypeError {
                expected: "array".to_string(),
                actual: self.value_type_name(&array),
            }),
        }
    }

    /// Execute a reduce operation on an array
    async fn execute_reduce(
        &self,
        array: Value,
        function: &ILExpression,
        mut accumulator: Value,
        context: &mut ILContext,
    ) -> Result<Value, ILError> {
        match array {
            Value::Array(items) => {
                for item in items {
                    // Bind both accumulator and current item
                    let acc_index = context.push_variable(accumulator.clone());
                    let item_index = context.push_variable(item);

                    // Create a nested bind for both parameters
                    let bind_expr = ILExpression::bind(
                        ILExpression::var(acc_index),
                        ILExpression::bind(ILExpression::var(item_index), function.clone()),
                    );

                    accumulator = self.execute(&bind_expr, context).await?;
                }
                Ok(accumulator)
            }
            _ => Err(ILError::TypeError {
                expected: "array".to_string(),
                actual: self.value_type_name(&array),
            }),
        }
    }

    /// Get a property from a value
    fn get_property(&self, object: &Value, property: &str) -> Result<Value, ILError> {
        match object {
            Value::Object(map) => map.get(property).cloned().ok_or_else(|| {
                ILError::ExecutionFailed(format!("Property '{}' not found", property))
            }),
            _ => Err(ILError::TypeError {
                expected: "object".to_string(),
                actual: self.value_type_name(object),
            }),
        }
    }

    /// Call a method on a capability
    async fn call_method(
        &self,
        _target: Value,
        _method: &str,
        _args: Vec<Value>,
    ) -> Result<Value, ILError> {
        // This would need to resolve the target to a capability
        // For now, return a placeholder
        Err(ILError::ExecutionFailed(
            "Method calls require capability resolution (not yet implemented)".to_string(),
        ))
    }

    /// Check if a value is truthy
    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(_) => true,
        }
    }

    /// Get a human-readable type name for a value
    fn value_type_name(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
}

impl Default for ILExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::il_extended::FilterExpression;
    use serde_json::json;

    #[tokio::test]
    async fn test_execute_literal() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        let expr = ILExpression::literal(json!(42));
        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!(42));
    }

    #[tokio::test]
    async fn test_execute_variable() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);
        context.set_variable(0, json!("hello")).unwrap();

        let expr = ILExpression::var(0);
        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[tokio::test]
    async fn test_execute_bind() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        let expr = ILExpression::bind(
            ILExpression::literal(json!(100)),
            ILExpression::var(0), // Reference the bound variable
        );

        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!(100));
    }

    #[tokio::test]
    async fn test_execute_if() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        // Test true condition
        let expr = ILExpression::if_expr(
            ILExpression::literal(json!(true)),
            ILExpression::literal(json!("then")),
            ILExpression::literal(json!("else")),
        );
        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!("then"));

        // Test false condition
        let expr = ILExpression::if_expr(
            ILExpression::literal(json!(false)),
            ILExpression::literal(json!("then")),
            ILExpression::literal(json!("else")),
        );
        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!("else"));
    }

    #[tokio::test]
    async fn test_execute_get() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        let expr = ILExpression::get(
            ILExpression::literal(json!({
                "name": "John",
                "age": 30
            })),
            "name".to_string(),
        );

        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert_eq!(result, json!("John"));
    }

    #[tokio::test]
    async fn test_execute_map() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        // Map that doubles each number
        let expr = ILExpression::map(
            ILExpression::literal(json!([1, 2, 3])),
            // This would normally be a more complex expression
            // For testing, we'll just return the item as-is
            ILExpression::var(0),
        );

        // Note: This test is simplified - in reality, the map function
        // would need proper transformation logic
        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        assert!(matches!(result, Value::Array(_)));
    }

    #[tokio::test]
    async fn test_execute_filter() {
        let executor = ILExecutor::new();
        let mut context = ILContext::new(vec![]);

        // Filter to keep only true values
        let expr = ILExpression::FilterOp {
            filter: FilterExpression {
                array: Box::new(ILExpression::literal(json!([true, false, true]))),
                predicate: Box::new(ILExpression::var(0)), // Use the item itself as the predicate
            },
        };

        let result = executor.execute_impl(&expr, &mut context).await.unwrap();
        match result {
            Value::Array(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], json!(true));
                assert_eq!(items[1], json!(true));
            }
            _ => panic!("Expected array result"),
        }
    }
}
