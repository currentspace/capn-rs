// Wire protocol handler for the server
// This module adds wire protocol support to the existing server

use serde_json::Value;
use capnweb_core::{
    WireMessage, WireExpression, PropertyKey, parse_wire_batch, serialize_wire_batch,
    CapId,
};
use tracing::{info, warn, error};

/// Convert WireExpression arguments to JSON Values for RPC calls
pub fn wire_expr_to_values(expr: &WireExpression) -> Vec<Value> {
    match expr {
        WireExpression::Array(items) => {
            items.iter().map(wire_expr_to_value).collect()
        }
        single => vec![wire_expr_to_value(single)]
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
        WireExpression::Object(map) => {
            Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), wire_expr_to_value(v)))
                    .collect()
            )
        }
        _ => {
            warn!("Unsupported WireExpression type: {:?}", expr);
            Value::String(format!("Unsupported: {:?}", expr))
        }
    }
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
            WireExpression::Object(
                map.into_iter()
                    .map(|(k, v)| (k, value_to_wire_expr(v)))
                    .collect()
            )
        }
    }
}