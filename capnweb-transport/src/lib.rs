pub mod capnweb_codec;
// #[cfg(feature = "http3")]  // Disabled: experimental
// pub mod http3;
#[cfg(feature = "http-batch")]
pub mod http_batch;
pub mod negotiate;
pub mod transport;
#[cfg(feature = "websocket")]
pub mod websocket;
#[cfg(feature = "webtransport")]
pub mod webtransport;

pub use capnweb_codec::{CapnWebCodec, CodecError, NewlineDelimitedCodec};
// #[cfg(feature = "http3")]  // Disabled: experimental
// pub use http3::{make_http3_client_endpoint, Http3Client, Http3Config, Http3Stats, Http3Transport};
#[cfg(feature = "http-batch")]
pub use http_batch::HttpBatchTransport;
pub use transport::{RpcTransport, TransportError, TransportEvent};
#[cfg(feature = "webtransport")]
pub use webtransport::{make_client_endpoint, WebTransportClient, WebTransportTransport};
