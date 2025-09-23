use crate::Server;
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message as WsMessage}},
    response::Response,
};
use capnweb_core::Message;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{debug, error, info};

/// WebSocket handler for Cap'n Web
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<Server>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, server))
}

async fn handle_socket(socket: WebSocket, server: Arc<Server>) {
    let session_id = uuid::Uuid::new_v4().to_string();
    info!("WebSocket connection established: {}", session_id);

    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                if let WsMessage::Binary(data) = msg {
                    // Decode the Cap'n Web message
                    match capnweb_core::decode_message(&data) {
                        Ok(message) => {
                            debug!("Received message: {:?}", message);

                            // Process the message
                            let response = process_message(message, &server).await;

                            // Encode and send response
                            match capnweb_core::encode_message(&response) {
                                Ok(encoded) => {
                                    if let Err(e) = sender.send(WsMessage::Binary(encoded.to_vec())).await {
                                        error!("Failed to send response: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to encode response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to decode message: {}", e);
                        }
                    }
                } else if let WsMessage::Close(_) = msg {
                    info!("WebSocket closing: {}", session_id);
                    break;
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up session
    // TODO: Add lifecycle management when available

    info!("WebSocket disconnected: {}", session_id);
}

async fn process_message(message: Message, server: &Arc<Server>) -> Message {
    // Use the server's existing process_message method
    server.process_message(message).await
}

/// Setup WebSocket routes for Axum
pub fn setup_websocket_routes(server: Arc<Server>) -> axum::Router {
    axum::Router::new()
        .route("/ws", axum::routing::get(websocket_handler))
        .with_state(server)
}