use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ids::{CallId, CapId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Cap(CapId),
    Special(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Outcome {
    Success { value: Value },
    Error { error: crate::error::RpcError },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Message {
    Call {
        id: CallId,
        target: Target,
        member: String,
        args: Vec<Value>,
    },
    Result {
        id: CallId,
        #[serde(flatten)]
        outcome: Outcome,
    },
    CapRef {
        id: CapId,
    },
    Dispose {
        caps: Vec<CapId>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::Call {
            id: CallId::new(1),
            target: Target::Cap(CapId::new(42)),
            member: "test".to_string(),
            args: vec![Value::String("arg".to_string())],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}