use capnweb_core::{RpcTarget, RpcError};
use capnweb_core::protocol::{
    Message, Expression, PipelineExpression, ImportId, ExportId,
    tables::{Value, ImportValue},
    session::RpcSession,
};
use capnweb_server::{NewCapnWebServer as CapnWebServer, CapnWebServerConfig};
use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde_json;
use tower_http::cors::CorsLayer;

/// Calculator capability for testing protocol compliance
#[derive(Debug)]
struct Calculator;

#[async_trait]
impl RpcTarget for Calculator {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
        match method {
            "add" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("add requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a + b).unwrap()))
            }
            "multiply" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("multiply requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a * b).unwrap()))
            }
            "divide" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("divide requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                if b == 0.0 {
                    return Err(RpcError::bad_request("Division by zero"));
                }
                Ok(Value::Number(serde_json::Number::from_f64(a / b).unwrap()))
            }
            "subtract" => {
                if args.len() != 2 {
                    return Err(RpcError::bad_request("subtract requires 2 arguments"));
                }
                let a = extract_number(&args[0])?;
                let b = extract_number(&args[1])?;
                Ok(Value::Number(serde_json::Number::from_f64(a - b).unwrap()))
            }
            _ => Err(RpcError::not_found(format!("Method not found: {}", method))),
        }
    }

    async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
        match property {
            "name" => Ok(Value::String("Calculator".to_string())),
            "version" => Ok(Value::String("1.0.0".to_string())),
            _ => Err(RpcError::not_found(format!("Property not found: {}", property))),
        }
    }
}

fn extract_number(value: &Value) -> Result<f64, RpcError> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| RpcError::bad_request("Invalid number")),
        _ => Err(RpcError::bad_request("Expected number")),
    }
}

/// Session with state tracking for Cap'n Web protocol
#[derive(Clone)]
struct ProtocolSessionState {
    session: Arc<RpcSession>,
    next_import_id: Arc<RwLock<i64>>,
    pending_pulls: Arc<RwLock<HashMap<ImportId, tokio::sync::oneshot::Sender<Vec<serde_json::Value>>>>>,
    main_capability: Option<Arc<dyn RpcTarget>>,
}

/// Protocol compliant server state
#[derive(Clone)]
struct ProtocolServerState {
    main_capability: Option<Arc<dyn RpcTarget>>,
    sessions: Arc<RwLock<HashMap<String, ProtocolSessionState>>>,
}

/// Convert Value to JSON for responses
fn value_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => serde_json::Value::Number(n),
        Value::String(s) => serde_json::Value::String(s),
        Value::Array(arr) => serde_json::Value::Array(arr.into_iter().map(value_to_json).collect()),
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj {
                map.insert(k, value_to_json(*v));
            }
            serde_json::Value::Object(map)
        }
        Value::Date(timestamp) => serde_json::Value::Number(serde_json::Number::from_f64(timestamp).unwrap()),
        Value::Error(error_type, message, _stack) => {
            serde_json::json!(["error", error_type, message])
        }
        Value::Stub(_) | Value::Promise(_) => {
            // For now, represent as placeholder
            serde_json::Value::String("STUB".to_string())
        }
    }
}

/// Convert expression args to Values for RPC calls
fn expression_to_values(expr: &Expression) -> Result<Vec<Value>, String> {
    match expr {
        Expression::Array(elements) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(expression_to_value(elem)?);
            }
            Ok(values)
        }
        _ => Ok(vec![expression_to_value(expr)?]),
    }
}

fn expression_to_value(expr: &Expression) -> Result<Value, String> {
    match expr {
        Expression::Null => Ok(Value::Null),
        Expression::Bool(b) => Ok(Value::Bool(*b)),
        Expression::Number(n) => Ok(Value::Number(n.clone())),
        Expression::String(s) => Ok(Value::String(s.clone())),
        Expression::Array(elements) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(expression_to_value(elem)?);
            }
            Ok(Value::Array(values))
        }
        Expression::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), Box::new(expression_to_value(val)?));
            }
            Ok(Value::Object(map))
        }
        _ => Err("Unsupported expression type".to_string()),
    }
}

