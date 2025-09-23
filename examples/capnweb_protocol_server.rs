use capnweb_core::{protocol::Value, RpcTarget, RpcError, ErrorCode};
use capnweb_server::{NewCapnWebServer, CapnWebServerConfig};
use async_trait::async_trait;
use std::sync::Arc;

/// Example Calculator capability that implements the Cap'n Web protocol
#[derive(Debug)]
struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "add requires exactly 2 arguments".to_string(),
                    });
                }

                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                let result = a + b;

                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }

            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "multiply requires exactly 2 arguments".to_string(),
                    });
                }

                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                let result = a * b;

                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }

            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "divide requires exactly 2 arguments".to_string(),
                    });
                }

                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;

                if b == 0.0 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "Division by zero".to_string(),
                    });
                }

                let result = a / b;
                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }

            _ => Err(RpcError {
                code: ErrorCode::NotFound,
                message: format!("Method '{}' not found", method),
            }),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("Calculator".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            _ => Err(RpcError {
                code: ErrorCode::NotFound,
                message: format!("Property '{}' not found", property),
            }),
        }
    }
}

fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => {
            n.as_f64().ok_or_else(|| RpcError {
                code: ErrorCode::BadRequest,
                message: "Invalid number".to_string(),
            })
        }
        _ => Err(RpcError {
            code: ErrorCode::BadRequest,
            message: "Expected number".to_string(),
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Create server configuration
    let config = CapnWebServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8080,
        max_batch_size: 100,
    };

    // Create server
    let mut server = NewCapnWebServer::new(config);

    // Register the Calculator as the main capability (ID 0)
    let calculator = Arc::new(Calculator);
    server.register_main(calculator);

    println!("🧮 Cap'n Web Calculator Server");
    println!("================================");
    println!("Protocol: Cap'n Web (array-based messages)");
    println!("Main capability: Calculator (ID 0)");
    println!("Methods: add, multiply, divide");
    println!();
    println!("Example request:");
    println!(r#"  ["push", ["import", 0, ["add"], [5, 3]]]"#);
    println!();
    println!("Starting server...");

    // Run the server
    server.run().await?;

    Ok(())
}