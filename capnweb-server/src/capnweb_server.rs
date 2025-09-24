use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap, header},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use capnweb_core::{
    protocol::{
        Message, Expression, ImportId, ExportId,
        RpcSession, Value, ImportValue,
    },
    RpcTarget,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Cap'n Web server configuration
#[derive(Debug, Clone)]
pub struct CapnWebServerConfig {
    pub host: String,
    pub port: u16,
    pub max_batch_size: usize,
}

impl Default for CapnWebServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_batch_size: 100,
        }
    }
}

/// Session with state tracking
#[derive(Clone)]
struct SessionState {
    session: Arc<RpcSession>,
    next_import_id: Arc<RwLock<i64>>,
    pending_pulls: Arc<RwLock<HashMap<ImportId, tokio::sync::oneshot::Sender<Message>>>>,
    last_activity: Arc<RwLock<std::time::Instant>>,
}

/// Cap'n Web server state
#[derive(Clone)]
struct ServerState {
    /// Main capability (ID 0)
    main_capability: Option<Arc<dyn RpcTarget>>,
    /// Registered capabilities by ID
    capabilities: Arc<RwLock<HashMap<i64, Arc<dyn RpcTarget>>>>,
    /// Active sessions with state tracking
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

/// Cap'n Web server
pub struct CapnWebServer {
    config: CapnWebServerConfig,
    state: ServerState,
}

impl CapnWebServer {
    /// Create a new Cap'n Web server
    pub fn new(config: CapnWebServerConfig) -> Self {
        Self {
            config,
            state: ServerState {
                main_capability: None,
                capabilities: Arc::new(RwLock::new(HashMap::new())),
                sessions: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }

    /// Register the main capability (ID 0)
    pub fn register_main(&mut self, capability: Arc<dyn RpcTarget>) {
        self.state.main_capability = Some(capability);
    }

    /// Register a capability with a specific ID
    pub fn register_capability(&self, id: i64, capability: Arc<dyn RpcTarget>) {
        let state = self.state.clone();
        tokio::spawn(async move {
            state.capabilities.write().await.insert(id, capability);
        });
    }

    /// Build the router
    fn build_router(state: ServerState) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/rpc/batch", post(handle_batch))
            .route("/rpc/ws", get(handle_websocket))
            .layer(CorsLayer::permissive())
            .with_state(state)
    }

    /// Run the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let state = self.state.clone();
        let app = Self::build_router(self.state);

        // Start session cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                cleanup_inactive_sessions(&state).await;
            }
        });

        println!("🚀 Cap'n Web server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Batch RPC endpoint handler - handles both JSON array and newline-delimited formats
#[tracing::instrument(skip(state, headers, body), fields(body_len = body.len()))]
async fn handle_batch(
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    tracing::info!("Received batch RPC request");

    // For HTTP batch, create a fresh session for each request
    // This ensures proper stateless operation as expected by the client
    let session_state = create_batch_session(&state).await;

    // Update last activity time
    *session_state.last_activity.write().await = std::time::Instant::now();

    let mut responses = Vec::new();

    // Check content type to determine format
    let content_type = headers.get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    tracing::debug!(content_type = %content_type, body_preview = %body.chars().take(200).collect::<String>(), "Parsing request");

    // Parse messages based on format
    let messages = if content_type.contains("application/json") {
        // JSON array format (for backward compatibility)
        match serde_json::from_str::<Vec<serde_json::Value>>(&body) {
            Ok(msgs) => msgs,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid JSON: {}", e)
                ).into_response();
            }
        }
    } else {
        // Newline-delimited format (official Cap'n Web client)
        let mut msgs = Vec::new();
        for line in body.lines() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(msg) => msgs.push(msg),
                Err(e) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        format!("Invalid message on line: {}", e)
                    ).into_response();
                }
            }
        }
        msgs
    };

    tracing::debug!(message_count = messages.len(), "Processing messages");

    // Process each message
    for (i, msg_json) in messages.iter().enumerate() {
        tracing::debug!(message_index = i, message = ?msg_json, "Processing message");

        // Parse message
        let message = match Message::from_json(msg_json) {
            Ok(msg) => msg,
            Err(e) => {
                // Return error response
                responses.push(serde_json::json!([
                    "abort",
                    ["error", "ParseError", e.to_string()]
                ]));
                continue;
            }
        };

        tracing::debug!(parsed_message = ?message, "Message parsed successfully");

        // Process message with session state
        match process_message(&state, &session_state, message).await {
            Ok(Some(response)) => {
                tracing::debug!(response = ?response, "Generated response");
                responses.push(response.to_json());
            }
            Ok(None) => {
                tracing::debug!("No response needed (e.g., for Push without Pull)");
            }
            Err(e) => {
                tracing::error!(error = %e, "Error processing message");
                responses.push(serde_json::json!([
                    "abort",
                    ["error", "ProcessError", e.to_string()]
                ]));
            }
        }
    }

    tracing::debug!(response_count = responses.len(), "Preparing response");

    // Return in the same format as received (no cookies for stateless operation)
    if content_type.contains("application/json") {
        tracing::debug!("Returning JSON array response");
        // Return as JSON array
        Json(responses).into_response()
    } else {
        tracing::debug!("Returning newline-delimited response");
        // Return as newline-delimited text (official Cap'n Web format)
        let response_body = responses
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");

        tracing::debug!(response_body = %response_body, "Final response body");

        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain")],
            response_body
        ).into_response()
    }
}

