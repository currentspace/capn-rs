// Variable State Management for Cap'n Web Protocol
// Implements setVariable, getVariable, and clearAllVariables functionality

use super::tables::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Variable state manager for capabilities
#[derive(Debug)]
pub struct VariableStateManager {
    /// Variables stored per session/capability
    variables: Arc<RwLock<HashMap<String, Value>>>,
    /// Maximum number of variables allowed
    max_variables: usize,
    /// Maximum variable name length
    max_name_length: usize,
}

impl VariableStateManager {
    /// Create a new variable state manager
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            max_variables: 1000, // Reasonable default limit
            max_name_length: 256, // Reasonable default limit
        }
    }

    /// Create a new variable state manager with custom limits
    pub fn with_limits(max_variables: usize, max_name_length: usize) -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            max_variables,
            max_name_length,
        }
    }

    /// Set a variable value
    pub async fn set_variable(&self, name: String, value: Value) -> Result<bool, VariableError> {
        // Validate variable name
        if name.is_empty() {
            return Err(VariableError::InvalidName("Variable name cannot be empty".to_string()));
        }

        if name.len() > self.max_name_length {
            return Err(VariableError::InvalidName(format!(
                "Variable name too long: {} > {}",
                name.len(),
                self.max_name_length
            )));
        }

        // Check for invalid characters (optional - could be customized)
        if name.chars().any(|c| c.is_control() || c == '\0') {
            return Err(VariableError::InvalidName("Variable name contains invalid characters".to_string()));
        }

        let mut variables = self.variables.write().await;

        // Check variable limits (only for new variables)
        if !variables.contains_key(&name) && variables.len() >= self.max_variables {
            return Err(VariableError::TooManyVariables(self.max_variables));
        }

        // Validate value (ensure it's serializable)
        self.validate_value(&value)?;

        tracing::debug!(
            name = %name,
            value_type = ?std::mem::discriminant(&value),
            "Setting variable"
        );

        variables.insert(name, value);
        Ok(true)
    }

    /// Get a variable value
    pub async fn get_variable(&self, name: &str) -> Result<Value, VariableError> {
        let variables = self.variables.read().await;

        match variables.get(name) {
            Some(value) => {
                tracing::debug!(
                    name = %name,
                    value_type = ?std::mem::discriminant(value),
                    "Retrieved variable"
                );
                Ok(value.clone())
            }
            None => Err(VariableError::VariableNotFound(name.to_string())),
        }
    }

    /// Check if a variable exists
    pub async fn has_variable(&self, name: &str) -> bool {
        let variables = self.variables.read().await;
        variables.contains_key(name)
    }

    /// Delete a variable
    pub async fn delete_variable(&self, name: &str) -> Result<bool, VariableError> {
        let mut variables = self.variables.write().await;

        match variables.remove(name) {
            Some(_) => {
                tracing::debug!(name = %name, "Variable deleted");
                Ok(true)
            }
            None => Err(VariableError::VariableNotFound(name.to_string())),
        }
    }

    /// Clear all variables
    pub async fn clear_all_variables(&self) -> Result<bool, VariableError> {
        let mut variables = self.variables.write().await;
        let count = variables.len();
        variables.clear();

        tracing::debug!(cleared_count = count, "All variables cleared");
        Ok(true)
    }

    /// Get all variable names
    pub async fn get_variable_names(&self) -> Vec<String> {
        let variables = self.variables.read().await;
        variables.keys().cloned().collect()
    }

    /// Get variable count
    pub async fn variable_count(&self) -> usize {
        let variables = self.variables.read().await;
        variables.len()
    }

    /// Export all variables as a HashMap (for serialization/debugging)
    pub async fn export_variables(&self) -> HashMap<String, Value> {
        let variables = self.variables.read().await;
        variables.clone()
    }

    /// Import variables from a HashMap (for deserialization/restoration)
    pub async fn import_variables(&self, vars: HashMap<String, Value>) -> Result<(), VariableError> {
        // Validate all variables first
        if vars.len() > self.max_variables {
            return Err(VariableError::TooManyVariables(self.max_variables));
        }

        for (name, value) in &vars {
            if name.len() > self.max_name_length {
                return Err(VariableError::InvalidName(format!(
                    "Variable name too long: {} > {}",
                    name.len(),
                    self.max_name_length
                )));
            }
            self.validate_value(value)?;
        }

        // If validation passes, import all variables
        let mut variables = self.variables.write().await;
        variables.clear();
        variables.extend(vars);

        tracing::debug!(imported_count = variables.len(), "Variables imported");
        Ok(())
    }

    /// Validate a value for storage
    fn validate_value(&self, value: &Value) -> Result<(), VariableError> {
        match value {
            // Simple types are always valid
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Date(_) => Ok(()),

            // Arrays and objects require recursive validation
            Value::Array(arr) => {
                if arr.len() > 1000 { // Prevent overly large arrays
                    return Err(VariableError::ValueTooComplex("Array too large".to_string()));
                }
                for item in arr {
                    self.validate_value(item)?;
                }
                Ok(())
            }

            Value::Object(obj) => {
                if obj.len() > 100 { // Prevent overly complex objects
                    return Err(VariableError::ValueTooComplex("Object too complex".to_string()));
                }
                for (key, val) in obj {
                    if key.len() > self.max_name_length {
                        return Err(VariableError::ValueTooComplex("Object key too long".to_string()));
                    }
                    self.validate_value(val)?;
                }
                Ok(())
            }

            // Error values are valid for storage
            Value::Error(_, _, _) => Ok(()),

            // Complex types (stubs, promises) cannot be stored as variables
            Value::Stub(_) => Err(VariableError::UnsupportedValueType("Cannot store stub as variable".to_string())),
            Value::Promise(_) => Err(VariableError::UnsupportedValueType("Cannot store promise as variable".to_string())),
        }
    }
}

