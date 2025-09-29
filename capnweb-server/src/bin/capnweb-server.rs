//! Cap'n Web Server Binary
//!
//! A simple server implementation for the Cap'n Web protocol.
//! This binary provides HTTP batch and WebSocket endpoints for RPC calls.

use anyhow::Result;
use async_trait::async_trait;
use capnweb_core::{CapId, RpcError};
use capnweb_server::{RpcTarget, Server, ServerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

/// Example calculator service for testing
#[derive(Debug)]
struct CalculatorService;

#[async_trait]
impl RpcTarget for CalculatorService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires exactly 2 arguments"));
                }
                // Simple implementation for testing
                Ok(json!({
                    "result": "sum of inputs",
                    "method": "add",
                    "args_count": args.len()
                }))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request(
                        "multiply requires exactly 2 arguments",
                    ));
                }
                Ok(json!({
                    "result": "product of inputs",
                    "method": "multiply",
                    "args_count": args.len()
                }))
            }
            "echo" => Ok(json!({
                "echoed": args,
                "method": "echo"
            })),
            _ => Err(RpcError::not_found(format!("Unknown method: {}", method))),
        }
    }
}

/// Bootstrap service that provides capability imports
/// This is the main interface (import_id=0) required by the Cap'n Web protocol
#[derive(Debug)]
struct BootstrapService;

#[async_trait]
impl RpcTarget for BootstrapService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "getCapability" => {
                // Extract capability ID from args
                if let Some(Value::Number(id)) = args.first() {
                    let cap_id = id.as_u64().unwrap_or(0);

                    // For now, we'll return a capability reference for known capabilities
                    // In a real implementation, this would check the capability table
                    match cap_id {
                        1 | 2 => {
                            // Return capability reference in Cap'n Web wire format
                            // The client expects an object with $capnweb.import_id
                            Ok(json!({
                                "$capnweb": {
                                    "import_id": cap_id
                                }
                            }))
                        }
                        _ => Err(RpcError::not_found(format!(
                            "Capability {} not found",
                            cap_id
                        ))),
                    }
                } else {
                    Err(RpcError::bad_request(
                        "getCapability requires a capability ID argument",
                    ))
                }
            }
            "echo" => {
                // Bootstrap echo for compatibility
                Ok(json!({
                    "echoed": args,
                    "method": "echo",
                    "source": "bootstrap"
                }))
            }
            _ => Err(RpcError::not_found(format!(
                "Unknown bootstrap method: {}",
                method
            ))),
        }
    }
}

/// Example echo service for testing
#[derive(Debug)]
struct EchoService;

#[async_trait]
impl RpcTarget for EchoService {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        Ok(json!({
            "service": "echo",
            "method": method,
            "args": args,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Cap'n Web Server");

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

    info!("Server configured with capabilities:");
    info!("  - CapId(0): Bootstrap Service (main interface)");
    info!("  - CapId(1): Calculator Service");
    info!("  - CapId(2): Echo Service");

    // Start server
    info!("Starting server on http://127.0.0.1:8080");
    info!("Endpoints:");
    info!("  - HTTP Batch: http://127.0.0.1:8080/rpc/batch");
    info!("  - WebSocket:  ws://127.0.0.1:8080/rpc/ws");

    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
