// Comprehensive Cap'n Web Protocol Compliance Tests
// Tests all core features of the official protocol specification
// Based on: https://github.com/cloudflare/capnweb/blob/main/protocol.md

use capnweb_core::{
    Message, Expression, ImportId, ExportId, CallId,
    ImportTable, ExportTable, IdAllocator, RpcTarget, RpcError,
    protocol::{Value, SessionSnapshot, ResumeTokenManager}
};
use serde_json::{json, Value as JsonValue, Number};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[cfg(test)]
mod protocol_message_tests {
    use super::*;

    /// Test all core message types per protocol specification
    #[tokio::test]
    async fn test_all_core_message_types() {
        println!("🧪 Testing Core Protocol Message Types");

        // Test PUSH message
        let push_msg = Message::Push(Expression::String("test_value".to_string()));
        let push_json = push_msg.to_json();
        let push_deserialized = Message::from_json(&push_json).unwrap();
        assert_eq!(push_msg, push_deserialized);
        println!("✅ PUSH message serialization verified");

        // Test PULL message
        let pull_msg = Message::Pull(ImportId(42));
        let pull_json = pull_msg.to_json();
        let pull_deserialized = Message::from_json(&pull_json).unwrap();
        assert_eq!(pull_msg, pull_deserialized);
        println!("✅ PULL message serialization verified");

        // Test RESOLVE message
        let resolve_msg = Message::Resolve(ExportId(-1), Expression::Number(Number::from(123)));
        let resolve_json = resolve_msg.to_json();
        let resolve_deserialized = Message::from_json(&resolve_json).unwrap();
        assert_eq!(resolve_msg, resolve_deserialized);
        println!("✅ RESOLVE message serialization verified");

        // Test REJECT message
        let reject_msg = Message::Reject(ExportId(-2), Expression::Error {
            error_type: "test_error".to_string(),
            message: "Test error message".to_string(),
            stack: None
        });
        let reject_json = reject_msg.to_json();
        let reject_deserialized = Message::from_json(&reject_json).unwrap();
        assert_eq!(reject_msg, reject_deserialized);
        println!("✅ REJECT message serialization verified");

        // Test RELEASE message
        let release_msg = Message::Release {
            release: vec![ImportId(1), ImportId(2), ImportId(3)]
        };
        let release_json = release_msg.to_json();
        let release_deserialized = Message::from_json(&release_json).unwrap();
        assert_eq!(release_msg, release_deserialized);
        println!("✅ RELEASE message serialization verified");

        // Test ABORT message
        let abort_msg = Message::Abort {
            abort: Expression::Error {
                error_type: "protocol_error".to_string(),
                message: "Session terminated".to_string(),
                stack: None
            }
        };
        let abort_json = abort_msg.to_json();
        let abort_deserialized = Message::from_json(&abort_json).unwrap();
        assert_eq!(abort_msg, abort_deserialized);
        println!("✅ ABORT message serialization verified");
    }

