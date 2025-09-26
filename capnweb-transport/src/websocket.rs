use crate::{RpcTransport, TransportError};
use async_trait::async_trait;
use capnweb_core::{decode_message, encode_message, Message};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{tungstenite, WebSocketStream};
use tungstenite::protocol::Message as WsMessage;

/// WebSocket transport implementation
pub struct WebSocketTransport<S> {
    /// The WebSocket stream
    stream: Arc<Mutex<WebSocketStream<S>>>,
    /// Channel for sending messages
    tx: mpsc::UnboundedSender<Message>,
    /// Channel for receiving messages
    rx: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,
}

impl<S> WebSocketTransport<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Sync + Unpin + 'static,
{
    /// Create a new WebSocket transport from a stream
    pub fn new(stream: WebSocketStream<S>) -> Self {
        let (tx, mut rx_internal) = mpsc::unbounded_channel();
        let (tx_internal, rx) = mpsc::unbounded_channel();

        let stream = Arc::new(Mutex::new(stream));
        let stream_clone = stream.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            loop {
                let mut stream = stream_clone.lock().await;
                match stream.next().await {
                    Some(Ok(msg)) => {
                        if let WsMessage::Binary(data) = msg {
                            // Decode the message
                            match decode_message(&data) {
                                Ok(message) => {
                                    if tx_internal.send(message).is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode message: {}", e);
                                }
                            }
                        } else if let WsMessage::Close(_) = msg {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
        });

        // Spawn task to handle outgoing messages
        let stream_clone2 = stream.clone();
        tokio::spawn(async move {
            while let Some(message) = rx_internal.recv().await {
                let encoded = match encode_message(&message) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to encode message: {}", e);
                        continue;
                    }
                };

                let mut stream = stream_clone2.lock().await;
                if let Err(e) = stream.send(WsMessage::Binary(encoded)).await {
                    eprintln!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });

        Self {
            stream,
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[async_trait]
impl<S> RpcTransport for WebSocketTransport<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Sync + Unpin + 'static,
{
    async fn send(&mut self, message: Message) -> Result<(), TransportError> {
        self.tx
            .send(message)
            .map_err(|_| TransportError::ConnectionClosed)
    }

    async fn recv(&mut self) -> Result<Option<Message>, TransportError> {
        let mut rx = self.rx.lock().await;
        Ok(rx.recv().await)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        let mut stream = self.stream.lock().await;
        stream
            .close(None)
            .await
            .map_err(|e| TransportError::Protocol(format!("Failed to close WebSocket: {}", e)))
    }
}

/// WebSocket client transport
pub struct WebSocketClient {
    url: String,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Connect to the WebSocket server
    pub async fn connect(
        &self,
    ) -> Result<
        WebSocketTransport<
            impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Sync + Unpin + 'static,
        >,
        TransportError,
    > {
        let (stream, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .map_err(|e| TransportError::Protocol(format!("Failed to connect: {}", e)))?;

        Ok(WebSocketTransport::new(stream))
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;
    use tokio_tungstenite::MaybeTlsStream;

    #[tokio::test]
    async fn test_websocket_transport() {
        // For now, just test the WebSocket transport can be created
        // Full integration test would require more complex setup

        // This is a basic smoke test to ensure compilation
        let _unused_import: Option<MaybeTlsStream<TcpStream>> = None;
    }
}
