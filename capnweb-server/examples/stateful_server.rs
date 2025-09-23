use capnweb_core::{RpcTarget, RpcError};
use capnweb_core::protocol::tables::Value;
use capnweb_server::{NewCapnWebServer as CapnWebServer, CapnWebServerConfig, init_logging};
use std::sync::Arc;
use async_trait::async_trait;

/// Calculator capability for testing
#[derive(Debug)]
struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a + b).unwrap()))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a * b).unwrap()))
            }
            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                if b == 0.0 {
                    return Err(RpcError::bad_request("Division by zero"));
                }
                Ok(Value::Number(serde_json::Number::from_f64(a / b).unwrap()))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a - b).unwrap()))
            }
            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("Calculator".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            "operations" => Ok(Value::Array(vec![
                Value::String("add".to_string()),
                Value::String("subtract".to_string()),
                Value::String("multiply".to_string()),
                Value::String("divide".to_string()),
            ])),
            _ => Err(RpcError::not_found(format!("Property not found: {}", property))),
        }
    }
}

fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number")),
        _ => Err(RpcError::bad_request("Expected number")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging("logs", "capnweb-server")?;

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Configure server
    let config = CapnWebServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        max_batch_size: 100,
    };

    // Create server with calculator as main capability
    let mut server = CapnWebServer::new(config);
    server.register_main(Arc::new(Calculator));

    println!("📐 Calculator Server with Stateful Session Support");
    println!("================================================");
    println!("✅ Push/Pull message handling with import allocation");
    println!("✅ Session persistence across requests");
    println!("✅ Promise resolution and waiting");
    println!("✅ Session cleanup after inactivity");
    println!("");
    println!("Test with:");
    println!("  cargo run --example official-client-test -p typescript-interop");
    println!("");

    // Run server
    server.run().await?;

    Ok(())
}