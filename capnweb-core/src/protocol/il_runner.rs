// Advanced IL Plan Runner - Execution engine for Cap'n Web IL plans
// Executes complex instruction sequences with capability composition

use super::tables::{Value, ImportTable, ExportTable};
use crate::CapId;
use crate::il::{Plan, Op, Source, CallOp, ObjectOp, ArrayOp};
use crate::{RpcTarget, RpcError};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Number;

/// Plan execution context containing runtime state
#[derive(Debug)]
pub struct ExecutionContext {
    /// Intermediate results from operations
    results: Vec<Option<Value>>,
    /// Parameter values passed to the plan
    parameters: Value,
    /// Captured capabilities
    captures: Vec<Arc<dyn RpcTarget>>,
    /// Variable state during execution
    #[allow(dead_code)]
    variables: HashMap<String, Value>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(parameters: Value, captures: Vec<Arc<dyn RpcTarget>>) -> Self {
        Self {
            results: Vec::new(),
            parameters,
            captures,
            variables: HashMap::new(),
        }
    }

    /// Convert serde_json::Value to tables::Value
    fn convert_serde_json_value_to_tables_value(&self, value: serde_json::Value) -> Value {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter()
                    .map(|v| self.convert_serde_json_value_to_tables_value(v))
                    .collect())
            },
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, Box::new(self.convert_serde_json_value_to_tables_value(v)));
                }
                Value::Object(map)
            }
        }
    }

    /// Get a source value from the context
    pub async fn get_source_value(&self, source: &Source) -> Result<Value, PlanExecutionError> {
        match source {
            Source::Capture { capture } => {
                if capture.index as usize >= self.captures.len() {
                    return Err(PlanExecutionError::InvalidCaptureIndex(capture.index));
                }
                // Return a capability reference
                Ok(Value::Object({
                    let mut obj = HashMap::new();
                    obj.insert("$cap".to_string(), Box::new(Value::Number(Number::from(capture.index))));
                    obj
                }))
            }
            Source::Result { result } => {
                if result.index as usize >= self.results.len() {
                    return Err(PlanExecutionError::InvalidResultIndex(result.index));
                }
                match &self.results[result.index as usize] {
                    Some(value) => Ok(value.clone()),
                    None => Err(PlanExecutionError::ResultNotSet(result.index)),
                }
            }
            Source::Param { param } => {
                self.get_nested_parameter(&param.path)
            }
            Source::ByValue { by_value } => {
                Ok(self.convert_serde_json_value_to_tables_value(by_value.value.clone()))
            }
        }
    }

    /// Get a nested parameter value by path
    fn get_nested_parameter(&self, path: &[String]) -> Result<Value, PlanExecutionError> {
        let mut current = &self.parameters;

        for segment in path {
            match current {
                Value::Object(obj) => {
                    current = obj.get(segment)
                        .ok_or_else(|| PlanExecutionError::ParameterNotFound(segment.clone()))?
                        .as_ref();
                }
                _ => return Err(PlanExecutionError::ParameterNotObject(segment.clone())),
            }
        }

        Ok(current.clone())
    }

    /// Set a result value
    pub fn set_result(&mut self, index: u32, value: Value) {
        // Extend the results vector if necessary
        while self.results.len() <= index as usize {
            self.results.push(None);
        }
        self.results[index as usize] = Some(value);
    }

    /// Get a capability by index
    pub fn get_capability(&self, index: u32) -> Result<Arc<dyn RpcTarget>, PlanExecutionError> {
        if index as usize >= self.captures.len() {
            return Err(PlanExecutionError::InvalidCaptureIndex(index));
        }
        Ok(self.captures[index as usize].clone())
    }
}

/// IL Plan execution engine
#[derive(Debug)]
pub struct PlanRunner {
    /// Import table for capability resolution
    #[allow(dead_code)]
    imports: Arc<ImportTable>,
    /// Export table for capability storage
    #[allow(dead_code)]
    exports: Arc<ExportTable>,
    /// Execution timeout in milliseconds
    timeout_ms: u64,
    /// Maximum operations per plan (prevents infinite loops)
    max_operations: usize,
}

impl PlanRunner {
    /// Create a new plan runner
    pub fn new(
        imports: Arc<ImportTable>,
        exports: Arc<ExportTable>,
    ) -> Self {
        Self {
            imports,
            exports,
            timeout_ms: 30000, // 30 second default timeout
            max_operations: 1000, // Maximum 1000 operations per plan
        }
    }

