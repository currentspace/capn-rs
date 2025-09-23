use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio;

/// A simple calculator capability for testing interoperability
struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
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
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires exactly 2 arguments"));
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
            "echo" => {
                if args.is_empty() {
                    return Err(RpcError::bad_request("echo requires at least 1 argument"));
                }

                // Return the first argument as-is
                Ok(args[0].clone())
            }
            _ => Err(RpcError::not_found(&format!("Method '{}' not found", member))),
        }
    }
}

/// A simple user management capability
struct UserManager;

#[async_trait]
impl RpcTarget for UserManager {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
            "getUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getUser requires exactly 1 argument"));
                }

                let user_id = args[0]
                    .as_u64()
                    .ok_or_else(|| RpcError::bad_request("User ID must be a number"))?;

                // Return a mock user object
                Ok(json!({
                    "id": user_id,
                    "name": "John Doe",
                    "email": "john.doe@example.com"
                }))
            }
            "getName" => {
                // This would typically be called on a user object
                Ok(json!("John Doe"))
            }
            "getAge" => {
                Ok(json!(30))
            }
            "getValue" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getValue requires exactly 1 argument"));
                }

                let multiplier = args[0]
                    .as_u64()
                    .ok_or_else(|| RpcError::bad_request("Argument must be a number"))?;

                // Return the input multiplied by 10
                Ok(json!(multiplier * 10))
            }
            _ => Err(RpcError::not_found(&format!("Method '{}' not found", member))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Cap'n Web Rust server...");

    // Create server configuration
    let config = ServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    println!("📡 Server configuration:");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Max batch size: {}", config.max_batch_size);

    // Create the server
    let server = Server::new(config);

    // Register capabilities
    server.register_capability(CapId::new(1), Arc::new(Calculator));
    server.register_capability(CapId::new(2), Arc::new(UserManager));

    println!("🎯 Registered capabilities:");
    println!("   1: Calculator (add, multiply, divide, echo)");
    println!("   2: UserManager (getUser, getName, getAge, getValue)");

    println!("✅ Server ready and listening for connections!");
    println!("🔗 Endpoints:");
    println!("   HTTP Batch: http://{}:{}/rpc/batch", config.host, config.port);
    println!("   WebSocket: ws://{}:{}/rpc/ws", config.host, config.port);
    println!("   Health check: http://{}:{}/health", config.host, config.port);

    // Run the server
    server.run().await
}