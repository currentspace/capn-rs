use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::Value;
use std::sync::Arc;

/// Calculator capability for testing WebSocket and HTTP batch transport
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
                Ok(serde_json::json!(a + b))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(serde_json::json!(a * b))
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
                Ok(serde_json::json!(a / b))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(serde_json::json!(a - b))
            }
            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }
}

fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| RpcError::bad_request("Invalid number")),
        _ => Err(RpcError::bad_request("Expected number")),
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9000);

    // Configure server
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port,
        max_batch_size: 100,
    };

    // Create server and register calculator as main capability
    let server = Server::new(config);
    server.register_capability(CapId::new(1), Arc::new(Calculator));

    println!("🧮 Calculator Server (HTTP Batch + WebSocket)");
    println!("============================================");
    println!("Port: {}", port);
    println!("HTTP Batch: http://127.0.0.1:{}/rpc/batch", port);
    println!("WebSocket:  ws://127.0.0.1:{}/rpc/ws", port);
    println!("Health:     http://127.0.0.1:{}/health", port);
    println!("");
    println!("Calculator capabilities:");
    println!("  add(a, b)      -> a + b");
    println!("  subtract(a, b) -> a - b");
    println!("  multiply(a, b) -> a * b");
    println!("  divide(a, b)   -> a / b");
    println!("");
    println!("Test with:");
    println!("  cd typescript-interop && pnpm test:tier2 {}", port);
    println!(
        "  cd typescript-interop && pnpm test:tier2-websocket {}",
        port
    );
    println!("");

    // Run server
    server.run().await
}
