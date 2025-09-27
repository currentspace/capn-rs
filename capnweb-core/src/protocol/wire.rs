// Cap'n Web Official Wire Protocol Implementation
// Specification: https://github.com/cloudflare/capnweb/blob/main/protocol.md
//
// The protocol uses newline-delimited JSON arrays as messages.
// Each message is an array where the first element is the message type.

use serde_json::{Number, Value as JsonValue};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Wire protocol message types
#[derive(Debug, Clone, PartialEq)]
pub enum WireMessage {
    /// ["push", expression] - Push an expression for evaluation
    Push(WireExpression),

    /// ["pull", import_id] - Pull a promise to get its resolved value
    Pull(i64),

    /// ["resolve", export_id, value] - Resolve a promise with a value
    Resolve(i64, WireExpression),

    /// ["reject", export_id, error] - Reject a promise with an error
    Reject(i64, WireExpression),

    /// ["release", [import_ids...]] - Release/dispose capabilities
    Release(Vec<i64>),

    /// ["abort", error] - Abort the session with an error
    Abort(WireExpression),
}

/// Wire protocol expressions
#[derive(Debug, Clone, PartialEq)]
pub enum WireExpression {
    // Literal values
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<WireExpression>),
    Object(HashMap<String, WireExpression>),

    // Special forms (represented as arrays in the protocol)
    /// ["error", type, message, stack?]
    Error {
        error_type: String,
        message: String,
        stack: Option<String>,
    },

    /// ["import", id]
    Import(i64),

    /// ["export", id, promise?]
    Export {
        id: i64,
        is_promise: bool,
    },

    /// ["promise", id]
    Promise(i64),

    /// ["pipeline", import_id, property_path?, args?]
    Pipeline {
        import_id: i64,
        property_path: Option<Vec<PropertyKey>>,
        args: Option<Box<WireExpression>>,
    },

    /// ["call", cap_id, property_path, args]
    Call {
        cap_id: i64,
        property_path: Vec<PropertyKey>,
        args: Box<WireExpression>,
    },

    /// ["date", timestamp]
    Date(f64),

    /// ["remap", plan]
    Remap(JsonValue), // IL plan, keep as raw JSON for now

    /// ["capref", id] - Reference to a capability for marshaling
    CapRef(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    String(String),
    Number(usize),
}

