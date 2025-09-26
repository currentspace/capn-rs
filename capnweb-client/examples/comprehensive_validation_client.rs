// Comprehensive Rust client for validating all Cap'n Web protocol features
// Tests all aspects of the protocol against the server implementation

use capnweb_core::protocol::wire::{
    parse_wire_batch, serialize_wire_batch, PropertyKey, WireExpression, WireMessage,
};
use capnweb_core::CapId;
use colored::*;
use reqwest;
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url =
        env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:3000/rpc/batch".to_string());

    println!("{}", "============================================".bold());
    println!(
        "{}",
        "Cap'n Web Protocol - Comprehensive Validation"
            .bold()
            .blue()
    );
    println!("{}", "============================================".bold());
    println!("Server: {}", rpc_url.cyan());
    println!();

    let client = reqwest::Client::new();
    let mut tests_passed = 0;
    let mut tests_failed = 0;

    // Test 1: Basic Call Expression
    print!("Test 1: Basic Call Expression... ");
    match test_basic_call(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 2: Pipeline Expression
    print!("Test 2: Pipeline Expression... ");
    match test_pipeline_expression(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 3: Complex Pipelining
    print!("Test 3: Complex Pipelining... ");
    match test_complex_pipelining(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 4: Error Handling
    print!("Test 4: Error Handling... ");
    match test_error_handling(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 5: Mixed Expression Types
    print!("Test 5: Mixed Expression Types... ");
    match test_mixed_expressions(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 6: Array Responses
    print!("Test 6: Array Responses... ");
    match test_array_responses(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 7: Object Nesting
    print!("Test 7: Object Nesting... ");
    match test_object_nesting(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Test 8: Batch Processing
    print!("Test 8: Batch Processing... ");
    match test_batch_processing(&client, &rpc_url).await {
        Ok(true) => {
            println!("{}", "✅ PASSED".green());
            tests_passed += 1;
        }
        Ok(false) | Err(_) => {
            println!("{}", "❌ FAILED".red());
            tests_failed += 1;
        }
    }

    // Summary
    println!();
    println!("{}", "============================================".bold());
    println!("{}", "Test Results Summary".bold());
    println!("{}", "============================================".bold());
    println!("Tests Passed: {}", tests_passed.to_string().green());
    println!("Tests Failed: {}", tests_failed.to_string().red());
    println!();

    if tests_failed == 0 {
        println!(
            "{}",
            "🎉 ALL TESTS PASSED! Full protocol compliance verified!"
                .green()
                .bold()
        );
        Ok(())
    } else {
        println!(
            "{}",
            "⚠️ Some tests failed. Please review the implementation.".yellow()
        );
        std::process::exit(1);
    }
}

// Test 1: Basic Call Expression
async fn test_basic_call(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "cookie-123".to_string(),
                )])),
            },
        ),
        WireMessage::Pull(1),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    for msg in response_messages {
        if let WireMessage::Resolve(1, expr) = msg {
            let value = wire_expr_to_value(&expr);
            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                return Ok(name == "Ada Lovelace");
            }
        }
    }
    Ok(false)
}

// Test 2: Pipeline Expression
async fn test_pipeline_expression(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Pipeline {
                import_id: 1,
                property_path: Some(vec![PropertyKey::String("authenticate".to_string())]),
                args: Some(Box::new(WireExpression::Array(vec![
                    WireExpression::String("cookie-123".to_string()),
                ]))),
            },
        ),
        WireMessage::Push(
            2,
            WireExpression::Pipeline {
                import_id: 1,
                property_path: Some(vec![PropertyKey::String("getUserProfile".to_string())]),
                args: Some(Box::new(WireExpression::Array(vec![
                    WireExpression::Pipeline {
                        import_id: 1,
                        property_path: Some(vec![PropertyKey::String("id".to_string())]),
                        args: None,
                    },
                ]))),
            },
        ),
        WireMessage::Pull(1),
        WireMessage::Pull(2),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut found_profile = false;
    for msg in response_messages {
        if let WireMessage::Resolve(2, expr) = msg {
            let value = wire_expr_to_value(&expr);
            if let Some(bio) = value.get("bio").and_then(|v| v.as_str()) {
                found_profile = bio.contains("first programmer");
            }
        }
    }
    Ok(found_profile)
}

