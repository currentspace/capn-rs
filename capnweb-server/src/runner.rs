use capnweb_core::{Plan, Source, Op, CapId, RpcError};
use serde_json::{Value, Map};
use std::collections::HashMap;
use crate::{RpcTarget, ServerConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Executes IL Plans against capabilities
pub struct PlanRunner {
    /// Configuration for the runner
    #[allow(dead_code)]
    config: ServerConfig,
}

impl PlanRunner {
    /// Create a new plan runner
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Execute a plan with captured capabilities
    pub async fn execute(
        &self,
        plan: &Plan,
        params: Option<Value>,
        captures: &HashMap<u32, Arc<RwLock<dyn RpcTarget>>>,
    ) -> Result<Value, RpcError> {
        // Validate the plan
        plan.validate()
            .map_err(|e| RpcError::bad_request(format!("Invalid plan: {}", e)))?;

        // Track results from operations
        let mut results: HashMap<u32, Value> = HashMap::new();

        // Execute operations in order
        for op in &plan.ops {
            let result = self.execute_op(op, params.as_ref(), captures, &results).await?;

            match op {
                Op::Call { result: idx, .. } |
                Op::Object { result: idx, .. } |
                Op::Array { result: idx, .. } => {
                    results.insert(*idx, result);
                }
            }
        }

        // Get the final result
        self.resolve_source(&plan.result, params.as_ref(), captures, &results)
    }

    /// Execute a single operation
    async fn execute_op(
        &self,
        op: &Op,
        params: Option<&Value>,
        captures: &HashMap<u32, Arc<RwLock<dyn RpcTarget>>>,
        results: &HashMap<u32, Value>,
    ) -> Result<Value, RpcError> {
        match op {
            Op::Call { target, member, args, .. } => {
                let target_value = self.resolve_source(target, params, captures, results)?;

                // Get the capability ID from the target value
                let cap_id = if let Value::Object(obj) = &target_value {
                    if let Some(Value::Number(n)) = obj.get("cap") {
                        CapId::new(n.as_u64().ok_or_else(||
                            RpcError::bad_request("Invalid capability ID")
                        )?)
                    } else {
                        return Err(RpcError::bad_request("Target is not a capability"));
                    }
                } else {
                    return Err(RpcError::bad_request("Target is not a capability"));
                };

                // Get the capability from captures
                let capability = captures
                    .get(&(cap_id.as_u64() as u32))
                    .ok_or_else(|| RpcError::not_found(format!("Capability not found: {:?}", cap_id)))?;

                // Resolve arguments
                let mut resolved_args = Vec::new();
                for arg_source in args {
                    resolved_args.push(self.resolve_source(arg_source, params, captures, results)?);
                }

                // Call the method on the capability
                let target = capability.read().await;
                target.call(member, resolved_args).await
            }

            Op::Object { fields, .. } => {
                let mut object = Map::new();
                for (key, source) in fields {
                    let value = self.resolve_source(source, params, captures, results)?;
                    object.insert(key.clone(), value);
                }
                Ok(Value::Object(object))
            }

            Op::Array { items, .. } => {
                let mut array = Vec::new();
                for source in items {
                    array.push(self.resolve_source(source, params, captures, results)?);
                }
                Ok(Value::Array(array))
            }
        }
    }

    /// Resolve a source to its value
    fn resolve_source(
        &self,
        source: &Source,
        params: Option<&Value>,
        captures: &HashMap<u32, Arc<RwLock<dyn RpcTarget>>>,
        results: &HashMap<u32, Value>,
    ) -> Result<Value, RpcError> {
        match source {
            Source::Capture { index } => {
                // Convert capability to a reference value
                captures
                    .get(index)
                    .map(|_| {
                        // Return a capability reference
                        serde_json::json!({ "cap": *index })
                    })
                    .ok_or_else(|| RpcError::not_found(format!("Capture {} not found", index)))
            }

            Source::Result { index } => {
                results
                    .get(index)
                    .cloned()
                    .ok_or_else(|| RpcError::not_found(format!("Result {} not found", index)))
            }

            Source::Param { path } => {
                let params = params.ok_or_else(||
                    RpcError::bad_request("No parameters provided")
                )?;

                // Navigate the path through the params
                let mut current = params;
                for segment in path {
                    current = current
                        .get(segment)
                        .ok_or_else(|| RpcError::bad_request(
                            format!("Parameter path not found: {}", path.join("."))
                        ))?;
                }
                Ok(current.clone())
            }

            Source::ByValue { value } => Ok(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use capnweb_core::{Plan, Source, Op};
    use async_trait::async_trait;

    /// Test implementation of RpcTarget
    struct TestTarget {
        name: String,
    }

    #[async_trait]
    impl RpcTarget for TestTarget {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            match method {
                "getName" => Ok(Value::String(self.name.clone())),
                "add" => {
                    if args.len() != 2 {
                        return Err(RpcError::bad_request("add requires 2 arguments"));
                    }
                    let a = args[0].as_f64().ok_or_else(||
                        RpcError::bad_request("First argument must be a number")
                    )?;
                    let b = args[1].as_f64().ok_or_else(||
                        RpcError::bad_request("Second argument must be a number")
                    )?;
                    Ok(serde_json::json!(a + b))
                }
                "echo" => {
                    Ok(args.first().cloned().unwrap_or(Value::Null))
                }
                _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
            }
        }
    }

    #[tokio::test]
    async fn test_execute_simple_call() {
        let runner = PlanRunner::new(ServerConfig::default());

        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![Op::Call {
                target: Source::Capture { index: 0 },
                member: "getName".to_string(),
                args: vec![],
                result: 0,
            }],
            Source::Result { index: 0 },
        );

        let mut captures = HashMap::new();
        captures.insert(0, Arc::new(RwLock::new(TestTarget {
            name: "test".to_string(),
        })) as Arc<RwLock<dyn RpcTarget>>);

        let result = runner.execute(&plan, None, &captures).await.unwrap();
        assert_eq!(result, Value::String("test".to_string()));
    }

    #[tokio::test]
    async fn test_execute_with_params() {
        let runner = PlanRunner::new(ServerConfig::default());

        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![Op::Call {
                target: Source::Capture { index: 0 },
                member: "add".to_string(),
                args: vec![
                    Source::Param { path: vec!["a".to_string()] },
                    Source::Param { path: vec!["b".to_string()] },
                ],
                result: 0,
            }],
            Source::Result { index: 0 },
        );

        let mut captures = HashMap::new();
        captures.insert(0, Arc::new(RwLock::new(TestTarget {
            name: "calculator".to_string(),
        })) as Arc<RwLock<dyn RpcTarget>>);

        let params = serde_json::json!({
            "a": 5,
            "b": 3
        });

        let result = runner.execute(&plan, Some(params), &captures).await.unwrap();
        assert_eq!(result, serde_json::json!(8.0));
    }

    #[tokio::test]
    async fn test_execute_object_construction() {
        let runner = PlanRunner::new(ServerConfig::default());

        let plan = Plan::new(
            vec![],
            vec![Op::Object {
                fields: vec![
                    ("name".to_string(), Source::ByValue { value: Value::String("test".to_string()) }),
                    ("value".to_string(), Source::ByValue { value: serde_json::json!(42) }),
                ].into_iter().collect(),
                result: 0,
            }],
            Source::Result { index: 0 },
        );

        let captures = HashMap::new();
        let result = runner.execute(&plan, None, &captures).await.unwrap();

        assert_eq!(result, serde_json::json!({
            "name": "test",
            "value": 42
        }));
    }

    #[tokio::test]
    async fn test_execute_array_construction() {
        let runner = PlanRunner::new(ServerConfig::default());

        let plan = Plan::new(
            vec![],
            vec![Op::Array {
                items: vec![
                    Source::ByValue { value: serde_json::json!(1) },
                    Source::ByValue { value: serde_json::json!(2) },
                    Source::ByValue { value: serde_json::json!(3) },
                ],
                result: 0,
            }],
            Source::Result { index: 0 },
        );

        let captures = HashMap::new();
        let result = runner.execute(&plan, None, &captures).await.unwrap();

        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[tokio::test]
    async fn test_execute_chained_operations() {
        let runner = PlanRunner::new(ServerConfig::default());

        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "echo".to_string(),
                    args: vec![Source::ByValue { value: Value::String("hello".to_string()) }],
                    result: 0,
                },
                Op::Object {
                    fields: vec![
                        ("message".to_string(), Source::Result { index: 0 }),
                        ("timestamp".to_string(), Source::ByValue { value: serde_json::json!(12345) }),
                    ].into_iter().collect(),
                    result: 1,
                }
            ],
            Source::Result { index: 1 },
        );

        let mut captures = HashMap::new();
        captures.insert(0, Arc::new(RwLock::new(TestTarget {
            name: "echo".to_string(),
        })) as Arc<RwLock<dyn RpcTarget>>);

        let result = runner.execute(&plan, None, &captures).await.unwrap();

        assert_eq!(result, serde_json::json!({
            "message": "hello",
            "timestamp": 12345
        }));
    }

    #[tokio::test]
    async fn test_invalid_plan() {
        let runner = PlanRunner::new(ServerConfig::default());

        // Invalid plan with forward reference
        let plan = Plan::new(
            vec![],
            vec![Op::Call {
                target: Source::Result { index: 1 }, // Forward reference
                member: "test".to_string(),
                args: vec![],
                result: 0,
            }],
            Source::Result { index: 0 },
        );

        let captures = HashMap::new();
        let result = runner.execute(&plan, None, &captures).await;

        assert!(result.is_err());
    }
}