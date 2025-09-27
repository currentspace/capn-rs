#![allow(clippy::io_other_error)]  // std::io::Error::other requires Rust 1.81+, we support 1.75+

use crate::{RpcTransport, TransportError};
use async_trait::async_trait;
use capnweb_core::{decode_message, encode_message, Message};
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use std::sync::Arc;
use tokio::sync::Mutex;

/// WebTransport implementation using QUIC
pub struct WebTransportTransport {
    connection: Arc<Connection>,
    send_stream: Arc<Mutex<Option<SendStream>>>,
    recv_stream: Arc<Mutex<Option<RecvStream>>>,
}

impl WebTransportTransport {
    /// Create a new WebTransport from a QUIC connection
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Arc::new(connection),
            send_stream: Arc::new(Mutex::new(None)),
            recv_stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize bidirectional stream for communication
    pub async fn init_stream(&mut self) -> Result<(), TransportError> {
        let (send, recv) = self.connection.open_bi().await.map_err(|e| {
            TransportError::Protocol(format!("Failed to open bidirectional stream: {}", e))
        })?;

        *self.send_stream.lock().await = Some(send);
        *self.recv_stream.lock().await = Some(recv);

        Ok(())
    }
}

#[async_trait]
impl RpcTransport for WebTransportTransport {
    async fn send(&mut self, message: Message) -> Result<(), TransportError> {
        let encoded = encode_message(&message).map_err(|e| TransportError::Codec(e.to_string()))?;

        let mut send_lock = self.send_stream.lock().await;
        let send_stream = send_lock
            .as_mut()
            .ok_or_else(|| TransportError::Protocol("Stream not initialized".to_string()))?;

        // Send length prefix (4 bytes, big-endian)
        let len = encoded.len() as u32;
        let len_bytes = len.to_be_bytes();
        send_stream
            .write_all(&len_bytes)
            .await
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Send message data
        send_stream
            .write_all(&encoded)
            .await
            .map_err(|e| TransportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    async fn recv(&mut self) -> Result<Option<Message>, TransportError> {
        let mut recv_lock = self.recv_stream.lock().await;
        let recv_stream = recv_lock
            .as_mut()
            .ok_or_else(|| TransportError::Protocol("Stream not initialized".to_string()))?;

        // Read length prefix
        let mut len_bytes = [0u8; 4];
        match recv_stream.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(_) => return Ok(None), // Stream closed
        }

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Read message data
        let mut data = vec![0u8; len];
        recv_stream
            .read_exact(&mut data)
            .await
            .map_err(|e| TransportError::Protocol(format!("Failed to read message: {}", e)))?;

        let message = decode_message(&data).map_err(|e| TransportError::Codec(e.to_string()))?;

        Ok(Some(message))
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.connection.close(0u32.into(), b"closing");
        Ok(())
    }
}

/// WebTransport client
pub struct WebTransportClient {
    endpoint: Endpoint,
}

impl WebTransportClient {
    /// Create a new WebTransport client
    pub fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }

    /// Connect to a WebTransport server
    pub async fn connect(&self, addr: &str) -> Result<WebTransportTransport, TransportError> {
        let connection = self
            .endpoint
            .connect(
                addr.parse()
                    .map_err(|e| TransportError::Protocol(format!("Invalid address: {}", e)))?,
                "localhost",
            )
            .map_err(|e| TransportError::Protocol(format!("Connection failed: {}", e)))?
            .await
            .map_err(|e| TransportError::Protocol(format!("Connection failed: {}", e)))?;

        let mut transport = WebTransportTransport::new(connection);
        transport.init_stream().await?;

        Ok(transport)
    }
}

/// Create a client endpoint with default configuration
pub fn make_client_endpoint() -> Result<Endpoint, TransportError> {
    let client_cfg = configure_client();
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
        .map_err(|e| TransportError::Protocol(format!("Failed to create endpoint: {}", e)))?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}

fn configure_client() -> quinn::ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).unwrap(),
    ))
}

/// Skip certificate verification for testing
/// WARNING: Only use this for testing!
#[derive(Debug)]
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_webtransport_creation() {
        // Basic test to ensure the module compiles
        // Full integration tests would require QUIC server setup
    }
}
