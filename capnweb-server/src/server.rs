use crate::CapTable;
use async_trait::async_trait;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use capnweb_core::{
    parse_wire_batch, serialize_wire_batch, CapId, PropertyKey, RpcError, WireExpression,
    WireMessage,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;

// Using wire protocol helper functions from the server_wire_handler module
use crate::server_wire_handler::{value_to_wire_expr, wire_expr_to_values_with_evaluation};

#[async_trait]
pub trait RpcTarget: Send + Sync {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
}

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub max_batch_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_batch_size: 100,
        }
    }
}

#[derive(Clone)]
pub struct Server {
    config: ServerConfig,
    cap_table: Arc<CapTable>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Server {
            config,
            cap_table: Arc::new(CapTable::new()),
        }
    }

    pub fn cap_table(&self) -> &Arc<CapTable> {
        &self.cap_table
    }

    pub fn register_capability(&self, id: CapId, target: Arc<dyn RpcTarget>) {
        self.cap_table.insert(id, target);
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let mut app = Router::new()
            .route("/rpc/batch", post(handle_batch))
            .route("/health", get(handle_health));

        // Add WebSocket support if the feature is enabled
        #[cfg(feature = "all-transports")]
        {
            // Use the new wire protocol WebSocket handler
            app = app.route("/rpc/ws", get(crate::ws_wire::websocket_wire_handler));
        }

        let app = app.with_state(Arc::new(self.clone()));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        println!("Server listening on {}", addr);
        println!("  HTTP Batch endpoint: http://{}/rpc/batch", addr);

        #[cfg(feature = "all-transports")]
        println!("  WebSocket endpoint: ws://{}/rpc/ws", addr);

        println!("  Health endpoint: http://{}/health", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    // Remove legacy message processing - we only support wire protocol now
}

// Session state to track push/pull flow per HTTP batch request
struct BatchSession {
    next_import_id: i64,
    // Map import IDs to their pushed expressions
    pushed_expressions: HashMap<i64, WireExpression>,
    // Map import IDs to their computed results
    results: HashMap<i64, WireExpression>,
}

async fn handle_batch(
    State(server): State<Arc<Server>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Enhanced tracing for debugging
    tracing::debug!("=== INCOMING CAP'N WEB WIRE PROTOCOL REQUEST ===");
    tracing::debug!("Headers: {:?}", headers);
    tracing::debug!("Body size: {} bytes", body.len());

    // Convert body to string
    let body_str = String::from_utf8_lossy(&body);
    tracing::debug!(
        "Raw body (first 500 chars): {}",
        &body_str.chars().take(500).collect::<String>()
    );

    // Create session state for this batch request
    let mut session = BatchSession {
        next_import_id: 1, // Start from 1 per protocol spec
        pushed_expressions: HashMap::new(),
        results: HashMap::new(),
    };

    // Parse the official Cap'n Web wire protocol (newline-delimited arrays ONLY)
    match parse_wire_batch(&body_str) {
        Ok(wire_messages) => {
            tracing::info!(
                "✅ Successfully parsed {} wire messages",
                wire_messages.len()
            );
            tracing::trace!("Messages: {:#?}", wire_messages);

            let mut responses = Vec::new();

            for (i, msg) in wire_messages.iter().enumerate() {
                tracing::debug!(
                    "Processing wire message {}/{}: {:?}",
                    i + 1,
                    wire_messages.len(),
                    msg
                );

                match msg {
                    WireMessage::Push(expr) => {
                        tracing::trace!("  PUSH expression details: {:#?}", expr);

                        // Assign the next import ID to this push
                        let assigned_import_id = session.next_import_id;
                        session.next_import_id += 1;

                        tracing::info!("  PUSH assigned import ID: {}", assigned_import_id);

                        // Store the expression for later evaluation
                        session
                            .pushed_expressions
                            .insert(assigned_import_id, expr.clone());

                        // Evaluate the expression immediately and store result
                        match expr {
                            WireExpression::Pipeline {
                                import_id,
                                property_path,
                                args,
                            } => {
                                tracing::info!(
                                    "  Pipeline call: import_id={}, path={:?}",
                                    import_id,
                                    property_path
                                );
                                tracing::info!("  Pipeline args raw wire expression: {:#?}", args);

                                // Validate and map import_id to capability
                                // Official protocol: import_id 0 is the main capability/bootstrap interface
                                // All import_ids map directly to their corresponding capability IDs

                                // Check for negative import_id values
                                if *import_id < 0 {
                                    tracing::error!(
                                        "Invalid negative import_id: {}. Import IDs must be non-negative.",
                                        import_id
                                    );
                                    session.results.insert(
                                        assigned_import_id,
                                        WireExpression::Error {
                                            error_type: "bad_request".to_string(),
                                            message: format!(
                                                "Invalid import_id: {}. Import IDs must be non-negative",
                                                import_id
                                            ),
                                            stack: None,
                                        },
                                    );
                                    continue;
                                }

                                // Safe to convert to u64 now that we've validated it's non-negative
                                let cap_id = CapId::new(*import_id as u64);

                                tracing::debug!(
                                    "  Mapped import_id {} to capability {}",
                                    import_id,
                                    cap_id
                                );

                                if let Some(capability) = server.cap_table.lookup(&cap_id) {
                                    if let Some(path) = property_path {
                                        if let Some(PropertyKey::String(method)) = path.first() {
                                            tracing::info!(
                                                "  Calling method '{}' on capability {}",
                                                method,
                                                cap_id
                                            );

                                            // Convert args from WireExpression to Value (with pipeline evaluation)
                                            let json_args = if let Some(args_expr) = args {
                                                wire_expr_to_values_with_evaluation(
                                                    args_expr,
                                                    &session.results,
                                                )
                                            } else {
                                                vec![]
                                            };

                                            tracing::info!(
                                                "  Method args (converted): {:?}",
                                                json_args
                                            );

                                            match capability.call(method, json_args).await {
                                                Ok(result) => {
                                                    tracing::info!(
                                                        "  ✅ Method '{}' succeeded",
                                                        method
                                                    );
                                                    tracing::trace!("  Result: {:?}", result);

                                                    // Store the result for this import ID
                                                    session.results.insert(
                                                        assigned_import_id,
                                                        value_to_wire_expr(result),
                                                    );
                                                }
                                                Err(err) => {
                                                    tracing::error!(
                                                        "  ❌ Method '{}' failed: {:?}",
                                                        method,
                                                        err
                                                    );

                                                    // Store the error for this import ID
                                                    session.results.insert(
                                                        assigned_import_id,
                                                        WireExpression::Error {
                                                            error_type: err.code.to_string(),
                                                            message: err.message.clone(),
                                                            stack: None,
                                                        },
                                                    );
                                                }
                                            }
                                        } else {
                                            tracing::warn!(
                                                "  No method name in property path: {:?}",
                                                path
                                            );
                                            session.results.insert(
                                                assigned_import_id,
                                                WireExpression::Error {
                                                    error_type: "bad_request".to_string(),
                                                    message: "No method specified".to_string(),
                                                    stack: None,
                                                },
                                            );
                                        }
                                    } else {
                                        tracing::warn!("  No property path in pipeline expression");
                                        session.results.insert(
                                            assigned_import_id,
                                            WireExpression::Error {
                                                error_type: "bad_request".to_string(),
                                                message: "No property path in pipeline".to_string(),
                                                stack: None,
                                            },
                                        );
                                    }
                                } else {
                                    tracing::error!(
                                        "  Capability {} not found in cap_table",
                                        cap_id
                                    );
                                    session.results.insert(
                                        assigned_import_id,
                                        WireExpression::Error {
                                            error_type: "not_found".to_string(),
                                            message: format!("Capability {} not found", import_id),
                                            stack: None,
                                        },
                                    );
                                }
                            }
                            WireExpression::Call {
                                cap_id,
                                property_path,
                                args,
                            } => {
                                tracing::info!(
                                    "  Call: cap_id={}, path={:?}",
                                    cap_id,
                                    property_path
                                );
                                tracing::trace!("  Call args: {:#?}", args);

                                let cap_id = CapId::new(*cap_id as u64);

                                if let Some(capability) = server.cap_table.lookup(&cap_id) {
                                    if let Some(PropertyKey::String(method)) = property_path.first()
                                    {
                                        tracing::info!(
                                            "  Calling method '{}' on capability {}",
                                            method,
                                            cap_id
                                        );

                                        // Convert args from WireExpression to Value (with pipeline evaluation)
                                        let json_args = wire_expr_to_values_with_evaluation(
                                            args,
                                            &session.results,
                                        );

                                        tracing::trace!(
                                            "  Method args (converted): {:?}",
                                            json_args
                                        );

                                        match capability.call(method, json_args).await {
                                            Ok(result) => {
                                                tracing::info!(
                                                    "  ✅ Method '{}' succeeded",
                                                    method
                                                );
                                                tracing::trace!("  Result: {:?}", result);

                                                // Store the result for this import ID
                                                session.results.insert(
                                                    assigned_import_id,
                                                    value_to_wire_expr(result),
                                                );
                                            }
                                            Err(err) => {
                                                tracing::error!(
                                                    "  ❌ Method '{}' failed: {:?}",
                                                    method,
                                                    err
                                                );

                                                // Store the error for this import ID
                                                session.results.insert(
                                                    assigned_import_id,
                                                    WireExpression::Error {
                                                        error_type: err.code.to_string(),
                                                        message: err.message.clone(),
                                                        stack: None,
                                                    },
                                                );
                                            }
                                        }
                                    } else {
                                        tracing::warn!(
                                            "  No method name in property path: {:?}",
                                            property_path
                                        );
                                        session.results.insert(
                                            assigned_import_id,
                                            WireExpression::Error {
                                                error_type: "bad_request".to_string(),
                                                message: "No method specified".to_string(),
                                                stack: None,
                                            },
                                        );
                                    }
                                } else {
                                    tracing::error!(
                                        "  Capability {} not found in cap_table",
                                        cap_id
                                    );
                                    session.results.insert(
                                        assigned_import_id,
                                        WireExpression::Error {
                                            error_type: "not_found".to_string(),
                                            message: format!("Capability {} not found", cap_id),
                                            stack: None,
                                        },
                                    );
                                }
                            }
                            _ => {
                                tracing::warn!("  Push expression is not a pipeline or call (unsupported): {:?}", expr);
                                session.results.insert(
                                    assigned_import_id,
                                    WireExpression::Error {
                                        error_type: "not_implemented".to_string(),
                                        message: "Only pipeline and call expressions are supported"
                                            .to_string(),
                                        stack: None,
                                    },
                                );
                            }
                        }
                    }

                    WireMessage::Pull(import_id) => {
                        tracing::debug!("  PULL for import_id: {}", import_id);

                        // Look up the result for this import ID
                        if let Some(result) = session.results.get(import_id) {
                            tracing::info!("  Found result for import ID {}", import_id);

                            // Check if it's an error
                            if let WireExpression::Error { .. } = result {
                                // Use the import ID as the export ID (per protocol spec)
                                responses.push(WireMessage::Reject(
                                    *import_id, // Use import ID as export ID
                                    result.clone(),
                                ));
                            } else {
                                // Use the import ID as the export ID (per protocol spec)
                                responses.push(WireMessage::Resolve(
                                    *import_id, // Use import ID as export ID
                                    result.clone(),
                                ));
                            }
                        } else {
                            tracing::warn!("  No result found for import ID {}", import_id);
                            responses.push(WireMessage::Reject(
                                *import_id,
                                WireExpression::Error {
                                    error_type: "not_found".to_string(),
                                    message: format!("No result for import ID {}", import_id),
                                    stack: None,
                                },
                            ));
                        }
                    }

                    WireMessage::Release(ids) => {
                        tracing::info!("  RELEASE capabilities: {:?}", ids);
                        // Handle capability disposal
                        for id in ids {
                            let cap_id = CapId::new(*id as u64);
                            server.cap_table.remove(&cap_id);
                        }
                    }

                    other => {
                        tracing::warn!(
                            "  Unhandled message type (not yet implemented): {:?}",
                            other
                        );
                    }
                }
            }

            // Serialize responses using official wire protocol (newline-delimited)
            let response_body = serialize_wire_batch(&responses);
            tracing::info!("📤 Sending {} response messages", responses.len());
            tracing::debug!(
                "Response body (first 500 chars): {}",
                &response_body.chars().take(500).collect::<String>()
            );

            (
                StatusCode::OK,
                [("content-type", "text/plain")],
                response_body,
            )
        }
        Err(e) => {
            tracing::error!("Failed to parse wire protocol: {}", e);
            tracing::debug!(
                "Invalid input was: {}",
                &body_str.chars().take(1000).collect::<String>()
            );
            let error_response = WireMessage::Reject(
                -1,
                WireExpression::Error {
                    error_type: "bad_request".to_string(),
                    message: format!("Invalid wire protocol: {}", e),
                    stack: None,
                },
            );
            let response = serialize_wire_batch(&[error_response]);
            (
                StatusCode::BAD_REQUEST,
                [("content-type", "text/plain")],
                response,
            )
        }
    }
}

async fn handle_health(State(server): State<Arc<Server>>) -> impl IntoResponse {
    let capability_count = server.cap_table.len();

    let mut endpoints = serde_json::json!({
        "batch": "/rpc/batch",
        "health": "/health"
    });

    // Add WebSocket endpoint if available
    #[cfg(feature = "all-transports")]
    {
        endpoints["websocket"] = serde_json::json!("/rpc/ws");
    }

    let health_response = serde_json::json!({
        "status": "healthy",
        "server": "capnweb-rust",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": capability_count,
        "max_batch_size": server.config.max_batch_size,
        "features": {
            "websocket": cfg!(feature = "all-transports"),
            "h3": cfg!(feature = "h3-server")
        },
        "endpoints": endpoints
    });

    (StatusCode::OK, Json(health_response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct TestTarget;

    #[async_trait]
    impl RpcTarget for TestTarget {
        async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            match member {
                "echo" => Ok(args.first().cloned().unwrap_or(Value::Null)),
                "add" => {
                    if args.len() != 2 {
                        return Err(RpcError::bad_request("add requires 2 arguments"));
                    }
                    let a = args[0]
                        .as_f64()
                        .ok_or_else(|| RpcError::bad_request("First arg must be number"))?;
                    let b = args[1]
                        .as_f64()
                        .ok_or_else(|| RpcError::bad_request("Second arg must be number"))?;
                    Ok(json!(a + b))
                }
                _ => Err(RpcError::not_found(format!(
                    "Method '{}' not found",
                    member
                ))),
            }
        }
    }

    #[tokio::test]
    async fn test_server_creation() {
        let config = ServerConfig::default();
        let server = Server::new(config);
        assert_eq!(server.config.port, 8080);
    }

    #[tokio::test]
    async fn test_register_capability() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(42);
        let target = Arc::new(TestTarget);

        server.register_capability(cap_id, target);
        assert!(server.cap_table.lookup(&cap_id).is_some());
    }

    #[tokio::test]
    async fn test_wire_protocol_push() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(1);
        server.register_capability(cap_id, Arc::new(TestTarget));

        // Simulate wire protocol push message for "echo" method
        // This would be the wire format:
        // let _wire_messages = vec![WireMessage::Push(WireExpression::Pipeline {
        //     import_id: 1, // Map to CapId(1)
        //     property_path: Some(vec![PropertyKey::String("echo".to_string())]),
        //     args: Some(Box::new(WireExpression::Array(vec![
        //         WireExpression::String("hello".to_string()),
        //     ]))),
        // })];

        // Process directly (simulating what handle_batch would do)
        let capability = server.cap_table.lookup(&cap_id).unwrap();
        let result = capability.call("echo", vec![json!("hello")]).await.unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[tokio::test]
    async fn test_wire_protocol_release() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(1);
        server.register_capability(cap_id, Arc::new(TestTarget));

        assert!(server.cap_table.lookup(&cap_id).is_some());

        // Simulate wire protocol release message
        server.cap_table.remove(&cap_id);

        assert!(server.cap_table.lookup(&cap_id).is_none());
    }

    #[tokio::test]
    async fn test_wire_protocol_unknown_capability() {
        let server = Server::new(ServerConfig::default());
        let cap_id = CapId::new(999);

        // Try to look up non-existent capability
        assert!(server.cap_table.lookup(&cap_id).is_none());
    }
}
