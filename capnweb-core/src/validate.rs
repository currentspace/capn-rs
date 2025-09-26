use crate::RpcError;
#[cfg(feature = "validation")]
use jsonschema;
use schemars::{schema_for, JsonSchema};
use serde_json::Value;

#[cfg(feature = "validation")]
pub struct Validator {
    schema: jsonschema::Validator,
}

#[cfg(feature = "validation")]
impl Validator {
    pub fn new(schema: Value) -> Result<Self, RpcError> {
        let compiled = jsonschema::validator_for(&schema)
            .map_err(|e| RpcError::bad_request(format!("Invalid schema: {}", e)))?;
        Ok(Validator { schema: compiled })
    }

    pub fn from_type<T: JsonSchema>() -> Result<Self, RpcError> {
        let schema = schema_for!(T);
        let schema_value = serde_json::to_value(schema)
            .map_err(|e| RpcError::internal(format!("Schema serialization error: {}", e)))?;
        Self::new(schema_value)
    }

    pub fn validate(&self, value: &Value) -> Result<(), Vec<String>> {
        if self.schema.is_valid(value) {
            Ok(())
        } else {
            Err(vec!["Validation failed".to_string()])
        }
    }

    pub fn is_valid(&self, value: &Value) -> bool {
        self.schema.is_valid(value)
    }
}

#[cfg(not(feature = "validation"))]
pub struct Validator;

#[cfg(not(feature = "validation"))]
impl Validator {
    pub fn new(_schema: Value) -> Result<Self, RpcError> {
        Err(RpcError::internal("Validation feature not enabled"))
    }

    pub fn validate(&self, _value: &Value) -> Result<(), Vec<String>> {
        Err(vec!["Validation feature not enabled".to_string()])
    }

    pub fn is_valid(&self, _value: &Value) -> bool {
        true
    }
}

#[cfg(all(test, feature = "validation"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_creation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name"]
        });

        let validator = Validator::new(schema).unwrap();
        assert!(validator.is_valid(&json!({"name": "Alice", "age": 30})));
        assert!(validator.is_valid(&json!({"name": "Bob"})));
        assert!(!validator.is_valid(&json!({"age": 30})));
    }

    #[test]
    fn test_validation_errors() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100
                }
            }
        });

        let validator = Validator::new(schema).unwrap();

        let valid = json!({"count": 50});
        assert!(validator.validate(&valid).is_ok());

        let invalid = json!({"count": 150});
        let result = validator.validate(&invalid);
        assert!(result.is_err());
    }

    #[derive(JsonSchema)]
    #[allow(dead_code)]
    struct TestStruct {
        name: String,
        age: Option<u32>,
    }

    #[test]
    fn test_from_type() {
        let validator = Validator::from_type::<TestStruct>().unwrap();

        assert!(validator.is_valid(&json!({
            "name": "Alice",
            "age": 30
        })));

        assert!(validator.is_valid(&json!({
            "name": "Bob"
        })));

        assert!(!validator.is_valid(&json!({
            "age": 30
        })));
    }
}
