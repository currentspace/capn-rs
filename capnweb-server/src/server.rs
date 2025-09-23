use async_trait::async_trait;
use serde_json::Value;
use capnweb_core::RpcError;

#[async_trait]
pub trait RpcTarget: Send + Sync {
    async fn call(&self, member: &str, args: Vec<Value>) -> Result<Value, RpcError>;
}

pub struct ServerConfig {
    pub port: u16,
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Server { config }
    }
}