impl Default for VariableStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors related to variable operations
#[derive(Debug, thiserror::Error)]
pub enum VariableError {
    #[error("Invalid variable name: {0}")]
    InvalidName(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Too many variables (limit: {0})")]
    TooManyVariables(usize),

    #[error("Unsupported value type: {0}")]
    UnsupportedValueType(String),

    #[error("Value too complex: {0}")]
    ValueTooComplex(String),
}

/// Enhanced RPC target trait with variable management
#[async_trait::async_trait]
pub trait VariableCapableRpcTarget: Send + Sync {
    /// Set a variable
    async fn set_variable(&self, name: String, value: Value) -> Result<Value, crate::RpcError>;

    /// Get a variable
    async fn get_variable(&self, name: String) -> Result<Value, crate::RpcError>;

    /// Clear all variables
    async fn clear_all_variables(&self) -> Result<Value, crate::RpcError>;

    /// Check if variable exists
    async fn has_variable(&self, name: String) -> Result<Value, crate::RpcError>;

    /// Get all variable names
    async fn list_variables(&self) -> Result<Value, crate::RpcError>;

    /// Regular RPC method calls (fallback)
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, crate::RpcError>;
}

/// Default implementation of variable-capable RPC target
#[derive(Debug)]
pub struct DefaultVariableCapableTarget {
    variable_manager: VariableStateManager,
    delegate: Arc<dyn crate::RpcTarget>, // Delegate for non-variable methods
}

impl DefaultVariableCapableTarget {
    pub fn new(delegate: Arc<dyn crate::RpcTarget>) -> Self {
        Self {
            variable_manager: VariableStateManager::new(),
            delegate,
        }
    }

