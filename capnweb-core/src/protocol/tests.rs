#[cfg(test)]
mod protocol_tests {
    use super::super::*;
    use serde_json::json;

    #[test]
    fn test_message_push_serialization() {
        let msg = Message::Push(Expression::String("test".to_string()));
        let json = msg.to_json();

        assert_eq!(json, json!(["push", "test"]));

        // Roundtrip test
        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_pull_serialization() {
        let msg = Message::Pull(ImportId(42));
        let json = msg.to_json();

        assert_eq!(json, json!(["pull", 42]));

        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_resolve_serialization() {
        let expr = Expression::Number(serde_json::Number::from(123));
        let msg = Message::Resolve(ExportId(-1), expr.clone());
        let json = msg.to_json();

        assert_eq!(json, json!(["resolve", -1, 123]));

        let deserialized = Message::from_json(&json).unwrap();
        match deserialized {
            Message::Resolve(id, ref e) => {
                assert_eq!(id, ExportId(-1));
                assert_eq!(e, &expr);
            }
            _ => panic!("Expected Resolve message"),
        }
    }

    #[test]
    fn test_message_reject_serialization() {
        let msg = Message::Reject(
            ExportId(-2),
            Expression::Error(ErrorExpression {
                error_type: "TypeError".to_string(),
                message: "Something went wrong".to_string(),
                stack: None,
            }),
        );
        let json = msg.to_json();

        assert_eq!(
            json,
            json!(["reject", -2, ["error", "TypeError", "Something went wrong"]])
        );
    }

    #[test]
    fn test_message_release_serialization() {
        let msg = Message::Release(ImportId(5), 2);
        let json = msg.to_json();

        assert_eq!(json, json!(["release", 5, 2]));

        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_message_abort_serialization() {
        let msg = Message::Abort(Expression::String("Connection lost".to_string()));
        let json = msg.to_json();

        assert_eq!(json, json!(["abort", "Connection lost"]));

        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_date_expression() {
        let expr = Expression::Date(1234567890.0);
        let json = expr.to_json();

        assert_eq!(json, json!(["date", 1234567890.0]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_error_expression() {
        let expr = Expression::Error(ErrorExpression {
            error_type: "ReferenceError".to_string(),
            message: "x is not defined".to_string(),
            stack: Some("at line 10".to_string()),
        });
        let json = expr.to_json();

        assert_eq!(
            json,
            json!(["error", "ReferenceError", "x is not defined", "at line 10"])
        );

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_import_expression() {
        let expr = Expression::Import(ImportExpression {
            import_id: ImportId(1),
            property_path: Some(vec![expression::PropertyKey::String("method".to_string())]),
            call_arguments: Some(Box::new(Expression::Array(vec![
                Expression::Number(serde_json::Number::from(1)),
                Expression::Number(serde_json::Number::from(2)),
            ]))),
        });
        let json = expr.to_json();

        assert_eq!(json, json!(["import", 1, ["method"], [1, 2]]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_pipeline_expression() {
        let expr = Expression::Pipeline(PipelineExpression {
            import_id: ImportId(3),
            property_path: Some(vec![
                expression::PropertyKey::String("users".to_string()),
                expression::PropertyKey::Number(0),
                expression::PropertyKey::String("name".to_string()),
            ]),
            call_arguments: None,
        });
        let json = expr.to_json();

        assert_eq!(json, json!(["pipeline", 3, ["users", 0, "name"]]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_export_expression() {
        let expr = Expression::Export(ExportExpression {
            export_id: ExportId(-5),
        });
        let json = expr.to_json();

        assert_eq!(json, json!(["export", -5]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_promise_expression() {
        let expr = Expression::Promise(PromiseExpression {
            export_id: ExportId(-10),
        });
        let json = expr.to_json();

        assert_eq!(json, json!(["promise", -10]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_escaped_array() {
        let expr = Expression::EscapedArray(vec![
            Expression::String("just".to_string()),
            Expression::String("an".to_string()),
            Expression::String("array".to_string()),
        ]);
        let json = expr.to_json();

        assert_eq!(json, json!([["just", "an", "array"]]));

        let parsed = Expression::from_json(&json).unwrap();
        assert_eq!(expr, parsed);
    }

    #[test]
    fn test_id_allocation() {
        let allocator = IdAllocator::new();

        // Test import allocation
        let import1 = allocator.allocate_import();
        let import2 = allocator.allocate_import();
        assert_eq!(import1, ImportId(1));
        assert_eq!(import2, ImportId(2));

        // Test export allocation
        let export1 = allocator.allocate_export();
        let export2 = allocator.allocate_export();
        assert_eq!(export1, ExportId(-1));
        assert_eq!(export2, ExportId(-2));
    }

    #[test]
    fn test_import_table_operations() {
        use std::sync::Arc;

        let allocator = Arc::new(IdAllocator::new());
        let table = ImportTable::new(allocator);

        let id = table.allocate_local();
        assert_eq!(id, ImportId(1));

        // Insert a value
        table
            .insert(id, ImportValue::Value(Value::String("test".to_string())))
            .unwrap();

        // Get the value
        match table.get(id).unwrap() {
            ImportValue::Value(Value::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }

        // Test refcounting
        table.add_ref(id).unwrap();
        assert!(!table.release(id, 1).unwrap()); // Should not remove
        assert!(table.release(id, 1).unwrap()); // Should remove
        assert!(table.get(id).is_none());
    }

    #[tokio::test]
    async fn test_export_table_promise() {
        use std::sync::Arc;

        let allocator = Arc::new(IdAllocator::new());
        let table = ExportTable::new(allocator);

        let (id, mut rx) = table.export_promise();
        assert_eq!(id, ExportId(-1));

        // Resolve the promise
        table
            .resolve(id, Value::Number(serde_json::Number::from(42)))
            .await
            .unwrap();

        // Check the receiver
        rx.changed().await.unwrap();
        {
            let borrowed = rx.borrow();
            match borrowed.as_ref().unwrap() {
                Ok(Value::Number(n)) => assert_eq!(n.as_i64(), Some(42)),
                _ => panic!("Expected resolved number"),
            }
        }
    }

    #[test]
    fn test_complex_message_roundtrip() {
        // Test a complex nested message
        let complex_expr = Expression::Object({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "method".to_string(),
                Box::new(Expression::String("getData".to_string())),
            );
            map.insert(
                "args".to_string(),
                Box::new(Expression::Array(vec![
                    Expression::Number(serde_json::Number::from(1)),
                    Expression::Bool(true),
                    Expression::Null,
                ])),
            );
            map
        });

        let msg = Message::Push(complex_expr);
        let json = msg.to_json();

        // Verify it can be deserialized
        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}