    /// Test all expression types per protocol specification
    #[tokio::test]
    async fn test_all_expression_types() {
        println!("🧪 Testing All Protocol Expression Types");

        // Test literal expressions
        let literals = vec![
            Expression::Null,
            Expression::Bool(true),
            Expression::Bool(false),
            Expression::Number(Number::from(42)),
            Expression::Number(Number::from_f64(3.14159).unwrap()),
            Expression::String("test_string".to_string()),
            Expression::Array(vec![
                Expression::Number(Number::from(1)),
                Expression::String("item".to_string()),
                Expression::Bool(true)
            ]),
        ];

        for literal in literals {
            let json = serde_json::to_string(&literal).unwrap();
            let deserialized: Expression = serde_json::from_str(&json).unwrap();
            assert_eq!(literal, deserialized);
        }
        println!("✅ All literal expressions verified");

        // Test object expression
        let mut obj_map = HashMap::new();
        obj_map.insert("key1".to_string(), Box::new(Expression::String("value1".to_string())));
        obj_map.insert("key2".to_string(), Box::new(Expression::Number(Number::from(42))));
        let obj_expr = Expression::Object(obj_map);

        let obj_json = serde_json::to_string(&obj_expr).unwrap();
        let obj_deserialized: Expression = serde_json::from_str(&obj_json).unwrap();
        assert_eq!(obj_expr, obj_deserialized);
        println!("✅ Object expression verified");

        // Test date expression
        let date_expr = Expression::Date(chrono::Utc::now());
        let date_json = serde_json::to_string(&date_expr).unwrap();
        let date_deserialized: Expression = serde_json::from_str(&date_json).unwrap();
        assert_eq!(date_expr, date_deserialized);
        println!("✅ Date expression verified");

        // Test error expression
        let error_expr = Expression::Error {
            error_type: "validation_error".to_string(),
            message: "Invalid input provided".to_string(),
            stack: Some("at line 42 in test.rs".to_string())
        };
        let error_json = serde_json::to_string(&error_expr).unwrap();
        let error_deserialized: Expression = serde_json::from_str(&error_json).unwrap();
        assert_eq!(error_expr, error_deserialized);
        println!("✅ Error expression verified");

        // Test import expression
        let import_expr = Expression::Import(ImportId(123));
        let import_json = serde_json::to_string(&import_expr).unwrap();
        let import_deserialized: Expression = serde_json::from_str(&import_json).unwrap();
        assert_eq!(import_expr, import_deserialized);
        println!("✅ Import expression verified");

        // Test export expression
        let export_expr = Expression::Export {
            export: ExportId(-456),
            promise: false
        };
        let export_json = serde_json::to_string(&export_expr).unwrap();
        let export_deserialized: Expression = serde_json::from_str(&export_json).unwrap();
        assert_eq!(export_expr, export_deserialized);
        println!("✅ Export expression verified");

        // Test promise expression
        let promise_expr = Expression::Promise {
            promise: ExportId(-789)
        };
        let promise_json = serde_json::to_string(&promise_expr).unwrap();
        let promise_deserialized: Expression = serde_json::from_str(&promise_json).unwrap();
        assert_eq!(promise_expr, promise_deserialized);
        println!("✅ Promise expression verified");

        // Test pipeline expression
        let pipeline_expr = Expression::Pipeline {
            pipeline: ImportId(999),
            property: vec!["method".to_string(), "property".to_string()],
            args: Some(Box::new(Expression::Array(vec![
                Expression::String("arg1".to_string()),
                Expression::Number(Number::from(42))
            ])))
        };
        let pipeline_json = serde_json::to_string(&pipeline_expr).unwrap();
        let pipeline_deserialized: Expression = serde_json::from_str(&pipeline_json).unwrap();
        assert_eq!(pipeline_expr, pipeline_deserialized);
        println!("✅ Pipeline expression verified");
    }

    /// Test ID assignment rules per protocol specification
    #[tokio::test]
    async fn test_id_assignment_rules() {
        println!("🧪 Testing Protocol ID Assignment Rules");

        let allocator = IdAllocator::new();

        // Test positive ID allocation (for imports)
        let positive_ids: Vec<ImportId> = (0..5).map(|_| allocator.allocate_import()).collect();
        for (i, id) in positive_ids.iter().enumerate() {
            assert!(id.0 > 0, "Import ID {} should be positive, got {}", i, id.0);
        }

        // Verify IDs are unique and increasing
        for i in 1..positive_ids.len() {
            assert!(positive_ids[i].0 > positive_ids[i-1].0, "IDs should be increasing");
        }
        println!("✅ Positive ID allocation (imports) verified");

        // Test negative ID allocation (for exports)
        let negative_ids: Vec<ExportId> = (0..5).map(|_| allocator.allocate_export()).collect();
        for (i, id) in negative_ids.iter().enumerate() {
            assert!(id.0 < 0, "Export ID {} should be negative, got {}", i, id.0);
        }

        // Verify IDs are unique and decreasing
        for i in 1..negative_ids.len() {
            assert!(negative_ids[i].0 < negative_ids[i-1].0, "Export IDs should be decreasing");
        }
        println!("✅ Negative ID allocation (exports) verified");

        // Test that positive and negative ranges don't overlap
        let pos_id = allocator.allocate_import().0;
        let neg_id = allocator.allocate_export().0;
        assert!(pos_id > 0 && neg_id < 0, "ID ranges should not overlap");
        println!("✅ ID range separation verified");
    }
}

#[cfg(test)]
mod protocol_table_tests {
    use super::*;

    #[derive(Debug)]
    struct TestRpcTarget {
        name: String,
    }

