use crate::CapId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Extended IL expressions for complete Cap'n Web protocol support
/// Includes variable references, bindings, conditionals, and plans
#[derive(Debug, Clone, PartialEq)]
pub enum ILExpression {
    /// Direct JSON value
    Literal(Value),

    /// Variable reference: ["var", index]
    Variable { var_ref: u32 },

    /// Plan execution: ["plan", ...operations]
    Plan { plan: ILPlan },

    /// Variable binding: ["bind", value, body]
    Bind { bind: BindExpression },

    /// Conditional: ["if", condition, then_expr, else_expr]
    If { if_expr: Box<IfExpression> },

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

// Custom serialization to match Cap'n Web protocol array notation
impl Serialize for ILExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde_json::json;

        let value = match self {
            ILExpression::Literal(v) => v.clone(),
            ILExpression::Variable { var_ref } => json!(["var", var_ref]),
            ILExpression::Plan { plan } => {
                let mut arr = vec![json!("plan")];
                // Add plan operations
                arr.push(json!(plan.captures));
                arr.push(json!(plan.operations));
                arr.push(serde_json::to_value(&*plan.result).unwrap());
                Value::Array(arr)
            }
            ILExpression::Bind { bind } => {
                json!(["bind", bind.value, bind.body])
            }
            ILExpression::If { if_expr } => {
                json!([
                    "if",
                    if_expr.condition,
                    if_expr.then_branch,
                    if_expr.else_branch
                ])
            }
            ILExpression::Get { get } => {
                json!(["get", get.object, get.property])
            }
            ILExpression::Call { call } => {
                let mut arr = vec![
                    json!("call"),
                    serde_json::to_value(&*call.target).unwrap(),
                    json!(call.method),
                ];
                for arg in &call.args {
                    arr.push(serde_json::to_value(arg).unwrap());
                }
                Value::Array(arr)
            }
            ILExpression::MapOp { map } => {
                json!(["map", map.array, map.function])
            }
            ILExpression::FilterOp { filter } => {
                json!(["filter", filter.array, filter.predicate])
            }
            ILExpression::ReduceOp { reduce } => {
                json!(["reduce", reduce.array, reduce.function, reduce.initial])
            }
        };

        value.serialize(serializer)
    }
}

// Custom deserialization to parse Cap'n Web protocol array notation
impl<'de> Deserialize<'de> for ILExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        // Check if this is an array with a type marker
        if let Value::Array(arr) = &value {
            if !arr.is_empty() {
                if let Some(Value::String(type_str)) = arr.first() {
                    return match type_str.as_str() {
                        "var" => {
                            if arr.len() != 2 {
                                return Err(serde::de::Error::custom(
                                    "var requires exactly 2 elements",
                                ));
                            }
                            let index = arr[1].as_u64().ok_or_else(|| {
                                serde::de::Error::custom("var index must be a number")
                            })? as u32;
                            Ok(ILExpression::Variable { var_ref: index })
                        }
                        "bind" => {
                            if arr.len() != 3 {
                                return Err(serde::de::Error::custom(
                                    "bind requires exactly 3 elements",
                                ));
                            }
                            let value = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let body = Box::new(
                                serde_json::from_value(arr[2].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::Bind {
                                bind: BindExpression { value, body },
                            })
                        }
                        "if" => {
                            if arr.len() != 4 {
                                return Err(serde::de::Error::custom(
                                    "if requires exactly 4 elements",
                                ));
                            }
                            let condition = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let then_branch = Box::new(
                                serde_json::from_value(arr[2].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let else_branch = Box::new(
                                serde_json::from_value(arr[3].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::If {
                                if_expr: Box::new(IfExpression {
                                    condition,
                                    then_branch,
                                    else_branch,
                                }),
                            })
                        }
                        "get" => {
                            if arr.len() != 3 {
                                return Err(serde::de::Error::custom(
                                    "get requires exactly 3 elements",
                                ));
                            }
                            let object = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let property = arr[2]
                                .as_str()
                                .ok_or_else(|| {
                                    serde::de::Error::custom("get property must be a string")
                                })?
                                .to_string();
                            Ok(ILExpression::Get {
                                get: GetExpression { object, property },
                            })
                        }
                        "call" => {
                            if arr.len() < 3 {
                                return Err(serde::de::Error::custom(
                                    "call requires at least 3 elements",
                                ));
                            }
                            let target = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let method = arr[2]
                                .as_str()
                                .ok_or_else(|| {
                                    serde::de::Error::custom("call method must be a string")
                                })?
                                .to_string();
                            let args = arr[3..]
                                .iter()
                                .map(|v| serde_json::from_value(v.clone()))
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(serde::de::Error::custom)?;
                            Ok(ILExpression::Call {
                                call: CallExpression {
                                    target,
                                    method,
                                    args,
                                },
                            })
                        }
                        "map" => {
                            if arr.len() != 3 {
                                return Err(serde::de::Error::custom(
                                    "map requires exactly 3 elements",
                                ));
                            }
                            let array = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let function = Box::new(
                                serde_json::from_value(arr[2].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::MapOp {
                                map: MapExpression { array, function },
                            })
                        }
                        "filter" => {
                            if arr.len() != 3 {
                                return Err(serde::de::Error::custom(
                                    "filter requires exactly 3 elements",
                                ));
                            }
                            let array = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let predicate = Box::new(
                                serde_json::from_value(arr[2].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::FilterOp {
                                filter: FilterExpression { array, predicate },
                            })
                        }
                        "reduce" => {
                            if arr.len() != 4 {
                                return Err(serde::de::Error::custom(
                                    "reduce requires exactly 4 elements",
                                ));
                            }
                            let array = Box::new(
                                serde_json::from_value(arr[1].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let function = Box::new(
                                serde_json::from_value(arr[2].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            let initial = Box::new(
                                serde_json::from_value(arr[3].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::ReduceOp {
                                reduce: ReduceExpression {
                                    array,
                                    function,
                                    initial,
                                },
                            })
                        }
                        "plan" => {
                            if arr.len() != 4 {
                                return Err(serde::de::Error::custom(
                                    "plan requires exactly 4 elements",
                                ));
                            }
                            let captures = serde_json::from_value(arr[1].clone())
                                .map_err(serde::de::Error::custom)?;
                            let operations = serde_json::from_value(arr[2].clone())
                                .map_err(serde::de::Error::custom)?;
                            let result = Box::new(
                                serde_json::from_value(arr[3].clone())
                                    .map_err(serde::de::Error::custom)?,
                            );
                            Ok(ILExpression::Plan {
                                plan: ILPlan {
                                    captures,
                                    operations,
                                    result,
                                },
                            })
                        }
                        _ => Ok(ILExpression::Literal(value)),
                    };
                }
            }
        }

        // If not a special array form, treat as literal
        Ok(ILExpression::Literal(value))
    }
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
    pub variables: Vec<Value>,
    pub captures: Vec<CapId>,
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
        assert_eq!(json, json!(["var", 0]));

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
