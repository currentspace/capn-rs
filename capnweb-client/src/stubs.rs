use thiserror::Error;

#[derive(Debug, Error)]
pub enum StubError {
    #[error("Stub error: {0}")]
    Error(String),
}

pub struct Capability<T> {
    _phantom: std::marker::PhantomData<T>,
}