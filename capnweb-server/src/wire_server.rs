// Official Cap'n Web Wire Protocol Server
// Implements the official Cap'n Web protocol using newline-delimited arrays

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
    parse_wire_batch, serialize_wire_batch, PropertyKey, RpcError, WireExpression, WireMessage,
};
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait WireCapability: Send + Sync {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError>;
}

#[derive(Clone)]
pub struct WireServer {
    config: WireServerConfig,
    capabilities: Arc<DashMap<i64, Arc<dyn WireCapability>>>,
    next_export_id: Arc<std::sync::atomic::AtomicI64>,
}

#[derive(Clone)]
pub struct WireServerConfig {
    pub port: u16,
    pub host: String,
    pub max_batch_size: usize,
}

impl Default for WireServerConfig {
    fn default() -> Self {
        WireServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_batch_size: 100,
        }
    }
}

impl WireServer {
    pub fn new(config: WireServerConfig) -> Self {
        WireServer {
            config,
            capabilities: Arc::new(DashMap::new()),
            next_export_id: Arc::new(std::sync::atomic::AtomicI64::new(-1)),
        }
    }

    pub fn register_capability(&self, id: i64, capability: Arc<dyn WireCapability>) {
        info!("Registering capability with ID: {}", id);
        self.capabilities.insert(id, capability);
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let app = Router::new()
            .route("/rpc/batch", post(handle_wire_batch))
            .route("/health", get(handle_health))
            .with_state(Arc::new(self));
        let listener = TcpListener::bind(&addr).await?;

        info!("🚀 Cap'n Web server listening on {}", addr);
        info!("  HTTP Batch endpoint: http://{}/rpc/batch", addr);
        info!("  Health endpoint: http://{}/health", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn process_wire_message(&self, message: WireMessage) -> Vec<WireMessage> {
        debug!("Processing wire message: {:?}", message);

        match message {
            WireMessage::Push(expr) => {
                info!("Processing PUSH message");
                self.handle_push_expression(expr).await
            }

            WireMessage::Pull(import_id) => {
                info!("Processing PULL message for import ID: {}", import_id);
                // For now, just return an error since we don't have promise resolution implemented
                vec![WireMessage::Reject(
                    -1, // Use a generic export ID
                    WireExpression::Error {
                        error_type: "not_implemented".to_string(),
                        message: "Promise resolution not yet implemented".to_string(),
                        stack: None,
                    },
                )]
            }

            WireMessage::Release(import_ids) => {
                info!("Processing RELEASE message for IDs: {:?}", import_ids);
                // Release capabilities
                for id in import_ids {
                    self.capabilities.remove(&id);
                }
                vec![] // No response for release
            }

            _ => {
                warn!("Unhandled message type: {:?}", message);
                vec![]
            }
        }
    }

    async fn handle_push_expression(&self, expr: WireExpression) -> Vec<WireMessage> {
        match expr {
            WireExpression::Pipeline {
                import_id,
                property_path,
                args,
            } => {
                info!(
                    "Handling pipeline expression: import_id={}, property_path={:?}",
                    import_id, property_path
                );

                // For now, treat import_id as a capability ID to call
                if let Some(capability) = self.capabilities.get(&import_id) {
                    // Extract method name from property path
                    if let Some(property_path) = property_path {
                        if let Some(PropertyKey::String(method)) = property_path.first() {
                            // Convert args from WireExpression to serde_json::Value
                            let json_args = if let Some(args_expr) = args {
                                self.wire_expression_to_json_args(*args_expr)
                            } else {
                                vec![]
                            };

                            // Call the capability
                            match capability.call(method, json_args).await {
                                Ok(result) => {
                                    let export_id = self
                                        .next_export_id
                                        .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                                    vec![WireMessage::Resolve(
                                        export_id,
                                        self.json_to_wire_expression(result),
                                    )]
                                }
                                Err(err) => {
                                    let export_id = self
                                        .next_export_id
                                        .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                                    vec![WireMessage::Reject(
                                        export_id,
                                        WireExpression::Error {
                                            error_type: err.code.to_string(),
                                            message: err.message.to_string(),
                                            stack: None,
                                        },
                                    )]
                                }
                            }
                        } else {
                            let export_id = self
                                .next_export_id
                                .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                            vec![WireMessage::Reject(
                                export_id,
                                WireExpression::Error {
                                    error_type: "bad_request".to_string(),
                                    message: "Invalid property path".to_string(),
                                    stack: None,
                                },
                            )]
                        }
                    } else {
                        let export_id = self
                            .next_export_id
                            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                        vec![WireMessage::Reject(
                            export_id,
                            WireExpression::Error {
                                error_type: "bad_request".to_string(),
                                message: "Missing property path".to_string(),
                                stack: None,
                            },
                        )]
                    }
                } else {
                    let export_id = self
                        .next_export_id
                        .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                    vec![WireMessage::Reject(
                        export_id,
                        WireExpression::Error {
                            error_type: "not_found".to_string(),
                            message: format!("Capability {} not found", import_id),
                            stack: None,
                        },
                    )]
                }
            }

            other => {
                warn!("Unhandled push expression: {:?}", other);
                let export_id = self
                    .next_export_id
                    .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                vec![WireMessage::Reject(
                    export_id,
                    WireExpression::Error {
                        error_type: "not_implemented".to_string(),
                        message: "Expression type not implemented".to_string(),
                        stack: None,
                    },
                )]
            }
        }
    }

    fn wire_expression_to_json_args(&self, expr: WireExpression) -> Vec<Value> {
        match expr {
            WireExpression::Array(items) => items
                .into_iter()
                .map(|item| self.wire_expression_to_json_value(item))
                .collect(),
            single => vec![self.wire_expression_to_json_value(single)],
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn wire_expression_to_json_value(&self, expr: WireExpression) -> Value {
        match expr {
            WireExpression::Null => Value::Null,
            WireExpression::Bool(b) => Value::Bool(b),
            WireExpression::Number(n) => Value::Number(n),
            WireExpression::String(s) => Value::String(s),
            WireExpression::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|item| self.wire_expression_to_json_value(item))
                    .collect(),
            ),
            WireExpression::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.wire_expression_to_json_value(v)))
                    .collect(),
            ),
            _ => Value::String(format!("Unsupported expression: {:?}", expr)),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn json_to_wire_expression(&self, value: Value) -> WireExpression {
        match value {
            Value::Null => WireExpression::Null,
            Value::Bool(b) => WireExpression::Bool(b),
            Value::Number(n) => WireExpression::Number(n),
            Value::String(s) => WireExpression::String(s),
            Value::Array(items) => WireExpression::Array(
                items
                    .into_iter()
                    .map(|item| self.json_to_wire_expression(item))
                    .collect(),
            ),
            Value::Object(map) => WireExpression::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.json_to_wire_expression(v)))
                    .collect(),
            ),
        }
    }
}

