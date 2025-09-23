use serde::{Deserialize, Serialize};
use serde_json::json;
use capnweb_core::{Plan, Op, Source, CapId, Message, CallId, Target, Outcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub rust_plan: Option<Plan>,
    pub js_plan: Option<serde_json::Value>,
    pub expected_result: serde_json::Value,
    pub capabilities: Vec<CapabilityFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    PlanExecution,
    MessageSerialization,
    PromisePipelining,
    ErrorHandling,
    CapabilityLifecycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityFixture {
    pub name: String,
    pub cap_id: u64,
    pub methods: Vec<MethodFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodFixture {
    pub name: String,
    pub params: Vec<serde_json::Value>,
    pub returns: serde_json::Value,
}

/// Load test fixtures for interoperability testing
pub fn load_fixtures() -> Vec<TestFixture> {
    vec![
        // Basic capability call test
        TestFixture {
            name: "basic_capability_call".to_string(),
            description: "Test basic capability method call".to_string(),
            test_type: TestType::PlanExecution,
            rust_plan: Some(Plan::new(
                vec![CapId::new(1)],
                vec![Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "add".to_string(),
                    args: vec![
                        Source::ByValue { value: json!(5) },
                        Source::ByValue { value: json!(3) },
                    ],
                    result: 0,
                }],
                Source::Result { index: 0 },
            )),
            js_plan: Some(json!({
                "captures": [1],
                "ops": [{
                    "call": {
                        "target": { "capture": { "index": 0 } },
                        "member": "add",
                        "args": [
                            { "byValue": { "value": 5 } },
                            { "byValue": { "value": 3 } }
                        ],
                        "result": 0
                    }
                }],
                "result": { "result": { "index": 0 } }
            })),
            expected_result: json!(8),
            capabilities: vec![CapabilityFixture {
                name: "calculator".to_string(),
                cap_id: 1,
                methods: vec![MethodFixture {
                    name: "add".to_string(),
                    params: vec![json!(5), json!(3)],
                    returns: json!(8),
                }],
            }],
        },

        // Promise pipelining test
        TestFixture {
            name: "promise_pipelining".to_string(),
            description: "Test promise pipelining with chained calls".to_string(),
            test_type: TestType::PromisePipelining,
            rust_plan: Some(Plan::new(
                vec![CapId::new(1)],
                vec![
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getUser".to_string(),
                        args: vec![Source::ByValue { value: json!(123) }],
                        result: 0,
                    },
                    Op::Call {
                        target: Source::Result { index: 0 },
                        member: "getName".to_string(),
                        args: vec![],
                        result: 1,
                    },
                ],
                Source::Result { index: 1 },
            )),
            js_plan: Some(json!({
                "captures": [1],
                "ops": [
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getUser",
                            "args": [{ "byValue": { "value": 123 } }],
                            "result": 0
                        }
                    },
                    {
                        "call": {
                            "target": { "result": { "index": 0 } },
                            "member": "getName",
                            "args": [],
                            "result": 1
                        }
                    }
                ],
                "result": { "result": { "index": 1 } }
            })),
            expected_result: json!("John Doe"),
            capabilities: vec![CapabilityFixture {
                name: "api".to_string(),
                cap_id: 1,
                methods: vec![
                    MethodFixture {
                        name: "getUser".to_string(),
                        params: vec![json!(123)],
                        returns: json!({"id": 123, "name": "John Doe"}),
                    },
                    MethodFixture {
                        name: "getName".to_string(),
                        params: vec![],
                        returns: json!("John Doe"),
                    },
                ],
            }],
        },

        // Object construction test
        TestFixture {
            name: "object_construction".to_string(),
            description: "Test object construction with multiple fields".to_string(),
            test_type: TestType::PlanExecution,
            rust_plan: Some(Plan::new(
                vec![CapId::new(1)],
                vec![
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getName".to_string(),
                        args: vec![],
                        result: 0,
                    },
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getAge".to_string(),
                        args: vec![],
                        result: 1,
                    },
                    Op::Object {
                        fields: {
                            let mut map = std::collections::BTreeMap::new();
                            map.insert("name".to_string(), Source::Result { index: 0 });
                            map.insert("age".to_string(), Source::Result { index: 1 });
                            map
                        },
                        result: 2,
                    },
                ],
                Source::Result { index: 2 },
            )),
            js_plan: Some(json!({
                "captures": [1],
                "ops": [
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getName",
                            "args": [],
                            "result": 0
                        }
                    },
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getAge",
                            "args": [],
                            "result": 1
                        }
                    },
                    {
                        "object": {
                            "fields": {
                                "name": { "result": { "index": 0 } },
                                "age": { "result": { "index": 1 } }
                            },
                            "result": 2
                        }
                    }
                ],
                "result": { "result": { "index": 2 } }
            })),
            expected_result: json!({"name": "Alice", "age": 30}),
            capabilities: vec![CapabilityFixture {
                name: "person".to_string(),
                cap_id: 1,
                methods: vec![
                    MethodFixture {
                        name: "getName".to_string(),
                        params: vec![],
                        returns: json!("Alice"),
                    },
                    MethodFixture {
                        name: "getAge".to_string(),
                        params: vec![],
                        returns: json!(30),
                    },
                ],
            }],
        },

        // Array construction test
        TestFixture {
            name: "array_construction".to_string(),
            description: "Test array construction with multiple items".to_string(),
            test_type: TestType::PlanExecution,
            rust_plan: Some(Plan::new(
                vec![CapId::new(1)],
                vec![
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getValue".to_string(),
                        args: vec![Source::ByValue { value: json!(1) }],
                        result: 0,
                    },
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getValue".to_string(),
                        args: vec![Source::ByValue { value: json!(2) }],
                        result: 1,
                    },
                    Op::Call {
                        target: Source::Capture { index: 0 },
                        member: "getValue".to_string(),
                        args: vec![Source::ByValue { value: json!(3) }],
                        result: 2,
                    },
                    Op::Array {
                        items: vec![
                            Source::Result { index: 0 },
                            Source::Result { index: 1 },
                            Source::Result { index: 2 },
                        ],
                        result: 3,
                    },
                ],
                Source::Result { index: 3 },
            )),
            js_plan: Some(json!({
                "captures": [1],
                "ops": [
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getValue",
                            "args": [{ "byValue": { "value": 1 } }],
                            "result": 0
                        }
                    },
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getValue",
                            "args": [{ "byValue": { "value": 2 } }],
                            "result": 1
                        }
                    },
                    {
                        "call": {
                            "target": { "capture": { "index": 0 } },
                            "member": "getValue",
                            "args": [{ "byValue": { "value": 3 } }],
                            "result": 2
                        }
                    },
                    {
                        "array": {
                            "items": [
                                { "result": { "index": 0 } },
                                { "result": { "index": 1 } },
                                { "result": { "index": 2 } }
                            ],
                            "result": 3
                        }
                    }
                ],
                "result": { "result": { "index": 3 } }
            })),
            expected_result: json!([10, 20, 30]),
            capabilities: vec![CapabilityFixture {
                name: "valueProvider".to_string(),
                cap_id: 1,
                methods: vec![
                    MethodFixture {
                        name: "getValue".to_string(),
                        params: vec![json!(1)],
                        returns: json!(10),
                    },
                    MethodFixture {
                        name: "getValue".to_string(),
                        params: vec![json!(2)],
                        returns: json!(20),
                    },
                    MethodFixture {
                        name: "getValue".to_string(),
                        params: vec![json!(3)],
                        returns: json!(30),
                    },
                ],
            }],
        },

        // Error handling test
        TestFixture {
            name: "error_handling".to_string(),
            description: "Test error propagation and handling".to_string(),
            test_type: TestType::ErrorHandling,
            rust_plan: Some(Plan::new(
                vec![CapId::new(1)],
                vec![Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "divide".to_string(),
                    args: vec![
                        Source::ByValue { value: json!(10) },
                        Source::ByValue { value: json!(0) },
                    ],
                    result: 0,
                }],
                Source::Result { index: 0 },
            )),
            js_plan: Some(json!({
                "captures": [1],
                "ops": [{
                    "call": {
                        "target": { "capture": { "index": 0 } },
                        "member": "divide",
                        "args": [
                            { "byValue": { "value": 10 } },
                            { "byValue": { "value": 0 } }
                        ],
                        "result": 0
                    }
                }],
                "result": { "result": { "index": 0 } }
            })),
            expected_result: json!({
                "error": {
                    "code": "DIVISION_BY_ZERO",
                    "message": "Cannot divide by zero"
                }
            }),
            capabilities: vec![CapabilityFixture {
                name: "calculator".to_string(),
                cap_id: 1,
                methods: vec![MethodFixture {
                    name: "divide".to_string(),
                    params: vec![json!(10), json!(0)],
                    returns: json!({
                        "error": {
                            "code": "DIVISION_BY_ZERO",
                            "message": "Cannot divide by zero"
                        }
                    }),
                }],
            }],
        },
    ]
}

