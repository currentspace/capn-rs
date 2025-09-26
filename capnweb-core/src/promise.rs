use crate::ids::{CallId, CapId, PromiseId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Represents a reference to a promise or capability in arguments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    /// Reference to a capability
    CapRef { cap: CapId },
    /// Reference to a promise result
    PromiseRef { promise: PromiseId },
    /// Reference to a field within a promise result
    PromiseField { promise: PromiseId, field: String },
    /// Direct JSON value (must be last for untagged deserialization)
    Value(Value),
}

/// Extended target that can reference promises
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtendedTarget {
    /// Target a capability directly
    Cap(CapId),
    /// Target a special built-in service
    Special(String),
    /// Target the result of a promise (expecting it to be a capability)
    Promise(PromiseId),
    /// Target a field within a promise result (expecting it to be a capability)
    PromiseField { promise: PromiseId, field: String },
}

/// Tracks dependencies between promises for topological sorting
#[derive(Debug, Default)]
pub struct PromiseDependencyGraph {
    /// Maps a promise to the promises it depends on
    dependencies: HashMap<PromiseId, HashSet<PromiseId>>,
    /// Maps a promise to the promises that depend on it
    dependents: HashMap<PromiseId, HashSet<PromiseId>>,
}

impl PromiseDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency: `promise` depends on `depends_on`
    pub fn add_dependency(&mut self, promise: PromiseId, depends_on: PromiseId) {
        self.dependencies
            .entry(promise)
            .or_default()
            .insert(depends_on);

        self.dependents
            .entry(depends_on)
            .or_default()
            .insert(promise);
    }

    /// Get all direct dependencies of a promise
    pub fn dependencies_of(&self, promise: &PromiseId) -> Option<&HashSet<PromiseId>> {
        self.dependencies.get(promise)
    }

    /// Get all promises that depend on this one
    pub fn dependents_of(&self, promise: &PromiseId) -> Option<&HashSet<PromiseId>> {
        self.dependents.get(promise)
    }

    /// Perform topological sort to get execution order
    /// Returns None if there's a cycle
    pub fn topological_sort(&self) -> Option<Vec<PromiseId>> {
        let mut in_degree = HashMap::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();

        // Collect all nodes
        let mut all_nodes = HashSet::new();
        for (promise, deps) in &self.dependencies {
            all_nodes.insert(*promise);
            for dep in deps {
                all_nodes.insert(*dep);
            }
        }

        // Initialize in-degrees for all nodes
        for &node in &all_nodes {
            in_degree.insert(node, 0);
        }

        // Calculate in-degrees
        // In-degree = number of dependencies this node has
        for (promise, deps) in &self.dependencies {
            *in_degree
                .get_mut(promise)
                .expect("Promise should exist in in_degree map") = deps.len();
        }

        // Find all nodes with in-degree 0 (no dependencies)
        for (&promise, &degree) in &in_degree {
            if degree == 0 {
                queue.push(promise);
            }
        }

        // Process queue
        while let Some(promise) = queue.pop() {
            result.push(promise);

            // For each promise that depends on this one
            if let Some(dependents) = self.dependents.get(&promise) {
                for &dependent in dependents {
                    let degree = in_degree
                        .get_mut(&dependent)
                        .expect("Dependent should exist in in_degree map");
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(dependent);
                    }
                }
            }
        }

        // Check if all nodes were processed (no cycle)
        if result.len() == all_nodes.len() {
            Some(result)
        } else {
            None
        }
    }

    /// Check if adding a dependency would create a cycle
    pub fn would_create_cycle(&self, promise: PromiseId, depends_on: PromiseId) -> bool {
        // Check if depends_on can reach promise
        let mut visited = HashSet::new();
        let mut stack = vec![depends_on];

        while let Some(current) = stack.pop() {
            if current == promise {
                return true;
            }

            if visited.insert(current) {
                if let Some(deps) = self.dependencies.get(&current) {
                    stack.extend(deps.iter().copied());
                }
            }
        }

        false
    }
}

/// Represents a pending promise waiting for resolution
#[derive(Debug)]
pub struct PendingPromise {
    pub id: PromiseId,
    pub call_id: CallId,
    pub dependencies: HashSet<PromiseId>,
    pub resolved: bool,
    pub result: Option<Value>,
}