async fn handle_wire_batch(
    State(server): State<Arc<WireServer>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    info!("=== WIRE PROTOCOL REQUEST ===");
    info!("Headers: {:?}", headers);
    info!("Body size: {} bytes", body.len());

    let body_str = String::from_utf8_lossy(&body);
    info!("Raw body: {}", body_str);

    // Parse wire protocol messages
    let wire_messages = match parse_wire_batch(&body_str) {
        Ok(messages) => {
            info!("Successfully parsed {} wire messages", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                debug!("Message {}: {:?}", i, msg);
            }
            messages
        }
        Err(e) => {
            error!("Failed to parse wire protocol: {}", e);
            let error_response = WireMessage::Reject(
                -1,
                WireExpression::Error {
                    error_type: "bad_request".to_string(),
                    message: format!("Invalid wire protocol: {}", e),
                    stack: None,
                },
            );
            let response = serialize_wire_batch(&[error_response]);
            return (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/plain")],
                response,
            );
        }
    };

    // Check batch size
    if wire_messages.len() > server.config.max_batch_size {
        let error_response = WireMessage::Reject(
            -1,
            WireExpression::Error {
                error_type: "bad_request".to_string(),
                message: format!(
                    "Batch size {} exceeds maximum {}",
                    wire_messages.len(),
                    server.config.max_batch_size
                ),
                stack: None,
            },
        );
        let response = serialize_wire_batch(&[error_response]);
        return (
            StatusCode::BAD_REQUEST,
            [("Content-Type", "text/plain")],
            response,
        );
    }

    // Process each wire message
    let mut response_messages = Vec::new();
    for message in wire_messages {
        let responses = server.process_wire_message(message).await;
        response_messages.extend(responses);
    }

    // Serialize response
    let response_body = serialize_wire_batch(&response_messages);
    info!("Response: {}", response_body);

    (
        StatusCode::OK,
        [("Content-Type", "text/plain")],
        response_body,
    )
}

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "server": "capnweb-rust",
        "version": "0.1.0",
        "protocol": "cap'n web wire protocol",
        "endpoints": {
            "batch": "/rpc/batch",
            "health": "/health"
        }
    }))
}

// Adapter for existing RpcTarget trait
pub struct RpcTargetAdapter<T: crate::RpcTarget> {
    inner: T,
}

impl<T: crate::RpcTarget> RpcTargetAdapter<T> {
    pub fn new(inner: T) -> Self {
        RpcTargetAdapter { inner }
    }
}

#[async_trait]
impl<T: crate::RpcTarget> WireCapability for RpcTargetAdapter<T> {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        self.inner.call(method, args).await
    }
}
