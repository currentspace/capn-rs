//! TypeScript Interoperability Test Server
//!
//! A specialized server for testing interoperability with the TypeScript Cap'n Web client.
//! This server includes workarounds for known TypeScript client issues and provides
//! comprehensive logging for debugging protocol interactions.

use anyhow::Result;
use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Bootstrap service that provides capability imports
/// This is the main interface (import_id=0) required by the Cap'n Web protocol
#[derive(Debug)]
struct BootstrapService;

#[async_trait]
impl RpcTarget for BootstrapService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("Bootstrap.{} called with {} args", method, args.len());
        debug!("Bootstrap.{} args: {:?}", method, args);

        match method {
            "getCapability" => {
                // Extract capability ID from args
                if let Some(Value::Number(id)) = args.first() {
                    let cap_id = id.as_u64().unwrap_or(0);
                    info!("getCapability requested for ID: {}", cap_id);

                    // For testing, we support capabilities 1-10
                    match cap_id {
                        1..=10 => {
                            // Return capability reference in Cap'n Web wire format
                            // The client expects an object that will be recognized as a capability
                            let response = json!({
                                "$capnweb": {
                                    "import_id": cap_id
                                }
                            });
                            info!("Returning capability reference for ID {}", cap_id);
                            Ok(response)
                        }
                        _ => {
                            warn!("Capability {} not found", cap_id);
                            Err(RpcError::not_found(format!(
                                "Capability {} not found",
                                cap_id
                            )))
                        }
                    }
                } else {
                    error!("getCapability called without proper ID argument");
                    Err(RpcError::bad_request(
                        "getCapability requires a capability ID argument",
                    ))
                }
            }
            "echo" => {
                // Handle echo with various argument patterns
                // TypeScript client has issues with empty arrays, so we handle that specially
                let response = if args.is_empty() {
                    json!({
                        "echoed": [],
                        "message": "Empty arguments received",
                        "method": "echo",
                        "source": "bootstrap"
                    })
                } else {
                    json!({
                        "echoed": args,
                        "method": "echo",
                        "source": "bootstrap"
                    })
                };
                info!("Bootstrap.echo returning response");
                Ok(response)
            }
            "ping" => {
                // Simple ping/pong for connection testing
                Ok(json!({ "pong": true, "timestamp": chrono::Utc::now().to_rfc3339() }))
            }
            "listCapabilities" => {
                // Return a list of available capabilities for discovery
                Ok(json!({
                    "capabilities": [
                        { "id": 1, "name": "Calculator", "methods": ["add", "subtract", "multiply", "divide", "echo"] },
                        { "id": 2, "name": "Echo", "methods": ["*"] },
                        { "id": 3, "name": "TypeScript Test", "methods": ["testEmpty", "testArrays", "testObjects"] }
                    ]
                }))
            }
            _ => {
                warn!("Unknown bootstrap method: {}", method);
                Err(RpcError::not_found(format!(
                    "Unknown bootstrap method: {}",
                    method
                )))
            }
        }
    }
}

/// Calculator service with proper error handling
#[derive(Debug)]
struct CalculatorService;

#[async_trait]
impl RpcTarget for CalculatorService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("Calculator.{} called with {} args", method, args.len());

        match method {
            "add" | "subtract" | "multiply" | "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(format!(
                        "{} requires exactly 2 arguments",
                        method
                    )));
                }

                let a = args[0]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("First argument must be a number"))?;
                let b = args[1]
                    .as_f64()
                    .ok_or_else(|| RpcError::bad_request("Second argument must be a number"))?;

                let result = match method {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(RpcError::bad_request("Division by zero"));
                        }
                        a / b
                    }
                    _ => unreachable!(),
                };

                Ok(json!({ "result": result, "operation": method }))
            }
            "echo" => Ok(json!({
                "echoed": args,
                "method": "echo",
                "service": "calculator"
            })),
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method))),
        }
    }
}

/// Echo service that accepts any method
#[derive(Debug)]
struct EchoService;

#[async_trait]
impl RpcTarget for EchoService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("Echo.{} called with {} args", method, args.len());

        Ok(json!({
            "service": "echo",
            "method": method,
            "args": args,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

/// TypeScript-specific test service for testing edge cases
#[derive(Debug)]
struct TypeScriptTestService;

#[async_trait]
impl RpcTarget for TypeScriptTestService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        info!("TypeScriptTest.{} called", method);

        match method {
            "testEmpty" => {
                // Test handling of empty responses
                Ok(json!({}))
            }
            "testArrays" => {
                // Test various array patterns that might cause issues
                Ok(json!({
                    "empty": [],
                    "single": [1],
                    "nested": [[1, 2], [3, 4]],
                    "mixed": [1, "two", true, null]
                }))
            }
            "testObjects" => {
                // Test nested object structures
                Ok(json!({
                    "simple": { "key": "value" },
                    "nested": {
                        "level1": {
                            "level2": {
                                "level3": "deep"
                            }
                        }
                    },
                    "withArrays": {
                        "items": [1, 2, 3],
                        "empty": []
                    }
                }))
            }
            "testCapabilityRef" => {
                // Test returning a capability reference
                Ok(json!({
                    "$capnweb": {
                        "import_id": 99
                    }
                }))
            }
            _ => Err(RpcError::not_found(format!(
                "Unknown test method: {}",
                method
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with more detail for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting TypeScript Interop Test Server");
    info!("This server includes workarounds for known TypeScript client issues");

    // Configure server
    let config = ServerConfig {
        port: 8080,
        host: "127.0.0.1".to_string(),
        max_batch_size: 100,
    };

    // Create server
    let server = Server::new(config);

    // Register capabilities
    // IMPORTANT: import_id=0 is the main interface/bootstrap service per Cap'n Web protocol
    server.register_capability(CapId::new(0), Arc::new(BootstrapService));
    server.register_capability(CapId::new(1), Arc::new(CalculatorService));
    server.register_capability(CapId::new(2), Arc::new(EchoService));
    server.register_capability(CapId::new(3), Arc::new(TypeScriptTestService));

    info!("Server configured with capabilities:");
    info!("  - CapId(0): Bootstrap Service (main interface)");
    info!("  - CapId(1): Calculator Service");
    info!("  - CapId(2): Echo Service");
    info!("  - CapId(3): TypeScript Test Service");

    // Start server
    info!("Starting server on http://127.0.0.1:8080");
    info!("Endpoints:");
    info!("  - HTTP Batch: http://127.0.0.1:8080/rpc/batch");
    info!("  - WebSocket:  ws://127.0.0.1:8080/rpc/ws");
    info!("");
    info!("TypeScript client known issues:");
    info!("  - Empty arrays in arguments may cause 'unknown special value' errors");
    info!("  - Workaround: Use single values instead of arrays when possible");

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