/// WebSocket endpoint handler
async fn handle_websocket(State(_state): State<ServerState>) -> impl IntoResponse {
    // TODO: Implement WebSocket support
    (StatusCode::NOT_IMPLEMENTED, "WebSocket not yet implemented")
}

/// Create a fresh session for HTTP batch requests
/// Each HTTP request is completely independent with its own import/export space
async fn create_batch_session(
    state: &ServerState,
) -> SessionState {
    // Create fresh session with import ID 0 pre-allocated to main capability
    let session = Arc::new(RpcSession::new());

    // Pre-allocate import ID 0 to the main capability
    if let Some(main_cap) = &state.main_capability {
        use capnweb_core::protocol::tables::StubReference;

        let stub_ref = StubReference::new(main_cap.clone());
        let _ = session.imports.insert(
            ImportId(0),
            ImportValue::Stub(stub_ref),
        );
    }

    SessionState {
        session,
        next_import_id: Arc::new(RwLock::new(1)), // Start from 1 since 0 is main
        pending_pulls: Arc::new(RwLock::new(HashMap::new())),
        last_activity: Arc::new(RwLock::new(std::time::Instant::now())),
    }
}

/// Process a Cap'n Web message
async fn process_message(
    state: &ServerState,
    session_state: &SessionState,
    message: Message,
) -> Result<Option<Message>, Box<dyn std::error::Error>> {
    match message {
        Message::Push(expr) => {
            // For HTTP batch, allocate a new import ID for the result
            // Each push gets a sequential import ID (1, 2, 3, ...)
            let mut next_id = session_state.next_import_id.write().await;
            let import_id = ImportId(*next_id);
            *next_id += 1;
            drop(next_id);

            // Clone values for async task
            let state_clone = state.clone();
            let session = session_state.session.clone();
            let expr_clone = expr.clone();
            let pending_pulls = session_state.pending_pulls.clone();

            // Spawn task to evaluate expression and resolve import
            tokio::spawn(async move {
                match evaluate_expression(&state_clone, &session, expr_clone).await {
                    Ok(value) => {
                        // Store the value in the import table
                        let _ = session.imports.insert(
                            import_id,
                            ImportValue::Value(value.clone()),
                        );

                        // Check if there's a pending pull waiting for this import
                        let mut pulls = pending_pulls.write().await;
                        if let Some(sender) = pulls.remove(&import_id) {
                            // Send resolution to waiting pull
                            let _ = sender.send(Message::Resolve(
                                ExportId(import_id.0),
                                value_to_expression(value),
                            ));
                        }
                    }
                    Err(e) => {
                        // Store error in import table
                        let error_expr = Expression::Error(
                            capnweb_core::protocol::ErrorExpression {
                                error_type: "EvalError".to_string(),
                                message: e.to_string(),
                                stack: None,
                            },
                        );

                        let _ = session.imports.insert(
                            import_id,
                            ImportValue::Value(Value::Error(
                                "EvalError".to_string(),
                                e.to_string(),
                                None,
                            )),
                        );

                        // Notify any waiting pulls
                        let mut pulls = pending_pulls.write().await;
                        if let Some(sender) = pulls.remove(&import_id) {
                            let _ = sender.send(Message::Reject(
                                ExportId(import_id.0),
                                error_expr,
                            ));
                        }
                    }
                }
            });

            // Push doesn't return immediate response
            Ok(None)
        }

        Message::Pull(import_id) => {
            // Check if the import is already resolved
            if let Some(import_value) = session_state.session.imports.get(import_id) {
                // Import exists and is resolved
                match import_value {
                    ImportValue::Value(value) => {
                        // Check if it's an error value
                        if let Value::Error(error_type, message, stack) = value {
                            Ok(Some(Message::Reject(
                                ExportId(import_id.0),
                                Expression::Error(capnweb_core::protocol::ErrorExpression {
                                    error_type,
                                    message,
                                    stack,
                                }),
                            )))
                        } else {
                            Ok(Some(Message::Resolve(
                                ExportId(import_id.0),
                                value_to_expression(value),
                            )))
                        }
                    }
                    _ => {
                        // Import is a stub or promise, not yet supported
                        Ok(Some(Message::Resolve(
                            ExportId(import_id.0),
                            Expression::String("Not yet implemented".to_string()),
                        )))
                    }
                }
            } else {
                // Import doesn't exist yet (push might still be processing)
                // Create a channel to wait for resolution
                let (tx, rx) = tokio::sync::oneshot::channel();
                session_state.pending_pulls.write().await.insert(import_id, tx);

                // Wait for resolution with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    rx,
                ).await {
                    Ok(Ok(message)) => Ok(Some(message)),
                    Ok(Err(_)) => {
                        // Channel closed without sending
                        Ok(Some(Message::Reject(
                            import_id.to_export_id(),
                            Expression::Error(capnweb_core::protocol::ErrorExpression {
                                error_type: "ChannelError".to_string(),
                                message: "Resolution channel closed".to_string(),
                                stack: None,
                            }),
                        )))
                    }
                    Err(_) => {
                        // Timeout
                        session_state.pending_pulls.write().await.remove(&import_id);
                        Ok(Some(Message::Reject(
                            import_id.to_export_id(),
                            Expression::Error(capnweb_core::protocol::ErrorExpression {
                                error_type: "Timeout".to_string(),
                                message: "Pull request timed out".to_string(),
                                stack: None,
                            }),
                        )))
                    }
                }
            }
        }

        Message::Resolve(export_id, expr) => {
            // Client is resolving an export - handle through session
            let _ = session_state.session.handle_message(Message::Resolve(export_id, expr)).await;
            Ok(None)
        }

        Message::Reject(export_id, expr) => {
            // Client is rejecting an export - handle through session
            let _ = session_state.session.handle_message(Message::Reject(export_id, expr)).await;
            Ok(None)
        }

        Message::Release(import_id, refcount) => {
            // Client is releasing an import - handle through session
            let _ = session_state.session.handle_message(Message::Release(import_id, refcount)).await;
            Ok(None)
        }

        Message::Abort(expr) => {
            // Session is being aborted - handle through session
            let _ = session_state.session.handle_message(Message::Abort(expr.clone())).await;

            // Clean up any pending pulls
            let mut pulls = session_state.pending_pulls.write().await;
            for (_, sender) in pulls.drain() {
                let _ = sender.send(Message::Abort(expr.clone()));
            }

            Ok(None)
        }
    }
}

