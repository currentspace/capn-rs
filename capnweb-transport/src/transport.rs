use async_trait::async_trait;
use thiserror::Error;
use bytes::Bytes;
use capnweb_core::Message;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Codec error: {0}")]
    Codec(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone)]
pub enum TransportEvent {
    Frame(Bytes),
    Closed(Option<TransportError>),
    Heartbeat,
}

#[async_trait]
pub trait RpcTransport: Send + Sync {
    async fn send(&mut self, msg: Message) -> Result<(), TransportError>;
    async fn recv(&mut self) -> Result<Option<Message>, TransportError>;
    async fn close(&mut self) -> Result<(), TransportError>;
}