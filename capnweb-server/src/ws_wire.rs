use crate::server_wire_handler::{value_to_wire_expr, wire_expr_to_values};
use crate::Server;
use axum::{
    extract::{
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use capnweb_core::{
    parse_wire_batch, serialize_wire_batch, CapId, PropertyKey, WireExpression, WireMessage,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WebSocket session state that persists across messages
struct WsSession {
    #[allow(dead_code)]
    session_id: String,
    next_import_id: i64,
    #[allow(dead_code)]
    next_export_id: i64,
    // Map import IDs to their expressions
    imports: HashMap<i64, WireExpression>,
    // Map export IDs to their values
    #[allow(dead_code)]
    exports: HashMap<i64, WireExpression>,
}

impl WsSession {
    fn new(session_id: String) -> Self {
        Self {
            session_id,
            next_import_id: 1,  // Client imports start at 1
            next_export_id: -1, // Server exports start at -1
            imports: HashMap::new(),
            exports: HashMap::new(),
        }
    }
}

/// WebSocket handler for Cap'n Web wire protocol
pub async fn websocket_wire_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<Server>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_wire_socket(socket, server))
}

async fn handle_wire_socket(socket: WebSocket, server: Arc<Server>) {
    let session_id = uuid::Uuid::new_v4().to_string();
    tracing::info!(
        "WebSocket wire protocol connection established: {}",
        session_id
    );

    let session = Arc::new(RwLock::new(WsSession::new(session_id.clone())));
    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    WsMessage::Text(text) => {
                        tracing::debug!("WS received: {}", text);

                        // Parse wire protocol messages
                        match parse_wire_batch(&text) {
                            Ok(messages) => {
                                let mut responses = Vec::new();
                                let mut session_guard = session.write().await;

                                for msg in messages {
                                    tracing::debug!("Processing WS message: {:?}", msg);

                                    match msg {
                                        WireMessage::Push(expr) => {
                                            // Assign import ID
                                            let import_id = session_guard.next_import_id;
                                            session_guard.next_import_id += 1;

                                            tracing::info!(
                                                "WS Push assigned import ID: {}",
                                                import_id
                                            );
                                            session_guard.imports.insert(import_id, expr.clone());

                                            // Process pipeline expression
                                            if let WireExpression::Pipeline {
                                                import_id: target_id,
                                                property_path,
                                                args,
                                            } = expr
                                            {
                                                let cap_id = if target_id == 0 {
                                                    CapId::new(1) // Main capability
                                                } else {
                                                    CapId::new(target_id as u64)
                                                };

                                                if let Some(capability) =
                                                    server.cap_table().lookup(&cap_id)
                                                {
                                                    if let Some(path) = property_path {
                                                        if let Some(PropertyKey::String(method)) =
                                                            path.first()
                                                        {
                                                            let json_args = args
                                                                .as_ref()
                                                                .map(|a| wire_expr_to_values(a))
                                                                .unwrap_or_else(Vec::new);

                                                            match capability
                                                                .call(method, json_args)
                                                                .await
                                                            {
                                                                Ok(result) => {
                                                                    let result_expr =
                                                                        value_to_wire_expr(result);
                                                                    session_guard.imports.insert(
                                                                        import_id,
                                                                        result_expr,
                                                                    );
                                                                }
                                                                Err(err) => {
                                                                    session_guard.imports.insert(
                                                                        import_id,
                                                                        WireExpression::Error {
                                                                            error_type: err
                                                                                .code
                                                                                .to_string(),
                                                                            message: err.message,
                                                                            stack: None,
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    session_guard.imports.insert(
                                                        import_id,
                                                        WireExpression::Error {
                                                            error_type: "not_found".to_string(),
                                                            message: format!(
                                                                "Capability {} not found",
                                                                target_id
                                                            ),
                                                            stack: None,
                                                        },
                                                    );
                                                }
                                            }
                                        }

                                        WireMessage::Pull(import_id) => {
                                            tracing::debug!("WS Pull for import_id: {}", import_id);

                                            if let Some(result) =
                                                session_guard.imports.get(&import_id)
                                            {
                                                if let WireExpression::Error { .. } = result {
                                                    responses.push(WireMessage::Reject(
                                                        import_id,
                                                        result.clone(),
                                                    ));
                                                } else {
                                                    responses.push(WireMessage::Resolve(
                                                        import_id,
                                                        result.clone(),
                                                    ));
                                                }
                                            } else {
                                                responses.push(WireMessage::Reject(
                                                    import_id,
                                                    WireExpression::Error {
                                                        error_type: "not_found".to_string(),
                                                        message: format!(
                                                            "No result for import ID {}",
                                                            import_id
                                                        ),
                                                        stack: None,
                                                    },
                                                ));
                                            }
                                        }

                                        WireMessage::Release(ids) => {
                                            tracing::info!("WS Release for IDs: {:?}", ids);
                                            // Remove released imports
                                            for id in ids {
                                                session_guard.imports.remove(&id);
                                            }
                                        }

                                        _ => {
                                            tracing::warn!("WS unhandled message type: {:?}", msg);
                                        }
                                    }
                                }

                                // Send responses
                                if !responses.is_empty() {
                                    let response_text = serialize_wire_batch(&responses);
                                    tracing::debug!("WS sending: {}", response_text);

                                    if let Err(e) =
                                        sender.send(WsMessage::Text(response_text.into())).await
                                    {
                                        tracing::error!("Failed to send WS response: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse WS wire protocol: {}", e);
                                let error_response = WireMessage::Reject(
                                    -1,
                                    WireExpression::Error {
                                        error_type: "bad_request".to_string(),
                                        message: format!("Invalid wire protocol: {}", e),
                                        stack: None,
                                    },
                                );
                                let response_text = serialize_wire_batch(&[error_response]);
                                if let Err(e) =
                                    sender.send(WsMessage::Text(response_text.into())).await
                                {
                                    tracing::error!("Failed to send error response: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    WsMessage::Binary(data) => {
                        tracing::warn!("Received binary WS message, trying as UTF-8");
                        if let Ok(_text) = String::from_utf8(data.to_vec()) {
                            // Process as text
                            continue;
                        }
                    }
                    WsMessage::Close(frame) => {
                        tracing::info!("WebSocket closing: {} (reason: {:?})", session_id, frame);
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    tracing::info!("WebSocket disconnected: {}", session_id);
}
