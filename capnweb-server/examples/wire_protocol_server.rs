// Official Cap'n Web Wire Protocol Server Example
// This example implements the official Cap'n Web protocol using newline-delimited arrays

use async_trait::async_trait;
use capnweb_core::RpcError;
use capnweb_server::{WireCapability, WireServer, WireServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing_subscriber;

/// A calculator capability that implements the official Cap'n Web protocol
#[derive(Debug)]
struct Calculator;

#[async_trait]
impl WireCapability for Calculator {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        println!("Calculator::{} called with args: {:?}", method, args);

        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a + b))
            }

            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "subtract requires exactly 2 arguments",
                    ));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a - b))
            }

            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "multiply requires exactly 2 arguments",
                    ));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;
                Ok(json!(a * b))
            }

            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires exactly 2 arguments"));
                }
                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;

                if b == 0.0 {
                    return Err(RpcError::bad_request("Division by zero"));
                }

                Ok(json!(a / b))
            }

            _ => Err(RpcError::not_found(format!(
                "Method '{}' not found on Calculator",
                method
            ))),
        }
    }
}

/// An echo service for testing
#[derive(Debug)]
struct EchoService;

#[async_trait]
impl WireCapability for EchoService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        println!("EchoService::{} called with args: {:?}", method, args);

        match method {
            "echo" => Ok(json!({
                "echoed": args,
                "count": args.len()
            })),

            "reverse" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("reverse requires exactly 1 argument"));
                }
                if let Some(s) = args[0].as_str() {
                    Ok(json!(s.chars().rev().collect::<String>()))
                } else {
                    Err(RpcError::bad_request("Argument must be a string"))
                }
            }

            _ => Err(RpcError::not_found(format!(
                "Method '{}' not found on EchoService",
                method
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("capnweb_server=info,wire_protocol_server=info")
        .init();

    println!("🚀 Starting Cap'n Web Wire Protocol Server");
    println!("==========================================");
    println!("This server implements the official Cap'n Web protocol:");
    println!("- Newline-delimited JSON arrays");
    println!("- Pipeline expressions for method calls");
    println!("- Import/export ID system");
    println!();

    // Create server configuration
    let config = WireServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    // Create and configure the server
    let server = WireServer::new(config);

    // Register capabilities
    // Use import ID 0 for Calculator (the TypeScript client pipeline uses ID 0)
    server.register_capability(0, Arc::new(Calculator));

    // Use import ID 1 for EchoService
    server.register_capability(1, Arc::new(EchoService));

    println!("📋 Registered capabilities:");
    println!("  - Calculator (ID: 0) - Methods: add, subtract, multiply, divide");
    println!("  - EchoService (ID: 1) - Methods: echo, reverse");
    println!();

    println!("💡 Example TypeScript client usage:");
    println!("  const session = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');");
    println!("  const result = await session.add(5, 3); // Returns 8");
    println!();

    println!("📡 Wire protocol format:");
    println!("  Request:  [\"push\",[\"pipeline\",0,[\"add\"],[5,3]]]");
    println!("            [\"pull\",1]");
    println!("  Response: [\"resolve\",-1,8]");
    println!();

    // Start the server
    server.run().await?;

    Ok(())
}
