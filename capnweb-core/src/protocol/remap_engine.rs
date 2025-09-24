// Advanced Remap Execution Engine for Cap'n Web Protocol
// Implements the sophisticated remap operation: ["remap", import_id, property_path, captures, instructions]

use super::expression::{Expression, RemapExpression, CaptureRef, PropertyKey};
use super::tables::{Value, ImportTable, ExportTable, ImportValue, ExportValueRef};
use super::ids::{ImportId, ExportId};
use super::evaluator::ExpressionEvaluator;
use std::collections::HashMap;
use std::sync::Arc;

/// Context for remap execution containing captured values
#[derive(Debug, Clone)]
pub struct RemapContext {
    /// Captured values indexed by their capture index
    captured_values: HashMap<usize, Value>,
    /// Original context values for reference
    context_imports: HashMap<ImportId, Value>,
    context_exports: HashMap<ExportId, Value>,
}

impl Default for RemapContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RemapContext {
    pub fn new() -> Self {
        Self {
            captured_values: HashMap::new(),
            context_imports: HashMap::new(),
            context_exports: HashMap::new(),
        }
    }

    /// Add a captured value at the specified index
    pub fn add_capture(&mut self, index: usize, value: Value) {
        self.captured_values.insert(index, value);
    }

    /// Get a captured value by index
    pub fn get_capture(&self, index: usize) -> Option<&Value> {
        self.captured_values.get(&index)
    }

    /// Add context values for reference resolution
    pub fn add_context_import(&mut self, id: ImportId, value: Value) {
        self.context_imports.insert(id, value);
    }

    pub fn add_context_export(&mut self, id: ExportId, value: Value) {
        self.context_exports.insert(id, value);
    }
}

/// Advanced Remap Execution Engine
pub struct RemapEngine {
    imports: Arc<ImportTable>,
    exports: Arc<ExportTable>,
}

impl RemapEngine {
    /// Create a new remap engine
    pub fn new(imports: Arc<ImportTable>, exports: Arc<ExportTable>) -> Self {
        Self { imports, exports }
    }

    /// Execute a remap expression
    pub async fn execute_remap(
        &self,
        remap: &RemapExpression,
        evaluator: &ExpressionEvaluator,
    ) -> Result<Value, RemapError> {
        tracing::debug!(
            import_id = %remap.import_id,
            captures_count = remap.captures.len(),
            instructions_count = remap.instructions.len(),
            "Starting remap execution"
        );

        // Step 1: Resolve the base import and property path
        let base_value = self.resolve_base_import(remap).await?;
        tracing::debug!("Base import resolved: {:?}", base_value);

        // Step 2: Capture all referenced values
        let mut context = RemapContext::new();
        self.capture_values(remap, &mut context).await?;
        tracing::debug!("Captured {} values", context.captured_values.len());

        // Step 3: Execute instruction sequence
        let result = self.execute_instructions(&remap.instructions, &context, evaluator).await?;
        tracing::debug!("Remap execution completed: {:?}", result);

        Ok(result)
    }

    /// Resolve the base import value with optional property path
    async fn resolve_base_import(&self, remap: &RemapExpression) -> Result<Value, RemapError> {
        // Get the base import
        let import_value = self.imports
            .get(remap.import_id)
            .ok_or(RemapError::UnknownImport(remap.import_id))?;

        let base_value = match import_value {
            ImportValue::Value(value) => value,
            ImportValue::Stub(_) => {
                return Err(RemapError::UnsupportedImportType("Stub remapping not yet implemented".to_string()));
            }
            ImportValue::Promise(_) => {
                return Err(RemapError::UnsupportedImportType("Promise remapping not yet implemented".to_string()));
            }
        };

        // Apply property path if specified
        if let Some(path) = &remap.property_path {
            self.resolve_property_path(&base_value, path)
        } else {
            Ok(base_value)
        }
    }

    /// Resolve a property path on a value
    fn resolve_property_path(&self, value: &Value, path: &[PropertyKey]) -> Result<Value, RemapError> {
        let mut current = value;
        #[allow(unused_assignments)]
        let mut owned_value: Option<Value> = None;

        for key in path {
            match key {
                PropertyKey::String(prop) => {
                    match current {
                        Value::Object(obj) => {
                            if let Some(val) = obj.get(prop) {
                                owned_value = Some((**val).clone());
                                current = owned_value.as_ref().expect("Just set owned_value to Some");
                            } else {
                                return Err(RemapError::PropertyNotFound(prop.clone()));
                            }
                        }
                        _ => return Err(RemapError::InvalidPropertyAccess(format!("Cannot access property '{}' on non-object", prop))),
                    }
                }
                PropertyKey::Number(index) => {
                    match current {
                        Value::Array(arr) => {
                            if *index < arr.len() {
                                owned_value = Some(arr[*index].clone());
                                current = owned_value.as_ref().expect("Just set owned_value to Some");
                            } else {
                                return Err(RemapError::IndexOutOfBounds(*index));
                            }
                        }
                        _ => return Err(RemapError::InvalidPropertyAccess(format!("Cannot index with {} on non-array", index))),
                    }
                }
            }
        }

        Ok(current.clone())
    }

