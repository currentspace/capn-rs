// Comprehensive integration tests for Cap'n Web Advanced Features
// Tests Resume Tokens, Nested Capabilities, Advanced IL Plan Runner, and transport integration

use capnweb_core::{
    protocol::{
        PlanRunner, PlanBuilder,
        ResumeTokenManager, PersistentSessionManager,
        CapabilityFactory, CapabilityMetadata, MethodMetadata, ParameterMetadata,
        CapabilityGraph, DefaultNestedCapableTarget, NestedCapableRpcTarget,
        ImportTable, ExportTable, IdAllocator, VariableStateManager,
        tables::Value
    },
    il::{Plan, Source, Op},
    RpcTarget, RpcError, CapId
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{json, Number};
use tokio::sync::RwLock;

// Mock RPC target for testing
#[derive(Debug)]
struct MockRpcTarget;

impl MockRpcTarget {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RpcTarget for MockRpcTarget {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        Ok(Value::String(format!("Mock call to {} with {} args", method, args.len())))
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        Ok(Value::String(format!("Mock property {}", property)))
    }
}

/// Advanced Calculator capability that supports nested operations
#[derive(Debug)]
struct AdvancedCalculatorCapability {
    precision: u32,
    history: Arc<RwLock<Vec<String>>>,
}

impl AdvancedCalculatorCapability {
    fn new(precision: u32) -> Self {
        Self {
            precision,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl RpcTarget for AdvancedCalculatorCapability {
    async fn call(&self, method: &str, args: Vec<capnweb_core::protocol::Value>) -> Result<capnweb_core::protocol::Value, RpcError> {
        use capnweb_core::protocol::Value;

        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires 2 arguments"));
                }

                let a = match &args[0] {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => return Err(RpcError::bad_request("First argument must be a number")),
                };

                let b = match &args[1] {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => return Err(RpcError::bad_request("Second argument must be a number")),
                };

                let result = a + b;

                // Add to history
                let operation = format!("{} + {} = {}", a, b, result);
                self.history.write().await.push(operation);

                Ok(Value::Number(Number::from_f64(result).unwrap()))
            }

            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires 2 arguments"));
                }

                let a = match &args[0] {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => return Err(RpcError::bad_request("First argument must be a number")),
                };

                let b = match &args[1] {
                    Value::Number(n) => n.as_f64().unwrap_or(0.0),
                    _ => return Err(RpcError::bad_request("Second argument must be a number")),
                };

                let result = a * b;

                // Add to history
                let operation = format!("{} * {} = {}", a, b, result);
                self.history.write().await.push(operation);

                Ok(Value::Number(Number::from_f64(result).unwrap()))
            }

            "getHistory" => {
                let history = self.history.read().await;
                let history_values: Vec<Value> = history.iter()
                    .map(|s| Value::String(s.clone()))
                    .collect();
                Ok(Value::Array(history_values))
            }

            "getPrecision" => {
                Ok(Value::Number(Number::from(self.precision)))
            }

            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<capnweb_core::protocol::Value, RpcError> {
        use capnweb_core::protocol::Value;

        match property {
            "precision" => Ok(Value::Number(Number::from(self.precision))),
            "type" => Ok(Value::String("AdvancedCalculator".to_string())),
            _ => Err(RpcError::not_found(format!("Property not found: {}", property))),
        }
    }
}

/// Mock factory for creating calculator capabilities
#[derive(Debug)]
struct CalculatorFactory;

#[async_trait::async_trait]
impl CapabilityFactory for CalculatorFactory {
    async fn create_capability(
        &self,
        capability_type: &str,
        config: capnweb_core::protocol::Value,
    ) -> Result<Arc<dyn RpcTarget>, capnweb_core::protocol::CapabilityError> {
        use capnweb_core::protocol::{Value, CapabilityError};

        match capability_type {
            "calculator" => {
                let precision = match config {
                    Value::Object(ref obj) => {
                        match obj.get("precision") {
                            Some(boxed_val) => match boxed_val.as_ref() {
                                Value::Number(n) => n.as_u64().unwrap_or(2) as u32,
                                _ => 2,
                            },
                            None => 2,
                        }
                    },
                    _ => 2,
                };

                Ok(Arc::new(AdvancedCalculatorCapability::new(precision)))
            }
            "advanced_calculator" => {
                let precision = match config {
                    Value::Object(ref obj) => {
                        match obj.get("precision") {
                            Some(boxed_val) => match boxed_val.as_ref() {
                                Value::Number(n) => n.as_u64().unwrap_or(4) as u32,
                                _ => 4,
                            },
                            None => 4,
                        }
                    },
                    _ => 4,
                };

                Ok(Arc::new(AdvancedCalculatorCapability::new(precision)))
            }
            _ => Err(CapabilityError::InvalidCapabilityType(capability_type.to_string())),
        }
    }

    fn list_capability_types(&self) -> Vec<String> {
        vec!["calculator".to_string(), "advanced_calculator".to_string()]
    }

    fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata> {
        match capability_type {
            "calculator" | "advanced_calculator" => Some(CapabilityMetadata {
                name: "Calculator".to_string(),
                description: "Mathematical calculation capability".to_string(),
                version: "2.0.0".to_string(),
                methods: vec![
                    MethodMetadata {
                        name: "add".to_string(),
                        description: "Add two numbers".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "a".to_string(),
                                type_name: "number".to_string(),
                                required: true,
                                description: "First number".to_string(),
                            },
                            ParameterMetadata {
                                name: "b".to_string(),
                                type_name: "number".to_string(),
                                required: true,
                                description: "Second number".to_string(),
                            },
                        ],
                        return_type: "number".to_string(),
                    },
                    MethodMetadata {
                        name: "multiply".to_string(),
                        description: "Multiply two numbers".to_string(),
                        parameters: vec![
                            ParameterMetadata {
                                name: "a".to_string(),
                                type_name: "number".to_string(),
                                required: true,
                                description: "First number".to_string(),
                            },
                            ParameterMetadata {
                                name: "b".to_string(),
                                type_name: "number".to_string(),
                                required: true,
                                description: "Second number".to_string(),
                            },
                        ],
                        return_type: "number".to_string(),
                    },
                    MethodMetadata {
                        name: "getHistory".to_string(),
                        description: "Get calculation history".to_string(),
                        parameters: vec![],
                        return_type: "array".to_string(),
                    },
                ],
                config_schema: Some(capnweb_core::protocol::Value::Object({
                    let mut schema = HashMap::new();
                    schema.insert("precision".to_string(), Box::new(capnweb_core::protocol::Value::String("number".to_string())));
                    schema
                })),
            }),
            _ => None,
        }
    }
}