/// Simple expression evaluator for testing
async fn evaluate_expression(
    state: &ServerState,
    session: &Arc<RpcSession>,
    expr: Expression,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    match expr {
        Expression::Null => Ok(Value::Null),
        Expression::Bool(b) => Ok(Value::Bool(b)),
        Expression::Number(n) => Ok(Value::Number(n)),
        Expression::String(s) => Ok(Value::String(s)),

        // For testing, handle simple import calls
        Expression::Import(import) => {
            // Check if this is the main capability (ID 0)
            if import.import_id.is_main() {
                if let Some(main) = &state.main_capability {
                    // Extract method name from property path
                    if let Some(path) = &import.property_path {
                        if let Some(capnweb_core::protocol::PropertyKey::String(method)) = path.first() {
                            // Extract call arguments
                            let args = if let Some(args_expr) = &import.call_arguments {
                                extract_args(&**args_expr)?
                            } else {
                                Vec::new()
                            };

                            // Call the method
                            return main.call(method, args).await
                                .map_err(|e| e.to_string().into());
                        }
                    }
                }
            }

            Ok(Value::String("Import not implemented".to_string()))
        }

        Expression::Pipeline(pipeline) => {
            // Handle pipeline expressions - look up the target capability
            if let Some(import_value) = session.imports.get(pipeline.import_id) {
                match import_value {
                    ImportValue::Stub(stub_ref) => {
                        // Extract method name from property path
                        if let Some(path) = &pipeline.property_path {
                            if let Some(capnweb_core::protocol::PropertyKey::String(method)) = path.first() {
                                // Extract call arguments
                                let args = if let Some(args_expr) = &pipeline.call_arguments {
                                    extract_args(&**args_expr)?
                                } else {
                                    Vec::new()
                                };

                                // Call the method on the capability
                                let cap = stub_ref.get();
                                return cap.call(method, args).await
                                    .map_err(|e| e.to_string().into());
                            }
                        }
                        return Err("Invalid pipeline path".into());
                    }
                    _ => {
                        return Err(format!("Import {} is not a capability stub", pipeline.import_id.0).into());
                    }
                }
            } else {
                return Err(format!("Import {} not found", pipeline.import_id.0).into());
            }
        }

        _ => Ok(Value::String("Expression type not implemented".to_string())),
    }
}