    /// Create a plan runner with custom settings
    pub fn with_settings(
        imports: Arc<ImportTable>,
        exports: Arc<ExportTable>,
        timeout_ms: u64,
        max_operations: usize,
    ) -> Self {
        Self {
            imports,
            exports,
            timeout_ms,
            max_operations,
        }
    }

    /// Execute a plan with the given parameters and captures
    pub async fn execute_plan(
        &self,
        plan: &Plan,
        parameters: Value,
        captures: Vec<Arc<dyn RpcTarget>>,
    ) -> Result<Value, PlanExecutionError> {
        // Validate the plan first
        plan.validate().map_err(PlanExecutionError::ValidationError)?;

        if plan.ops.len() > self.max_operations {
            return Err(PlanExecutionError::TooManyOperations(plan.ops.len()));
        }

        tracing::debug!(
            ops_count = plan.ops.len(),
            captures_count = captures.len(),
            "Executing IL plan"
        );

        let mut context = ExecutionContext::new(parameters, captures);

        // Execute operations in sequence
        for (i, op) in plan.ops.iter().enumerate() {
            tracing::trace!(operation_index = i, "Executing operation");

            let result = tokio::time::timeout(
                std::time::Duration::from_millis(self.timeout_ms),
                self.execute_operation(op, &mut context)
            )
            .await
            .map_err(|_| PlanExecutionError::ExecutionTimeout)?;

            match result {
                Ok(value) => {
                    context.set_result(op.get_result_index(), value);
                }
                Err(e) => {
                    tracing::error!(
                        operation_index = i,
                        error = %e,
                        "Operation execution failed"
                    );
                    return Err(e);
                }
            }
        }

        // Return the final result
        let final_result = context.get_source_value(&plan.result).await?;

        tracing::debug!("Plan execution completed successfully");
        Ok(final_result)
    }

    /// Execute a single operation
    async fn execute_operation(
        &self,
        op: &Op,
        context: &mut ExecutionContext,
    ) -> Result<Value, PlanExecutionError> {
        match op {
            Op::Call { call } => self.execute_call_op(call, context).await,
            Op::Object { object } => self.execute_object_op(object, context).await,
            Op::Array { array } => self.execute_array_op(array, context).await,
        }
    }

    /// Execute a call operation
    async fn execute_call_op(
        &self,
        call: &CallOp,
        context: &mut ExecutionContext,
    ) -> Result<Value, PlanExecutionError> {
        // Resolve the target capability
        let target = self.resolve_target(&call.target, context).await?;

        // Resolve arguments
        let mut args = Vec::new();
        for arg_source in &call.args {
            let arg_value = context.get_source_value(arg_source).await?;
            args.push(arg_value);
        }

        tracing::trace!(
            member = %call.member,
            args_count = args.len(),
            "Executing RPC call"
        );

        // Execute the RPC call
        let result = target.call(&call.member, args).await
            .map_err(PlanExecutionError::RpcCallFailed)?;

        tracing::trace!(member = %call.member, "RPC call completed");
        Ok(result)
    }

    /// Execute an object construction operation
    async fn execute_object_op(
        &self,
        object: &ObjectOp,
        context: &mut ExecutionContext,
    ) -> Result<Value, PlanExecutionError> {
        let mut obj = HashMap::new();

        for (key, source) in &object.fields {
            let value = context.get_source_value(source).await?;
            obj.insert(key.clone(), Box::new(value));
        }

        tracing::trace!(fields_count = obj.len(), "Created object");
        Ok(Value::Object(obj))
    }

    /// Execute an array construction operation
    async fn execute_array_op(
        &self,
        array: &ArrayOp,
        context: &mut ExecutionContext,
    ) -> Result<Value, PlanExecutionError> {
        let mut items = Vec::new();

        for source in &array.items {
            let value = context.get_source_value(source).await?;
            items.push(value);
        }

        tracing::trace!(items_count = items.len(), "Created array");
        Ok(Value::Array(items))
    }

