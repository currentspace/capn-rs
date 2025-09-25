use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    BadRequest,
    NotFound,
    CapRevoked,
    PermissionDenied,
    Canceled,
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorCode::BadRequest => "bad_request",
            ErrorCode::NotFound => "not_found",
            ErrorCode::CapRevoked => "cap_revoked",
            ErrorCode::PermissionDenied => "permission_denied",
            ErrorCode::Canceled => "canceled",
            ErrorCode::Internal => "internal",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        RpcError {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: ErrorCode, message: impl Into<String>, data: Value) -> Self {
        RpcError {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn cap_revoked(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::CapRevoked, message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, message)
    }

    pub fn canceled(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Canceled, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

impl From<serde_json::Error> for RpcError {
    fn from(err: serde_json::Error) -> Self {
        RpcError::bad_request(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for RpcError {
    fn from(err: std::io::Error) -> Self {
        RpcError::internal(format!("IO error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RpcError::new(ErrorCode::BadRequest, "Invalid input");
        assert_eq!(err.code, ErrorCode::BadRequest);
        assert_eq!(err.message, "Invalid input");
        assert_eq!(err.data, None);
    }

    #[test]
    fn test_error_with_data() {
        let data = serde_json::json!({"field": "value"});
        let err = RpcError::with_data(ErrorCode::Internal, "Server error", data.clone());
        assert_eq!(err.code, ErrorCode::Internal);
        assert_eq!(err.message, "Server error");
        assert_eq!(err.data, Some(data));
    }

    #[test]
    fn test_convenience_constructors() {
        let err = RpcError::bad_request("Bad input");
        assert_eq!(err.code, ErrorCode::BadRequest);

        let err = RpcError::not_found("Resource not found");
        assert_eq!(err.code, ErrorCode::NotFound);

        let err = RpcError::cap_revoked("Capability revoked");
        assert_eq!(err.code, ErrorCode::CapRevoked);

        let err = RpcError::permission_denied("Access denied");
        assert_eq!(err.code, ErrorCode::PermissionDenied);

        let err = RpcError::canceled("Operation canceled");
        assert_eq!(err.code, ErrorCode::Canceled);

        let err = RpcError::internal("Internal error");
        assert_eq!(err.code, ErrorCode::Internal);
    }

    #[test]
    fn test_error_serialization() {
        let err = RpcError::new(ErrorCode::NotFound, "Resource not found");
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: RpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, deserialized);
    }

    #[test]
    fn test_error_serialization_with_data() {
        let data = serde_json::json!({"id": 123});
        let err = RpcError::with_data(ErrorCode::BadRequest, "Invalid ID", data);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"data\""));
        let deserialized: RpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, deserialized);
    }

    #[test]
    fn test_error_display() {
        let err = RpcError::internal("Something went wrong");
        let display = format!("{}", err);
        assert!(display.contains("Internal"));
        assert!(display.contains("Something went wrong"));
    }
}