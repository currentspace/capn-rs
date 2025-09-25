// Cap'n Web IL (Intermediate Language) Execution Tests
// Tests the complete IL plan execution and operation features
// Specification: IL enables complex operations to be executed efficiently

use capnweb_core::{
    il::{Source, Op, Plan, CallOp, ObjectOp, ArrayOp, CaptureRef, ResultRef, ParamRef, ValueRef},
    CapId, Expression, Value, ImportId, ExportId, RpcTarget, RpcError,
    protocol::il_runner::{IlRunner, IlRunnerError, ExecutionContext},
};
use serde_json::{json, Number};
use std::sync::Arc;
use std::collections::{HashMap, BTreeMap};
use tokio::sync::Mutex;

#[cfg(test)]
mod il_basic_tests {
    use super::*;

    #[derive(Debug)]
    struct TestCapability {
        name: String,
        data: HashMap<String, Value>,
    }

    #[async_trait::async_trait]
    impl RpcTarget for TestCapability {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            match method {
                "add" => {
                    if args.len() != 2 {
                        return Err(RpcError::InvalidArguments("add requires 2 arguments".into()));
                    }
                    match (&args[0], &args[1]) {
                        (Value::Number(a), Value::Number(b)) => {
                            let sum = a.as_f64().unwrap() + b.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(sum).unwrap()))
                        }
                        _ => Err(RpcError::InvalidArguments("add requires numbers".into()))
                    }
                }
                "multiply" => {
                    if args.len() != 2 {
                        return Err(RpcError::InvalidArguments("multiply requires 2 arguments".into()));
                    }
                    match (&args[0], &args[1]) {
                        (Value::Number(a), Value::Number(b)) => {
                            let product = a.as_f64().unwrap() * b.as_f64().unwrap();
                            Ok(Value::Number(Number::from_f64(product).unwrap()))
                        }
                        _ => Err(RpcError::InvalidArguments("multiply requires numbers".into()))
                    }
                }
                "concat" => {
                    let result = args.iter()
                        .filter_map(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            _ => None
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    Ok(Value::String(result))
                }
                "getData" => {
                    Ok(Value::Object(self.data.clone()))
                }
                _ => Err(RpcError::MethodNotFound(method.to_string()))
            }
        }

        async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
            self.data.get(property)
                .cloned()
                .ok_or_else(|| RpcError::PropertyNotFound(property.to_string()))
        }
    }

    /// Test basic IL plan creation and structure
    #[tokio::test]
    async fn test_il_plan_creation() {
        println!("🧪 Testing IL Plan Creation");

        // Create a simple IL plan
        let plan = Plan {
            captures: vec![CapId(1), CapId(2)],
            ops: vec![
                Op::Call {
                    call: CallOp {
                        target: Source::capture(0),
                        member: "add".to_string(),
                        args: vec![
                            Source::by_value(Value::Number(Number::from(10))),
                            Source::by_value(Value::Number(Number::from(20))),
                        ],
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        assert_eq!(plan.captures.len(), 2);
        assert_eq!(plan.ops.len(), 1);
        println!("✅ IL plan structure verified");

        // Test serialization
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, deserialized);
        println!("✅ IL plan serialization verified");
    }

    /// Test IL source types
    #[tokio::test]
    async fn test_il_source_types() {
        println!("🧪 Testing IL Source Types");

        // Test capture source
        let capture = Source::capture(42);
        assert_eq!(capture.get_capture_index(), Some(42));
        println!("✅ Capture source verified");

        // Test result source
        let result = Source::result(10);
        assert_eq!(result.get_result_index(), Some(10));
        println!("✅ Result source verified");

        // Test param source
        let param = Source::param(vec!["user".to_string(), "name".to_string()]);
        match param {
            Source::Param { param: ref p } => {
                assert_eq!(p.path, vec!["user", "name"]);
            }
            _ => panic!("Should be param source")
        }
        println!("✅ Param source verified");

        // Test by-value source
        let by_value = Source::by_value(Value::String("test".to_string()));
        match by_value {
            Source::ByValue { by_value: ref v } => {
                assert_eq!(v.value, Value::String("test".to_string()));
            }
            _ => panic!("Should be by-value source")
        }
        println!("✅ By-value source verified");
    }

    /// Test IL call operations
    #[tokio::test]
    async fn test_il_call_operations() {
        println!("🧪 Testing IL Call Operations");

        let mut runner = IlRunner::new();

        // Set up test capability
        let mut data = HashMap::new();
        data.insert("count".to_string(), Value::Number(Number::from(42)));
        let cap = Arc::new(TestCapability {
            name: "calculator".to_string(),
            data,
        });

        // Create execution context
        let mut context = ExecutionContext::new();
        context.add_capture(CapId(1), cap.clone());

        // Create call operation plan
        let plan = Plan {
            captures: vec![CapId(1)],
            ops: vec![
                Op::Call {
                    call: CallOp {
                        target: Source::capture(0),
                        member: "add".to_string(),
                        args: vec![
                            Source::by_value(Value::Number(Number::from(5))),
                            Source::by_value(Value::Number(Number::from(3))),
                        ],
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await;
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Number(n) => {
                assert_eq!(n.as_f64().unwrap(), 8.0);
            }
            _ => panic!("Should return number")
        }
        println!("✅ IL call operation verified");
    }

    /// Test IL object operations
    #[tokio::test]
    async fn test_il_object_operations() {
        println!("🧪 Testing IL Object Operations");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Create object operation
        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), Source::by_value(Value::String("Alice".to_string())));
        fields.insert("age".to_string(), Source::by_value(Value::Number(Number::from(30))));
        fields.insert("active".to_string(), Source::by_value(Value::Bool(true)));

        let plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Object {
                    object: ObjectOp {
                        fields: fields.clone(),
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("name"), Some(&Value::String("Alice".to_string())));
                assert_eq!(obj.get("age"), Some(&Value::Number(Number::from(30))));
                assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
            }
            _ => panic!("Should return object")
        }
        println!("✅ IL object operation verified");
    }

    /// Test IL array operations
    #[tokio::test]
    async fn test_il_array_operations() {
        println!("🧪 Testing IL Array Operations");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Create array operation
        let items = vec![
            Source::by_value(Value::String("item1".to_string())),
            Source::by_value(Value::Number(Number::from(42))),
            Source::by_value(Value::Bool(true)),
            Source::by_value(Value::Null),
        ];

        let plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Array {
                    array: ArrayOp {
                        items: items.clone(),
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr[0], Value::String("item1".to_string()));
                assert_eq!(arr[1], Value::Number(Number::from(42)));
                assert_eq!(arr[2], Value::Bool(true));
                assert_eq!(arr[3], Value::Null);
            }
            _ => panic!("Should return array")
        }
        println!("✅ IL array operation verified");
    }
}

#[cfg(test)]
mod il_complex_tests {
    use super::*;

    /// Test complex IL plan with multiple operations
    #[tokio::test]
    async fn test_complex_il_plan_execution() {
        println!("🧪 Testing Complex IL Plan Execution");

        let mut runner = IlRunner::new();

        // Set up test capabilities
        let mut data1 = HashMap::new();
        data1.insert("value".to_string(), Value::Number(Number::from(10)));
        let cap1 = Arc::new(TestCapability {
            name: "calc1".to_string(),
            data: data1,
        });

        let mut data2 = HashMap::new();
        data2.insert("value".to_string(), Value::Number(Number::from(20)));
        let cap2 = Arc::new(TestCapability {
            name: "calc2".to_string(),
            data: data2,
        });

        let mut context = ExecutionContext::new();
        context.add_capture(CapId(1), cap1);
        context.add_capture(CapId(2), cap2);

        // Complex plan:
        // 1. Call add on cap1 with values 5 and 3
        // 2. Call multiply on cap2 with result of step 1 and 2
        // 3. Create object with both results
        let plan = Plan {
            captures: vec![CapId(1), CapId(2)],
            ops: vec![
                // Op 0: cap1.add(5, 3) -> 8
                Op::Call {
                    call: CallOp {
                        target: Source::capture(0),
                        member: "add".to_string(),
                        args: vec![
                            Source::by_value(Value::Number(Number::from(5))),
                            Source::by_value(Value::Number(Number::from(3))),
                        ],
                        result: 0,
                    }
                },
                // Op 1: cap2.multiply(result[0], 2) -> 16
                Op::Call {
                    call: CallOp {
                        target: Source::capture(1),
                        member: "multiply".to_string(),
                        args: vec![
                            Source::result(0),
                            Source::by_value(Value::Number(Number::from(2))),
                        ],
                        result: 1,
                    }
                },
                // Op 2: Create object with results
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("addition".to_string(), Source::result(0));
                            fields.insert("multiplication".to_string(), Source::result(1));
                            fields.insert("original".to_string(), Source::by_value(Value::String("test".to_string())));
                            fields
                        },
                        result: 2,
                    }
                }
            ],
            result: Source::result(2),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("addition"), Some(&Value::Number(Number::from(8))));
                assert_eq!(obj.get("multiplication"), Some(&Value::Number(Number::from(16))));
                assert_eq!(obj.get("original"), Some(&Value::String("test".to_string())));
            }
            _ => panic!("Should return object")
        }
        println!("✅ Complex IL plan execution verified");
    }

    /// Test IL plan with parameter references
    #[tokio::test]
    async fn test_il_with_parameters() {
        println!("🧪 Testing IL with Parameters");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Parameters passed to the plan
        let mut params = HashMap::new();
        params.insert("user".to_string(), Value::Object({
            let mut user = HashMap::new();
            user.insert("name".to_string(), Value::String("Bob".to_string()));
            user.insert("age".to_string(), Value::Number(Number::from(25)));
            user
        }));
        params.insert("config".to_string(), Value::Object({
            let mut config = HashMap::new();
            config.insert("debug".to_string(), Value::Bool(true));
            config.insert("timeout".to_string(), Value::Number(Number::from(5000)));
            config
        }));

        // Plan that uses parameter references
        let plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("userName".to_string(), Source::param(vec!["user".to_string(), "name".to_string()]));
                            fields.insert("userAge".to_string(), Source::param(vec!["user".to_string(), "age".to_string()]));
                            fields.insert("debugMode".to_string(), Source::param(vec!["config".to_string(), "debug".to_string()]));
                            fields.insert("static".to_string(), Source::by_value(Value::String("fixed".to_string())));
                            fields
                        },
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&plan, &mut context, &params).await.unwrap();

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("userName"), Some(&Value::String("Bob".to_string())));
                assert_eq!(obj.get("userAge"), Some(&Value::Number(Number::from(25))));
                assert_eq!(obj.get("debugMode"), Some(&Value::Bool(true)));
                assert_eq!(obj.get("static"), Some(&Value::String("fixed".to_string())));
            }
            _ => panic!("Should return object")
        }
        println!("✅ IL parameter references verified");
    }

    /// Test IL plan with nested operations
    #[tokio::test]
    async fn test_nested_il_operations() {
        println!("🧪 Testing Nested IL Operations");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Create nested structure plan
        let plan = Plan {
            captures: vec![],
            ops: vec![
                // Create inner array
                Op::Array {
                    array: ArrayOp {
                        items: vec![
                            Source::by_value(Value::Number(Number::from(1))),
                            Source::by_value(Value::Number(Number::from(2))),
                            Source::by_value(Value::Number(Number::from(3))),
                        ],
                        result: 0,
                    }
                },
                // Create inner object
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("type".to_string(), Source::by_value(Value::String("nested".to_string())));
                            fields.insert("count".to_string(), Source::by_value(Value::Number(Number::from(3))));
                            fields
                        },
                        result: 1,
                    }
                },
                // Create outer object with nested structures
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("array".to_string(), Source::result(0));
                            fields.insert("metadata".to_string(), Source::result(1));
                            fields.insert("version".to_string(), Source::by_value(Value::Number(Number::from(1))));
                            fields
                        },
                        result: 2,
                    }
                }
            ],
            result: Source::result(2),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Object(obj) => {
                // Check array field
                match obj.get("array") {
                    Some(Value::Array(arr)) => {
                        assert_eq!(arr.len(), 3);
                        assert_eq!(arr[0], Value::Number(Number::from(1)));
                    }
                    _ => panic!("Should have array field")
                }

                // Check metadata field
                match obj.get("metadata") {
                    Some(Value::Object(meta)) => {
                        assert_eq!(meta.get("type"), Some(&Value::String("nested".to_string())));
                        assert_eq!(meta.get("count"), Some(&Value::Number(Number::from(3))));
                    }
                    _ => panic!("Should have metadata object")
                }

                assert_eq!(obj.get("version"), Some(&Value::Number(Number::from(1))));
            }
            _ => panic!("Should return object")
        }
        println!("✅ Nested IL operations verified");
    }

    /// Test IL error handling
    #[tokio::test]
    async fn test_il_error_handling() {
        println!("🧪 Testing IL Error Handling");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Plan with invalid capture reference
        let invalid_plan = Plan {
            captures: vec![CapId(1)], // Declared but not in context
            ops: vec![
                Op::Call {
                    call: CallOp {
                        target: Source::capture(0),
                        member: "test".to_string(),
                        args: vec![],
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&invalid_plan, &mut context, &HashMap::new()).await;
        assert!(result.is_err());
        println!("✅ Missing capture error handling verified");

        // Plan with invalid result reference
        let invalid_result_plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Array {
                    array: ArrayOp {
                        items: vec![Source::by_value(Value::Number(Number::from(1)))],
                        result: 0,
                    }
                }
            ],
            result: Source::result(1), // References non-existent result
        };

        let result = runner.execute(&invalid_result_plan, &mut context, &HashMap::new()).await;
        assert!(result.is_err());
        println!("✅ Invalid result reference error handling verified");

        // Plan with invalid parameter path
        let invalid_param_plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("value".to_string(), Source::param(vec!["nonexistent".to_string(), "path".to_string()]));
                            fields
                        },
                        result: 0,
                    }
                }
            ],
            result: Source::result(0),
        };

        let result = runner.execute(&invalid_param_plan, &mut context, &HashMap::new()).await;
        assert!(result.is_err());
        println!("✅ Invalid parameter path error handling verified");
    }
}