/// Create message serialization test fixtures
pub fn message_serialization_fixtures() -> Vec<(Message, serde_json::Value)> {
    vec![
        // Call message
        (
            Message::Call {
                id: CallId::new(1),
                target: Target::Cap(CapId::new(42)),
                member: "test".to_string(),
                args: vec![json!({"param": 123})],
            },
            json!({
                "call": {
                    "id": 1,
                    "target": {"cap": {"id": 42}},
                    "member": "test",
                    "args": [{"param": 123}]
                }
            }),
        ),
        // Return message
        (
            Message::Result {
                id: CallId::new(1),
                outcome: Outcome::Success { value: json!("success") },
            },
            json!({
                "result": {
                    "id": 1,
                    "success": {
                        "value": "success"
                    }
                }
            }),
        ),
        // CapRef message
        (
            Message::CapRef { id: CapId::new(1) },
            json!({
                "capRef": {
                    "id": 1
                }
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_load() {
        let fixtures = load_fixtures();
        assert!(!fixtures.is_empty());

        // Verify each fixture has required fields
        for fixture in fixtures {
            assert!(!fixture.name.is_empty());
            assert!(!fixture.description.is_empty());
            assert!(!fixture.capabilities.is_empty());
        }
    }

    #[test]
    fn test_message_serialization_fixtures() {
        let fixtures = message_serialization_fixtures();
        assert!(!fixtures.is_empty());

        // Test that we can serialize/deserialize the messages
        for (message, expected_json) in fixtures {
            let serialized = serde_json::to_value(&message).unwrap();
            assert_eq!(serialized, expected_json);
        }
    }

    #[test]
    fn test_plan_serialization() {
        let fixtures = load_fixtures();

        for fixture in fixtures {
            if let Some(rust_plan) = &fixture.rust_plan {
                // Test that Rust plan can be serialized
                let serialized = serde_json::to_value(rust_plan).unwrap();
                assert!(serialized.is_object());

                // Verify it has the expected structure
                assert!(serialized.get("captures").is_some());
                assert!(serialized.get("ops").is_some());
                assert!(serialized.get("result").is_some());
            }
        }
    }
}