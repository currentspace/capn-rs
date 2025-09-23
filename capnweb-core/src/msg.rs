use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ids::{CallId, CapId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Cap { cap: CapIdWrapper },
    Special { special: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapIdWrapper {
    pub id: CapId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Outcome {
    Success { value: Value },
    Error { error: crate::error::RpcError },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Call { call: CallMessage },
    Result { result: ResultMessage },
    CapRef { capRef: CapRefMessage },
    Dispose { dispose: DisposeMessage },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallMessage {
    pub id: CallId,
    pub target: Target,
    pub member: String,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultMessage {
    pub id: CallId,
    #[serde(flatten)]
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapRefMessage {
    pub id: CapId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisposeMessage {
    pub caps: Vec<CapId>,
}

impl Target {
    pub fn cap(id: CapId) -> Self {
        Target::Cap { cap: CapIdWrapper { id } }
    }

    pub fn special(name: String) -> Self {
        Target::Special { special: name }
    }
}

impl Message {
    pub fn call(id: CallId, target: Target, member: String, args: Vec<Value>) -> Self {
        Message::Call {
            call: CallMessage {
                id,
                target,
                member,
                args,
            },
        }
    }

    pub fn result(id: CallId, outcome: Outcome) -> Self {
        Message::Result {
            result: ResultMessage { id, outcome },
        }
    }

    pub fn cap_ref(id: CapId) -> Self {
        Message::CapRef {
            capRef: CapRefMessage { id },
        }
    }

    pub fn dispose(caps: Vec<CapId>) -> Self {
        Message::Dispose {
            dispose: DisposeMessage { caps },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::call(
            CallId::new(1),
            Target::cap(CapId::new(42)),
            "test".to_string(),
            vec![Value::String("arg".to_string())],
        );

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}