#[cfg(test)]
mod il_optimization_tests {
    use super::*;

    /// Test IL plan optimization and efficiency
    #[tokio::test]
    async fn test_il_plan_optimization() {
        println!("🧪 Testing IL Plan Optimization");

        // Test result reuse - operations can reference previous results
        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Plan that reuses intermediate results multiple times
        let plan = Plan {
            captures: vec![],
            ops: vec![
                // Op 0: Create base value
                Op::Array {
                    array: ArrayOp {
                        items: vec![
                            Source::by_value(Value::Number(Number::from(10))),
                            Source::by_value(Value::Number(Number::from(20))),
                        ],
                        result: 0,
                    }
                },
                // Op 1: Reference result 0 multiple times
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("original".to_string(), Source::result(0));
                            fields.insert("copy1".to_string(), Source::result(0));
                            fields.insert("copy2".to_string(), Source::result(0));
                            fields
                        },
                        result: 1,
                    }
                }
            ],
            result: Source::result(1),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Object(obj) => {
                let expected_array = Value::Array(vec![
                    Value::Number(Number::from(10)),
                    Value::Number(Number::from(20))
                ]);
                assert_eq!(obj.get("original"), Some(&expected_array));
                assert_eq!(obj.get("copy1"), Some(&expected_array));
                assert_eq!(obj.get("copy2"), Some(&expected_array));
            }
            _ => panic!("Should return object")
        }
        println!("✅ IL result reuse optimization verified");
    }

    /// Test IL plan with large data structures
    #[tokio::test]
    async fn test_il_with_large_data() {
        println!("🧪 Testing IL with Large Data Structures");

        let mut runner = IlRunner::new();
        let mut context = ExecutionContext::new();

        // Create plan with large array
        let large_array_items: Vec<Source> = (0..1000)
            .map(|i| Source::by_value(Value::Number(Number::from(i))))
            .collect();

        let plan = Plan {
            captures: vec![],
            ops: vec![
                Op::Array {
                    array: ArrayOp {
                        items: large_array_items,
                        result: 0,
                    }
                },
                Op::Object {
                    object: ObjectOp {
                        fields: {
                            let mut fields = BTreeMap::new();
                            fields.insert("data".to_string(), Source::result(0));
                            fields.insert("count".to_string(), Source::by_value(Value::Number(Number::from(1000))));
                            fields
                        },
                        result: 1,
                    }
                }
            ],
            result: Source::result(1),
        };

        let result = runner.execute(&plan, &mut context, &HashMap::new()).await.unwrap();

        match result {
            Value::Object(obj) => {
                match obj.get("data") {
                    Some(Value::Array(arr)) => {
                        assert_eq!(arr.len(), 1000);
                        // Spot check some values
                        assert_eq!(arr[0], Value::Number(Number::from(0)));
                        assert_eq!(arr[500], Value::Number(Number::from(500)));
                        assert_eq!(arr[999], Value::Number(Number::from(999)));
                    }
                    _ => panic!("Should have data array")
                }
                assert_eq!(obj.get("count"), Some(&Value::Number(Number::from(1000))));
            }
            _ => panic!("Should return object")
        }
        println!("✅ IL with large data structures verified");
    }
}