#[tokio::test]
async fn test_comprehensive_advanced_features_integration() {
    // Initialize core components
    let imports = Arc::new(ImportTable::with_default_allocator());
    let exports = Arc::new(ExportTable::with_default_allocator());
    let allocator = Arc::new(IdAllocator::new());
    let variable_manager = VariableStateManager::new();

    // 1. Test Resume Token functionality
    println!("🔄 Testing Resume Token functionality...");

    let secret_key = ResumeTokenManager::generate_secret_key();
    let token_manager = ResumeTokenManager::new(secret_key);
    let session_manager = PersistentSessionManager::new(token_manager);

    // Create a session snapshot
    let token = session_manager.snapshot_session(
        "test-session",
        &allocator,
        &imports,
        &exports,
        Some(&variable_manager)
    ).await;

    assert!(token.is_ok(), "Failed to create resume token");
    let token = token.unwrap();
    println!("✅ Resume token created successfully: {}", &token.session_id);

    // Restore the session
    let restored_session_id = session_manager.restore_session(
        &token,
        &allocator,
        &imports,
        &exports,
        Some(&variable_manager)
    ).await;

    assert!(restored_session_id.is_ok(), "Failed to restore session");
    println!("✅ Session restored successfully: {}", restored_session_id.unwrap());

    // 2. Test Nested Capability Creation
    println!("\n🏗️  Testing Nested Capability Creation...");

    let factory = Arc::new(CalculatorFactory);
    let graph = Arc::new(CapabilityGraph::new());
    let base_calculator = Arc::new(AdvancedCalculatorCapability::new(2));

    let nested_target = DefaultNestedCapableTarget::new(
        "parent_calculator".to_string(),
        factory,
        graph.clone(),
        base_calculator,
    );

    // Test listing capability types
    let types = nested_target.list_capability_types().await;
    assert!(types.is_ok(), "Failed to list capability types");
    println!("✅ Available capability types: {:?}", types.unwrap());

    // Create a sub-capability
    let config = capnweb_core::protocol::Value::Object({
        let mut obj = HashMap::new();
        obj.insert("precision".to_string(), Box::new(capnweb_core::protocol::Value::Number(Number::from(4))));
        obj
    });

    let sub_cap_result = nested_target.create_sub_capability("advanced_calculator", config).await;
    assert!(sub_cap_result.is_ok(), "Failed to create sub-capability");
    println!("✅ Sub-capability created: {:?}", sub_cap_result.unwrap());

    // Test graph statistics
    let stats = graph.get_stats().await;
    println!("✅ Capability graph stats: {:?}", stats);
    assert!(stats.total_capabilities > 0, "Graph should have capabilities");

    // 3. Test Advanced IL Plan Runner
    println!("\n🚀 Testing Advanced IL Plan Runner...");

    let runner = PlanRunner::new(imports.clone(), exports.clone());

    // Create a simple calculator capability for testing
    let calc_target = Arc::new(AdvancedCalculatorCapability::new(2));
    let captures = vec![calc_target];

    // Build a complex plan that performs multiple operations
    let mut builder = PlanBuilder::new();
    let cap_index = builder.add_capture(CapId::new(1));

    // First operation: add 5 + 3
    let add_result = builder.add_call(
        Source::capture(cap_index),
        "add".to_string(),
        vec![
            Source::by_value(json!(5)),
            Source::by_value(json!(3)),
        ],
    );

    // Second operation: multiply result by 2
    let multiply_result = builder.add_call(
        Source::capture(cap_index),
        "multiply".to_string(),
        vec![
            Source::result(add_result),
            Source::by_value(json!(2)),
        ],
    );

    // Create an object with the results
    let mut fields = HashMap::new();
    fields.insert("add_result".to_string(), Source::result(add_result));
    fields.insert("final_result".to_string(), Source::result(multiply_result));
    fields.insert("operation".to_string(), Source::by_value(json!("Complex calculation")));
    let object_result = builder.add_object(fields);

    let plan = builder.build(Source::result(object_result));

    // Validate the plan
    assert!(plan.validate().is_ok(), "Plan validation failed");
    println!("✅ IL Plan validation passed");

    // Execute the plan
    let parameters = Value::Object(std::collections::HashMap::new());
    let execution_result = runner.execute_plan(&plan, parameters, captures.into_iter().map(|c| c as Arc<dyn RpcTarget>).collect()).await;

    assert!(execution_result.is_ok(), "Plan execution failed: {:?}", execution_result.err());
    let result = execution_result.unwrap();
    println!("✅ IL Plan executed successfully: {:?}", result);

    // Verify the result structure
    match result {
        capnweb_core::protocol::Value::Object(obj) => {
            assert!(obj.contains_key("add_result"), "Missing add_result");
            assert!(obj.contains_key("final_result"), "Missing final_result");
            assert!(obj.contains_key("operation"), "Missing operation");

            // Verify the calculation: (5 + 3) * 2 = 16
            if let Some(boxed_final) = obj.get("final_result") {
                if let capnweb_core::protocol::Value::Number(n) = boxed_final.as_ref() {
                    assert_eq!(n.as_f64(), Some(16.0), "Calculation result should be 16");
                    println!("✅ Calculation result verified: {}", n);
                }
            }
        }
        _ => panic!("Expected object result"),
    }

    // 4. Test Plan Complexity Analysis
    println!("\n📊 Testing Plan Complexity Analysis...");

    let complexity = capnweb_core::protocol::PlanOptimizer::analyze_complexity(&plan);
    println!("✅ Plan complexity: {:?}", complexity);

    assert!(complexity.total_operations > 0, "Should have operations");
    assert!(complexity.call_operations > 0, "Should have call operations");
    assert!(complexity.object_operations > 0, "Should have object operations");

    // 5. Test Advanced Features Integration
    println!("\n🔗 Testing Advanced Features Integration...");

    // Create a plan that uses nested capabilities
    let factory = Arc::new(CalculatorFactory);
    let graph = Arc::new(CapabilityGraph::new());
    let base_calc = Arc::new(AdvancedCalculatorCapability::new(4));

    let nested_calc = DefaultNestedCapableTarget::new(
        "nested_calculator".to_string(),
        factory.clone(),
        graph,
        base_calc,
    );

    // Test both base and nested functionality
    let base_result = nested_calc.call("add", vec![
        capnweb_core::protocol::Value::Number(Number::from(10)),
        capnweb_core::protocol::Value::Number(Number::from(5)),
    ]).await;

    assert!(base_result.is_ok(), "Base calculation failed");
    println!("✅ Base calculation: 10 + 5 = {:?}", base_result.unwrap());

    // Create sub-capability through nested interface
    let sub_config = capnweb_core::protocol::Value::Object({
        let mut obj = HashMap::new();
        obj.insert("precision".to_string(), Box::new(capnweb_core::protocol::Value::Number(Number::from(6))));
        obj
    });

    let nested_result = nested_calc.create_sub_capability("calculator", sub_config).await;
    assert!(nested_result.is_ok(), "Nested capability creation failed");
    println!("✅ Nested capability integration successful");

    // 6. Test Error Handling and Edge Cases
    println!("\n⚠️  Testing Error Handling...");

    // Test invalid plan
    let invalid_plan = Plan::new(
        vec![CapId::new(1)],
        vec![Op::call(
            Source::result(99), // Invalid result reference
            "test".to_string(),
            vec![],
            0,
        )],
        Source::result(0),
    );

    let invalid_execution = runner.execute_plan(&invalid_plan, Value::Object(std::collections::HashMap::new()), vec![Arc::new(MockRpcTarget::new())]).await;
    assert!(invalid_execution.is_err(), "Should fail with invalid plan");
    println!("✅ Error handling for invalid plans works");

    // Test capability factory error handling
    let bad_capability = factory.create_capability("nonexistent", capnweb_core::protocol::Value::Null).await;
    assert!(bad_capability.is_err(), "Should fail with invalid capability type");
    println!("✅ Error handling for invalid capability types works");

    println!("\n🎉 All Advanced Features Integration Tests Passed! 🎉");
}

