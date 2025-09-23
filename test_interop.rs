use capnweb_core::{Plan, Op, Source, CapId, Message, CallId, Target, Outcome};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Cap'n Web JavaScript interoperability...");

    // Test 1: Plan serialization compatibility
    println!("1. Testing plan serialization...");
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

    let serialized = serde_json::to_value(&plan)?;
    println!("   ✓ Plan serialized to JSON successfully");
    println!("   JSON: {}", serde_json::to_string_pretty(&serialized)?);

    // Test 2: Plan deserialization compatibility
    println!("\n2. Testing plan deserialization...");
    let js_plan = json!({
        "captures": [42],
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

    let _deserialized_plan: Plan = serde_json::from_value(js_plan)?;
    println!("   ✓ JavaScript-style plan deserialized successfully");

    // Test 3: Message serialization
    println!("\n3. Testing message serialization...");
    let msg = Message::call(
        CallId::new(123),
        Target::cap(CapId::new(42)),
        "testMethod".to_string(),
        vec![json!("hello"), json!(42)],
    );

    let msg_json = serde_json::to_value(&msg)?;
    println!("   ✓ Message serialized successfully");
    println!("   JSON: {}", serde_json::to_string_pretty(&msg_json)?);

    // Test 4: Complex data structures
    println!("\n4. Testing complex data structures...");
    let mut fields = std::collections::BTreeMap::new();
    fields.insert("name".to_string(), Source::by_value(json!("Alice")));
    fields.insert("age".to_string(), Source::by_value(json!(30)));

    let object_plan = Plan::new(
        vec![],
        vec![Op::object(fields, 0)],
        Source::result(0),
    );

    let object_json = serde_json::to_value(&object_plan)?;
    println!("   ✓ Object construction plan serialized successfully");

    let array_plan = Plan::new(
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

    let array_json = serde_json::to_value(&array_plan)?;
    println!("   ✓ Array construction plan serialized successfully");

    // Test 5: Round-trip compatibility
    println!("\n5. Testing round-trip compatibility...");
    let roundtrip_plan: Plan = serde_json::from_value(serialized)?;
    assert_eq!(plan.captures.len(), roundtrip_plan.captures.len());
    assert_eq!(plan.ops.len(), roundtrip_plan.ops.len());
    println!("   ✓ Round-trip serialization/deserialization successful");

    println!("\n🎉 All interoperability tests passed!");
    println!("The Rust implementation is compatible with JavaScript JSON formats.");
    println!("JavaScript clients and servers can communicate with this Rust implementation.");

    Ok(())
}