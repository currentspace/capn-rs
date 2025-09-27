// Wire protocol handler for the server
// This module adds wire protocol support to the existing server

use capnweb_core::{PropertyKey, WireExpression};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Convert WireExpression arguments to JSON Values for RPC calls
pub fn wire_expr_to_values(expr: &WireExpression) -> Vec<Value> {
    match expr {
        WireExpression::Array(items) => items.iter().map(wire_expr_to_value).collect(),
        single => vec![wire_expr_to_value(single)],
    }
}

/// Convert WireExpression arguments to JSON Values with pipeline evaluation
pub fn wire_expr_to_values_with_evaluation(
    expr: &WireExpression,
    results: &HashMap<i64, WireExpression>,
) -> Vec<Value> {
    match expr {
        WireExpression::Array(items) => items
            .iter()
            .map(|e| wire_expr_to_value_with_evaluation(e, results))
            .collect(),
        single => vec![wire_expr_to_value_with_evaluation(single, results)],
    }
}

/// Convert a single WireExpression to a JSON Value
pub fn wire_expr_to_value(expr: &WireExpression) -> Value {
    match expr {
        WireExpression::Null => Value::Null,
        WireExpression::Bool(b) => Value::Bool(*b),
        WireExpression::Number(n) => Value::Number(n.clone()),
        WireExpression::String(s) => Value::String(s.clone()),
        WireExpression::Array(items) => {
            Value::Array(items.iter().map(wire_expr_to_value).collect())
        }
        WireExpression::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), wire_expr_to_value(v)))
                .collect(),
        ),
        WireExpression::CapRef(id) => {
            // Marshal capability reference as special JSON object
            // This follows TypeScript implementation pattern
            serde_json::json!({
                "_type": "capability",
                "id": id
            })
        }
        _ => {
            warn!("Unsupported WireExpression type: {:?}", expr);
            Value::String(format!("Unsupported: {:?}", expr))
        }
    }
}

/// Convert a single WireExpression to a JSON Value with pipeline evaluation
pub fn wire_expr_to_value_with_evaluation(
    expr: &WireExpression,
    results: &HashMap<i64, WireExpression>,
) -> Value {
    match expr {
        // Handle pipeline expressions by evaluating them
        WireExpression::Pipeline {
            import_id,
            property_path,
            args: _,
        } => {
            debug!(
                "Evaluating pipeline: import_id={}, path={:?}",
                import_id, property_path
            );

            // Look up the result for this import_id
            if let Some(result_expr) = results.get(import_id) {
                debug!(
                    "Found result for import_id {}: {:?}",
                    import_id, result_expr
                );

                // Navigate the property path if present
                if let Some(path) = property_path {
                    let result_value = wire_expr_to_value(result_expr);
                    navigate_property_path(&result_value, path)
                } else {
                    // No path, return the whole result
                    wire_expr_to_value(result_expr)
                }
            } else {
                warn!(
                    "No result found for import_id {} during pipeline evaluation",
                    import_id
                );
                Value::Null
            }
        }
        // For non-pipeline expressions, use the regular conversion
        other => wire_expr_to_value(other),
    }
}

/// Navigate a property path through a JSON value
fn navigate_property_path(value: &Value, path: &[PropertyKey]) -> Value {
    let mut current = value.clone();

    for key in path {
        match key {
            PropertyKey::String(s) => {
                if let Value::Object(map) = current {
                    current = map.get(s).cloned().unwrap_or(Value::Null);
                } else {
                    return Value::Null;
                }
            }
            PropertyKey::Number(n) => {
                if let Value::Array(arr) = current {
                    let index = *n; // n is already a usize
                    current = arr.get(index).cloned().unwrap_or(Value::Null);
                } else {
                    return Value::Null;
                }
            }
        }
    }

    current
}

/// Convert a JSON Value back to WireExpression
pub fn value_to_wire_expr(value: Value) -> WireExpression {
    match value {
        Value::Null => WireExpression::Null,
        Value::Bool(b) => WireExpression::Bool(b),
        Value::Number(n) => WireExpression::Number(n),
        Value::String(s) => WireExpression::String(s),
        Value::Array(items) => {
            WireExpression::Array(items.into_iter().map(value_to_wire_expr).collect())
        }
        Value::Object(map) => {
            // Check if this is a capability reference
            if let (Some(type_val), Some(id_val)) = (map.get("_type"), map.get("id")) {
                if type_val.as_str() == Some("capability") {
                    if let Some(id) = id_val.as_i64() {
                        return WireExpression::CapRef(id);
                    }
                }
            }

            // Regular object
            WireExpression::Object(
                map.into_iter()
                    .map(|(k, v)| (k, value_to_wire_expr(v)))
                    .collect(),
            )
        }
    }
}