#[tokio::test]
async fn test_resume_token_persistence_and_recovery() {
    println!("🔄 Testing Resume Token Persistence and Recovery...");

    let secret_key = ResumeTokenManager::generate_secret_key();
    let token_manager = ResumeTokenManager::new(secret_key);

    // Create session with some variables
    let mut variables = HashMap::new();
    variables.insert("user_id".to_string(), capnweb_core::protocol::Value::Number(Number::from(12345)));
    variables.insert("session_data".to_string(), capnweb_core::protocol::Value::Object({
        let mut obj = HashMap::new();
        obj.insert("theme".to_string(), Box::new(capnweb_core::protocol::Value::String("dark".to_string())));
        obj.insert("language".to_string(), Box::new(capnweb_core::protocol::Value::String("en".to_string())));
        obj
    }));

    let snapshot = capnweb_core::protocol::SessionSnapshot {
        session_id: "persistence-test".to_string(),
        created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        last_activity: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        version: 1,
        next_positive_id: 10,
        next_negative_id: -5,
        imports: HashMap::new(),
        exports: HashMap::new(),
        variables,
        max_age_seconds: 3600,
        capabilities: vec!["calculator".to_string(), "advanced_calculator".to_string()],
    };

    // Generate token
    let token = token_manager.generate_token(snapshot.clone()).unwrap();
    assert_eq!(token.session_id, "persistence-test");
    println!("✅ Token generated for session: {}", token.session_id);

    // Parse token back
    let restored_snapshot = token_manager.parse_token(&token).unwrap();
    assert_eq!(restored_snapshot.session_id, snapshot.session_id);
    assert_eq!(restored_snapshot.variables.len(), 2);

    // Verify variable restoration
    assert!(restored_snapshot.variables.contains_key("user_id"));
    assert!(restored_snapshot.variables.contains_key("session_data"));
    println!("✅ Session variables preserved: {} items", restored_snapshot.variables.len());

    println!("✅ Resume Token Persistence and Recovery Test Passed!");
}

