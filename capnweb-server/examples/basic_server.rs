use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;

/// A simple calculator capability
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
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires exactly 2 arguments"));
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
            _ => Err(RpcError::not_found(format!(
                "Method '{}' not found on Calculator",
                member
            ))),
        }
    }
}

/// A simple echo service capability
struct EchoService;

#[async_trait]
impl RpcTarget for EchoService {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match member {
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
                member
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the server
    let config = ServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    // Create the server
    let server = Server::new(config);

    // Register capabilities
    let calculator = Arc::new(Calculator);
    let echo = Arc::new(EchoService);

    server.register_capability(CapId::new(1), calculator);
    server.register_capability(CapId::new(2), echo);

    println!("Starting Cap'n Web server with the following capabilities:");
    println!("  - Calculator (ID: 1) - Methods: add, subtract, multiply, divide");
    println!("  - EchoService (ID: 2) - Methods: echo, reverse");
    println!();
    println!("Example request:");
    println!("  POST http://127.0.0.1:8080/rpc/batch");
    println!("  Content-Type: application/json");
    println!();
    println!(r#"  [{{"type":"call","id":1,"target":1,"member":"add","args":[5,3]}}]"#);
    println!();

    // Run the server
    server.run().await?;

    Ok(())
}