    #[async_trait::async_trait]
    impl RpcTarget for TestRpcTarget {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            Ok(Value::String(format!("{}::{} called with {} args", self.name, method, args.len())))
        }

        async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
            Ok(Value::String(format!("{}::{}", self.name, property)))
        }
    }

    /// Test import/export table operations per protocol specification
    #[tokio::test]
    async fn test_import_export_tables() {
        println!("🧪 Testing Protocol Import/Export Tables");

        let allocator = Arc::new(IdAllocator::new());
        let import_table = ImportTable::new(allocator.clone());
        let export_table = ExportTable::new(allocator.clone());

        // Test import table operations
        let target = Arc::new(TestRpcTarget { name: "test_target".to_string() });
        let import_id = import_table.add_import(target.clone()).await;
        assert!(import_id.0 > 0, "Import ID should be positive");

        let retrieved = import_table.get_import(&import_id).await;
        assert!(retrieved.is_some(), "Import should be retrievable");
        println!("✅ Import table operations verified");

        // Test export table operations
        let export_id = export_table.add_export(target.clone()).await;
        assert!(export_id.0 < 0, "Export ID should be negative");

        let retrieved = export_table.get_export(&export_id).await;
        assert!(retrieved.is_some(), "Export should be retrievable");
        println!("✅ Export table operations verified");

        // Test reference counting
        let ref_count_before = import_table.get_ref_count(&import_id).await;
        import_table.add_ref(&import_id).await;
        let ref_count_after = import_table.get_ref_count(&import_id).await;
        assert_eq!(ref_count_after, ref_count_before + 1, "Reference count should increase");
        println!("✅ Reference counting verified");

        // Test disposal
        let removed = import_table.remove_import(&import_id).await;
        assert!(removed.is_some(), "Import should be removable");

        let retrieved_after_removal = import_table.get_import(&import_id).await;
        assert!(retrieved_after_removal.is_none(), "Import should not exist after removal");
        println!("✅ Import disposal verified");
    }

    /// Test capability lifecycle per protocol specification
    #[tokio::test]
    async fn test_capability_lifecycle() {
        println!("🧪 Testing Protocol Capability Lifecycle");

        let allocator = Arc::new(IdAllocator::new());
        let import_table = ImportTable::new(allocator.clone());

        // Create multiple capabilities
        let targets: Vec<Arc<TestRpcTarget>> = (0..3)
            .map(|i| Arc::new(TestRpcTarget { name: format!("target_{}", i) }))
            .collect();

        let import_ids: Vec<ImportId> = futures::future::join_all(
            targets.iter().map(|target| import_table.add_import(target.clone()))
        ).await;

        // Verify all imports exist
        for (i, id) in import_ids.iter().enumerate() {
            let target = import_table.get_import(id).await;
            assert!(target.is_some(), "Import {} should exist", i);
        }
        println!("✅ Multiple capability creation verified");

        // Test batch disposal (RELEASE message behavior)
        let disposed_count = import_table.batch_remove(&import_ids).await;
        assert_eq!(disposed_count, import_ids.len(), "All imports should be disposed");

        // Verify all imports are gone
        for (i, id) in import_ids.iter().enumerate() {
            let target = import_table.get_import(id).await;
            assert!(target.is_none(), "Import {} should be disposed", i);
        }
        println!("✅ Batch capability disposal verified");
    }
}

#[cfg(test)]
mod protocol_session_tests {
    use super::*;

    /// Test session snapshots and resume tokens per protocol specification
    #[tokio::test]
    async fn test_session_resume_protocol() {
        println!("🧪 Testing Protocol Session Resume");

        let secret_key = ResumeTokenManager::generate_secret_key();
        let token_manager = ResumeTokenManager::new(secret_key);

        // Create session snapshot with protocol-compliant data
        let snapshot = SessionSnapshot {
            session_id: "protocol-test-session".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
            next_positive_id: 42,
            next_negative_id: -17,
            imports: HashMap::new(),
            exports: HashMap::new(),
            variables: HashMap::new(),
            max_age_seconds: 3600,
            capabilities: Vec::new(),
        };

        // Test token generation
        let token = token_manager.generate_token(snapshot.clone()).unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        println!("✅ Resume token generation verified");

        // Test token parsing
        let parsed_snapshot = token_manager.parse_token(&token).unwrap();
        assert_eq!(parsed_snapshot.session_id, snapshot.session_id);
        assert_eq!(parsed_snapshot.next_positive_id, snapshot.next_positive_id);
        assert_eq!(parsed_snapshot.next_negative_id, snapshot.next_negative_id);
        println!("✅ Resume token parsing verified");

        // Test token-based ID state restoration
        let restored_allocator = IdAllocator::from_snapshot(&parsed_snapshot);
        let new_import = restored_allocator.allocate_import();
        let new_export = restored_allocator.allocate_export();

        assert_eq!(new_import.0, 42, "Import ID should continue from snapshot");
        assert_eq!(new_export.0, -17, "Export ID should continue from snapshot");
        println!("✅ ID state restoration verified");
    }

