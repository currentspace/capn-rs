// Rust client demonstrating batch pipelining - mirrors TypeScript batch-pipelining example
// Shows:
// - Batching + pipelining: multiple dependent calls, one round trip
// - Non-batched sequential calls: multiple round trips
// - Full protocol validation against the server

use std::time::Instant;
use std::env;
use serde_json::{json, Value};
use reqwest;
use colored::*;
use capnweb_core::protocol::wire::{
    WireMessage, WireExpression, PropertyKey,
    serialize_wire_batch, parse_wire_batch
};
use capnweb_core::CapId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:3000/rpc/batch".to_string());

    println!("========================================");
    println!("Cap'n Web Rust Client - Batch Pipelining");
    println!("========================================");
    println!("Server: {}", rpc_url);
    println!();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Test 1: Pipelined batch (single round trip)
    println!("{}", "--- Running pipelined (batched, single round trip) ---".blue());
    let (pipelined_result, pipelined_time) = run_pipelined(&client, &rpc_url).await?;

    println!("HTTP POSTs: {}", "1".green());
    println!("Time: {:.2} ms", pipelined_time);
    println!("Authenticated user: {:?}", pipelined_result.user);
    println!("Profile: {:?}", pipelined_result.profile);
    println!("Notifications: {:?}", pipelined_result.notifications);
    println!();

    // Test 2: Sequential calls (multiple round trips)
    println!("{}", "--- Running sequential (non-batched, multiple round trips) ---".blue());
    let (sequential_result, sequential_time) = run_sequential(&client, &rpc_url).await?;

    println!("HTTP POSTs: {}", "3".yellow());
    println!("Time: {:.2} ms", sequential_time);
    println!("Authenticated user: {:?}", sequential_result.user);
    println!("Profile: {:?}", sequential_result.profile);
    println!("Notifications: {:?}", sequential_result.notifications);
    println!();

    // Summary
    println!("{}", "========================================".green());
    println!("{}", "Summary:".bold());
    println!("Pipelined: {} POST, {:.2} ms", "1".green(), pipelined_time);
    println!("Sequential: {} POSTs, {:.2} ms", "3".yellow(), sequential_time);
    println!("Speedup: {:.1}x faster with pipelining", sequential_time / pipelined_time);
    println!("{}", "========================================".green());

    Ok(())
}

#[derive(Debug)]
struct TestResult {
    user: Value,
    profile: Value,
    notifications: Value,
}

async fn run_pipelined(
    client: &reqwest::Client,
    url: &str
) -> Result<(TestResult, f64), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // Build batch request with pipelining
    let messages = vec![
        // 1. Authenticate with session token
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::String("cookie-123".to_string())
                ])),
            }
        ),
        // 2. Get user profile using pipelined user ID
        WireMessage::Push(
            2,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getUserProfile".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::Pipeline {
                        import_id: 1,
                        property_path: Some(vec![PropertyKey::String("id".to_string())]),
                        args: None,
                    }
                ])),
            }
        ),
        // 3. Get notifications using pipelined user ID
        WireMessage::Push(
            3,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getNotifications".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::Pipeline {
                        import_id: 1,
                        property_path: Some(vec![PropertyKey::String("id".to_string())]),
                        args: None,
                    }
                ])),
            }
        ),
        // Pull all results
        WireMessage::Pull(1),
        WireMessage::Pull(2),
        WireMessage::Pull(3),
    ];

    // Serialize and send
    let request_body = serialize_wire_batch(&messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    // Extract results
    let mut user = Value::Null;
    let mut profile = Value::Null;
    let mut notifications = Value::Null;

    for msg in response_messages {
        match msg {
            WireMessage::Resolve(id, expr) => {
                let value = wire_expr_to_value(&expr);
                match id {
                    1 => user = value,
                    2 => profile = value,
                    3 => notifications = value,
                    _ => {}
                }
            }
            WireMessage::Reject(id, expr) => {
                eprintln!("Request {} rejected: {:?}", id, expr);
            }
            _ => {}
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok((TestResult { user, profile, notifications }, elapsed))
}

async fn run_sequential(
    client: &reqwest::Client,
    url: &str
) -> Result<(TestResult, f64), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // 1. Authenticate (first round trip)
    let auth_messages = vec![
        WireMessage::Push(
            1,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("authenticate".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::String("cookie-123".to_string())
                ])),
            }
        ),
        WireMessage::Pull(1),
    ];

    let request_body = serialize_wire_batch(&auth_messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut user = Value::Null;
    let mut user_id = String::new();

    for msg in response_messages {
        if let WireMessage::Resolve(1, expr) = msg {
            user = wire_expr_to_value(&expr);
            if let Some(id) = user.get("id").and_then(|v| v.as_str()) {
                user_id = id.to_string();
            }
        }
    }

    // 2. Get profile (second round trip)
    let profile_messages = vec![
        WireMessage::Push(
            2,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getUserProfile".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::String(user_id.clone())
                ])),
            }
        ),
        WireMessage::Pull(2),
    ];

    let request_body = serialize_wire_batch(&profile_messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut profile = Value::Null;
    for msg in response_messages {
        if let WireMessage::Resolve(2, expr) = msg {
            profile = wire_expr_to_value(&expr);
        }
    }

    // 3. Get notifications (third round trip)
    let notif_messages = vec![
        WireMessage::Push(
            3,
            WireExpression::Call {
                cap_id: 1,
                property_path: vec![PropertyKey::String("getNotifications".to_string())],
                args: Box::new(WireExpression::Array(vec![
                    WireExpression::String(user_id)
                ])),
            }
        ),
        WireMessage::Pull(3),
    ];

    let request_body = serialize_wire_batch(&notif_messages)?;
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    let response_messages = parse_wire_batch(&response_text)?;

    let mut notifications = Value::Null;
    for msg in response_messages {
        if let WireMessage::Resolve(3, expr) = msg {
            notifications = wire_expr_to_value(&expr);
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok((TestResult { user, profile, notifications }, elapsed))
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