impl PendingPromise {
    pub fn new(id: PromiseId, call_id: CallId) -> Self {
        Self {
            id,
            call_id,
            dependencies: HashSet::new(),
            resolved: false,
            result: None,
        }
    }

    pub fn add_dependency(&mut self, promise: PromiseId) {
        self.dependencies.insert(promise);
    }

    pub fn resolve(&mut self, result: Value) {
        self.resolved = true;
        self.result = Some(result);
    }

    pub fn is_ready(&self, resolved_promises: &HashSet<PromiseId>) -> bool {
        self.dependencies.is_subset(resolved_promises)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_value_serialization() {
        // Test Value variant (simple values work fine)
        let arg = ArgValue::Value(serde_json::json!(42));
        let json = serde_json::to_string(&arg).unwrap();
        assert_eq!(json, "42");
        let deserialized: ArgValue = serde_json::from_str(&json).unwrap();
        assert_eq!(arg, deserialized);

        // Test CapRef variant
        let arg = ArgValue::CapRef { cap: CapId::new(1) };
        let json = serde_json::to_string(&arg).unwrap();
        assert_eq!(json, r#"{"cap":1}"#);

        // Test PromiseRef variant
        let arg = ArgValue::PromiseRef {
            promise: PromiseId::new(2),
        };
        let json = serde_json::to_string(&arg).unwrap();
        assert_eq!(json, r#"{"promise":2}"#);

        // Test PromiseField variant (ensure it has both fields)
        let arg = ArgValue::PromiseField {
            promise: PromiseId::new(3),
            field: "result".to_string(),
        };
        let json = serde_json::to_string(&arg).unwrap();
        // Check that it serializes both fields
        assert!(json.contains("\"promise\":3"));
        assert!(json.contains("\"field\":\"result\""));
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = PromiseDependencyGraph::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);
        let p3 = PromiseId::new(3);

        // p2 depends on p1, p3 depends on p2
        // This means p1 must execute before p2, and p2 before p3
        graph.add_dependency(p2, p1);
        graph.add_dependency(p3, p2);

        // Check dependencies
        assert!(graph
            .dependencies_of(&p2)
            .map(|deps| deps.contains(&p1))
            .unwrap_or(false));
        assert!(graph
            .dependents_of(&p1)
            .map(|deps| deps.contains(&p2))
            .unwrap_or(false));

        // Check topological sort
        println!("Graph dependencies: {:?}", graph.dependencies);
        println!("Graph dependents: {:?}", graph.dependents);
        let sorted = graph.topological_sort();
        assert!(
            sorted.is_some(),
            "Topological sort should succeed (no cycle)"
        );
        let sorted = sorted.unwrap();

        // The sort should include all three nodes
        assert_eq!(sorted.len(), 3);

        // p1 should come before p2
        let p1_index = sorted
            .iter()
            .position(|&p| p == p1)
            .expect("p1 should be in sorted list");
        let p2_index = sorted
            .iter()
            .position(|&p| p == p2)
            .expect("p2 should be in sorted list");
        let p3_index = sorted
            .iter()
            .position(|&p| p == p3)
            .expect("p3 should be in sorted list");

        assert!(p1_index < p2_index, "p1 should come before p2");
        assert!(p2_index < p3_index, "p2 should come before p3");
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = PromiseDependencyGraph::new();

        let p1 = PromiseId::new(1);
        let p2 = PromiseId::new(2);
        let p3 = PromiseId::new(3);

        graph.add_dependency(p2, p1);
        graph.add_dependency(p3, p2);

        // This would create a cycle
        assert!(graph.would_create_cycle(p1, p3));

        // This wouldn't create a cycle
        assert!(!graph.would_create_cycle(p3, p1));
    }

    #[test]
    fn test_pending_promise() {
        let mut promise = PendingPromise::new(PromiseId::new(1), CallId::new(1));

        promise.add_dependency(PromiseId::new(2));
        promise.add_dependency(PromiseId::new(3));

        let mut resolved = HashSet::new();
        assert!(!promise.is_ready(&resolved));

        resolved.insert(PromiseId::new(2));
        assert!(!promise.is_ready(&resolved));

        resolved.insert(PromiseId::new(3));
        assert!(promise.is_ready(&resolved));

        promise.resolve(serde_json::json!({"result": true}));
        assert!(promise.resolved);
        assert!(promise.result.is_some());
    }
}