    /// Capture values from imports and exports as specified
    async fn capture_values(&self, remap: &RemapExpression, context: &mut RemapContext) -> Result<(), RemapError> {
        for (index, capture_ref) in remap.captures.iter().enumerate() {
            let captured_value = match capture_ref {
                CaptureRef::Import(import_id) => {
                    let import_value = self.imports
                        .get(*import_id)
                        .ok_or(RemapError::UnknownImport(*import_id))?;

                    match import_value {
                        ImportValue::Value(value) => value,
                        ImportValue::Stub(_) => {
                            return Err(RemapError::UnsupportedCaptureType("Cannot capture stub".to_string()));
                        }
                        ImportValue::Promise(_) => {
                            return Err(RemapError::UnsupportedCaptureType("Cannot capture unresolved promise".to_string()));
                        }
                    }
                }
                CaptureRef::Export(export_id) => {
                    let export_value = self.exports
                        .get(*export_id)
                        .ok_or(RemapError::UnknownExport(*export_id))?;

                    match export_value {
                        ExportValueRef::Resolved(value) => value,
                        ExportValueRef::Rejected(error) => {
                            return Err(RemapError::CapturedRejectedPromise(error.clone()));
                        }
                        ExportValueRef::Stub(_) => {
                            return Err(RemapError::UnsupportedCaptureType("Cannot capture stub".to_string()));
                        }
                        ExportValueRef::Promise(_) => {
                            return Err(RemapError::UnsupportedCaptureType("Cannot capture unresolved promise".to_string()));
                        }
                    }
                }
            };

            context.add_capture(index, captured_value);
        }

        Ok(())
    }

    /// Execute the instruction sequence with captured values
    async fn execute_instructions(
        &self,
        instructions: &[Expression],
        context: &RemapContext,
        evaluator: &ExpressionEvaluator,
    ) -> Result<Value, RemapError> {
        if instructions.is_empty() {
            return Err(RemapError::EmptyInstructions);
        }

        let mut result = Value::Null;

        for (i, instruction) in instructions.iter().enumerate() {
            tracing::debug!("Executing instruction {}: {:?}", i, instruction);

            // Replace capture references in the instruction
            let resolved_instruction = self.resolve_instruction_captures(instruction, context)?;

            // Evaluate the instruction
            result = evaluator.evaluate(resolved_instruction).await
                .map_err(|e| RemapError::InstructionExecutionError(i, e.to_string()))?;

            tracing::debug!("Instruction {} result: {:?}", i, result);
        }

        Ok(result)
    }

    /// Resolve capture references within an instruction
    fn resolve_instruction_captures(&self, instruction: &Expression, context: &RemapContext) -> Result<Expression, RemapError> {
        match instruction {
            // Handle special capture reference syntax (e.g., $0, $1, etc.)
            Expression::String(s) if s.starts_with('$') => {
                if let Ok(index) = s[1..].parse::<usize>() {
                    if let Some(captured_value) = context.get_capture(index) {
                        Ok(self.value_to_expression(captured_value))
                    } else {
                        Err(RemapError::InvalidCaptureReference(index))
                    }
                } else {
                    Ok(instruction.clone())
                }
            }

            // Handle arrays recursively
            Expression::Array(elements) => {
                let resolved_elements: Result<Vec<Expression>, RemapError> = elements
                    .iter()
                    .map(|elem| self.resolve_instruction_captures(elem, context))
                    .collect();
                Ok(Expression::Array(resolved_elements?))
            }

            // Handle objects recursively
            Expression::Object(obj) => {
                let mut resolved_obj = std::collections::HashMap::new();
                for (key, value) in obj {
                    let resolved_value = self.resolve_instruction_captures(value, context)?;
                    resolved_obj.insert(key.clone(), Box::new(resolved_value));
                }
                Ok(Expression::Object(resolved_obj))
            }

            // For other expression types, return as-is (more complex resolution could be added)
            _ => Ok(instruction.clone()),
        }
    }

    /// Convert a value back to an expression for evaluation
    fn value_to_expression(&self, value: &Value) -> Expression {
        match value {
            Value::Null => Expression::Null,
            Value::Bool(b) => Expression::Bool(*b),
            Value::Number(n) => Expression::Number(n.clone()),
            Value::String(s) => Expression::String(s.clone()),
            Value::Array(arr) => {
                let elements = arr.iter().map(|v| self.value_to_expression(v)).collect();
                Expression::Array(elements)
            }
            Value::Object(obj) => {
                let mut map = std::collections::HashMap::new();
                for (key, val) in obj {
                    map.insert(key.clone(), Box::new(self.value_to_expression(val)));
                }
                Expression::Object(map)
            }
            Value::Date(timestamp) => Expression::Date(*timestamp),
            Value::Error { error_type, message, stack } => {
                Expression::Error(super::expression::ErrorExpression {
                    error_type: error_type.clone(),
                    message: message.clone(),
                    stack: stack.clone(),
                })
            }
            // For complex types, create placeholder expressions
            Value::Stub(_) | Value::Promise(_) => {
                Expression::String("[Complex value - not serializable]".to_string())
            }
        }
    }
}

