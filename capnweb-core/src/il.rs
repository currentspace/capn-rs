use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use crate::CapId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Source {
    Capture { index: u32 },
    Result { index: u32 },
    Param { path: Vec<String> },
    ByValue { value: Value },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Op {
    Call {
        target: Source,
        member: String,
        args: Vec<Source>,
        result: u32,
    },
    Object {
        fields: BTreeMap<String, Source>,
        result: u32,
    },
    Array {
        items: Vec<Source>,
        result: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub captures: Vec<CapId>,
    pub ops: Vec<Op>,
    pub result: Source,
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
            let result_index = match op {
                Op::Call { result, .. } |
                Op::Object { result, .. } |
                Op::Array { result, .. } => *result,
            };

            if !result_indices.insert(result_index) {
                return Err(format!("Duplicate result index: {}", result_index));
            }
        }

        for (i, op) in self.ops.iter().enumerate() {
            let sources = match op {
                Op::Call { target, args, .. } => {
                    let mut sources = vec![target];
                    sources.extend(args);
                    sources
                }
                Op::Object { fields, .. } => {
                    fields.values().collect()
                }
                Op::Array { items, .. } => {
                    items.iter().collect()
                }
            };

            for source in sources {
                match source {
                    Source::Result { index } => {
                        let found = self.ops[..i].iter().any(|prev_op| {
                            match prev_op {
                                Op::Call { result, .. } |
                                Op::Object { result, .. } |
                                Op::Array { result, .. } => *result == *index,
                            }
                        });

                        if !found {
                            return Err(format!("Result {} referenced before being defined", index));
                        }
                    }
                    Source::Capture { index } => {
                        if *index as usize >= self.captures.len() {
                            return Err(format!("Capture index {} out of bounds", index));
                        }
                    }
                    _ => {}
                }
            }
        }

        self.validate_source(&self.result, self.ops.len())?;

        Ok(())
    }

    fn validate_source(&self, source: &Source, _ops_count: usize) -> Result<(), String> {
        match source {
            Source::Capture { index } => {
                if *index as usize >= self.captures.len() {
                    return Err(format!("Capture index {} out of bounds", index));
                }
            }
            Source::Result { index } => {
                let found = self.ops.iter().any(|op| {
                    match op {
                        Op::Call { result, .. } |
                        Op::Object { result, .. } |
                        Op::Array { result, .. } => *result == *index,
                    }
                });

                if !found {
                    return Err(format!("Result {} not found in ops", index));
                }
            }
            _ => {}
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
            Source::Capture { index: 0 },
            Source::Result { index: 1 },
            Source::Param { path: vec!["user".to_string(), "name".to_string()] },
            Source::ByValue { value: json!(42) },
        ];

        for source in sources {
            let json = serde_json::to_string(&source).unwrap();
            let decoded: Source = serde_json::from_str(&json).unwrap();
            assert_eq!(source, decoded);
        }
    }

    #[test]
    fn test_op_serialization() {
        let op = Op::Call {
            target: Source::Capture { index: 0 },
            member: "method".to_string(),
            args: vec![Source::ByValue { value: json!("arg") }],
            result: 0,
        };

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_plan_validation_valid() {
        let plan = Plan {
            captures: vec![CapId::new(1)],
            ops: vec![
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "method1".to_string(),
                    args: vec![],
                    result: 0,
                },
                Op::Call {
                    target: Source::Result { index: 0 },
                    member: "method2".to_string(),
                    args: vec![],
                    result: 1,
                },
            ],
            result: Source::Result { index: 1 },
        };

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_plan_validation_duplicate_result() {
        let plan = Plan {
            captures: vec![CapId::new(1)],
            ops: vec![
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "method1".to_string(),
                    args: vec![],
                    result: 0,
                },
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "method2".to_string(),
                    args: vec![],
                    result: 0, // Duplicate!
                },
            ],
            result: Source::Result { index: 0 },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_undefined_result() {
        let plan = Plan {
            captures: vec![CapId::new(1)],
            ops: vec![
                Op::Call {
                    target: Source::Result { index: 99 }, // Undefined!
                    member: "method".to_string(),
                    args: vec![],
                    result: 0,
                },
            ],
            result: Source::Result { index: 0 },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_forward_reference() {
        let plan = Plan {
            captures: vec![CapId::new(1)],
            ops: vec![
                Op::Call {
                    target: Source::Result { index: 1 }, // Forward reference!
                    member: "method1".to_string(),
                    args: vec![],
                    result: 0,
                },
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "method2".to_string(),
                    args: vec![],
                    result: 1,
                },
            ],
            result: Source::Result { index: 0 },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_capture_out_of_bounds() {
        let plan = Plan {
            captures: vec![CapId::new(1)],
            ops: vec![
                Op::Call {
                    target: Source::Capture { index: 1 }, // Out of bounds!
                    member: "method".to_string(),
                    args: vec![],
                    result: 0,
                },
            ],
            result: Source::Result { index: 0 },
        };

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_object_op() {
        let op = Op::Object {
            fields: BTreeMap::from([
                ("field1".to_string(), Source::ByValue { value: json!(123) }),
                ("field2".to_string(), Source::Result { index: 0 }),
            ]),
            result: 1,
        };

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_array_op() {
        let op = Op::Array {
            items: vec![
                Source::ByValue { value: json!(1) },
                Source::ByValue { value: json!(2) },
                Source::Result { index: 0 },
            ],
            result: 1,
        };

        let json = serde_json::to_string(&op).unwrap();
        let decoded: Op = serde_json::from_str(&json).unwrap();
        assert_eq!(op, decoded);
    }

    #[test]
    fn test_complex_plan() {
        let plan = Plan {
            captures: vec![CapId::new(1), CapId::new(2)],
            ops: vec![
                Op::Call {
                    target: Source::Capture { index: 0 },
                    member: "getData".to_string(),
                    args: vec![],
                    result: 0,
                },
                Op::Object {
                    fields: BTreeMap::from([
                        ("data".to_string(), Source::Result { index: 0 }),
                        ("extra".to_string(), Source::ByValue { value: json!("info") }),
                    ]),
                    result: 1,
                },
                Op::Array {
                    items: vec![
                        Source::Result { index: 1 },
                        Source::Capture { index: 1 },
                    ],
                    result: 2,
                },
            ],
            result: Source::Result { index: 2 },
        };

        assert!(plan.validate().is_ok());

        let json = serde_json::to_string(&plan).unwrap();
        let decoded: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, decoded);
    }
}