#[tokio::test]
async fn test_nested_capability_lifecycle() {
    println!("🏗️  Testing Nested Capability Lifecycle...");

    let factory = Arc::new(CalculatorFactory);
    let graph = Arc::new(CapabilityGraph::new());
    let base_calc = Arc::new(AdvancedCalculatorCapability::new(2));

    let nested_target = DefaultNestedCapableTarget::new(
        "lifecycle_test".to_string(),
        factory,
        graph.clone(),
        base_calc,
    );

    // Create multiple sub-capabilities
    let configs = vec![
        ("calc1", 2),
        ("calc2", 4),
        ("calc3", 8),
    ];

    let mut created_ids = Vec::new();

    for (name, precision) in configs {
        let config = capnweb_core::protocol::Value::Object({
            let mut obj = HashMap::new();
            obj.insert("precision".to_string(), Box::new(capnweb_core::protocol::Value::Number(Number::from(precision))));
            obj.insert("name".to_string(), Box::new(capnweb_core::protocol::Value::String(name.to_string())));
            obj
        });

        let result = nested_target.create_sub_capability("calculator", config).await.unwrap();

        // Extract capability ID
        if let capnweb_core::protocol::Value::Object(obj) = result {
            if let Some(boxed_id) = obj.get("capability_id") {
                if let capnweb_core::protocol::Value::String(id) = boxed_id.as_ref() {
                    created_ids.push(id.clone());
                    println!("✅ Created capability: {} with precision {}", name, precision);
                }
            }
        }
    }

    // Verify graph state
    let stats = graph.get_stats().await;
    assert_eq!(stats.total_capabilities, 3); // 3 children capabilities
    println!("✅ Graph contains {} capabilities", stats.total_capabilities);

    // List all child capabilities
    let children = nested_target.list_child_capabilities().await.unwrap();
    if let capnweb_core::protocol::Value::Array(child_array) = children {
        assert_eq!(child_array.len(), 3);
        println!("✅ Listed {} child capabilities", child_array.len());
    }

    // Dispose of one capability
    if let Some(id_to_dispose) = created_ids.first() {
        let dispose_result = nested_target.dispose_child_capability(id_to_dispose).await.unwrap();
        if let capnweb_core::protocol::Value::Bool(true) = dispose_result {
            println!("✅ Successfully disposed capability: {}", id_to_dispose);
        }
    }

    // Verify graph state after disposal
    let stats_after = graph.get_stats().await;
    assert_eq!(stats_after.total_capabilities, 2); // 2 remaining children after disposal
    println!("✅ Graph now contains {} capabilities after disposal", stats_after.total_capabilities);

    println!("✅ Nested Capability Lifecycle Test Passed!");
}