    pub fn with_variable_limits(
        delegate: Arc<dyn crate::RpcTarget>,
        max_variables: usize,
        max_name_length: usize,
    ) -> Self {
        Self {
            variable_manager: VariableStateManager::with_limits(max_variables, max_name_length),
            delegate,
        }
    }
}

#[async_trait::async_trait]
impl VariableCapableRpcTarget for DefaultVariableCapableTarget {
    async fn set_variable(&self, name: String, value: Value) -> Result<Value, crate::RpcError> {
        let result = self.variable_manager.set_variable(name, value).await
            .map_err(|e| crate::RpcError::bad_request(&e.to_string()))?;
        Ok(Value::Bool(result))
    }

    async fn get_variable(&self, name: String) -> Result<Value, crate::RpcError> {
        let value = self.variable_manager.get_variable(&name).await
            .map_err(|e| crate::RpcError::bad_request(&e.to_string()))?;
        Ok(value)
    }

    async fn clear_all_variables(&self) -> Result<Value, crate::RpcError> {
        let result = self.variable_manager.clear_all_variables().await
            .map_err(|e| crate::RpcError::bad_request(&e.to_string()))?;
        Ok(Value::Bool(result))
    }

    async fn has_variable(&self, name: String) -> Result<Value, crate::RpcError> {
        let exists = self.variable_manager.has_variable(&name).await;
        Ok(Value::Bool(exists))
    }

    async fn list_variables(&self) -> Result<Value, crate::RpcError> {
        let names = self.variable_manager.get_variable_names().await;
        let values: Vec<Value> = names.into_iter().map(Value::String).collect();
        Ok(Value::Array(values))
    }

    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, crate::RpcError> {
        match method {
            "setVariable" => {
                if args.len() != 2 {
                    return Err(crate::RpcError::bad_request("setVariable requires exactly 2 arguments (name, value)"));
                }

                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("Variable name must be a string")),
                };

                self.set_variable(name, args[1].clone()).await
            }

            "getVariable" => {
                if args.len() != 1 {
                    return Err(crate::RpcError::bad_request("getVariable requires exactly 1 argument (name)"));
                }

                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("Variable name must be a string")),
                };

                self.get_variable(name).await
            }

            "clearAllVariables" => {
                if !args.is_empty() {
                    return Err(crate::RpcError::bad_request("clearAllVariables takes no arguments"));
                }
                self.clear_all_variables().await
            }

            "hasVariable" => {
                if args.len() != 1 {
                    return Err(crate::RpcError::bad_request("hasVariable requires exactly 1 argument (name)"));
                }

                let name = match &args[0] {
                    Value::String(s) => s.clone(),
                    _ => return Err(crate::RpcError::bad_request("Variable name must be a string")),
                };

                self.has_variable(name).await
            }

            "listVariables" => {
                if !args.is_empty() {
                    return Err(crate::RpcError::bad_request("listVariables takes no arguments"));
                }
                self.list_variables().await
            }

            // Delegate other methods to the underlying target
            _ => self.delegate.call(method, args).await,
        }
    }
}

#[async_trait::async_trait]
impl crate::RpcTarget for DefaultVariableCapableTarget {
    async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, crate::RpcError> {
        VariableCapableRpcTarget::call(self, method, args).await
    }