impl WireMessage {
    /// Parse a wire message from a JSON array
    pub fn from_json_array(arr: &[JsonValue]) -> Result<Self, String> {
        trace!(
            "Parsing wire message from JSON array with {} elements",
            arr.len()
        );

        if arr.is_empty() {
            warn!("Attempted to parse empty message array");
            return Err("Empty message array".into());
        }

        let msg_type = arr[0].as_str().ok_or_else(|| {
            warn!("Message type is not a string: {:?}", arr[0]);
            "Message type must be a string".to_string()
        })?;

        debug!("Parsing message type: {}", msg_type);

        match msg_type {
            "push" => {
                if arr.len() != 2 {
                    warn!("push message has {} elements, expected 2", arr.len());
                    return Err("push requires exactly 2 elements".into());
                }
                trace!("Parsing push expression: {:?}", arr[1]);
                let expr = WireExpression::from_json(&arr[1])?;
                Ok(WireMessage::Push(expr))
            }

            "pull" => {
                if arr.len() != 2 {
                    warn!("pull message has {} elements, expected 2", arr.len());
                    return Err("pull requires exactly 2 elements".into());
                }
                trace!("Parsing pull with import ID: {:?}", arr[1]);
                let id = arr[1]
                    .as_i64()
                    .ok_or_else(|| "pull requires an integer import ID".to_string())?;
                Ok(WireMessage::Pull(id))
            }

            "resolve" => {
                if arr.len() != 3 {
                    return Err("resolve requires exactly 3 elements".into());
                }
                let id = arr[1]
                    .as_i64()
                    .ok_or_else(|| "resolve requires an integer export ID".to_string())?;
                let value = WireExpression::from_json(&arr[2])?;
                Ok(WireMessage::Resolve(id, value))
            }

            "reject" => {
                if arr.len() != 3 {
                    return Err("reject requires exactly 3 elements".into());
                }
                let id = arr[1]
                    .as_i64()
                    .ok_or_else(|| "reject requires an integer export ID".to_string())?;
                let error = WireExpression::from_json(&arr[2])?;
                Ok(WireMessage::Reject(id, error))
            }

            "release" => {
                if arr.len() != 2 {
                    return Err("release requires exactly 2 elements".into());
                }
                let ids = arr[1]
                    .as_array()
                    .ok_or_else(|| "release requires an array of import IDs".to_string())?
                    .iter()
                    .map(|v| {
                        v.as_i64()
                            .ok_or_else(|| "release IDs must be integers".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(WireMessage::Release(ids))
            }

            "abort" => {
                if arr.len() != 2 {
                    return Err("abort requires exactly 2 elements".into());
                }
                let error = WireExpression::from_json(&arr[1])?;
                Ok(WireMessage::Abort(error))
            }

            _ => {
                warn!("Unknown message type: {}", msg_type);
                Err(format!("Unknown message type: {}", msg_type))
            }
        }
    }

    /// Convert to JSON array for wire format
    pub fn to_json_array(&self) -> Vec<JsonValue> {
        match self {
            WireMessage::Push(expr) => {
                vec![JsonValue::String("push".into()), expr.to_json()]
            }
            WireMessage::Pull(id) => {
                vec![
                    JsonValue::String("pull".into()),
                    JsonValue::Number(Number::from(*id)),
                ]
            }
            WireMessage::Resolve(id, value) => {
                vec![
                    JsonValue::String("resolve".into()),
                    JsonValue::Number(Number::from(*id)),
                    value.to_json(),
                ]
            }
            WireMessage::Reject(id, error) => {
                vec![
                    JsonValue::String("reject".into()),
                    JsonValue::Number(Number::from(*id)),
                    error.to_json(),
                ]
            }
            WireMessage::Release(ids) => {
                vec![
                    JsonValue::String("release".into()),
                    JsonValue::Array(
                        ids.iter()
                            .map(|id| JsonValue::Number(Number::from(*id)))
                            .collect(),
                    ),
                ]
            }
            WireMessage::Abort(error) => {
                vec![JsonValue::String("abort".into()), error.to_json()]
            }
        }
    }
}

impl WireExpression {
    /// Parse an expression from JSON value
    pub fn from_json(value: &JsonValue) -> Result<Self, String> {
        trace!("Parsing expression from JSON: {:?}", value);

        match value {
            JsonValue::Null => Ok(WireExpression::Null),
            JsonValue::Bool(b) => Ok(WireExpression::Bool(*b)),
            JsonValue::Number(n) => Ok(WireExpression::Number(n.clone())),
            JsonValue::String(s) => Ok(WireExpression::String(s.clone())),

            JsonValue::Array(arr) if !arr.is_empty() => {
                // Check if it's a special form
                if let Some(JsonValue::String(type_str)) = arr.first() {
                    debug!("Parsing special form: {}", type_str);
                    match type_str.as_str() {
                        "error" => {
                            if arr.len() < 3 || arr.len() > 4 {
                                return Err("error requires 3-4 elements".into());
                            }
                            let error_type = arr[1]
                                .as_str()
                                .ok_or("error type must be string")?
                                .to_string();
                            let message = arr[2]
                                .as_str()
                                .ok_or("error message must be string")?
                                .to_string();
                            let stack = arr.get(3).and_then(|v| v.as_str()).map(|s| s.to_string());
                            Ok(WireExpression::Error {
                                error_type,
                                message,
                                stack,
                            })
                        }

                        "import" => {
                            if arr.len() != 2 {
                                return Err("import requires exactly 2 elements".into());
                            }
                            let id = arr[1].as_i64().ok_or("import ID must be integer")?;
                            Ok(WireExpression::Import(id))
                        }

                        "export" => {
                            if arr.len() < 2 || arr.len() > 3 {
                                return Err("export requires 2-3 elements".into());
                            }
                            let id = arr[1].as_i64().ok_or("export ID must be integer")?;
                            let is_promise = arr.get(2).and_then(|v| v.as_bool()).unwrap_or(false);
                            Ok(WireExpression::Export { id, is_promise })
                        }

                        "promise" => {
                            if arr.len() != 2 {
                                return Err("promise requires exactly 2 elements".into());
                            }
                            let id = arr[1].as_i64().ok_or("promise ID must be integer")?;
                            Ok(WireExpression::Promise(id))
                        }

                        "pipeline" => {
                            if arr.len() < 2 || arr.len() > 4 {
                                warn!("pipeline has {} elements, expected 2-4", arr.len());
                                return Err("pipeline requires 2-4 elements".into());
                            }
                            let import_id = arr[1]
                                .as_i64()
                                .ok_or("pipeline import ID must be integer")?;

                            trace!("Pipeline: import_id={}, elements={}", import_id, arr.len());

                            let property_path = arr
                                .get(2)
                                .and_then(|v| v.as_array())
                                .map(|path| {
                                    path.iter()
                                        .map(|key| {
                                            if let Some(s) = key.as_str() {
                                                Ok(PropertyKey::String(s.to_string()))
                                            } else if let Some(n) = key.as_u64() {
                                                Ok(PropertyKey::Number(n as usize))
                                            } else {
                                                Err("Property key must be string or number"
                                                    .to_string())
                                            }
                                        })
                                        .collect::<Result<Vec<_>, _>>()
                                })
                                .transpose()?;

                            let args = arr
                                .get(3)
                                .map(WireExpression::from_json)
                                .transpose()?
                                .map(Box::new);

                            Ok(WireExpression::Pipeline {
                                import_id,
                                property_path,
                                args,
                            })
                        }

                        "call" => {
                            if arr.len() != 4 {
                                warn!("call has {} elements, expected 4", arr.len());
                                return Err("call requires exactly 4 elements".into());
                            }
                            let cap_id = arr[1].as_i64().ok_or("call cap ID must be integer")?;

                            trace!("Call: cap_id={}, elements={}", cap_id, arr.len());

                            let property_path = arr[2]
                                .as_array()
                                .ok_or("call property path must be array")?
                                .iter()
                                .map(|key| {
                                    if let Some(s) = key.as_str() {
                                        Ok(PropertyKey::String(s.to_string()))
                                    } else if let Some(n) = key.as_u64() {
                                        Ok(PropertyKey::Number(n as usize))
                                    } else {
                                        Err("Property key must be string or number".to_string())
                                    }
                                })
                                .collect::<Result<Vec<_>, _>>()?;

                            let args = Box::new(WireExpression::from_json(&arr[3])?);

                            Ok(WireExpression::Call {
                                cap_id,
                                property_path,
                                args,
                            })
                        }

                        "date" => {
                            if arr.len() != 2 {
                                return Err("date requires exactly 2 elements".into());
                            }
                            let timestamp =
                                arr[1].as_f64().ok_or("date timestamp must be number")?;
                            Ok(WireExpression::Date(timestamp))
                        }

                        "remap" => {
                            if arr.len() != 2 {
                                return Err("remap requires exactly 2 elements".into());
                            }
                            Ok(WireExpression::Remap(arr[1].clone()))
                        }

                        "capref" => {
                            if arr.len() != 2 {
                                return Err("capref requires exactly 2 elements".into());
                            }
                            let id = arr[1].as_i64().ok_or("capref ID must be integer")?;
                            Ok(WireExpression::CapRef(id))
                        }

                        _ => {
                            // Not a special form, just a regular array
                            let items = arr
                                .iter()
                                .map(WireExpression::from_json)
                                .collect::<Result<Vec<_>, _>>()?;
                            Ok(WireExpression::Array(items))
                        }
                    }
                } else {
                    // Regular array
                    let items = arr
                        .iter()
                        .map(WireExpression::from_json)
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(WireExpression::Array(items))
                }
            }

            JsonValue::Array(_arr) => Ok(WireExpression::Array(vec![])), // Empty array

            JsonValue::Object(obj) => {
                let map = obj
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), WireExpression::from_json(v)?)))
                    .collect::<Result<HashMap<_, _>, String>>()?;
                Ok(WireExpression::Object(map))
            }
        }
    }

    /// Convert to JSON for wire format
    pub fn to_json(&self) -> JsonValue {
        match self {
            WireExpression::Null => JsonValue::Null,
            WireExpression::Bool(b) => JsonValue::Bool(*b),
            WireExpression::Number(n) => JsonValue::Number(n.clone()),
            WireExpression::String(s) => JsonValue::String(s.clone()),

            WireExpression::Array(items) => {
                JsonValue::Array(items.iter().map(|e| e.to_json()).collect())
            }

            WireExpression::Object(map) => {
                JsonValue::Object(map.iter().map(|(k, v)| (k.clone(), v.to_json())).collect())
            }

            WireExpression::Error {
                error_type,
                message,
                stack,
            } => {
                let mut arr = vec![
                    JsonValue::String("error".into()),
                    JsonValue::String(error_type.clone()),
                    JsonValue::String(message.clone()),
                ];
                if let Some(s) = stack {
                    arr.push(JsonValue::String(s.clone()));
                }
                JsonValue::Array(arr)
            }

            WireExpression::Import(id) => JsonValue::Array(vec![
                JsonValue::String("import".into()),
                JsonValue::Number(Number::from(*id)),
            ]),

            WireExpression::Export { id, is_promise } => {
                let mut arr = vec![
                    JsonValue::String("export".into()),
                    JsonValue::Number(Number::from(*id)),
                ];
                if *is_promise {
                    arr.push(JsonValue::Bool(true));
                }
                JsonValue::Array(arr)
            }

            WireExpression::Promise(id) => JsonValue::Array(vec![
                JsonValue::String("promise".into()),
                JsonValue::Number(Number::from(*id)),
            ]),

            WireExpression::Pipeline {
                import_id,
                property_path,
                args,
            } => {
                let mut arr = vec![
                    JsonValue::String("pipeline".into()),
                    JsonValue::Number(Number::from(*import_id)),
                ];

                if let Some(path) = property_path {
                    let path_json: Vec<JsonValue> = path
                        .iter()
                        .map(|key| match key {
                            PropertyKey::String(s) => JsonValue::String(s.clone()),
                            PropertyKey::Number(n) => JsonValue::Number(Number::from(*n)),
                        })
                        .collect();
                    arr.push(JsonValue::Array(path_json));

                    if let Some(a) = args {
                        arr.push(a.to_json());
                    }
                } else if let Some(a) = args {
                    // If no property path but has args, need empty array for path
                    arr.push(JsonValue::Array(vec![]));
                    arr.push(a.to_json());
                }

                JsonValue::Array(arr)
            }

            WireExpression::Date(timestamp) => JsonValue::Array(vec![
                JsonValue::String("date".into()),
                JsonValue::Number(Number::from_f64(*timestamp).unwrap_or_else(|| Number::from(0))), // Use 0 for invalid timestamps
            ]),

            WireExpression::Remap(plan) => {
                JsonValue::Array(vec![JsonValue::String("remap".into()), plan.clone()])
            }

            WireExpression::CapRef(id) => JsonValue::Array(vec![
                JsonValue::String("capref".into()),
                JsonValue::Number(Number::from(*id)),
            ]),

            WireExpression::Call {
                cap_id,
                property_path,
                args,
            } => {
                let mut arr = vec![
                    JsonValue::String("call".into()),
                    JsonValue::Number(Number::from(*cap_id)),
                ];

                let path_json: Vec<JsonValue> = property_path
                    .iter()
                    .map(|key| match key {
                        PropertyKey::String(s) => JsonValue::String(s.clone()),
                        PropertyKey::Number(n) => JsonValue::Number(Number::from(*n)),
                    })
                    .collect();
                arr.push(JsonValue::Array(path_json));
                arr.push(args.to_json());

                JsonValue::Array(arr)
            }
        }
    }
}

