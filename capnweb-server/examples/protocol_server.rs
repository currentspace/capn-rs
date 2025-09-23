use capnweb_core::{protocol::Value, RpcTarget, RpcError, ErrorCode};
use capnweb_server::capnweb_server::{CapnWebServer, CapnWebServerConfig};
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
                        data: None,
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
                        data: None,
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
                        data: None,
                    });
                }

                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;

                if b == 0.0 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "Division by zero".to_string(),
                        data: None,
                    });
                }

                let result = a / b;
                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }

            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError {
                        code: ErrorCode::BadRequest,
                        message: "subtract requires exactly 2 arguments".to_string(),
                        data: None,
                    });
                }

                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                let result = a - b;

                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }

            _ => Err(RpcError {
                code: ErrorCode::NotFound,
                message: format!("Method '{}' not found", method),
                data: None,
            }),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("Calculator".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            "pi" => Ok(Value::Number(serde_json::Number::from_f64(std::f64::consts::PI).unwrap())),
            _ => Err(RpcError {
                code: ErrorCode::NotFound,
                message: format!("Property '{}' not found", property),
                data: None,
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
                data: None,
            })
        }
        _ => Err(RpcError {
            code: ErrorCode::BadRequest,
            message: "Expected number".to_string(),
            data: None,
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
    let mut server = CapnWebServer::new(config);

    // Register the Calculator as the main capability (ID 0)
    let calculator = Arc::new(Calculator);
    server.register_main(calculator);

    println!("🧮 Cap'n Web Protocol Server (Calculator Example)");
    println!("=================================================");
    println!();
    println!("Protocol: Cap'n Web (array-based messages)");
    println!("Endpoint: http://127.0.0.1:8080/rpc/batch");
    println!("Main capability: Calculator (ID 0)");
    println!("Methods: add, multiply, divide, subtract");
    println!("Properties: name, version, pi");
    println!();
    println!("Example requests:");
    println!();
    println!("1. Simple add:");
    println!(r#"   curl -X POST http://localhost:8080/rpc/batch \"#);
    println!(r#"     -H "Content-Type: application/json" \"#);
    println!(r#"     -d '[["push", ["import", 0, ["add"], [5, 3]]]]'"#);
    println!();
    println!("2. Multiple operations:");
    println!(r#"   curl -X POST http://localhost:8080/rpc/batch \"#);
    println!(r#"     -H "Content-Type: application/json" \"#);
    println!(r#"     -d '["#);
    println!(r#"       ["push", ["import", 0, ["add"], [10, 20]]],"#);
    println!(r#"       ["push", ["import", 0, ["multiply"], [5, 7]]],"#);
    println!(r#"       ["push", ["import", 0, ["divide"], [100, 4]]]"#);
    println!(r#"     ]'"#);
    println!();
    println!("Starting server on http://127.0.0.1:8080...");

    // Run the server
    server.run().await?;

    Ok(())
}