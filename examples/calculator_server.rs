//! Example Calculator Server
//!
//! Demonstrates a complete Cap'n Web server implementation with:
//! - Capability registration and lifecycle management
//! - WebSocket and HTTP batch endpoints
//! - Error handling and logging
//! - Graceful shutdown

use capnweb_core::{CapId, RpcError};
use capnweb_server::{Server, ServerConfig, RpcTarget};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, Level};
use tracing_subscriber;

/// Calculator capability implementation
#[derive(Debug)]
struct Calculator {
    name: String,
}

impl Calculator {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl RpcTarget for Calculator {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("Calculator '{}' called method: {} with args: {:?}", self.name, member, args);

        match member {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }

                let a = args[0].as_f64().ok_or_else(|| RpcError::bad_request("first argument must be a number"))?;
                let b = args[1].as_f64().ok_or_else(|| RpcError::bad_request("second argument must be a number"))?;

                Ok(json!(a + b))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires exactly 2 arguments"));
                }

                let a = args[0].as_f64().ok_or_else(|| RpcError::bad_request("first argument must be a number"))?;
                let b = args[1].as_f64().ok_or_else(|| RpcError::bad_request("second argument must be a number"))?;

                Ok(json!(a - b))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires exactly 2 arguments"));
                }

                let a = args[0].as_f64().ok_or_else(|| RpcError::bad_request("first argument must be a number"))?;
                let b = args[1].as_f64().ok_or_else(|| RpcError::bad_request("second argument must be a number"))?;

                Ok(json!(a * b))
            }
            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires exactly 2 arguments"));
                }

                let a = args[0].as_f64().ok_or_else(|| RpcError::bad_request("first argument must be a number"))?;
                let b = args[1].as_f64().ok_or_else(|| RpcError::bad_request("second argument must be a number"))?;

                if b == 0.0 {
                    return Err(RpcError::bad_request("division by zero is not allowed"));
                }

                Ok(json!(a / b))
            }
            "power" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("power requires exactly 2 arguments"));
                }

                let base = args[0].as_f64().ok_or_else(|| RpcError::bad_request("base must be a number"))?;
                let exp = args[1].as_f64().ok_or_else(|| RpcError::bad_request("exponent must be a number"))?;

                Ok(json!(base.powf(exp)))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("sqrt requires exactly 1 argument"));
                }

                let value = args[0].as_f64().ok_or_else(|| RpcError::bad_request("argument must be a number"))?;

                if value < 0.0 {
                    return Err(RpcError::bad_request("cannot take square root of negative number"));
                }

                Ok(json!(value.sqrt()))
            }
            "factorial" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("factorial requires exactly 1 argument"));
                }

                let n = args[0].as_i64().ok_or_else(|| RpcError::bad_request("argument must be an integer"))?;

                if n < 0 {
                    return Err(RpcError::bad_request("factorial is not defined for negative numbers"));
                }

                if n > 20 {
                    return Err(RpcError::bad_request("factorial too large (max 20)"));
                }

                let mut result = 1i64;
                for i in 1..=n {
                    result *= i;
                }

                Ok(json!(result))
            }
            _ => Err(RpcError::not_found(format!("method '{}' not found", member))),
        }
    }
}

/// User management capability
#[derive(Debug)]
struct UserManager;

impl RpcTarget for UserManager {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("UserManager called method: {} with args: {:?}", member, args);

        match member {
            "getUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("getUser requires exactly 1 argument"));
                }

                let user_id = args[0].as_i64().ok_or_else(|| RpcError::bad_request("user ID must be a number"))?;

                // Simulate user lookup
                let user = match user_id {
                    1 => json!({
                        "id": 1,
                        "name": "Alice",
                        "email": "alice@example.com",
                        "role": "admin"
                    }),
                    2 => json!({
                        "id": 2,
                        "name": "Bob",
                        "email": "bob@example.com",
                        "role": "user"
                    }),
                    3 => json!({
                        "id": 3,
                        "name": "Charlie",
                        "email": "charlie@example.com",
                        "role": "user"
                    }),
                    _ => return Err(RpcError::not_found("user not found")),
                };

