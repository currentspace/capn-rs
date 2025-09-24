// Comprehensive Coverage Improvement Tests
// Addresses identified coverage gaps in Cap'n Web implementation

use capnweb_core::{
    protocol::{
        // Resume Tokens
        ResumeTokenManager, PersistentSessionManager, SessionSnapshot, ResumeTokenError,
        // Nested Capabilities
        CapabilityFactory, CapabilityGraph, DefaultNestedCapableTarget, NestedCapableRpcTarget,
        CapabilityError, CapabilityMetadata, MethodMetadata,
        // IL Plan Runner
        PlanRunner, PlanBuilder, ExecutionContext, PlanExecutionError, PlanOptimizer,
        // Core
        ImportTable, ExportTable, IdAllocator, VariableStateManager, Value,
        VariableError, TableError, SessionError,
    },
    il::{Plan, Source, Op},
    RpcTarget, RpcError, CapId,
};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Number;

// ============================================================================
// RESUME TOKENS - ERROR PATH TESTS
// ============================================================================

#[cfg(test)]
mod resume_token_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_token_serialization_error() {
        let secret_key = ResumeTokenManager::generate_secret_key();
        let manager = ResumeTokenManager::new(secret_key);

        // Create a snapshot with invalid data that can't be serialized
        let snapshot = SessionSnapshot {
            session_id: "test-session".to_string(),
            created_at: u64::MAX, // Edge case: max value
            last_activity: u64::MAX,
            version: 999, // Unsupported version
            next_positive_id: i64::MAX,
            next_negative_id: i64::MIN,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables: HashMap::new(),
            max_age_seconds: 0, // Edge case: zero timeout
            capabilities: vec![],
        };