#[tokio::test]
async fn test_il_plan_runner_edge_cases() {
    println!("🚀 Testing IL Plan Runner Edge Cases...");

    let imports = Arc::new(ImportTable::with_default_allocator());
    let exports = Arc::new(ExportTable::with_default_allocator());
    let runner = PlanRunner::with_settings(imports, exports, 5000, 100); // 5s timeout, max 100 ops

    // Test 1: Empty plan
    let empty_plan = Plan::new(
        vec![],
        vec![],
        Source::by_value(json!("empty")),
    );

    let empty_result = runner.execute_plan(&empty_plan, Value::Object(std::collections::HashMap::new()), vec![]).await;
    assert!(empty_result.is_ok(), "Empty plan should execute");
    println!("✅ Empty plan execution successful");

    // Test 2: Parameter access
    let calc_target = Arc::new(AdvancedCalculatorCapability::new(2));
    let mut builder = PlanBuilder::new();
    let cap_index = builder.add_capture(CapId::new(1));

    // Use parameters in calculation
    let param_result = builder.add_call(
        Source::capture(cap_index),
        "add".to_string(),
        vec![
            Source::param(vec!["numbers".to_string(), "a".to_string()]),
            Source::param(vec!["numbers".to_string(), "b".to_string()]),
        ],
    );

    let plan = builder.build(Source::result(param_result));

    let mut numbers_obj = std::collections::HashMap::new();
    numbers_obj.insert("a".to_string(), Box::new(Value::Number(Number::from(15))));
    numbers_obj.insert("b".to_string(), Box::new(Value::Number(Number::from(25))));

    let mut parameters_obj = std::collections::HashMap::new();
    parameters_obj.insert("numbers".to_string(), Box::new(Value::Object(numbers_obj)));

    let parameters = Value::Object(parameters_obj);

    let param_execution = runner.execute_plan(&plan, parameters, vec![calc_target.clone()]).await;
    assert!(param_execution.is_ok(), "Parameter-based plan should execute");

    if let Ok(capnweb_core::protocol::Value::Number(n)) = param_execution {
        assert_eq!(n.as_f64(), Some(40.0), "15 + 25 should equal 40");
        println!("✅ Parameter access: 15 + 25 = {}", n);
    }

    // Test 3: Complex nested structures
    let mut complex_builder = PlanBuilder::new();
    let cap_index = complex_builder.add_capture(CapId::new(1));

    // Multiple operations creating nested data
    let val1 = complex_builder.add_call(
        Source::capture(cap_index),
        "add".to_string(),
        vec![
            Source::by_value(json!(1)),
            Source::by_value(json!(2)),
        ],
    );

    let val2 = complex_builder.add_call(
        Source::capture(cap_index),
        "multiply".to_string(),
        vec![
            Source::result(val1),
            Source::by_value(json!(3)),
        ],
    );

    // Create nested object
    let mut inner_fields = HashMap::new();
    inner_fields.insert("calculation1".to_string(), Source::result(val1));
    inner_fields.insert("calculation2".to_string(), Source::result(val2));
    let inner_obj = complex_builder.add_object(inner_fields);

    // Create array with mixed types
    let array_result = complex_builder.add_array(vec![
        Source::result(inner_obj),
        Source::by_value(json!("metadata")),
        Source::by_value(json!(42)),
    ]);

    // Final outer object
    let mut outer_fields = HashMap::new();
    outer_fields.insert("data".to_string(), Source::result(array_result));
    outer_fields.insert("timestamp".to_string(), Source::by_value(json!(1234567890)));
    let final_obj = complex_builder.add_object(outer_fields);

    let complex_plan = complex_builder.build(Source::result(final_obj));

    let complex_result = runner.execute_plan(&complex_plan, Value::Object(std::collections::HashMap::new()), vec![calc_target]).await;
    assert!(complex_result.is_ok(), "Complex nested plan should execute");
    println!("✅ Complex nested structure execution successful");

    // Test 4: Plan complexity analysis
    let complexity = capnweb_core::protocol::PlanOptimizer::analyze_complexity(&complex_plan);
    println!("✅ Complex plan analysis: {:?}", complexity);
    assert!(complexity.total_operations >= 5, "Should have multiple operations");
    assert!(complexity.call_operations >= 2, "Should have call operations");
    assert!(complexity.object_operations >= 2, "Should have object operations");
    assert!(complexity.array_operations >= 1, "Should have array operations");

    println!("✅ IL Plan Runner Edge Cases Test Passed!");
}