    async fn get_property(&self, property: &str) -> Result<Value, crate::RpcError> {
        // For variables, we could support getting variables as properties
        if let Ok(value) = self.variable_manager.get_variable(property).await {
            Ok(value)
        } else {
            // Delegate to underlying target
            self.delegate.get_property(property).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Number;

    #[tokio::test]
    async fn test_basic_variable_operations() {
        let manager = VariableStateManager::new();

        // Set a variable
        let result = manager.set_variable("test".to_string(), Value::Number(Number::from(42))).await.unwrap();
        assert!(result);

        // Get the variable
        let value = manager.get_variable("test").await.unwrap();
        match value {
            Value::Number(n) => assert_eq!(n.as_i64(), Some(42)),
            _ => panic!("Expected number value"),
        }

        // Check if variable exists
        assert!(manager.has_variable("test").await);
        assert!(!manager.has_variable("nonexistent").await);

        // Variable count
        assert_eq!(manager.variable_count().await, 1);
    }

    #[tokio::test]
    async fn test_variable_validation() {
        let manager = VariableStateManager::with_limits(2, 10);

        // Test name length validation
        let long_name = "a".repeat(20);
        let result = manager.set_variable(long_name, Value::Number(Number::from(1))).await;
        assert!(result.is_err());

        // Test empty name
        let result = manager.set_variable("".to_string(), Value::Number(Number::from(1))).await;
        assert!(result.is_err());

        // Test variable limit
        manager.set_variable("var1".to_string(), Value::Number(Number::from(1))).await.unwrap();
        manager.set_variable("var2".to_string(), Value::Number(Number::from(2))).await.unwrap();

        let result = manager.set_variable("var3".to_string(), Value::Number(Number::from(3))).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complex_values() {
        let manager = VariableStateManager::new();

        // Test array storage
        let array_val = Value::Array(vec![
            Value::Number(Number::from(1)),
            Value::String("test".to_string()),
            Value::Bool(true),
        ]);
        manager.set_variable("array".to_string(), array_val).await.unwrap();

        let retrieved = manager.get_variable("array").await.unwrap();
        match retrieved {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }

        // Test object storage
        let mut obj = std::collections::HashMap::new();
        obj.insert("name".to_string(), Box::new(Value::String("Alice".to_string())));
        obj.insert("age".to_string(), Box::new(Value::Number(Number::from(30))));

        let obj_val = Value::Object(obj);
        manager.set_variable("user".to_string(), obj_val).await.unwrap();

        let retrieved = manager.get_variable("user").await.unwrap();
        match retrieved {
            Value::Object(obj) => {
                assert_eq!(obj.len(), 2);
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("age"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[tokio::test]
    async fn test_clear_operations() {
        let manager = VariableStateManager::new();

        // Set multiple variables
        manager.set_variable("var1".to_string(), Value::Number(Number::from(1))).await.unwrap();
        manager.set_variable("var2".to_string(), Value::String("test".to_string())).await.unwrap();
        manager.set_variable("var3".to_string(), Value::Bool(true)).await.unwrap();

        assert_eq!(manager.variable_count().await, 3);

        // Delete one variable
        manager.delete_variable("var2").await.unwrap();
        assert_eq!(manager.variable_count().await, 2);
        assert!(!manager.has_variable("var2").await);

        // Clear all variables
        manager.clear_all_variables().await.unwrap();
        assert_eq!(manager.variable_count().await, 0);
    }

    #[tokio::test]
    async fn test_variable_names_list() {
        let manager = VariableStateManager::new();

        // Set variables
        manager.set_variable("alpha".to_string(), Value::Number(Number::from(1))).await.unwrap();
        manager.set_variable("beta".to_string(), Value::Number(Number::from(2))).await.unwrap();
        manager.set_variable("gamma".to_string(), Value::Number(Number::from(3))).await.unwrap();

        let names = manager.get_variable_names().await;
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
        assert!(names.contains(&"gamma".to_string()));
    }

    #[tokio::test]
    async fn test_import_export_variables() {
        let manager = VariableStateManager::new();

        // Set up some variables
        manager.set_variable("var1".to_string(), Value::Number(Number::from(42))).await.unwrap();
        manager.set_variable("var2".to_string(), Value::String("hello".to_string())).await.unwrap();

        // Export variables
        let exported = manager.export_variables().await;
        assert_eq!(exported.len(), 2);

        // Create new manager and import
        let manager2 = VariableStateManager::new();
        manager2.import_variables(exported).await.unwrap();

        assert_eq!(manager2.variable_count().await, 2);

        let val = manager2.get_variable("var1").await.unwrap();
        match val {
            Value::Number(n) => assert_eq!(n.as_i64(), Some(42)),
            _ => panic!("Expected number"),
        }
    }
}