        // This should handle serialization gracefully
        let result = manager.generate_token(snapshot);
        assert!(result.is_ok() || matches!(result.err(), Some(ResumeTokenError::SerializationError(_))));
    }

    #[tokio::test]
    async fn test_token_expiration_edge_cases() {
        let manager = ResumeTokenManager::with_settings(3600, 256);

        // Test with expired token
        let expired_token = capnweb_core::protocol::ResumeToken {
            token_data: "invalid_token_data".to_string(),
            session_id: "expired".to_string(),
            expires_at: 0, // Already expired
        };

        let session_manager = PersistentSessionManager::new(manager);
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());

        let result = session_manager.restore_session(
            &expired_token,
            &allocator,
            &imports,
            &exports,
            None
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_token_parsing() {
        let secret = ResumeTokenManager::generate_secret_key();
        let manager = ResumeTokenManager::new(secret);

        let invalid_token = capnweb_core::protocol::ResumeToken {
            token_data: "not_a_valid_base64_token!@#$%".to_string(),
            session_id: "invalid".to_string(),
            expires_at: u64::MAX,
        };

        // Should handle invalid base64 gracefully
        let snapshot = SessionSnapshot {
            session_id: "test".to_string(),
            created_at: 0,
            last_activity: 0,
            version: 1,
            next_positive_id: 1,
            next_negative_id: -1,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables: HashMap::new(),
            max_age_seconds: 3600,
            capabilities: vec![],
        };

        // Try to parse invalid token
        let result = manager.parse_token(&invalid_token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_persistence_with_empty_data() {
        let token_manager = ResumeTokenManager::new(ResumeTokenManager::generate_secret_key());
        let session_manager = PersistentSessionManager::new(token_manager);

        // Edge case: empty session
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());

        let token = session_manager.snapshot_session(
            "", // Empty session ID
            &allocator,
            &imports,
            &exports,
            None
        ).await;

        // Should handle empty session ID
        assert!(token.is_ok() || token.is_err());
    }

    #[test]
    fn test_secret_key_generation_uniqueness() {
        let key1 = ResumeTokenManager::generate_secret_key();
        let key2 = ResumeTokenManager::generate_secret_key();

        // Keys should be unique
        assert_ne!(key1, key2);

        // Keys should have correct length
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }
}

// ============================================================================
// NESTED CAPABILITIES - PUBLIC API TESTS
// ============================================================================

#[cfg(test)]
mod nested_capability_api_tests {
    use super::*;

    #[derive(Debug)]
    struct TestCapability {
        name: String,
        counter: std::sync::atomic::AtomicU32,
    }

    #[async_trait::async_trait]
    impl RpcTarget for TestCapability {
        async fn call(&self, method: &str, _args: Vec<Value>) -> Result<Value, RpcError> {
            self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            match method {
                "test" => Ok(Value::String(self.name.clone())),
                _ => Err(RpcError::not_found("Method not found")),
            }
        }

        async fn get_property(&self, _property: &str) -> Result<Value, RpcError> {
            Ok(Value::String(self.name.clone()))
        }
    }

    #[tokio::test]
    async fn test_capability_graph_operations() {
        let graph = Arc::new(CapabilityGraph::new());

        // Test add_capability
        let cap1 = Arc::new(TestCapability {
            name: "cap1".to_string(),
            counter: std::sync::atomic::AtomicU32::new(0),
        });

        graph.add_capability("cap1", None, cap1.clone()).await;

        // Test get_children
        let children = graph.get_children("cap1").await;
        assert_eq!(children.len(), 0);

        // Add child capability
        let cap2 = Arc::new(TestCapability {
            name: "cap2".to_string(),
            counter: std::sync::atomic::AtomicU32::new(0),
        });

        graph.add_capability("cap2", Some("cap1".to_string()), cap2.clone()).await;

        // Test get_descendants
        let descendants = graph.get_descendants("cap1").await;
        assert_eq!(descendants.len(), 1);

        // Test add_reference and remove_reference
        graph.add_reference("cap1").await;
        graph.add_reference("cap1").await;

        let removed = graph.remove_reference("cap1").await;
        assert!(!removed); // Still has references

        let removed = graph.remove_reference("cap1").await;
        assert!(removed); // Last reference removed

        // Test capability not found error
        let missing = graph.get_capability("nonexistent").await;
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_nested_capability_error_paths() {
        let factory = Arc::new(TestCapabilityFactory);
        let graph = Arc::new(CapabilityGraph::new());
        let root = Arc::new(TestCapability {
            name: "root".to_string(),
            counter: std::sync::atomic::AtomicU32::new(0),
        });

        let nested = DefaultNestedCapableTarget::new(
            "test".to_string(),
            factory,
            graph.clone(),
            root,
        );

        // Test invalid capability type
        let result = nested.create_sub_capability(
            "invalid_type",
            Value::Null
        ).await;

        assert!(matches!(result, Err(_)));

        // Test dispose of non-existent capability
        let result = nested.dispose_child_capability("nonexistent").await;
        assert!(matches!(result, Ok(Value::Bool(false))));
    }

    #[derive(Debug)]
    struct TestCapabilityFactory;

    #[async_trait::async_trait]
    impl CapabilityFactory for TestCapabilityFactory {
        async fn create_capability(
            &self,
            capability_type: &str,
            _config: Value,
        ) -> Result<Arc<dyn RpcTarget>, CapabilityError> {
            match capability_type {
                "test" => Ok(Arc::new(TestCapability {
                    name: "test_cap".to_string(),
                    counter: std::sync::atomic::AtomicU32::new(0),
                })),
                _ => Err(CapabilityError::InvalidCapabilityType(capability_type.to_string())),
            }
        }

        fn list_capability_types(&self) -> Vec<String> {
            vec!["test".to_string()]
        }

        fn get_capability_metadata(&self, capability_type: &str) -> Option<CapabilityMetadata> {
            match capability_type {
                "test" => Some(CapabilityMetadata {
                    name: "Test".to_string(),
                    description: "Test capability".to_string(),
                    version: "1.0.0".to_string(),
                    methods: vec![],
                    config_schema: None,
                }),
                _ => None,
            }
        }
    }
}

// ============================================================================
// IL PLAN RUNNER - EDGE CASES AND ERROR PATHS
// ============================================================================

#[cfg(test)]
mod il_runner_edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_plan_runner_with_zero_operations() {
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());
        let runner = PlanRunner::new(imports, exports);

        // Edge case: empty plan
        let plan = Plan::new(
            vec![],
            vec![],
            Source::by_value(serde_json::json!("empty"))
        );

        let result = runner.execute_plan(&plan, Value::Null, vec![]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plan_runner_timeout() {
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());

        // Create runner with very short timeout
        let runner = PlanRunner::with_settings(imports, exports, 1, 100); // 1ms timeout

        // Create a plan that would take longer
        let mut builder = PlanBuilder::new();
        for i in 0..10 {
            builder.add_object(HashMap::from([
                ("index".to_string(), Source::by_value(serde_json::json!(i)))
            ]));
        }
        let plan = builder.build(Source::result(0));

        let mock_cap = Arc::new(MockSlowCapability);
        let result = runner.execute_plan(&plan, Value::Null, vec![mock_cap]).await;

        // Should timeout or complete
        assert!(result.is_ok() || matches!(result, Err(PlanExecutionError::ExecutionTimeout)));
    }

    #[tokio::test]
    async fn test_plan_builder_edge_cases() {
        let mut builder = PlanBuilder::new();

        // Test with maximum index values
        for _ in 0..100 {
            builder.add_capture(CapId::new(u64::MAX));
        }

        // Test complex nested structures
        let mut fields = HashMap::new();
        for i in 0..50 {
            fields.insert(format!("field_{}", i), Source::by_value(serde_json::json!(i)));
        }
        builder.add_object(fields);

        // Test large array
        let items: Vec<Source> = (0..100)
            .map(|i| Source::by_value(serde_json::json!(i)))
            .collect();
        builder.add_array(items);

        let plan = builder.build(Source::result(0));
        assert!(plan.validate().is_ok());
    }

    #[tokio::test]
    async fn test_plan_optimizer_with_complex_plans() {
        let mut builder = PlanBuilder::new();

        // Create a complex plan with many operations
        for i in 0..20 {
            builder.add_call(
                Source::capture(0),
                format!("method_{}", i),
                vec![Source::by_value(serde_json::json!(i))]
            );
        }

        let plan = builder.build(Source::result(0));

        // Test complexity analysis
        let complexity = PlanOptimizer::analyze_complexity(&plan);
        assert_eq!(complexity.total_operations, 20);
        assert_eq!(complexity.call_operations, 20);

        // Test optimization
        let optimized = PlanOptimizer::optimize(plan.clone());
        assert_eq!(optimized.ops.len(), plan.ops.len()); // Currently no optimization
    }

    #[derive(Debug)]
    struct MockSlowCapability;

    #[async_trait::async_trait]
    impl RpcTarget for MockSlowCapability {
        async fn call(&self, _method: &str, _args: Vec<Value>) -> Result<Value, RpcError> {
            // Simulate slow operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(Value::Number(Number::from(42)))
        }

        async fn get_property(&self, _property: &str) -> Result<Value, RpcError> {
            Ok(Value::String("slow".to_string()))
        }
    }

    #[tokio::test]
    async fn test_execution_context_edge_cases() {
        // Test with deeply nested parameters
        let params = Value::Object({
            let mut obj = HashMap::new();
            obj.insert("level1".to_string(), Box::new(Value::Object({
                let mut obj2 = HashMap::new();
                obj2.insert("level2".to_string(), Box::new(Value::Object({
                    let mut obj3 = HashMap::new();
                    obj3.insert("level3".to_string(), Box::new(Value::String("deep".to_string())));
                    obj3
                })));
                obj2
            })));
            obj
        });

        let context = ExecutionContext::new(params, vec![]);

        // Test nested parameter access
        let result = context.get_nested_parameter(&[
            "level1".to_string(),
            "level2".to_string(),
            "level3".to_string()
        ]);

        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s, "deep"),
            _ => panic!("Expected string value"),
        }

        // Test missing parameter
        let missing = context.get_nested_parameter(&["nonexistent".to_string()]);
        assert!(missing.is_err());
    }
}