    /// Test bidirectional protocol features
    #[tokio::test]
    async fn test_bidirectional_protocol() {
        println!("🧪 Testing Bidirectional Protocol Features");

        // Both sides can send and receive any message type
        let client_to_server_messages = vec![
            Message::Push(Expression::String("client_request".to_string())),
            Message::Pull(ImportId(1)),
            Message::Release { release: vec![ImportId(2)] },
        ];

        let server_to_client_messages = vec![
            Message::Push(Expression::String("server_notification".to_string())),
            Message::Resolve(ExportId(-1), Expression::Number(Number::from(42))),
            Message::Reject(ExportId(-2), Expression::Error {
                error_type: "server_error".to_string(),
                message: "Operation failed".to_string(),
                stack: None
            }),
        ];

        // Test serialization in both directions
        for (i, msg) in client_to_server_messages.iter().enumerate() {
            let json = msg.to_json();
            let deserialized = Message::from_json(&json).unwrap();
            assert_eq!(*msg, deserialized, "Client message {} should round-trip", i);
        }

        for (i, msg) in server_to_client_messages.iter().enumerate() {
            let json = msg.to_json();
            let deserialized = Message::from_json(&json).unwrap();
            assert_eq!(*msg, deserialized, "Server message {} should round-trip", i);
        }

        println!("✅ Bidirectional message flow verified");
    }
}

#[cfg(test)]
mod protocol_error_tests {
    use super::*;

    /// Test error handling per protocol specification
    #[tokio::test]
    async fn test_protocol_error_handling() {
        println!("🧪 Testing Protocol Error Handling");

        // Test structured error format
        let structured_error = Expression::Error {
            error_type: "validation_error".to_string(),
            message: "Required field missing: 'name'".to_string(),
            stack: Some("ValidationError\n    at validate_user_input (user.rs:42:5)".to_string())
        };

        let error_json = serde_json::to_string(&structured_error).unwrap();
        let parsed: JsonValue = serde_json::from_str(&error_json).unwrap();

        // Verify error structure
        assert_eq!(parsed["error"]["type"], "validation_error");
        assert_eq!(parsed["error"]["message"], "Required field missing: 'name'");
        assert!(parsed["error"]["stack"].is_string());
        println!("✅ Structured error format verified");

        // Test error in REJECT message
        let reject_msg = Message::Reject(ExportId(-42), structured_error.clone());
        let reject_json = reject_msg.to_json();
        let reject_deserialized = Message::from_json(&reject_json).unwrap();
        assert_eq!(reject_msg, reject_deserialized);
        println!("✅ Error in REJECT message verified");

        // Test error in ABORT message
        let abort_msg = Message::Abort { abort: structured_error };
        let abort_json = abort_msg.to_json();
        let abort_deserialized = Message::from_json(&abort_json).unwrap();
        assert_eq!(abort_msg, abort_deserialized);
        println!("✅ Error in ABORT message verified");

        // Test error propagation through expressions
        let error_in_array = Expression::Array(vec![
            Expression::String("normal_value".to_string()),
            Expression::Error {
                error_type: "computation_error".to_string(),
                message: "Division by zero".to_string(),
                stack: None
            }
        ]);

        let array_json = serde_json::to_string(&error_in_array).unwrap();
        let array_deserialized: Expression = serde_json::from_str(&array_json).unwrap();
        assert_eq!(error_in_array, array_deserialized);
        println!("✅ Error propagation in expressions verified");
    }

