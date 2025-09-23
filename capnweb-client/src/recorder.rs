use capnweb_core::{Plan, Op, Source, CapId};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Records method calls to build an IL Plan
#[derive(Clone)]
pub struct Recorder {
    inner: Arc<Mutex<RecorderInner>>,
}

struct RecorderInner {
    captures: Vec<CapId>,
    ops: Vec<Op>,
    next_result_index: u32,
    capability_map: BTreeMap<String, u32>,
}

impl Recorder {
    /// Create a new recorder
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RecorderInner {
                captures: Vec::new(),
                ops: Vec::new(),
                next_result_index: 0,
                capability_map: BTreeMap::new(),
            })),
        }
    }

    /// Record a capability capture
    pub fn capture(&self, name: &str, cap_id: CapId) -> RecordedCapability {
        let mut inner = self.inner.lock().unwrap();
        let index = inner.captures.len() as u32;
        inner.captures.push(cap_id);
        inner.capability_map.insert(name.to_string(), index);

        RecordedCapability {
            recorder: self.clone(),
            index,
            name: name.to_string(),
        }
    }

    /// Record a method call
    pub fn call(
        &self,
        target: Source,
        method: &str,
        args: Vec<Source>,
    ) -> RecordedResult {
        let mut inner = self.inner.lock().unwrap();
        let result_index = inner.next_result_index;
        inner.next_result_index += 1;

        inner.ops.push(Op::Call {
            target,
            member: method.to_string(),
            args,
            result: result_index,
        });

        RecordedResult {
            recorder: self.clone(),
            index: result_index,
        }
    }

    /// Record object construction
    pub fn object(&self, fields: BTreeMap<String, Source>) -> RecordedResult {
        let mut inner = self.inner.lock().unwrap();
        let result_index = inner.next_result_index;
        inner.next_result_index += 1;

        inner.ops.push(Op::Object {
            fields,
            result: result_index,
        });

        RecordedResult {
            recorder: self.clone(),
            index: result_index,
        }
    }

    /// Record array construction
    pub fn array(&self, items: Vec<Source>) -> RecordedResult {
        let mut inner = self.inner.lock().unwrap();
        let result_index = inner.next_result_index;
        inner.next_result_index += 1;

        inner.ops.push(Op::Array {
            items,
            result: result_index,
        });

        RecordedResult {
            recorder: self.clone(),
            index: result_index,
        }
    }

    /// Build the final plan
    pub fn build(&self, result: Source) -> Plan {
        let inner = self.inner.lock().unwrap();
        Plan::new(
            inner.captures.clone(),
            inner.ops.clone(),
            result,
        )
    }

    /// Get a capability source by name
    pub fn cap(&self, name: &str) -> Option<Source> {
        let inner = self.inner.lock().unwrap();
        inner.capability_map.get(name).map(|&index| {
            Source::Capture { index }
        })
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a recorded capability
pub struct RecordedCapability {
    recorder: Recorder,
    index: u32,
    #[allow(dead_code)]
    name: String,
}

impl RecordedCapability {
    /// Call a method on this capability
    pub fn call(&self, method: &str, args: Vec<Source>) -> RecordedResult {
        self.recorder.call(
            Source::Capture { index: self.index },
            method,
            args,
        )
    }

    /// Get the source for this capability
    pub fn as_source(&self) -> Source {
        Source::Capture { index: self.index }
    }
}

/// Represents a recorded result
pub struct RecordedResult {
    recorder: Recorder,
    index: u32,
}

impl RecordedResult {
    /// Call a method on this result
    pub fn call(&self, method: &str, args: Vec<Source>) -> RecordedResult {
        self.recorder.call(
            Source::Result { index: self.index },
            method,
            args,
        )
    }

    /// Get the source for this result
    pub fn as_source(&self) -> Source {
        Source::Result { index: self.index }
    }

    /// Access a field of this result
    pub fn field(&self, name: &str) -> RecordedField {
        RecordedField {
            recorder: self.recorder.clone(),
            result_index: self.index,
            field_name: name.to_string(),
        }
    }
}

/// Represents a field access on a result
pub struct RecordedField {
    recorder: Recorder,
    result_index: u32,
    #[allow(dead_code)]
    field_name: String,
}

impl RecordedField {
    /// Call a method on this field
    pub fn call(&self, method: &str, args: Vec<Source>) -> RecordedResult {
        // First, get the field value
        let field_source = self.as_source();
        self.recorder.call(field_source, method, args)
    }

    /// Get the source for this field
    pub fn as_source(&self) -> Source {
        // In a real implementation, this would extract the field
        // For now, we'll use the parent result
        Source::Result { index: self.result_index }
    }
}

/// Helper to create parameter sources
pub struct Param;

impl Param {
    /// Create a parameter source from a path
    pub fn path(segments: &[&str]) -> Source {
        Source::Param {
            path: segments.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create a value source
    pub fn value(val: Value) -> Source {
        Source::ByValue { value: val }
    }
}

/// Convenience builder for recorded plans
pub struct RecordedPlan {
    recorder: Recorder,
}

impl RecordedPlan {
    /// Create a new plan recorder
    pub fn new() -> Self {
        Self {
            recorder: Recorder::new(),
        }
    }

    /// Capture a capability
    pub fn capture(&self, name: &str, cap_id: CapId) -> RecordedCapability {
        self.recorder.capture(name, cap_id)
    }

    /// Build an object
    pub fn object(&self, fields: BTreeMap<String, Source>) -> RecordedResult {
        self.recorder.object(fields)
    }

    /// Build an array
    pub fn array(&self, items: Vec<Source>) -> RecordedResult {
        self.recorder.array(items)
    }

    /// Finish building the plan
    pub fn finish(self, result: Source) -> Plan {
        self.recorder.build(result)
    }
}

impl Default for RecordedPlan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_basic() {
        let recorder = Recorder::new();

        // Capture a capability
        let cap = recorder.capture("calculator", CapId::new(1));

        // Call a method
        let result = cap.call("add", vec![
            Param::value(serde_json::json!(5)),
            Param::value(serde_json::json!(3)),
        ]);

        // Build the plan
        let plan = recorder.build(result.as_source());

        // Verify the plan structure
        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.ops.len(), 1);
    }

    #[test]
    fn test_recorded_plan_builder() {
        let plan_builder = RecordedPlan::new();

        // Capture capabilities
        let calc = plan_builder.capture("calc", CapId::new(1));

        // Perform operations
        let sum = calc.call("add", vec![
            Param::path(&["a"]),
            Param::path(&["b"]),
        ]);

        // Build final plan
        let plan = plan_builder.finish(sum.as_source());

        assert_eq!(plan.captures.len(), 1);
        assert_eq!(plan.ops.len(), 1);
    }

    #[test]
    fn test_chained_calls() {
        let recorder = Recorder::new();

        let api = recorder.capture("api", CapId::new(1));

        // Chain multiple calls
        let result = api
            .call("getUser", vec![Param::value(serde_json::json!(123))])
            .call("getName", vec![]);

        let plan = recorder.build(result.as_source());

        assert_eq!(plan.ops.len(), 2);
    }

    #[test]
    fn test_object_construction() {
        let recorder = Recorder::new();

        let api = recorder.capture("api", CapId::new(1));
        let name = api.call("getName", vec![]);
        let age = api.call("getAge", vec![]);

        let mut fields = BTreeMap::new();
        fields.insert("name".to_string(), name.as_source());
        fields.insert("age".to_string(), age.as_source());

        let obj = recorder.object(fields);
        let plan = recorder.build(obj.as_source());

        assert_eq!(plan.ops.len(), 3); // 2 calls + 1 object
    }

    #[test]
    fn test_array_construction() {
        let recorder = Recorder::new();

        let api = recorder.capture("api", CapId::new(1));

        let items = vec![
            api.call("getValue", vec![Param::value(serde_json::json!(1))]).as_source(),
            api.call("getValue", vec![Param::value(serde_json::json!(2))]).as_source(),
            api.call("getValue", vec![Param::value(serde_json::json!(3))]).as_source(),
        ];

        let arr = recorder.array(items);
        let plan = recorder.build(arr.as_source());

        assert_eq!(plan.ops.len(), 4); // 3 calls + 1 array
    }
}