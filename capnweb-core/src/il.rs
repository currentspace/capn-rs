use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use crate::CapId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Source {
    Capture { capture: CaptureRef },
    Result { result: ResultRef },
    Param { param: ParamRef },
    ByValue { byValue: ValueRef },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRef {
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultRef {
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParamRef {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueRef {
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Op {
    Call { call: CallOp },
    Object { object: ObjectOp },
    Array { array: ArrayOp },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallOp {
    pub target: Source,
    pub member: String,
    pub args: Vec<Source>,
    pub result: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectOp {
    pub fields: BTreeMap<String, Source>,
    pub result: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayOp {
    pub items: Vec<Source>,
    pub result: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub captures: Vec<CapId>,
    pub ops: Vec<Op>,
    pub result: Source,
}

impl Source {
    pub fn capture(index: u32) -> Self {
        Source::Capture { capture: CaptureRef { index } }
    }

    pub fn result(index: u32) -> Self {
        Source::Result { result: ResultRef { index } }
    }

    pub fn param(path: Vec<String>) -> Self {
        Source::Param { param: ParamRef { path } }
    }

    pub fn by_value(value: Value) -> Self {
        Source::ByValue { byValue: ValueRef { value } }
    }

    pub fn get_capture_index(&self) -> Option<u32> {
        match self {
            Source::Capture { capture } => Some(capture.index),
            _ => None,
        }
    }

    pub fn get_result_index(&self) -> Option<u32> {
        match self {
            Source::Result { result } => Some(result.index),
            _ => None,
        }
    }
}

impl Op {
    pub fn call(target: Source, member: String, args: Vec<Source>, result: u32) -> Self {
        Op::Call {
            call: CallOp {
                target,
                member,
                args,
                result,
            },
        }
    }

    pub fn object(fields: BTreeMap<String, Source>, result: u32) -> Self {
        Op::Object {
            object: ObjectOp { fields, result },
        }
    }

    pub fn array(items: Vec<Source>, result: u32) -> Self {
        Op::Array {
            array: ArrayOp { items, result },
        }
    }

    pub fn get_result_index(&self) -> u32 {
        match self {
            Op::Call { call } => call.result,
            Op::Object { object } => object.result,
            Op::Array { array } => array.result,
        }
    }
}

impl Plan {
    pub fn new(captures: Vec<CapId>, ops: Vec<Op>, result: Source) -> Self {
        Plan {
            captures,
            ops,
            result,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut result_indices = std::collections::HashSet::new();

        for op in &self.ops {
            let result_index = op.get_result_index();

            if !result_indices.insert(result_index) {
                return Err(format!("Duplicate result index: {}", result_index));
            }
        }

        for (i, op) in self.ops.iter().enumerate() {
            let sources = match op {
                Op::Call { call } => {
                    let mut sources = vec![&call.target];
                    sources.extend(&call.args);
                    sources
                }
                Op::Object { object } => {
                    object.fields.values().collect()
                }
                Op::Array { array } => {
                    array.items.iter().collect()
                }
            };

            for source in sources {
                if let Some(index) = source.get_result_index() {
                    let found = self.ops[..i].iter().any(|prev_op| {
                        prev_op.get_result_index() == index
                    });

                    if !found {
                        return Err(format!("Result {} referenced before being defined", index));
                    }
                }

                if let Some(index) = source.get_capture_index() {
                    if index as usize >= self.captures.len() {
                        return Err(format!("Capture index {} out of bounds", index));
                    }
                }
            }
        }

        self.validate_source(&self.result, self.ops.len())?;

        Ok(())
    }

    fn validate_source(&self, source: &Source, _ops_count: usize) -> Result<(), String> {
        if let Some(index) = source.get_capture_index() {
            if index as usize >= self.captures.len() {
                return Err(format!("Capture index {} out of bounds", index));
            }
        }

        if let Some(index) = source.get_result_index() {
            let found = self.ops.iter().any(|op| op.get_result_index() == index);

            if !found {
                return Err(format!("Result {} not found in ops", index));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_source_serialization() {
        let sources = vec![
            Source::capture(0),
            Source::result(1),
            Source::param(vec!["user".to_string(), "name".to_string()]),
            Source::by_value(json!(42)),
        ];

        for source in sources {
            let json = serde_json::to_string(&source).unwrap();
            let decoded: Source = serde_json::from_str(&json).unwrap();
            assert_eq!(source, decoded);
        }
    }

    #[test]
    fn test_op_serialization() {
        let op = Op::call(
            Source::capture(0),
            "method".to_string(),
            vec![Source::by_value(json!("arg"))],
            0,
        );

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_plan_validation_valid() {
        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::call(
                    Source::capture(0),
                    "method1".to_string(),
                    vec![],
                    0,
                ),
                Op::call(
                    Source::result(0),
                    "method2".to_string(),
                    vec![],
                    1,
                ),
            ],
            Source::result(1),
        );

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_plan_validation_duplicate_result() {
        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::call(
                    Source::capture(0),
                    "method1".to_string(),
                    vec![],
                    0,
                ),
                Op::call(
                    Source::capture(0),
                    "method2".to_string(),
                    vec![],
                    0, // Duplicate!
                ),
            ],
            Source::result(0),
        );

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_undefined_result() {
        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::call(
                    Source::result(99), // Undefined!
                    "method".to_string(),
                    vec![],
                    0,
                ),
            ],
            Source::result(0),
        );

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_forward_reference() {
        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::call(
                    Source::result(1), // Forward reference!
                    "method1".to_string(),
                    vec![],
                    0,
                ),
                Op::call(
                    Source::capture(0),
                    "method2".to_string(),
                    vec![],
                    1,
                ),
            ],
            Source::result(0),
        );

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_capture_out_of_bounds() {
        let plan = Plan::new(
            vec![CapId::new(1)],
            vec![
                Op::call(
                    Source::capture(1), // Out of bounds!
                    "method".to_string(),
                    vec![],
                    0,
                ),
            ],
            Source::result(0),
        );

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_object_op() {
        let op = Op::object(
            BTreeMap::from([
                ("field1".to_string(), Source::by_value(json!(123))),
                ("field2".to_string(), Source::result(0)),
            ]),
            1,
        );

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_array_op() {
        let op = Op::array(
            vec![
                Source::by_value(json!(1)),
                Source::by_value(json!(2)),
                Source::result(0),
            ],
            1,
        );

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_complex_plan() {
        let plan = Plan::new(
            vec![CapId::new(1), CapId::new(2)],
            vec![
                Op::call(
                    Source::capture(0),
                    "getData".to_string(),
                    vec![],
                    0,
                ),
                Op::object(
                    BTreeMap::from([
                        ("data".to_string(), Source::result(0)),
                        ("extra".to_string(), Source::by_value(json!("info"))),
                    ]),
                    1,
                ),
                Op::array(
                    vec![
                        Source::result(1),
                        Source::capture(1),
                    ],
                    2,
                ),
            ],
            Source::result(2),
        );

        assert!(plan.validate().is_ok());

        let json = serde_json::to_string(&plan).unwrap();
        let decoded: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, decoded);
    }
}