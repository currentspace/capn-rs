use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    pub name: String,
    pub description: String,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
}

pub fn load_fixtures() -> Vec<TestFixture> {
    vec![]
}