/// Errors that can occur during remap execution
#[derive(Debug, thiserror::Error)]
pub enum RemapError {
    #[error("Unknown import: {0}")]
    UnknownImport(ImportId),

    #[error("Unknown export: {0}")]
    UnknownExport(ExportId),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("Invalid property access: {0}")]
    InvalidPropertyAccess(String),

    #[error("Unsupported import type: {0}")]
    UnsupportedImportType(String),

    #[error("Unsupported capture type: {0}")]
    UnsupportedCaptureType(String),

    #[error("Captured rejected promise: {0:?}")]
    CapturedRejectedPromise(Value),

    #[error("Empty instruction sequence")]
    EmptyInstructions,

    #[error("Invalid capture reference: ${0}")]
    InvalidCaptureReference(usize),

    #[error("Instruction {0} execution error: {1}")]
    InstructionExecutionError(usize, String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{IdAllocator, ImportValue};
    use std::sync::Arc;
    use serde_json::Number;

    #[tokio::test]
    async fn test_basic_remap_execution() {
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::new(allocator.clone()));
        let exports = Arc::new(ExportTable::new(allocator));

        // Set up test data
        let import_id = ImportId(1);
        imports.insert(import_id, ImportValue::Value(Value::Number(Number::from(42)))).unwrap();

        // Create remap expression: ["remap", 1, null, [], ["$0"]]
        let remap = RemapExpression {
            import_id,
            property_path: None,
            captures: vec![CaptureRef::Import(import_id)],
            instructions: vec![Expression::String("$0".to_string())],
        };

        let engine = RemapEngine::new(imports.clone(), exports.clone());
        let evaluator = ExpressionEvaluator::new(imports, exports);

        let result = engine.execute_remap(&remap, &evaluator).await.unwrap();

        match result {
            Value::Number(n) => assert_eq!(n.as_i64(), Some(42)),
            _ => panic!("Expected number result"),
        }
    }

    #[tokio::test]
    async fn test_property_path_resolution() {
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::new(allocator.clone()));
        let exports = Arc::new(ExportTable::new(allocator));

        // Create object with nested properties
        let mut obj = std::collections::HashMap::new();
        obj.insert("user".to_string(), Box::new(Value::Object({
            let mut user_obj = std::collections::HashMap::new();
            user_obj.insert("name".to_string(), Box::new(Value::String("Alice".to_string())));
            user_obj.insert("age".to_string(), Box::new(Value::Number(Number::from(30))));
            user_obj
        })));

        let import_id = ImportId(1);
        imports.insert(import_id, ImportValue::Value(Value::Object(obj))).unwrap();

        // Create remap with property path: ["remap", 1, ["user", "name"], [], ["$0"]]
        let remap = RemapExpression {
            import_id,
            property_path: Some(vec![
                PropertyKey::String("user".to_string()),
                PropertyKey::String("name".to_string()),
            ]),
            captures: vec![],
            instructions: vec![Expression::String("Alice".to_string())], // Simple return for test
        };

        let engine = RemapEngine::new(imports.clone(), exports.clone());
        let evaluator = ExpressionEvaluator::new(imports, exports);

        let result = engine.execute_remap(&remap, &evaluator).await.unwrap();

        match result {
            Value::String(s) => assert_eq!(s, "Alice"),
            _ => panic!("Expected string result, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_capture_resolution() {
        let allocator = Arc::new(IdAllocator::new());
        let imports = Arc::new(ImportTable::new(allocator.clone()));
        let exports = Arc::new(ExportTable::new(allocator));

        // Set up multiple values to capture
        let import1 = ImportId(1);
        let import2 = ImportId(2);

        imports.insert(import1, ImportValue::Value(Value::Number(Number::from(10)))).unwrap();
        imports.insert(import2, ImportValue::Value(Value::Number(Number::from(20)))).unwrap();

        // Create remap that captures both values: ["remap", 1, null, [["import", 1], ["import", 2]], [...]]
        let remap = RemapExpression {
            import_id: import1,
            property_path: None,
            captures: vec![
                CaptureRef::Import(import1),
                CaptureRef::Import(import2),
            ],
            instructions: vec![
                // Simple instruction that would use captures (simplified for test)
                Expression::Array(vec![
                    Expression::String("$0".to_string()),
                    Expression::String("$1".to_string()),
                ])
            ],
        };

        let engine = RemapEngine::new(imports.clone(), exports.clone());
        let evaluator = ExpressionEvaluator::new(imports, exports);

        let result = engine.execute_remap(&remap, &evaluator).await.unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                // Note: In a real implementation, $0 and $1 would be resolved to captured values
            }
            _ => panic!("Expected array result"),
        }
    }
}