use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use super::expression::Expression;
use super::ids::{ImportId, ExportId};

/// Cap'n Web protocol messages
/// Messages are represented as JSON arrays with the message type as the first element
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// ["push", expression] - Evaluate an expression and assign it an import ID
    Push(Expression),

    /// ["pull", importId] - Request resolution of an import
    Pull(ImportId),

    /// ["resolve", exportId, expression] - Resolve an export with a value
    Resolve(ExportId, Expression),

    /// ["reject", exportId, expression] - Reject an export with an error
    Reject(ExportId, Expression),

    /// ["release", importId, refcount] - Release an import
    Release(ImportId, u32),

    /// ["abort", expression] - Terminate the session with an error
    Abort(Expression),
}

impl Message {
    /// Parse a message from a JSON value
    pub fn from_json(value: &JsonValue) -> Result<Self, MessageError> {
        let arr = value.as_array()
            .ok_or(MessageError::NotAnArray)?;

        if arr.is_empty() {
            return Err(MessageError::EmptyMessage);
        }

        let msg_type = arr[0].as_str()
            .ok_or(MessageError::InvalidMessageType)?;

        match msg_type {
            "push" => {
                if arr.len() != 2 {
                    return Err(MessageError::InvalidPush);
                }
                let expr = Expression::from_json(&arr[1])?;
                Ok(Message::Push(expr))
            }

            "pull" => {
                if arr.len() != 2 {
                    return Err(MessageError::InvalidPull);
                }
                let import_id = arr[1].as_i64()
                    .ok_or(MessageError::InvalidImportId)?;
                Ok(Message::Pull(ImportId(import_id)))
            }

            "resolve" => {
                if arr.len() != 3 {
                    return Err(MessageError::InvalidResolve);
                }
                let export_id = arr[1].as_i64()
                    .ok_or(MessageError::InvalidExportId)?;
                let expr = Expression::from_json(&arr[2])?;
                Ok(Message::Resolve(ExportId(export_id), expr))
            }

            "reject" => {
                if arr.len() != 3 {
                    return Err(MessageError::InvalidReject);
                }
                let export_id = arr[1].as_i64()
                    .ok_or(MessageError::InvalidExportId)?;
                let expr = Expression::from_json(&arr[2])?;
                Ok(Message::Reject(ExportId(export_id), expr))
            }

            "release" => {
                if arr.len() != 3 {
                    return Err(MessageError::InvalidRelease);
                }
                let import_id = arr[1].as_i64()
                    .ok_or(MessageError::InvalidImportId)?;
                let refcount = arr[2].as_u64()
                    .ok_or(MessageError::InvalidRefcount)? as u32;
                Ok(Message::Release(ImportId(import_id), refcount))
            }

            "abort" => {
                if arr.len() != 2 {
                    return Err(MessageError::InvalidAbort);
                }
                let expr = Expression::from_json(&arr[1])?;
                Ok(Message::Abort(expr))
            }

            _ => Err(MessageError::UnknownMessageType(msg_type.to_string()))
        }
    }

    /// Convert the message to a JSON value
    pub fn to_json(&self) -> JsonValue {
        match self {
            Message::Push(expr) => {
                serde_json::json!(["push", expr.to_json()])
            }
            Message::Pull(import_id) => {
                serde_json::json!(["pull", import_id.0])
            }
            Message::Resolve(export_id, expr) => {
                serde_json::json!(["resolve", export_id.0, expr.to_json()])
            }
            Message::Reject(export_id, expr) => {
                serde_json::json!(["reject", export_id.0, expr.to_json()])
            }
            Message::Release(import_id, refcount) => {
                serde_json::json!(["release", import_id.0, refcount])
            }
            Message::Abort(expr) => {
                serde_json::json!(["abort", expr.to_json()])
            }
        }
    }
}

/// Custom serialization for Message
impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

/// Custom deserialization for Message
impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = JsonValue::deserialize(deserializer)?;
        Message::from_json(&value)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Message must be a JSON array")]
    NotAnArray,

    #[error("Message array cannot be empty")]
    EmptyMessage,

    #[error("Message type must be a string")]
    InvalidMessageType,

    #[error("Invalid push message format")]
    InvalidPush,

    #[error("Invalid pull message format")]
    InvalidPull,

    #[error("Invalid resolve message format")]
    InvalidResolve,

    #[error("Invalid reject message format")]
    InvalidReject,

    #[error("Invalid release message format")]
    InvalidRelease,

    #[error("Invalid abort message format")]
    InvalidAbort,

    #[error("Invalid import ID")]
    InvalidImportId,

    #[error("Invalid export ID")]
    InvalidExportId,

    #[error("Invalid refcount")]
    InvalidRefcount,

    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),

    #[error("Expression error: {0}")]
    ExpressionError(#[from] super::expression::ExpressionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_push_message() {
        let json = json!(["push", "hello"]);
        let msg = Message::from_json(&json).unwrap();

        match &msg {
            Message::Push(expr) => {
                assert_eq!(expr, &Expression::String("hello".to_string()));
            }
            _ => panic!("Expected Push message"),
        }

        assert_eq!(msg.to_json(), json);
    }

    #[test]
    fn test_pull_message() {
        let json = json!(["pull", 42]);
        let msg = Message::from_json(&json).unwrap();

        match msg {
            Message::Pull(id) => {
                assert_eq!(id, ImportId(42));
            }
            _ => panic!("Expected Pull message"),
        }

        assert_eq!(msg.to_json(), json);
    }

    #[test]
    fn test_resolve_message() {
        let json = json!(["resolve", -1, "result"]);
        let msg = Message::from_json(&json).unwrap();

        match &msg {
            Message::Resolve(id, expr) => {
                assert_eq!(id, &ExportId(-1));
                assert_eq!(expr, &Expression::String("result".to_string()));
            }
            _ => panic!("Expected Resolve message"),
        }

        assert_eq!(msg.to_json(), json);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = Message::Push(Expression::Number(serde_json::Number::from(42)));
        let json = serde_json::to_value(&original).unwrap();
        let deserialized: Message = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(original, deserialized);
        assert_eq!(json, json!(["push", 42]));
    }
}