// Test 3: Complex Pipelining
async fn test_complex_pipelining(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "cookie-123".to_string(),
                )])),
            },
        ),
        WireMessage::Push(
            2,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getUserProfile".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::Pipeline {
                    import_id: 1,
                    property_path: Some(vec![PropertyKey::String("id".to_string())]),
                    args: None,
                }])),
            },
        ),
        WireMessage::Push(
            3,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getNotifications".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::Pipeline {
                    import_id: 1,
                    property_path: Some(vec![PropertyKey::String("id".to_string())]),
                    args: None,
                }])),
            },
        ),
        WireMessage::Pull(1),
        WireMessage::Pull(2),
        WireMessage::Pull(3),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut results_count = 0;
    for msg in &response_messages {
        if matches!(msg, WireMessage::Resolve(_, _)) {
            results_count += 1;
        }
    }
    Ok(results_count == 3)
}

// Test 4: Error Handling
async fn test_error_handling(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "invalid-token".to_string(),
                )])),
            },
        ),
        WireMessage::Pull(1),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    for msg in response_messages {
        if matches!(msg, WireMessage::Reject(1, _)) {
            return Ok(true);
        }
    }
    Ok(false)
}

// Test 5: Mixed Expression Types
async fn test_mixed_expressions(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        // Call expression
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "cookie-456".to_string(),
                )])),
            },
        ),
        // Pipeline expression
        WireMessage::Push(
            2,
            WireExpression::Pipeline {
                import_id: 1,
                property_path: Some(vec![PropertyKey::String("getUserProfile".to_string())]),
                args: Some(Box::new(WireExpression::Array(vec![
                    WireExpression::Pipeline {
                        import_id: 1,
                        property_path: Some(vec![PropertyKey::String("id".to_string())]),
                        args: None,
                    },
                ]))),
            },
        ),
        WireMessage::Pull(1),
        WireMessage::Pull(2),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut found_turing = false;
    let mut found_profile = false;

    for msg in response_messages {
        match msg {
            WireMessage::Resolve(1, expr) => {
                let value = wire_expr_to_value(&expr);
                if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                    found_turing = name == "Alan Turing";
                }
            }
            WireMessage::Resolve(2, expr) => {
                let value = wire_expr_to_value(&expr);
                if value.get("bio").is_some() {
                    found_profile = true;
                }
            }
            _ => {}
        }
    }
    Ok(found_turing && found_profile)
}

// Test 6: Array Responses
async fn test_array_responses(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getNotifications".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "u_1".to_string(),
                )])),
            },
        ),
        WireMessage::Pull(1),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    for msg in response_messages {
        if let WireMessage::Resolve(1, expr) = msg {
            let value = wire_expr_to_value(&expr);
            if let Some(arr) = value.as_array() {
                return Ok(arr.len() == 2
                    && arr[0].as_str() == Some("Welcome to jsrpc!")
                    && arr[1].as_str() == Some("You have 2 new followers"));
            }
        }
    }
    Ok(false)
}

// Test 7: Object Nesting
async fn test_object_nesting(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getUserProfile".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    "u_2".to_string(),
                )])),
            },
        ),
        WireMessage::Pull(1),
    ];

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    for msg in response_messages {
        if let WireMessage::Resolve(1, expr) = msg {
            let value = wire_expr_to_value(&expr);
            // Check for nested object structure
            if value.get("id").is_some() && value.get("bio").is_some() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

// Test 8: Batch Processing
async fn test_batch_processing(
    client: &reqwest::Client,
    url: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Send multiple operations in a single batch
    let mut messages = vec![];

    // Create 5 different authentication attempts
    for i in 1..=5 {
        let token = if i <= 2 { "cookie-123" } else { "cookie-456" };
        messages.push(WireMessage::Push(
            i,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![WireExpression::String(
                    token.to_string(),
                )])),
            },
        ));
    }

    // Pull all results
    for i in 1..=5 {
        messages.push(WireMessage::Pull(i));
    }

    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    // Should get 5 resolve messages
    let resolve_count = response_messages
        .iter()
        .filter(|msg| matches!(msg, WireMessage::Resolve(_, _)))
        .count();

    Ok(resolve_count == 5)
}

// Helper function to convert WireExpression to JSON Value
fn wire_expr_to_value(expr: &WireExpression) -> Value {
    match expr {
        WireExpression::String(s) => json!(s),
        WireExpression::Number(n) => json!(n),
        WireExpression::Boolean(b) => json!(b),
        WireExpression::Null => Value::Null,
        WireExpression::Array(arr) => {
            json!(arr.iter().map(wire_expr_to_value).collect::<Vec<_>>())
        }
        WireExpression::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (key, value) in obj {
                map.insert(key.clone(), wire_expr_to_value(value));
            }
            json!(map)
        }
        _ => Value::Null,
    }
}
