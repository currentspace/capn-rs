// use serde::{Deserialize, Serialize}; // TODO: Remove when serialization is implemented
use serde_json::{Value as JsonValue, Number};
use std::collections::HashMap;
use super::ids::{ImportId, ExportId};

/// Cap'n Web expressions represent values and operations in the protocol
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literal JSON values
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Expression>),
    Object(std::collections::HashMap<String, Box<Expression>>),

    // Special typed expressions (parsed from arrays)
    EscapedArray(Vec<Expression>),
    Date(f64),
    Error(ErrorExpression),
    Import(ImportExpression),
    Pipeline(PipelineExpression),
    Remap(RemapExpression),
    Export(ExportExpression),
    Promise(PromiseExpression),
}

impl Expression {
    /// Parse an expression from a JSON value
    pub fn from_json(value: &JsonValue) -> Result<Self, ExpressionError> {
        match value {
            JsonValue::Null => Ok(Expression::Null),
            JsonValue::Bool(b) => Ok(Expression::Bool(*b)),
            JsonValue::Number(n) => Ok(Expression::Number(n.clone())),
            JsonValue::String(s) => Ok(Expression::String(s.clone())),

            JsonValue::Array(arr) if arr.is_empty() => {
                Ok(Expression::Array(Vec::new()))
            }

            JsonValue::Array(arr) => {
                // Check if this is a special typed array
                if let Some(JsonValue::String(type_code)) = arr.first() {
                    Self::parse_typed_array(type_code, arr)
                } else if let Some(JsonValue::Array(inner)) = arr.first() {
                    // This is an escaped array: [[...]]
                    if arr.len() == 1 {
                        let elements = inner.iter()
                            .map(Self::from_json)
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Expression::EscapedArray(elements))
                    } else {
                        // Regular array with array as first element
                        let elements = arr.iter()
                            .map(Self::from_json)
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Expression::Array(elements))
                    }
                } else {
                    // Regular array
                    let elements = arr.iter()
                        .map(Self::from_json)
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Expression::Array(elements))
                }
            }

            JsonValue::Object(obj) => {
                let mut map = HashMap::new();
                for (key, val) in obj {
                    map.insert(key.clone(), Box::new(Self::from_json(val)?));
                }
                Ok(Expression::Object(map))
            }
        }
    }

    /// Convert the expression to a JSON value
    pub fn to_json(&self) -> JsonValue {
        match self {
            Expression::Null => JsonValue::Null,
            Expression::Bool(b) => JsonValue::Bool(*b),
            Expression::Number(n) => JsonValue::Number(n.clone()),
            Expression::String(s) => JsonValue::String(s.clone()),

            Expression::Array(elements) => {
                JsonValue::Array(elements.iter().map(|e| e.to_json()).collect())
            }

            Expression::Object(map) => {
                let mut obj = serde_json::Map::new();
                for (key, val) in map {
                    obj.insert(key.clone(), val.to_json());
                }
                JsonValue::Object(obj)
            }

            Expression::EscapedArray(elements) => {
                // Wrap in outer array for escaping
                let inner = elements.iter().map(|e| e.to_json()).collect();
                JsonValue::Array(vec![JsonValue::Array(inner)])
            }

            Expression::Date(millis) => {
                serde_json::json!(["date", millis])
            }

            Expression::Error(err) => {
                if let Some(stack) = &err.stack {
                    serde_json::json!(["error", &err.error_type, &err.message, stack])
                } else {
                    serde_json::json!(["error", &err.error_type, &err.message])
                }
            }

            Expression::Import(import) => import.to_json(),
            Expression::Pipeline(pipeline) => pipeline.to_json(),
            Expression::Remap(remap) => remap.to_json(),
            Expression::Export(export) => export.to_json(),
            Expression::Promise(promise) => promise.to_json(),
        }
    }

    fn parse_typed_array(type_code: &str, arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        match type_code {
            "date" => {
                if arr.len() != 2 {
                    return Err(ExpressionError::InvalidDate);
                }
                let millis = arr[1].as_f64()
                    .ok_or(ExpressionError::InvalidDate)?;
                Ok(Expression::Date(millis))
            }

            "error" => {
                if arr.len() < 3 || arr.len() > 4 {
                    return Err(ExpressionError::InvalidError);
                }
                let error_type = arr[1].as_str()
                    .ok_or(ExpressionError::InvalidError)?
                    .to_string();
                let message = arr[2].as_str()
                    .ok_or(ExpressionError::InvalidError)?
                    .to_string();
                let stack = arr.get(3)
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Ok(Expression::Error(ErrorExpression {
                    error_type,
                    message,
                    stack,
                }))
            }

            "import" => {
                ImportExpression::from_array(arr).map(Expression::Import)
            }

            "pipeline" => {
                PipelineExpression::from_array(arr).map(Expression::Pipeline)
            }

            "remap" => {
                RemapExpression::from_array(arr).map(Expression::Remap)
            }

            "export" => {
                ExportExpression::from_array(arr).map(Expression::Export)
            }

            "promise" => {
                PromiseExpression::from_array(arr).map(Expression::Promise)
            }

            _ => {
                // Unknown type code, treat as regular array
                let elements = arr.iter()
                    .map(Self::from_json)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Expression::Array(elements))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorExpression {
    pub error_type: String,
    pub message: String,
    pub stack: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub call_arguments: Option<Box<Expression>>,
}

impl ImportExpression {
    fn from_array(arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        if arr.len() < 2 || arr.len() > 4 {
            return Err(ExpressionError::InvalidImport);
        }

        let import_id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidImport)?;

        let property_path = arr.get(2)
            .map(PropertyKey::parse_path)
            .transpose()?;

        let call_arguments = arr.get(3)
            .map(|v| Expression::from_json(v).map(Box::new))
            .transpose()?;

        Ok(ImportExpression {
            import_id: ImportId(import_id),
            property_path,
            call_arguments,
        })
    }

    fn to_json(&self) -> JsonValue {
        let mut arr = vec![
            JsonValue::String("import".to_string()),
            JsonValue::Number(Number::from(self.import_id.0)),
        ];

        if let Some(path) = &self.property_path {
            arr.push(PropertyKey::path_to_json(path));
        }

        if let Some(args) = &self.call_arguments {
            arr.push(args.to_json());
        }

        JsonValue::Array(arr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub call_arguments: Option<Box<Expression>>,
}

impl PipelineExpression {
    fn from_array(arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        if arr.len() < 2 || arr.len() > 4 {
            return Err(ExpressionError::InvalidPipeline);
        }

        let import_id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidPipeline)?;

        let property_path = arr.get(2)
            .map(PropertyKey::parse_path)
            .transpose()?;

        let call_arguments = arr.get(3)
            .map(|v| Expression::from_json(v).map(Box::new))
            .transpose()?;

        Ok(PipelineExpression {
            import_id: ImportId(import_id),
            property_path,
            call_arguments,
        })
    }

    fn to_json(&self) -> JsonValue {
        let mut arr = vec![
            JsonValue::String("pipeline".to_string()),
            JsonValue::Number(Number::from(self.import_id.0)),
        ];

        if let Some(path) = &self.property_path {
            arr.push(PropertyKey::path_to_json(path));
        }

        if let Some(args) = &self.call_arguments {
            arr.push(args.to_json());
        }

        JsonValue::Array(arr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemapExpression {
    pub import_id: ImportId,
    pub property_path: Option<Vec<PropertyKey>>,
    pub captures: Vec<CaptureRef>,
    pub instructions: Vec<Expression>,
}

impl RemapExpression {
    fn from_array(arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        if arr.len() != 5 {
            return Err(ExpressionError::InvalidRemap);
        }

        let import_id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidRemap)?;

        let property_path = if !arr[2].is_null() {
            Some(PropertyKey::parse_path(&arr[2])?)
        } else {
            None
        };

        let captures = arr[3].as_array()
            .ok_or(ExpressionError::InvalidRemap)?
            .iter()
            .map(CaptureRef::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let instructions = arr[4].as_array()
            .ok_or(ExpressionError::InvalidRemap)?
            .iter()
            .map(Expression::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RemapExpression {
            import_id: ImportId(import_id),
            property_path,
            captures,
            instructions,
        })
    }

    fn to_json(&self) -> JsonValue {
        let path = self.property_path.as_ref()
            .map(|p| PropertyKey::path_to_json(p))
            .unwrap_or(JsonValue::Null);

        let captures: Vec<JsonValue> = self.captures.iter()
            .map(|c| c.to_json())
            .collect();

        let instructions: Vec<JsonValue> = self.instructions.iter()
            .map(|i| i.to_json())
            .collect();

        serde_json::json!([
            "remap",
            self.import_id.0,
            path,
            captures,
            instructions
        ])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportExpression {
    pub export_id: ExportId,
}

impl ExportExpression {
    fn from_array(arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        if arr.len() != 2 {
            return Err(ExpressionError::InvalidExport);
        }

        let export_id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidExport)?;

        Ok(ExportExpression {
            export_id: ExportId(export_id),
        })
    }

    fn to_json(&self) -> JsonValue {
        serde_json::json!(["export", self.export_id.0])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromiseExpression {
    pub export_id: ExportId,
}

impl PromiseExpression {
    fn from_array(arr: &[JsonValue]) -> Result<Self, ExpressionError> {
        if arr.len() != 2 {
            return Err(ExpressionError::InvalidPromise);
        }

        let export_id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidPromise)?;

        Ok(PromiseExpression {
            export_id: ExportId(export_id),
        })
    }

    fn to_json(&self) -> JsonValue {
        serde_json::json!(["promise", self.export_id.0])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    String(String),
    Number(usize),
}

impl PropertyKey {
    fn parse_path(value: &JsonValue) -> Result<Vec<PropertyKey>, ExpressionError> {
        let arr = value.as_array()
            .ok_or(ExpressionError::InvalidPropertyPath)?;

        arr.iter()
            .map(|v| {
                if let Some(s) = v.as_str() {
                    Ok(PropertyKey::String(s.to_string()))
                } else if let Some(n) = v.as_u64() {
                    Ok(PropertyKey::Number(n as usize))
                } else {
                    Err(ExpressionError::InvalidPropertyPath)
                }
            })
            .collect()
    }

    fn path_to_json(path: &[PropertyKey]) -> JsonValue {
        let elements: Vec<JsonValue> = path.iter()
            .map(|key| match key {
                PropertyKey::String(s) => JsonValue::String(s.clone()),
                PropertyKey::Number(n) => JsonValue::Number(Number::from(*n)),
            })
            .collect();

        JsonValue::Array(elements)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureRef {
    Import(ImportId),
    Export(ExportId),
}

impl CaptureRef {
    fn from_json(value: &JsonValue) -> Result<Self, ExpressionError> {
        let arr = value.as_array()
            .ok_or(ExpressionError::InvalidCapture)?;

        if arr.len() != 2 {
            return Err(ExpressionError::InvalidCapture);
        }

        let type_str = arr[0].as_str()
            .ok_or(ExpressionError::InvalidCapture)?;

        let id = arr[1].as_i64()
            .ok_or(ExpressionError::InvalidCapture)?;

        match type_str {
            "import" => Ok(CaptureRef::Import(ImportId(id))),
            "export" => Ok(CaptureRef::Export(ExportId(id))),
            _ => Err(ExpressionError::InvalidCapture),
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            CaptureRef::Import(id) => serde_json::json!(["import", id.0]),
            CaptureRef::Export(id) => serde_json::json!(["export", id.0]),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExpressionError {
    #[error("Invalid date expression")]
    InvalidDate,

    #[error("Invalid error expression")]
    InvalidError,

    #[error("Invalid import expression")]
    InvalidImport,

    #[error("Invalid pipeline expression")]
    InvalidPipeline,

    #[error("Invalid remap expression")]
    InvalidRemap,

    #[error("Invalid export expression")]
    InvalidExport,

    #[error("Invalid promise expression")]
    InvalidPromise,

    #[error("Invalid property path")]
    InvalidPropertyPath,

    #[error("Invalid capture reference")]
    InvalidCapture,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_literal_expressions() {
        assert_eq!(
            Expression::from_json(&json!(null)).unwrap(),
            Expression::Null
        );

        assert_eq!(
            Expression::from_json(&json!(true)).unwrap(),
            Expression::Bool(true)
        );

        assert_eq!(
            Expression::from_json(&json!(42)).unwrap(),
            Expression::Number(Number::from(42))
        );

        assert_eq!(
            Expression::from_json(&json!("hello")).unwrap(),
            Expression::String("hello".to_string())
        );
    }

    #[test]
    fn test_date_expression() {
        let json = json!(["date", 1234567890.0]);
        let expr = Expression::from_json(&json).unwrap();

        match expr {
            Expression::Date(millis) => assert_eq!(millis, 1234567890.0),
            _ => panic!("Expected Date expression"),
        }

        assert_eq!(expr.to_json(), json);
    }

    #[test]
    fn test_error_expression() {
        let json = json!(["error", "TypeError", "Something went wrong", "stack trace"]);
        let expr = Expression::from_json(&json).unwrap();

        match expr {
            Expression::Error(err) => {
                assert_eq!(err.error_type, "TypeError");
                assert_eq!(err.message, "Something went wrong");
                assert_eq!(err.stack, Some("stack trace".to_string()));
            }
            _ => panic!("Expected Error expression"),
        }
    }

    #[test]
    fn test_import_expression() {
        let json = json!(["import", 42, ["method"], [1, 2, 3]]);
        let expr = Expression::from_json(&json).unwrap();

        match expr {
            Expression::Import(import) => {
                assert_eq!(import.import_id, ImportId(42));
                assert_eq!(import.property_path, Some(vec![PropertyKey::String("method".to_string())]));
                assert!(import.call_arguments.is_some());
            }
            _ => panic!("Expected Import expression"),
        }
    }

    #[test]
    fn test_escaped_array() {
        let json = json!([["just", "an", "array"]]);
        let expr = Expression::from_json(&json).unwrap();

        match expr {
            Expression::EscapedArray(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expression::String("just".to_string()));
            }
            _ => panic!("Expected EscapedArray expression"),
        }
    }
}