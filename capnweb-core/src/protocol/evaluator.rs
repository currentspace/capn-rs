// Expression evaluator for Cap'n Web protocol
// This module evaluates expressions to produce values

use super::expression::Expression;
use super::tables::{Value, ImportTable, ExportTable};
use super::ids::{ImportId, ExportId};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

/// Expression evaluator context
pub struct ExpressionEvaluator {
    #[allow(dead_code)] // TODO: Use these when evaluation is implemented
    imports: Arc<ImportTable>,
    #[allow(dead_code)] // TODO: Use these when evaluation is implemented
    exports: Arc<ExportTable>,
}

impl ExpressionEvaluator {
    /// Create a new expression evaluator
    pub fn new(imports: Arc<ImportTable>, exports: Arc<ExportTable>) -> Self {
        Self { imports, exports }
    }

    /// Evaluate an expression to produce a value
    pub fn evaluate(&self, expr: Expression) -> Pin<Box<dyn Future<Output = Result<Value, EvaluatorError>> + Send + '_>> {
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

                Expression::Error(err) => {
                    Ok(Value::Error(err.error_type, err.message, err.stack))
                }

                // TODO: Implement import, pipeline, remap, export, promise evaluation
                _ => Err(EvaluatorError::NotImplemented),
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
}