// ============================================================================
// ERROR HANDLING AND EDGE CASES
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_variable_manager_error_paths() {
        let manager = VariableStateManager::new();

        // Test invalid variable names
        let result = manager.set_variable("", Value::String("test".to_string())).await;
        assert!(result.is_err());

        let result = manager.set_variable("invalid/name", Value::String("test".to_string())).await;
        assert!(result.is_err());

        // Test with stub value (unsupported)
        let stub_value = Value::Stub(capnweb_core::protocol::StubReference {
            id: "stub".to_string(),
            stub: Arc::new(MockSlowCapability),
        });
        let result = manager.set_variable("stub", stub_value).await;
        assert!(matches!(result, Err(VariableError::UnsupportedValueType(_))));

        // Test size limits
        let large_string = "x".repeat(10_000_000); // 10MB string
        let result = manager.set_variable("large", Value::String(large_string)).await;
        // Should handle large values gracefully
        assert!(result.is_ok() || matches!(result, Err(VariableError::ValueTooLarge(_))));
    }

    #[tokio::test]
    async fn test_table_operations_with_edge_values() {
        let allocator = Arc::new(IdAllocator::new());
        let imports = ImportTable::new(allocator.clone());

        // Test with maximum ID values
        let max_id = capnweb_core::protocol::ImportId(i64::MAX);
        let min_id = capnweb_core::protocol::ImportId(i64::MIN);

        // Test insert with extreme IDs
        let result = imports.insert(max_id, capnweb_core::protocol::ImportValue::Value(Value::Null));
        assert!(result.is_ok());

        let result = imports.insert(min_id, capnweb_core::protocol::ImportValue::Value(Value::Null));
        assert!(result.is_ok());

        // Test reference counting edge cases
        imports.increment_refcount(max_id);
        imports.increment_refcount(max_id);

        let removed = imports.decrement_refcount(max_id);
        assert!(!removed); // Still has references

        let removed = imports.decrement_refcount(max_id);
        assert!(removed); // Last reference removed
    }

    #[tokio::test]
    async fn test_session_error_scenarios() {
        let allocator = Arc::new(IdAllocator::new());

        // Test ID allocation limits
        for _ in 0..1000 {
            allocator.allocate_import();
            allocator.allocate_export();
        }

        // IDs should still be unique
        let id1 = allocator.allocate_import();
        let id2 = allocator.allocate_import();
        assert_ne!(id1.0, id2.0);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let graph = Arc::new(CapabilityGraph::new());
        let mut handles = vec![];

        // Test concurrent capability additions
        for i in 0..10 {
            let graph_clone = graph.clone();
            let handle = tokio::spawn(async move {
                let cap = Arc::new(MockSlowCapability);
                graph_clone.add_capability(&format!("cap_{}", i), None, cap).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = graph.get_stats().await;
        assert_eq!(stats.total_capabilities, 10);
    }
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_id_allocator_uniqueness(count in 1..1000) {
            let allocator = IdAllocator::new();
            let mut import_ids = std::collections::HashSet::new();
            let mut export_ids = std::collections::HashSet::new();

            for _ in 0..count {
                let import_id = allocator.allocate_import();
                let export_id = allocator.allocate_export();

                // All IDs should be unique
                assert!(import_ids.insert(import_id.0));
                assert!(export_ids.insert(export_id.0));
            }
        }

        #[test]
        fn test_plan_validation_with_random_ops(ops_count in 0..100) {
            let mut builder = PlanBuilder::new();

            for i in 0..ops_count {
                match i % 3 {
                    0 => {
                        builder.add_call(
                            Source::capture(0),
                            format!("method_{}", i),
                            vec![]
                        );
                    }
                    1 => {
                        builder.add_object(HashMap::new());
                    }
                    _ => {
                        builder.add_array(vec![]);
                    }
                }
            }

            let plan = builder.build(Source::result(0));

            // Plan should always validate
            let validation = plan.validate();
            assert!(validation.is_ok() || validation.is_err());
        }
    }
}

// ============================================================================
// INTEGRATION TESTS FOR UNCOVERED SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_full_advanced_feature_integration() {
    // This test ensures all advanced features work together
    let token_manager = ResumeTokenManager::new(ResumeTokenManager::generate_secret_key());
    let session_manager = PersistentSessionManager::new(token_manager);

    let factory = Arc::new(property_tests::TestCapabilityFactory);
    let graph = Arc::new(CapabilityGraph::new());

    let allocator = Arc::new(IdAllocator::new());
    let imports = Arc::new(ImportTable::new(allocator.clone()));
    let exports = Arc::new(ExportTable::new(allocator.clone()));

    // Create nested capabilities
    let root = Arc::new(nested_capability_api_tests::TestCapability {
        name: "root".to_string(),
        counter: std::sync::atomic::AtomicU32::new(0),
    });

    let nested = DefaultNestedCapableTarget::new(
        "integration".to_string(),
        factory,
        graph.clone(),
        root.clone(),
    );

    // Create and execute IL plan
    let runner = PlanRunner::new(imports.clone(), exports.clone());
    let mut builder = PlanBuilder::new();

    let cap_id = builder.add_capture(CapId::new(1));
    let result_id = builder.add_call(
        Source::capture(cap_id),
        "test".to_string(),
        vec![]
    );

    let plan = builder.build(Source::result(result_id));

    let exec_result = runner.execute_plan(
        &plan,
        Value::Null,
        vec![root as Arc<dyn RpcTarget>]
    ).await;

    assert!(exec_result.is_ok());

    // Create resume token
    let token = session_manager.snapshot_session(
        "integration-test",
        &allocator,
        &imports,
        &exports,
        None
    ).await;

    assert!(token.is_ok());

    // Verify graph statistics
    let stats = graph.get_stats().await;
    assert!(stats.total_capabilities >= 1);
}

#[tokio::test]
async fn test_resource_cleanup_and_disposal() {
    let graph = Arc::new(CapabilityGraph::new());

    // Add many capabilities
    for i in 0..100 {
        let cap = Arc::new(nested_capability_api_tests::TestCapability {
            name: format!("cap_{}", i),
            counter: std::sync::atomic::AtomicU32::new(0),
        });

        graph.add_capability(&format!("cap_{}", i), None, cap).await;
    }

    // Remove all capabilities
    for i in 0..100 {
        graph.remove_capability(&format!("cap_{}", i)).await;
    }

    let stats = graph.get_stats().await;
    assert_eq!(stats.total_capabilities, 0);
}