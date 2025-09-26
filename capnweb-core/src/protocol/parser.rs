// Expression parser for Cap'n Web protocol
// This module is responsible for parsing JSON values into typed expressions

use super::expression::Expression;
use serde_json::Value as JsonValue;

pub struct ExpressionParser;

impl ExpressionParser {
    /// Parse a JSON value into an Expression
    pub fn parse(value: &JsonValue) -> Result<Expression, ParseError> {
        Expression::from_json(value).map_err(ParseError::ExpressionError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Expression parsing error: {0}")]
    ExpressionError(#[from] super::expression::ExpressionError),

    #[error("Invalid JSON structure")]
    InvalidJson,

    #[error("Unknown expression type")]
    UnknownType,
}