/// Extract arguments from an expression
fn extract_args(expr: &Expression) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    match expr {
        Expression::Array(elements) => {
            let mut args = Vec::new();
            for elem in elements {
                args.push(expr_to_value(elem)?);
            }
            Ok(args)
        }
        _ => Ok(vec![expr_to_value(expr)?]),
    }
}

/// Convert expression to value
fn expr_to_value(expr: &Expression) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    match expr {
        Expression::Null => Ok(Value::Null),
        Expression::Bool(b) => Ok(Value::Bool(*b)),
        Expression::Number(n) => Ok(Value::Number(n.clone())),
        Expression::String(s) => Ok(Value::String(s.clone())),
        Expression::Array(elements) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(expr_to_value(elem)?);
            }
            Ok(Value::Array(values))
        }
        _ => Err("Unsupported expression type for conversion".into()),
    }
}

/// Convert value back to expression
fn value_to_expression(value: Value) -> Expression {
    match value {
        Value::Null => Expression::Null,
        Value::Bool(b) => Expression::Bool(b),
        Value::Number(n) => Expression::Number(n),
        Value::String(s) => Expression::String(s),
        Value::Array(values) => {
            let elements = values.into_iter().map(value_to_expression).collect();
            Expression::Array(elements)
        }
        Value::Object(obj) => {
            // Convert object to Object expression
            let mut map = std::collections::HashMap::new();
            for (key, val) in obj {
                map.insert(key, Box::new(value_to_expression(*val)));
            }
            Expression::Object(map)
        }
        Value::Date(timestamp) => Expression::Date(timestamp),
        Value::Error(error_type, message, stack) => {
            Expression::Error(capnweb_core::protocol::ErrorExpression {
                error_type,
                message,
                stack,
            })
        }
        Value::Stub(_) | Value::Promise(_) => {
            // For now, return a placeholder
            Expression::String("[Stub/Promise not yet supported]".to_string())
        }
    }
}

/// Clean up inactive sessions
async fn cleanup_inactive_sessions(state: &ServerState) {
    let mut sessions = state.sessions.write().await;
    let now = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300); // 5 minute timeout

    sessions.retain(|_id, session_state| {
        if let Ok(last_activity) = session_state.last_activity.try_read() {
            now.duration_since(*last_activity) < timeout
        } else {
            true // Keep if we can't read the timestamp
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let config = CapnWebServerConfig::default();
        let server = CapnWebServer::new(config);
        assert!(server.state.main_capability.is_none());
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}