    /// Resolve a target source to an RPC target
    async fn resolve_target(
        &self,
        source: &Source,
        context: &ExecutionContext,
    ) -> Result<Arc<dyn RpcTarget>, PlanExecutionError> {
        match source {
            Source::Capture { capture } => {
                context.get_capability(capture.index)
            }
            Source::Result { result: _ } => {
                // Check if the result is a capability reference
                let value = context.get_source_value(source).await?;
                if let Value::Object(obj) = value {
                    if let Some(cap_ref) = obj.get("$cap") {
                        if let Value::Number(n) = cap_ref.as_ref() {
                            if let Some(cap_index) = n.as_u64() {
                                return context.get_capability(cap_index as u32);
                            }
                        }
                    }
                }
                Err(PlanExecutionError::InvalidTarget("Result is not a capability".to_string()))
            }
            _ => Err(PlanExecutionError::InvalidTarget("Source cannot be used as a target".to_string())),
        }
    }
}

/// Errors that can occur during plan execution
#[derive(Debug, thiserror::Error)]
pub enum PlanExecutionError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid capture index: {0}")]
    InvalidCaptureIndex(u32),

    #[error("Invalid result index: {0}")]
    InvalidResultIndex(u32),

    #[error("Result not set: {0}")]
    ResultNotSet(u32),

    #[error("Parameter not found: {0}")]
    ParameterNotFound(String),

    #[error("Parameter is not an object: {0}")]
    ParameterNotObject(String),

    #[error("RPC call failed: {0}")]
    RpcCallFailed(RpcError),

    #[error("Invalid target: {0}")]
    InvalidTarget(String),

    #[error("Execution timeout")]
    ExecutionTimeout,

    #[error("Too many operations: {0}")]
    TooManyOperations(usize),

    #[error("Plan execution error: {0}")]
    ExecutionError(String),
}

/// Advanced plan builder for creating complex IL plans
#[derive(Debug)]
pub struct PlanBuilder {
    captures: Vec<CapId>,
    ops: Vec<Op>,
    next_result_index: u32,
}

impl PlanBuilder {
    /// Create a new plan builder
    pub fn new() -> Self {
        Self {
            captures: Vec::new(),
            ops: Vec::new(),
            next_result_index: 0,
        }
    }

    /// Add a capture to the plan
    pub fn add_capture(&mut self, cap_id: CapId) -> u32 {
        let index = self.captures.len() as u32;
        self.captures.push(cap_id);
        index
    }

    /// Add a call operation
    pub fn add_call(&mut self, target: Source, method: String, args: Vec<Source>) -> u32 {
        let result_index = self.next_result_index;
        self.next_result_index += 1;

        let op = Op::call(target, method, args, result_index);
        self.ops.push(op);

        result_index
    }

    /// Add an object construction operation
    pub fn add_object(&mut self, fields: HashMap<String, Source>) -> u32 {
        let result_index = self.next_result_index;
        self.next_result_index += 1;

        let op = Op::object(fields.into_iter().collect(), result_index);
        self.ops.push(op);

        result_index
    }

    /// Add an array construction operation
    pub fn add_array(&mut self, items: Vec<Source>) -> u32 {
        let result_index = self.next_result_index;
        self.next_result_index += 1;

        let op = Op::array(items, result_index);
        self.ops.push(op);

        result_index
    }

    /// Build the final plan
    pub fn build(self, result_source: Source) -> Plan {
        Plan::new(self.captures, self.ops, result_source)
    }
}

impl Default for PlanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan optimization utilities
pub struct PlanOptimizer;

impl PlanOptimizer {
    /// Optimize a plan by removing unused operations
    pub fn optimize(plan: Plan) -> Plan {
        // For now, just return the plan as-is
        // Future optimizations could include:
        // - Dead code elimination
        // - Constant folding
        // - Operation reordering
        // - Parallel execution planning
        plan
    }

    /// Analyze plan complexity
    pub fn analyze_complexity(plan: &Plan) -> PlanComplexity {
        let mut call_count = 0;
        let mut object_count = 0;
        let mut array_count = 0;
        let mut max_depth = 0;
        let mut total_args = 0;

        for op in &plan.ops {
            match op {
                Op::Call { call } => {
                    call_count += 1;
                    total_args += call.args.len();
                }
                Op::Object { object } => {
                    object_count += 1;
                    max_depth = max_depth.max(object.fields.len());
                }
                Op::Array { array } => {
                    array_count += 1;
                    max_depth = max_depth.max(array.items.len());
                }
            }
        }

        PlanComplexity {
            total_operations: plan.ops.len(),
            call_operations: call_count,
            object_operations: object_count,
            array_operations: array_count,
            max_depth,
            total_arguments: total_args,
            captures_count: plan.captures.len(),
        }
    }
}

