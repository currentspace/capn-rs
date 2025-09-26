use crate::CapId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Extended IL expressions for complete Cap'n Web protocol support
/// Includes variable references, bindings, conditionals, and plans
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ILExpression {
    /// Direct JSON value
    Literal(Value),

    /// Variable reference: ["var", index]
    #[serde(rename_all = "camelCase")]
    Variable {
        #[serde(rename = "var")]
        var_ref: u32,
    },

    /// Plan execution: ["plan", ...operations]
    Plan { plan: ILPlan },

    /// Variable binding: ["bind", value, body]
    Bind { bind: BindExpression },

    /// Conditional: ["if", condition, then_expr, else_expr]
    If {
        #[serde(rename = "if")]
        if_expr: Box<IfExpression>,
    },

    /// Property access: ["get", object, property]
    Get { get: GetExpression },

    /// Function call: ["call", target, method, ...args]
    Call { call: CallExpression },

    /// Array map operation: ["map", array, function]
    MapOp { map: MapExpression },

    /// Filter operation: ["filter", array, predicate]
    FilterOp { filter: FilterExpression },

    /// Reduce operation: ["reduce", array, function, initial]
    ReduceOp { reduce: ReduceExpression },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ILPlan {
    pub captures: Vec<CapId>,
    pub operations: Vec<ILOperation>,
    pub result: Box<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindExpression {
    pub value: Box<ILExpression>,
    pub body: Box<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfExpression {
    pub condition: Box<ILExpression>,
    pub then_branch: Box<ILExpression>,
    pub else_branch: Box<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetExpression {
    pub object: Box<ILExpression>,
    pub property: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpression {
    pub target: Box<ILExpression>,
    pub method: String,
    pub args: Vec<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapExpression {
    pub array: Box<ILExpression>,
    pub function: Box<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterExpression {
    pub array: Box<ILExpression>,
    pub predicate: Box<ILExpression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReduceExpression {
    pub array: Box<ILExpression>,
    pub function: Box<ILExpression>,
    pub initial: Box<ILExpression>,
}

/// IL operation within a plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ILOperation {
    /// Store a value in a variable slot
    Store { store: StoreOperation },

    /// Execute an expression
    Execute { execute: Box<ILExpression> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreOperation {
    pub slot: u32,
    pub value: Box<ILExpression>,
}

/// Context for IL execution with variable bindings
pub struct ILContext {
    variables: Vec<Value>,
    captures: Vec<CapId>,
}

impl ILContext {
    pub fn new(captures: Vec<CapId>) -> Self {
        Self {
            variables: Vec::new(),
            captures,
        }
    }

    pub fn with_capacity(capacity: usize, captures: Vec<CapId>) -> Self {
        Self {
            variables: Vec::with_capacity(capacity),
            captures,
        }
    }

    pub fn get_variable(&self, index: u32) -> Option<&Value> {
        self.variables.get(index as usize)
    }

    pub fn set_variable(&mut self, index: u32, value: Value) -> Result<(), ILError> {
        let idx = index as usize;
        if idx >= self.variables.len() {
            self.variables.resize_with(idx + 1, || Value::Null);
        }
        self.variables[idx] = value;
        Ok(())
    }

    pub fn push_variable(&mut self, value: Value) -> u32 {
        let index = self.variables.len() as u32;
        self.variables.push(value);
        index
    }

    pub fn get_capture(&self, index: u32) -> Option<&CapId> {
        self.captures.get(index as usize)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ILError {
    #[error("Variable not found: {0}")]
    VariableNotFound(u32),

    #[error("Capture not found: {0}")]
    CaptureNotFound(u32),

    #[error("Type error: expected {expected}, got {actual}")]
    TypeError { expected: String, actual: String },

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl ILExpression {
    /// Create a variable reference
    pub fn var(index: u32) -> Self {
        ILExpression::Variable { var_ref: index }
    }

    /// Create a literal value
    pub fn literal(value: Value) -> Self {
        ILExpression::Literal(value)
    }

    /// Create a bind expression
    pub fn bind(value: ILExpression, body: ILExpression) -> Self {
        ILExpression::Bind {
            bind: BindExpression {
                value: Box::new(value),
                body: Box::new(body),
            },
        }
    }

    /// Create an if expression
    pub fn if_expr(
        condition: ILExpression,
        then_branch: ILExpression,
        else_branch: ILExpression,
    ) -> Self {
        ILExpression::If {
            if_expr: Box::new(IfExpression {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }),
        }
    }

    /// Create a property get expression
    pub fn get(object: ILExpression, property: String) -> Self {
        ILExpression::Get {
            get: GetExpression {
                object: Box::new(object),
                property,
            },
        }
    }

    /// Create a method call expression
    pub fn call(target: ILExpression, method: String, args: Vec<ILExpression>) -> Self {
        ILExpression::Call {
            call: CallExpression {
                target: Box::new(target),
                method,
                args,
            },
        }
    }

    /// Create a map expression for array transformation
    pub fn map(array: ILExpression, function: ILExpression) -> Self {
        ILExpression::MapOp {
            map: MapExpression {
                array: Box::new(array),
                function: Box::new(function),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_expression() {
        let expr = ILExpression::var(0);
        let json = serde_json::to_value(&expr).unwrap();
        assert_eq!(json, json!({"var": 0}));

        let deserialized: ILExpression = serde_json::from_value(json).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_bind_expression() {
        let expr = ILExpression::bind(ILExpression::literal(json!(42)), ILExpression::var(0));

        let json = serde_json::to_value(&expr).unwrap();
        let deserialized: ILExpression = serde_json::from_value(json).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_if_expression() {
        let expr = ILExpression::if_expr(
            ILExpression::var(0),
            ILExpression::literal(json!("true branch")),
            ILExpression::literal(json!("false branch")),
        );

        let json = serde_json::to_value(&expr).unwrap();
        let deserialized: ILExpression = serde_json::from_value(json).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_map_expression() {
        let expr = ILExpression::map(
            ILExpression::var(0),
            ILExpression::bind(
                ILExpression::var(1),
                ILExpression::call(ILExpression::var(1), "toString".to_string(), vec![]),
            ),
        );

        let json = serde_json::to_value(&expr).unwrap();
        let deserialized: ILExpression = serde_json::from_value(json).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_il_context() {
        let mut context = ILContext::new(vec![CapId::new(1)]);

        context.set_variable(0, json!("first")).unwrap();
        context.set_variable(2, json!("third")).unwrap();

        assert_eq!(context.get_variable(0), Some(&json!("first")));
        assert_eq!(context.get_variable(1), Some(&Value::Null));
        assert_eq!(context.get_variable(2), Some(&json!("third")));

        let index = context.push_variable(json!("pushed"));
        assert_eq!(index, 3);
        assert_eq!(context.get_variable(3), Some(&json!("pushed")));
    }
}
