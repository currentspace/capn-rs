use async_trait::async_trait;
use capnweb_core::Message;
use crate::{RpcTransport, TransportError};
use std::collections::VecDeque;

/// HTTP Batch transport collects messages and sends them as a single HTTP request
#[cfg(feature = "http-batch")]
pub struct HttpBatchTransport {
    /// URL endpoint for the batch RPC
    endpoint: String,
    /// Outgoing message queue
    outgoing: VecDeque<Message>,
    /// Incoming message queue
    incoming: VecDeque<Message>,
}

#[cfg(feature = "http-batch")]
impl HttpBatchTransport {
    pub fn new(endpoint: String) -> Self {
        HttpBatchTransport {
            endpoint,
            outgoing: VecDeque::new(),
            incoming: VecDeque::new(),
        }
    }

    /// Execute the batch - send all queued messages and receive responses
    /// This is a placeholder implementation that will need an actual HTTP client
    pub async fn execute(&mut self) -> Result<(), TransportError> {
        if self.outgoing.is_empty() {
            return Ok(());
        }

        // Collect all outgoing messages
        let messages: Vec<Message> = self.outgoing.drain(..).collect();

        // TODO: Implement actual HTTP request using a client library
        // For now, we'll just encode the messages to verify the structure
        let _body = serde_json::to_vec(&messages)
            .map_err(|e| TransportError::Codec(e.to_string()))?;

        // Placeholder: In a real implementation, we would:
        // 1. Send HTTP POST to self.endpoint
        // 2. Parse response
        // 3. Queue incoming messages

        Ok(())
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn pending_outgoing(&self) -> usize {
        self.outgoing.len()
    }

    pub fn pending_incoming(&self) -> usize {
        self.incoming.len()
    }
}

#[cfg(feature = "http-batch")]
#[async_trait]
impl RpcTransport for HttpBatchTransport {
    async fn send(&mut self, msg: Message) -> Result<(), TransportError> {
        self.outgoing.push_back(msg);
        Ok(())
    }

    async fn recv(&mut self) -> Result<Option<Message>, TransportError> {
        Ok(self.incoming.pop_front())
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        // Execute any pending messages before closing
        if !self.outgoing.is_empty() {
            self.execute().await?;
        }
        self.outgoing.clear();
        self.incoming.clear();
        Ok(())
    }
}

#[cfg(all(test, feature = "http-batch"))]
mod tests {
    use super::*;
    use capnweb_core::{CallId, CapId, Target};
    use serde_json::json;

    #[tokio::test]
    async fn test_batch_transport_queue() {
        let mut transport = HttpBatchTransport::new("http://localhost:8080/rpc/batch".to_string());

        let msg = Message::call(
            CallId::new(1),
            Target::cap(CapId::new(42)),
            "test".to_string(),
            vec![json!("hello")],
        );

        transport.send(msg.clone()).await.unwrap();
        assert_eq!(transport.pending_outgoing(), 1);
        assert_eq!(transport.pending_incoming(), 0);

        // Without execution, recv returns None
        assert!(transport.recv().await.unwrap().is_none());
    }

    #[test]
    fn test_endpoint() {
        let transport = HttpBatchTransport::new("http://example.com/rpc".to_string());
        assert_eq!(transport.endpoint(), "http://example.com/rpc");
    }

    #[tokio::test]
    async fn test_close_clears_queues() {
        let mut transport = HttpBatchTransport::new("http://localhost:8080/rpc/batch".to_string());

        let msg = Message::cap_ref(CapId::new(1));
        transport.send(msg).await.unwrap();

        // Note: close will try to execute, which will fail without a server
        // but it should still clear the queues
        let _ = transport.close().await;

        assert_eq!(transport.pending_outgoing(), 0);
        assert_eq!(transport.pending_incoming(), 0);
    }
}