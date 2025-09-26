// Expression evaluator for Cap'n Web protocol
// This module evaluates expressions to produce values

use super::expression::Expression;
use super::ids::{ExportId, ImportId};
use super::remap_engine::{RemapEngine, RemapError};
use super::tables::{ExportTable, ImportTable, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Expression evaluator context
pub struct ExpressionEvaluator {
    #[allow(dead_code)]
    imports: Arc<ImportTable>,
    #[allow(dead_code)]
    exports: Arc<ExportTable>,
    remap_engine: RemapEngine,
}

impl ExpressionEvaluator {
    /// Create a new expression evaluator
    pub fn new(imports: Arc<ImportTable>, exports: Arc<ExportTable>) -> Self {
        let remap_engine = RemapEngine::new(imports.clone(), exports.clone());
        Self {
            imports,
            exports,
            remap_engine,
        }
    }

    /// Evaluate an expression to produce a value
    pub fn evaluate(
        &self,
        expr: Expression,
    ) -> Pin<Box<dyn Future<Output = Result<Value, EvaluatorError>> + Send + '_>> {
        Box::pin(async move {
            match expr {
                Expression::Null => Ok(Value::Null),
                Expression::Bool(b) => Ok(Value::Bool(b)),
                Expression::Number(n) => Ok(Value::Number(n)),
                Expression::String(s) => Ok(Value::String(s)),

                Expression::Array(elements) => {
                    let mut values = Vec::new();
                    for elem in elements {
                        values.push(self.evaluate(elem).await?);
                    }
                    Ok(Value::Array(values))
                }

                Expression::Object(map) => {
                    let mut result = std::collections::HashMap::new();
                    for (key, val) in map {
                        result.insert(key, Box::new(self.evaluate(*val).await?));
                    }
                    Ok(Value::Object(result))
                }

                Expression::Date(millis) => Ok(Value::Date(millis)),

                Expression::Error(err) => Ok(Value::Error {
                    error_type: err.error_type,
                    message: err.message,
                    stack: err.stack,
                }),

                Expression::EscapedArray(elements) => {
                    // Escaped arrays are just regular arrays in evaluation
                    let mut values = Vec::new();
                    for elem in elements {
                        values.push(self.evaluate(elem).await?);
                    }
                    Ok(Value::Array(values))
                }

                Expression::Remap(remap) => {
                    // Execute remap using the remap engine
                    self.remap_engine
                        .execute_remap(&remap, self)
                        .await
                        .map_err(EvaluatorError::RemapError)
                }

                // TODO: Implement import, pipeline, export, promise evaluation
                Expression::Import(_) => Err(EvaluatorError::NotImplemented),
                Expression::Pipeline(_) => Err(EvaluatorError::NotImplemented),
                Expression::Export(_) => Err(EvaluatorError::NotImplemented),
                Expression::Promise(_) => Err(EvaluatorError::NotImplemented),
            }
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluatorError {
    #[error("Expression evaluation not yet implemented")]
    NotImplemented,

    #[error("Unknown import: {0}")]
    UnknownImport(ImportId),

    #[error("Unknown export: {0}")]
    UnknownExport(ExportId),

    #[error("Invalid operation")]
    InvalidOperation,

    #[error("Remap execution error: {0}")]
    RemapError(#[from] RemapError),
}