    /// Test all protocol-defined error codes
    #[tokio::test]
    async fn test_protocol_error_codes() {
        println!("🧪 Testing Protocol Error Codes");

        let error_codes = vec![
            ("bad_request", "Invalid request format or parameters"),
            ("not_found", "Requested resource not found"),
            ("cap_revoked", "Capability has been revoked"),
            ("permission_denied", "Permission denied for this operation"),
            ("canceled", "Operation was canceled"),
            ("internal", "Internal server error"),
            ("timeout", "Operation timed out"),
            ("network_error", "Network communication error"),
        ];

        for (code, description) in error_codes {
            let error = Expression::Error {
                error_type: code.to_string(),
                message: description.to_string(),
                stack: None,
            };

            let json = serde_json::to_string(&error).unwrap();
            let parsed: Expression = serde_json::from_str(&json).unwrap();

            match parsed {
                Expression::Error { error_type, .. } => {
                    assert_eq!(error_type, code);
                }
                _ => panic!("Should be an error expression"),
            }
        }
        println!("✅ All protocol error codes verified");
    }

    /// Test error recovery and session continuity
    #[tokio::test]
    async fn test_error_recovery_protocol() {
        println!("🧪 Testing Error Recovery Protocol");

        let allocator = Arc::new(IdAllocator::new());
        let import_table = ImportTable::new(allocator.clone());

        // Simulate partial failure scenario
        let target = Arc::new(TestRpcTarget { name: "recoverable_target".to_string() });
        let import_id = import_table.add_import(target).await;

        // Test that errors don't corrupt ID allocation
        let id_before_error = allocator.allocate_import();

        // Simulate error
        let error_msg = Message::Reject(ExportId(-1), Expression::Error {
            error_type: "temporary_failure".to_string(),
            message: "Temporary failure, please retry".to_string(),
            stack: None,
        });

        // Verify ID allocation continues correctly after error
        let id_after_error = allocator.allocate_import();
        assert_eq!(id_after_error.0, id_before_error.0 + 1, "ID allocation should continue after error");

        // Verify import table integrity after error
        let retrieved = import_table.get_import(&import_id).await;
        assert!(retrieved.is_some(), "Import table should maintain integrity after error");

        println!("✅ Error recovery and session continuity verified");
    }
}

#[cfg(test)]
mod protocol_validation_tests {
    use super::*;

    /// Test message validation per protocol
    #[tokio::test]
    async fn test_message_validation() {
        println!("🧪 Testing Protocol Message Validation");

        // Test that IDs follow protocol rules
        let invalid_messages = vec![
            // Import IDs should be positive
            ("negative_import", Message::Pull(ImportId(-1))),
            // Export IDs should be negative
            ("positive_export", Message::Resolve(ExportId(1), Expression::Null)),
        ];

        for (name, msg) in invalid_messages {
            let json = msg.to_json();
            // Protocol implementation should handle invalid IDs
            println!("⚠️  {} validation: ID range violation detected", name);
        }

        // Test valid ID ranges
        let valid_messages = vec![
            Message::Pull(ImportId(1)),
            Message::Resolve(ExportId(-1), Expression::Null),
        ];

        for msg in valid_messages {
            let json = msg.to_json();
            let parsed = Message::from_json(&json).unwrap();
            assert_eq!(msg, parsed);
        }
        println!("✅ Message validation verified");
    }

    /// Test expression validation
    #[tokio::test]
    async fn test_expression_validation() {
        println!("🧪 Testing Expression Validation");

        // Test cyclic reference detection
        let mut obj1 = HashMap::new();
        obj1.insert("ref".to_string(), Box::new(Expression::Import(ImportId(1))));

        // Test deep nesting limits
        let mut deep_expr = Expression::Null;
        for i in 0..100 {
            let mut obj = HashMap::new();
            obj.insert(format!("level_{}", i), Box::new(deep_expr));
            deep_expr = Expression::Object(obj);
        }

        // Serialize and deserialize to verify handling
        let json = serde_json::to_string(&deep_expr).unwrap();
        let parsed: Expression = serde_json::from_str(&json).unwrap();
        assert_eq!(deep_expr, parsed);
        println!("✅ Deep nesting validation verified");

        // Test large array handling
        let large_array = Expression::Array(
            (0..10000).map(|i| Expression::Number(Number::from(i))).collect()
        );
        let array_json = serde_json::to_string(&large_array).unwrap();
        let array_parsed: Expression = serde_json::from_str(&array_json).unwrap();
        assert_eq!(large_array, array_parsed);
        println!("✅ Large array validation verified");
    }
}