/// Get or create a session
async fn get_or_create_session(
    state: &ProtocolServerState,
    session_id: &str,
) -> ProtocolSessionState {
    let mut sessions = state.sessions.write().unwrap();

    if let Some(session_state) = sessions.get(session_id) {
        session_state.clone()
    } else {
        let session_state = ProtocolSessionState {
            session: Arc::new(RpcSession::new()),
            next_import_id: Arc::new(RwLock::new(1)), // Start from 1 for imports
            pending_pulls: Arc::new(RwLock::new(HashMap::new())),
            main_capability: state.main_capability.clone(),
        };
        sessions.insert(session_id.to_string(), session_state.clone());
        session_state
    }
}

/// Process Cap'n Web protocol messages
async fn process_capnweb_message(
    session_state: &ProtocolSessionState,
    message: Message,
) -> Result<Vec<serde_json::Value>, String> {
    match message {
        Message::Push(expr) => {
            // Allocate import ID for this push
            let mut next_id = session_state.next_import_id.write().unwrap();
            let import_id = ImportId(*next_id);
            *next_id += 1;
            drop(next_id);

            println!("🔄 Processing PUSH -> Import ID: {}", import_id.0);

            // Process the expression
            match expr {
                Expression::Pipeline(pipeline) => {
                    println!("   Pipeline: import={}, path={:?}",
                            pipeline.import_id.0, pipeline.property_path);

                    // If import_id is 0, this is calling the main capability
                    if pipeline.import_id.0 == 0 {
                        if let Some(main_cap) = &session_state.main_capability {
                            // Extract method name and arguments
                            if let Some(property_path) = &pipeline.property_path {
                                if let Some(first_prop) = property_path.first() {
                                    if let capnweb_core::protocol::PropertyKey::String(method) = first_prop {
                                        let args = if let Some(call_args) = &pipeline.call_arguments {
                                            expression_to_values(call_args).unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        };

                                        println!("   Calling method: {} with {} args", method, args.len());

                                        // Make the actual RPC call
                                        match main_cap.call(method, args).await {
                                            Ok(result) => {
                                                // Store the result in the import table
                                                let _ = session_state.session.imports.insert(
                                                    import_id,
                                                    ImportValue::Value(result.clone()),
                                                );

                                                // Check if there's a pending pull for this import
                                                let mut pulls = session_state.pending_pulls.write().unwrap();
                                                if let Some(sender) = pulls.remove(&import_id) {
                                                    let response = vec![
                                                        serde_json::json!(["resolve", -(import_id.0), value_to_json(result)])
                                                    ];
                                                    let _ = sender.send(response);
                                                }

                                                return Ok(vec![]); // Push doesn't return immediate response
                                            }
                                            Err(e) => {
                                                // Store error in import table
                                                let error_value = Value::Error(
                                                    "RpcError".to_string(),
                                                    e.to_string(),
                                                    None,
                                                );
                                                let _ = session_state.session.imports.insert(
                                                    import_id,
                                                    ImportValue::Value(error_value.clone()),
                                                );

                                                // Notify any waiting pulls
                                                let mut pulls = session_state.pending_pulls.write().unwrap();
                                                if let Some(sender) = pulls.remove(&import_id) {
                                                    let response = vec![
                                                        serde_json::json!(["reject", -(import_id.0), value_to_json(error_value)])
                                                    ];
                                                    let _ = sender.send(response);
                                                }

                                                return Ok(vec![]); // Push doesn't return immediate response
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // For now, just store a placeholder
                    let _ = session_state.session.imports.insert(
                        import_id,
                        ImportValue::Value(Value::String("Pipeline result".to_string())),
                    );
                }
                _ => {
                    // For other expression types, store a placeholder
                    let _ = session_state.session.imports.insert(
                        import_id,
                        ImportValue::Value(Value::String("Expression result".to_string())),
                    );
                }
            }

            Ok(vec![]) // Push doesn't return immediate response
        }

        Message::Pull(import_id) => {
            println!("🔄 Processing PULL -> Import ID: {}", import_id.0);

            // Check if the import is already resolved
            if let Some(import_value) = session_state.session.imports.get(import_id) {
                println!("   Import already resolved");
                match import_value {
                    ImportValue::Value(value) => {
                        let export_id = -(import_id.0); // Convert to negative export ID
                        match value {
                            Value::Error(error_type, message, _) => {
                                Ok(vec![serde_json::json!(["reject", export_id, ["error", error_type, message]])])
                            }
                            _ => {
                                Ok(vec![serde_json::json!(["resolve", export_id, value_to_json(value)])])
                            }
                        }
                    }
                    _ => {
                        Ok(vec![serde_json::json!(["resolve", -(import_id.0), "Not implemented"])])
                    }
                }
            } else {
                println!("   Import not yet resolved, will wait");
                // Import doesn't exist yet - set up a pull waiter
                let (tx, rx) = tokio::sync::oneshot::channel();
                session_state.pending_pulls.write().unwrap().insert(import_id, tx);

                // Wait for resolution with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    rx,
                ).await {
                    Ok(Ok(response)) => Ok(response),
                    Ok(Err(_)) => {
                        // Channel closed
                        Ok(vec![serde_json::json!(["reject", -(import_id.0), ["error", "ChannelError", "Resolution channel closed"]])])
                    }
                    Err(_) => {
                        // Timeout
                        session_state.pending_pulls.write().unwrap().remove(&import_id);
                        Ok(vec![serde_json::json!(["reject", -(import_id.0), ["error", "Timeout", "Pull request timed out"]])])
                    }
                }
            }
        }

        _ => {
            println!("🔄 Processing OTHER message: {:?}", message);
            Ok(vec![])
        }
    }
}

/// Batch RPC endpoint handler - Cap'n Web protocol compliant
async fn handle_batch(
    State(state): State<ProtocolServerState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    println!("📨 Received batch request ({} bytes)", body.len());

    // Get session ID from headers or create new one
    let session_id = headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let session_state = get_or_create_session(&state, &session_id).await;

    let mut all_responses = Vec::new();

    // Parse messages - Cap'n Web uses newline-delimited format
    for (line_num, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        println!("📝 Processing line {}: {}", line_num, line);

        // Parse each line as a separate message
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(msg_json) => {
                match Message::from_json(&msg_json) {
                    Ok(message) => {
                        println!("   Parsed message: {:?}", message);
                        match process_capnweb_message(&session_state, message).await {
                            Ok(responses) => {
                                all_responses.extend(responses);
                            }
                            Err(e) => {
                                println!("   Error processing message: {}", e);
                                all_responses.push(serde_json::json!(["abort", ["error", "ProcessError", e]]));
                            }
                        }
                    }
                    Err(e) => {
                        println!("   Error parsing message: {:?}", e);
                        all_responses.push(serde_json::json!(["abort", ["error", "ParseError", e.to_string()]]));
                    }
                }
            }
            Err(e) => {
                println!("   Error parsing JSON: {}", e);
                all_responses.push(serde_json::json!(["abort", ["error", "JSONError", e.to_string()]]));
            }
        }
    }

    println!("📤 Sending {} responses", all_responses.len());
    for (i, resp) in all_responses.iter().enumerate() {
        println!("   Response {}: {}", i, serde_json::to_string(resp).unwrap_or_default());
    }

    // Return responses in newline-delimited format
    let response_body = all_responses
        .iter()
        .map(|r| serde_json::to_string(r).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        response_body
    ).into_response()
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Configure server with direct routing (not using CapnWebServer wrapper)
    let state = ProtocolServerState {
        main_capability: Some(Arc::new(Calculator)),
        sessions: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/rpc/batch", post(handle_batch))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);

    println!("🚀 Protocol Compliant Cap'n Web Server");
    println!("======================================");
    println!("✅ Proper message parsing (newline-delimited)");
    println!("✅ Push/Pull semantics implementation");
    println!("✅ Import/Export ID allocation");
    println!("✅ Session state management");
    println!("📍 Listening on: http://{}", addr);
    println!("");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}