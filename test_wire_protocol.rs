// Test the Cap'n Web wire protocol parsing independently

use capnweb_core::{parse_wire_batch, serialize_wire_batch, WireMessage, WireExpression, PropertyKey};
use serde_json::Number;

fn main() {
    println!("🧪 Testing Cap'n Web Wire Protocol");
    println!("==================================");

    // Test parsing the TypeScript client format
    let typescript_request = r#"["push",["pipeline",0,["add"],[5,3]]]
["pull",1]"#;

    println!("📥 Parsing TypeScript client request:");
    println!("{}", typescript_request);

    match parse_wire_batch(typescript_request) {
        Ok(messages) => {
            println!("\n✅ Successfully parsed {} messages:", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                println!("  Message {}: {:?}", i, msg);
            }

            // Test response generation
            println!("\n📤 Generating response:");
            let response_messages = vec![
                WireMessage::Resolve(-1, WireExpression::Number(Number::from(8)))
            ];

            let response = serialize_wire_batch(&response_messages);
            println!("Response: {}", response);

        }
        Err(e) => {
            println!("❌ Failed to parse: {}", e);
        }
    }

    // Test individual message parsing
    println!("\n🔍 Testing individual message types:");

    // Test push with pipeline
    test_message_parse(r#"["push",["pipeline",0,["add"],[5,3]]]"#, "Push with Pipeline");

    // Test pull
    test_message_parse(r#"["pull",1]"#, "Pull");

    // Test resolve
    test_message_parse(r#"["resolve",-1,8]"#, "Resolve");

    // Test reject with error
    test_message_parse(r#"["reject",-1,["error","bad_request","Invalid arguments"]]"#, "Reject with Error");

    // Test release
    test_message_parse(r#"["release",[1,2,3]]"#, "Release");
}

fn test_message_parse(json_str: &str, description: &str) {
    println!("\n  Testing {}: {}", description, json_str);

    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            println!("    ❌ Invalid JSON: {}", e);
            return;
        }
    };

    let array = match json.as_array() {
        Some(arr) => arr,
        None => {
            println!("    ❌ Not an array");
            return;
        }
    };

    match WireMessage::from_json_array(array) {
        Ok(msg) => println!("    ✅ Parsed: {:?}", msg),
        Err(e) => println!("    ❌ Failed to parse: {}", e),
    }
}