/// Parse a newline-delimited batch of messages
pub fn parse_wire_batch(input: &str) -> Result<Vec<WireMessage>, String> {
    debug!("Parsing wire batch, input length: {} chars", input.len());
    let mut messages = Vec::new();
    let mut line_num = 0;

    for line in input.lines() {
        line_num += 1;
        let line = line.trim();
        if line.is_empty() {
            trace!("Skipping empty line {}", line_num);
            continue;
        }

        trace!("Parsing line {}: {}", line_num, line);

        let json: JsonValue = serde_json::from_str(line).map_err(|e| {
            warn!("Failed to parse JSON on line {}: {}", line_num, e);
            format!("Invalid JSON on line {}: {}", line_num, e)
        })?;

        let arr = json.as_array().ok_or_else(|| {
            warn!("Line {} is not an array: {:?}", line_num, json);
            format!("Message on line {} must be an array", line_num)
        })?;

        let msg = WireMessage::from_json_array(arr)?;
        debug!(
            "Successfully parsed message on line {}: {:?}",
            line_num, msg
        );
        messages.push(msg);
    }

    debug!(
        "Successfully parsed {} messages from wire batch",
        messages.len()
    );
    Ok(messages)
}

/// Serialize messages to newline-delimited format
pub fn serialize_wire_batch(messages: &[WireMessage]) -> String {
    debug!("Serializing {} messages to wire format", messages.len());

    let result = messages
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            trace!("Serializing message {}: {:?}", i, msg);
            let arr = msg.to_json_array();
            serde_json::to_string(&arr).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n");

    debug!("Serialized wire batch: {} bytes", result.len());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_push_message() {
        // Test the exact format the TypeScript client sends
        let input = r#"["push",["pipeline",0,["add"],[5,3]]]"#;
        let json: JsonValue = serde_json::from_str(input).unwrap();
        let arr = json.as_array().unwrap();
        let msg = WireMessage::from_json_array(arr).unwrap();

        match msg {
            WireMessage::Push(WireExpression::Pipeline {
                import_id,
                property_path,
                args,
            }) => {
                assert_eq!(import_id, 0);
                assert_eq!(property_path, Some(vec![PropertyKey::String("add".into())]));
                assert!(args.is_some());
            }
            _ => panic!("Expected Push with Pipeline"),
        }
    }

    #[test]
    fn test_parse_pull_message() {
        let input = r#"["pull",1]"#;
        let json: JsonValue = serde_json::from_str(input).unwrap();
        let arr = json.as_array().unwrap();
        let msg = WireMessage::from_json_array(arr).unwrap();

        assert_eq!(msg, WireMessage::Pull(1));
    }

    #[test]
    fn test_parse_batch() {
        // Test exact TypeScript client batch format
        let input = r#"["push",["pipeline",0,["add"],[5,3]]]
["pull",1]"#;
        let messages = parse_wire_batch(input).unwrap();
        assert_eq!(messages.len(), 2);

        // Verify first message is push
        match &messages[0] {
            WireMessage::Push(_) => {}
            _ => panic!("Expected first message to be Push"),
        }

        // Verify second message is pull with ID 1
        match &messages[1] {
            WireMessage::Pull(id) => assert_eq!(*id, 1),
            _ => panic!("Expected second message to be Pull"),
        }
    }

    #[test]
    fn test_serialize_response() {
        // Test serializing responses like the server should
        let messages = vec![WireMessage::Resolve(
            1,
            WireExpression::Number(serde_json::Number::from(8)),
        )];

        let output = serialize_wire_batch(&messages);
        assert_eq!(output, r#"["resolve",1,8]"#);
    }

    #[test]
    fn test_full_protocol_flow() {
        // Test the full push/pull/resolve flow

        // Client sends push and pull
        let client_batch = r#"["push",["pipeline",0,["add"],[5,3]]]
["pull",1]"#;
        let client_messages = parse_wire_batch(client_batch).unwrap();
        assert_eq!(client_messages.len(), 2);

        // Server should respond with resolve using the same ID from pull
        let server_response = WireMessage::Resolve(
            1, // Use the import ID from the pull
            WireExpression::Number(serde_json::Number::from(8)),
        );

        let response_str = serialize_wire_batch(&[server_response]);
        assert_eq!(response_str, r#"["resolve",1,8]"#);
    }

    #[test]
    fn test_capref_wire_expression() {
        // Test CapRef parsing and serialization
        let input = r#"["capref",42]"#;
        let json: JsonValue = serde_json::from_str(input).unwrap();
        let expr = WireExpression::from_json(&json).unwrap();

        match expr {
            WireExpression::CapRef(id) => assert_eq!(id, 42),
            _ => panic!("Expected CapRef expression"),
        }

        // Test serialization
        let serialized = expr.to_json();
        let expected = serde_json::json!(["capref", 42]);
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_capability_passing_in_args() {
        // Test capability references passed as arguments
        let input = r#"["push",["pipeline",0,["method"],[["capref",5],"regular_arg"]]]"#;
        let json: JsonValue = serde_json::from_str(input).unwrap();
        let arr = json.as_array().unwrap();
        let msg = WireMessage::from_json_array(arr).unwrap();

        match msg {
            WireMessage::Push(WireExpression::Pipeline {
                import_id,
                property_path,
                args,
            }) => {
                assert_eq!(import_id, 0);
                assert_eq!(
                    property_path,
                    Some(vec![PropertyKey::String("method".into())])
                );

                if let Some(args_expr) = args {
                    match args_expr.as_ref() {
                        WireExpression::Array(items) => {
                            assert_eq!(items.len(), 2);
                            match &items[0] {
                                WireExpression::CapRef(id) => assert_eq!(*id, 5),
                                _ => panic!("Expected first arg to be CapRef"),
                            }
                            match &items[1] {
                                WireExpression::String(s) => assert_eq!(s, "regular_arg"),
                                _ => panic!("Expected second arg to be string"),
                            }
                        }
                        _ => panic!("Expected args to be array"),
                    }
                }
            }
            _ => panic!("Expected Push with Pipeline"),
        }
    }
}
