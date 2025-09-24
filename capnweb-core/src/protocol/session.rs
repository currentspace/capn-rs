// RPC session management for Cap'n Web protocol
// Handles message processing and session state

use super::message::Message;
use super::expression::Expression;
use super::ids::{ImportId, IdAllocator};
// use super::ids::ExportId; // TODO: Remove when export handling is implemented
use super::tables::{ImportTable, ExportTable, Value};
use super::evaluator::ExpressionEvaluator;
use std::sync::Arc;
use tokio::sync::Mutex;

/// RPC session state
pub struct RpcSession {
    pub allocator: Arc<IdAllocator>,
    pub imports: Arc<ImportTable>,
    pub exports: Arc<ExportTable>,
    pub evaluator: Arc<Mutex<ExpressionEvaluator>>,
}

impl RpcSession {
    /// Create a new RPC session
    pub fn new() -> Self {
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::new(allocator.clone()));
        let exports = Arc::new(ExportTable::new(allocator.clone()));
        let evaluator = Arc::new(Mutex::new(ExpressionEvaluator::new(
            imports.clone(),
            exports.clone(),
        )));

        Self {
            allocator,
            imports,
            exports,
            evaluator,
        }
    }

    /// Handle an incoming message
    pub async fn handle_message(&self, msg: Message) -> Result<(), SessionError> {
        match msg {
            Message::Push(expr) => {
                // Allocate import ID and evaluate expression
                let _import_id = self.imports.allocate_local();

                // TODO: Evaluate expression and store result
                let _ = self.evaluator.lock().await.evaluate(expr).await?;

                Ok(())
            }

            Message::Pull(_import_id) => {
                // Request resolution of an import
                // TODO: Send resolve message for the import
                Ok(())
            }

            Message::Resolve(export_id, expr) => {
                // Resolve an export with a value
                let value = self.evaluator.lock().await.evaluate(expr).await?;
                self.exports.resolve(export_id, value).await?;
                Ok(())
            }

            Message::Reject(export_id, expr) => {
                // Reject an export with an error
                let error = self.evaluator.lock().await.evaluate(expr).await?;
                self.exports.reject(export_id, error).await?;
                Ok(())
            }

            Message::Release(import_id, refcount) => {
                // Release an import
                self.imports.release(import_id, refcount)?;
                Ok(())
            }

            Message::Abort(expr) => {
                // Terminate the session
                let _ = self.evaluator.lock().await.evaluate(expr).await?;
                // TODO: Clean up session
                Ok(())
            }
        }
    }

    /// Send a push message
    pub async fn push(&self, _expr: Expression) -> ImportId {
        let import_id = self.imports.allocate_local();

        // TODO: Send push message over transport

        import_id
    }

    /// Send a pull message
    pub async fn pull(&self, _import_id: ImportId) -> Result<Value, SessionError> {
        // TODO: Send pull message and wait for resolution
        Err(SessionError::NotImplemented)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Feature not yet implemented")]
    NotImplemented,

    #[error("Evaluator error: {0}")]
    EvaluatorError(#[from] super::evaluator::EvaluatorError),

    #[error("Table error: {0}")]
    TableError(#[from] super::tables::TableError),

    #[error("Transport error")]
    TransportError,
}