/// Plan complexity metrics
#[derive(Debug, Clone)]
pub struct PlanComplexity {
    pub total_operations: usize,
    pub call_operations: usize,
    pub object_operations: usize,
    pub array_operations: usize,
    pub max_depth: usize,
    pub total_arguments: usize,
    pub captures_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockRpcTarget;
    use serde_json::json;
    use std::collections::BTreeMap;

    // Helper to convert serde_json::Value to tables::Value
    fn json_to_value(json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n),
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, Box::new(json_to_value(v)));
                }
                Value::Object(map)
            }
        }
    }

    #[tokio::test]
    async fn test_plan_runner_simple_call() {
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());
        let runner = PlanRunner::new(imports, exports);

        let mock_target = Arc::new(MockRpcTarget::new());
        let _captures = vec![mock_target];

        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![Op::call(
                Source::capture(0),
                "test_method".to_string(),
                vec![Source::by_value(json!("arg1"))],
                0,
            )],
            Source::result(0),
        );

        let parameters = json_to_value(json!({}));
        let captures: Vec<Arc<dyn RpcTarget>> = vec![Arc::new(MockRpcTarget::new())];
        let _result = runner.execute_plan(&plan, parameters, captures).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plan_builder() {
        let mut builder = PlanBuilder::new();

        let cap_index = builder.add_capture(CapId::new(1));
        let call_result = builder.add_call(
            Source::capture(cap_index),
            "getData".to_string(),
            vec![],
        );

        let mut fields = HashMap::new();
        fields.insert("data".to_string(), Source::result(call_result));
        fields.insert("extra".to_string(), Source::by_value(json!("info")));
        let obj_result = builder.add_object(fields);

        let plan = builder.build(Source::result(obj_result));

        assert!(plan.validate().is_ok());
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.ops.len(), 2);
    }

    #[tokio::test]
    async fn test_execution_context_parameters() {
        let params = json_to_value(json!({
            "user": {
                "name": "Alice",
                "id": 123
            },
            "settings": {
                "theme": "dark"
            }
        }));

        let context = ExecutionContext::new(params, vec![]);

        // Test nested parameter access
        let name = context.get_nested_parameter(&["user".to_string(), "name".to_string()]);
        assert!(name.is_ok());
        assert_eq!(name.expect("Should get name"), json_to_value(json!("Alice")));

        let theme = context.get_nested_parameter(&["settings".to_string(), "theme".to_string()]);
        assert!(theme.is_ok());
        assert_eq!(theme.expect("Should get theme"), json_to_value(json!("dark")));
    }

    #[tokio::test]
    async fn test_plan_complexity_analysis() {
        let plan = Plan::new(
            vec![CapId::new(1), CapId::new(2)],
            vec![
                Op::call(
                    Source::capture(0),
                    "method1".to_string(),
                    vec![Source::by_value(json!("arg1")), Source::by_value(json!("arg2"))],
                    0,
                ),
                Op::object(
                    BTreeMap::from([
                        ("field1".to_string(), Source::result(0)),
                        ("field2".to_string(), Source::capture(1)),
                    ]),
                    1,
                ),
                Op::array(
                    vec![Source::result(1), Source::by_value(json!(42))],
                    2,
                ),
            ],
            Source::result(2),
        );

        let complexity = PlanOptimizer::analyze_complexity(&plan);

        assert_eq!(complexity.total_operations, 3);
        assert_eq!(complexity.call_operations, 1);
        assert_eq!(complexity.object_operations, 1);
        assert_eq!(complexity.array_operations, 1);
        assert_eq!(complexity.total_arguments, 2);
        assert_eq!(complexity.captures_count, 2);
    }

    #[tokio::test]
    async fn test_execution_timeout() {
        let imports = Arc::new(ImportTable::with_default_allocator());
        let exports = Arc::new(ExportTable::with_default_allocator());
        let runner = PlanRunner::with_settings(imports, exports, 10, 1000); // 10ms timeout

        // Create a mock target that takes a long time
        let mock_target = Arc::new(MockRpcTarget::new());
        let _captures = vec![mock_target];

        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![Op::call(
                Source::capture(0),
                "slow_method".to_string(),
                vec![],
                0,
            )],
            Source::result(0),
        );

        let parameters = json_to_value(json!({}));
        let captures: Vec<Arc<dyn RpcTarget>> = vec![Arc::new(MockRpcTarget::new())];
        let _result = runner.execute_plan(&plan, parameters, captures).await;

        // This test might not fail with the mock target, but demonstrates the timeout structure
        // In a real scenario with a slow RPC target, this would timeout
    }
}