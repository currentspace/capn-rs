//! Simplified interop tests focusing on JSON serialization compatibility

use capnweb_core::{Plan, Op, Source, CapId, Message, CallId, Target, Outcome};
use serde_json::json;

/// Test that Rust plans can be serialized to JSON in a JavaScript-compatible format
pub fn test_plan_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple plan with a capability call
    let plan = Plan::new(
        vec![CapId::new(1)],
        vec![Op::call(
            Source::capture(0),
            "add".to_string(),
            vec![
                Source::by_value(json!(5)),
                Source::by_value(json!(3)),
            ],
            0,
        )],
        Source::result(0),
    );

    // Serialize to JSON
    let serialized = serde_json::to_value(&plan)?;

    // Verify structure matches expected JavaScript format
    assert!(serialized.get("captures").is_some());
    assert!(serialized.get("ops").is_some());
    assert!(serialized.get("result").is_some());

    let captures = serialized["captures"].as_array().unwrap();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0], json!(1));

    let ops = serialized["ops"].as_array().unwrap();
    assert_eq!(ops.len(), 1);

    println!("✓ Plan serialization test passed");
    Ok(())
}

/// Test that JavaScript-style plans can be deserialized to Rust
pub fn test_plan_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let js_plan = json!({
        "captures": [1],
        "ops": [{
            "call": {
                "target": { "capture": { "index": 0 } },
                "member": "multiply",
                "args": [
                    { "byValue": { "value": 6 } },
                    { "byValue": { "value": 7 } }
                ],
                "result": 0
            }
        }],
        "result": { "result": { "index": 0 } }
    });

    // Deserialize from JSON
    let plan: Plan = serde_json::from_value(js_plan)?;

    // Verify structure
    assert_eq!(plan.captures.len(), 1);
    assert_eq!(plan.ops.len(), 1);

    if let Op::Call { call } = &plan.ops[0] {
        assert_eq!(call.member, "multiply");
        assert_eq!(call.args.len(), 2);
        assert_eq!(call.result, 0);

        if let Source::Capture { capture } = &call.target {
            assert_eq!(capture.index, 0);
        } else {
            panic!("Expected capture target");
        }
    } else {
        panic!("Expected call operation");
    }

    println!("✓ Plan deserialization test passed");
    Ok(())
}

/// Test message serialization compatibility
pub fn test_message_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test Call message
    let call_msg = Message::call(
        CallId::new(123),
        Target::cap(CapId::new(42)),
        "testMethod".to_string(),
        vec![json!("hello"), json!(42)],
    );

    let serialized = serde_json::to_value(&call_msg)?;

    // Verify structure
    assert!(serialized.get("call").is_some());
    let call_data = &serialized["call"];
    assert_eq!(call_data["id"], json!(123));
    assert_eq!(call_data["member"], json!("testMethod"));
    assert!(call_data["args"].is_array());

    // Test Result message
    let result_msg = Message::result(
        CallId::new(123),
        Outcome::Success { value: json!("result") },
    );

    let serialized = serde_json::to_value(&result_msg)?;
    assert!(serialized.get("result").is_some());

    println!("✓ Message serialization test passed");
    Ok(())
}

/// Test complex data structure serialization
pub fn test_complex_structures() -> Result<(), Box<dyn std::error::Error>> {
    // Test object construction
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("name".to_string(), Source::by_value(json!("Alice")));
    fields.insert("age".to_string(), Source::by_value(json!(30)));

    let plan = Plan::new(
        vec![],
        vec![Op::object(fields, 0)],
        Source::result(0),
    );

    let serialized = serde_json::to_value(&plan)?;

    // Verify object structure
    let ops = serialized["ops"].as_array().unwrap();
    let obj_op = &ops[0]["object"];
    assert!(obj_op["fields"].is_object());

    // Test array construction
    let plan = Plan::new(
        vec![],
        vec![Op::array(
            vec![
                Source::by_value(json!(1)),
                Source::by_value(json!(2)),
                Source::by_value(json!(3)),
            ],
            0,
        )],
        Source::result(0),
    );

    let serialized = serde_json::to_value(&plan)?;
    let ops = serialized["ops"].as_array().unwrap();
    let array_op = &ops[0]["array"];
    assert_eq!(array_op["items"].as_array().unwrap().len(), 3);

    println!("✓ Complex structures test passed");
    Ok(())
}

/// Run all interop tests
pub fn run_all_interop_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running JavaScript interoperability tests...");

    test_plan_serialization()?;
    test_plan_deserialization()?;
    test_message_serialization()?;
    test_complex_structures()?;

    println!("✓ All interoperability tests passed!");
    println!("The Rust implementation is compatible with JavaScript JSON formats.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_interop() {
        run_all_interop_tests().expect("All interop tests should pass");
    }

    #[test]
    fn test_plan_roundtrip() {
        // Test that we can serialize and deserialize a plan
        let original_plan = Plan::new(
            vec![CapId::new(1)],
            vec![Op::call(
                Source::capture(0),
                "test".to_string(),
                vec![Source::by_value(json!(42))],
                0,
            )],
            Source::result(0),
        );

        let json_value = serde_json::to_value(&original_plan).unwrap();
        let deserialized_plan: Plan = serde_json::from_value(json_value).unwrap();

        // Verify they match
        assert_eq!(original_plan.captures.len(), deserialized_plan.captures.len());
        assert_eq!(original_plan.ops.len(), deserialized_plan.ops.len());
    }

    #[test]
    fn test_message_roundtrip() {
        let original_msg = Message::call(
            CallId::new(42),
            Target::cap(CapId::new(1)),
            "test".to_string(),
            vec![json!("arg1"), json!(123)],
        );

        let json_value = serde_json::to_value(&original_msg).unwrap();
        let deserialized_msg: Message = serde_json::from_value(json_value).unwrap();

        // Verify message types match
        match (&original_msg, &deserialized_msg) {
            (Message::Call { call: call1 }, Message::Call { call: call2 }) => {
                assert_eq!(call1.id, call2.id);
            }
            _ => panic!("Message types should match"),
        }
    }

    #[test]
    fn test_error_outcome_serialization() {
        let error_msg = Message::result(
            CallId::new(1),
            Outcome::Error {
                error: capnweb_core::RpcError::bad_request("test error"),
            },
        );

        let json_value = serde_json::to_value(&error_msg).unwrap();
        assert!(json_value.get("result").is_some());

        // Should be able to deserialize back
        let _: Message = serde_json::from_value(json_value).unwrap();
    }
}