                Ok(user)
            }
            "createUser" => {
                if args.len() != 1 {
                    return Err(RpcError::bad_request("createUser requires exactly 1 argument"));
                }

                let user_data = &args[0];
                if !user_data.is_object() {
                    return Err(RpcError::bad_request("user data must be an object"));
                }

                // Simulate user creation
                let new_user = json!({
                    "id": 999,
                    "name": user_data.get("name").unwrap_or(&json!("Unknown")),
                    "email": user_data.get("email").unwrap_or(&json!("unknown@example.com")),
                    "role": "user",
                    "created": true
                });

                Ok(new_user)
            }
            _ => Err(RpcError::not_found(format!("method '{}' not found", member))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cap'n Web Calculator Server...");

    // Create server configuration
    let config = ServerConfig {
        http_bind_addr: "0.0.0.0:8080".to_string(),
        websocket_endpoint: "/ws".to_string(),
        batch_endpoint: "/batch".to_string(),
        max_connections: 1000,
        max_batch_size: 100,
        rate_limit_requests_per_second: Some(100),
        enable_cors: true,
        ..Default::default()
    };

    // Create server
    let server = Arc::new(Server::new(config));

    // Register capabilities
    info!("Registering capabilities...");

    // Register multiple calculator instances
    let basic_calc = Arc::new(Calculator::new("Basic Calculator".to_string()));
    let scientific_calc = Arc::new(Calculator::new("Scientific Calculator".to_string()));
    let user_manager = Arc::new(UserManager);

    server.register_capability(CapId::new(1), basic_calc)?;
    server.register_capability(CapId::new(2), scientific_calc)?;
    server.register_capability(CapId::new(100), user_manager)?;

    info!("Capabilities registered:");
    info!("  - Basic Calculator (ID: 1)");
    info!("  - Scientific Calculator (ID: 2)");
    info!("  - User Manager (ID: 100)");

    // Start server
    info!("Starting server...");
    info!("HTTP server listening on: http://0.0.0.0:8080");
    info!("WebSocket endpoint: ws://0.0.0.0:8080/ws");
    info!("Batch endpoint: http://0.0.0.0:8080/batch");

    let server_handle = {
        let server = server.clone();
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!("Server error: {}", e);
            }
        })
    };

    // Wait for shutdown signal
    info!("Server started! Press Ctrl+C to shutdown.");
    info!("");
    info!("Example usage:");
    info!("  WebSocket: Connect to ws://localhost:8080/ws");
    info!("  HTTP Batch: POST to http://localhost:8080/batch");
    info!("");
    info!("Available capabilities:");
    info!("  Calculator (ID: 1, 2): add, subtract, multiply, divide, power, sqrt, factorial");
    info!("  UserManager (ID: 100): getUser, createUser");

    // Wait for Ctrl+C
    signal::ctrl_c().await?;

    info!("Shutdown signal received, stopping server...");

    // Shutdown server gracefully
    server.shutdown().await;
    server_handle.abort();

    info!("Server stopped.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_calculator_operations() {
        let calc = Calculator::new("Test Calculator".to_string());

        // Test addition
        let result = calc.call("add", vec![json!(5), json!(3)]).await.unwrap();
        assert_eq!(result, json!(8));

        // Test division
        let result = calc.call("divide", vec![json!(10), json!(2)]).await.unwrap();
        assert_eq!(result, json!(5));

        // Test error handling
        let result = calc.call("divide", vec![json!(10), json!(0)]).await;
        assert!(result.is_err());

        // Test factorial
        let result = calc.call("factorial", vec![json!(5)]).await.unwrap();
        assert_eq!(result, json!(120));
    }

    #[tokio::test]
    async fn test_user_manager() {
        let user_mgr = UserManager;

        // Test user lookup
        let result = user_mgr.call("getUser", vec![json!(1)]).await.unwrap();
        assert_eq!(result["name"], json!("Alice"));

        // Test user creation
        let user_data = json!({"name": "Test User", "email": "test@example.com"});
        let result = user_mgr.call("createUser", vec![user_data]).await.unwrap();
        assert_eq!(result["created"], json!(true));

        // Test error handling
        let result = user_mgr.call("getUser", vec![json!(999)]).await;
        assert!(result.is_err());
    }
}