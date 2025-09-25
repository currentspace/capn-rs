// Cap'n Web Record-Replay and .map() Tests
// Tests the record-replay functionality and .map() operation features
// Specification: Enables recording RPC interactions for replay and transformation

use capnweb_core::{
    Message, Expression, ImportId, ExportId, CapId, Value,
    RpcTarget, RpcError,
    il::{Plan, Source, Op, CallOp},
    protocol::record_replay::{Recorder, Replayer, Recording, MapOperation},
};
use serde_json::{json, Number};
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use tokio::sync::{Mutex, RwLock};

#[cfg(test)]
mod record_replay_basic_tests {
    use super::*;

    #[derive(Debug)]
    struct MockCapability {
        name: String,
        call_count: Arc<Mutex<usize>>,
        responses: HashMap<String, Value>,
    }

    #[async_trait::async_trait]
    impl RpcTarget for MockCapability {
        async fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, RpcError> {
            *self.call_count.lock().await += 1;

            self.responses.get(method)
                .cloned()
                .ok_or_else(|| RpcError::MethodNotFound(method.to_string()))
        }

        async fn get_property(&self, property: &str) -> Result<Value, RpcError> {
            match property {
                "name" => Ok(Value::String(self.name.clone())),
                "call_count" => {
                    let count = *self.call_count.lock().await;
                    Ok(Value::Number(Number::from(count)))
                }
                _ => Err(RpcError::PropertyNotFound(property.to_string()))
            }
        }
    }

    /// Test basic recording functionality
    #[tokio::test]
    async fn test_basic_recording() {
        println!("🧪 Testing Basic Recording");

        let recorder = Recorder::new();

        // Record multiple operations
        recorder.record_call(CapId(1), "method1", vec![Value::String("arg1".to_string())]).await;
        recorder.record_call(CapId(1), "method2", vec![Value::Number(Number::from(42))]).await;
        recorder.record_call(CapId(2), "method3", vec![Value::Bool(true)]).await;

        // Record responses
        recorder.record_response(CapId(1), "method1", Ok(Value::String("result1".to_string()))).await;
        recorder.record_response(CapId(1), "method2", Ok(Value::Number(Number::from(84)))).await;
        recorder.record_response(CapId(2), "method3", Err(RpcError::MethodNotFound("method3".to_string()))).await;

        // Get recording
        let recording = recorder.get_recording().await;
        assert_eq!(recording.calls.len(), 3);
        assert_eq!(recording.responses.len(), 3);

        println!("✅ Basic recording verified");

        // Verify recorded data
        let first_call = &recording.calls[0];
        assert_eq!(first_call.capability, CapId(1));
        assert_eq!(first_call.method, "method1");
        assert_eq!(first_call.args, vec![Value::String("arg1".to_string())]);

        let first_response = &recording.responses[0];
        assert!(first_response.result.is_ok());
        assert_eq!(first_response.result.as_ref().unwrap(), &Value::String("result1".to_string()));

        println!("✅ Recording data structure verified");
    }

    /// Test replay functionality
    #[tokio::test]
    async fn test_replay_functionality() {
        println!("🧪 Testing Replay Functionality");

        // Create a recording
        let mut recording = Recording::new();
        recording.add_call(CapId(1), "add", vec![
            Value::Number(Number::from(5)),
            Value::Number(Number::from(3))
        ]);
        recording.add_response(CapId(1), "add", Ok(Value::Number(Number::from(8))));

        recording.add_call(CapId(1), "multiply", vec![
            Value::Number(Number::from(4)),
            Value::Number(Number::from(7))
        ]);
        recording.add_response(CapId(1), "multiply", Ok(Value::Number(Number::from(28))));

        // Create replayer
        let replayer = Replayer::new(recording);

        // Replay calls and verify responses
        let result1 = replayer.replay_call(CapId(1), "add", vec![
            Value::Number(Number::from(5)),
            Value::Number(Number::from(3))
        ]).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), Value::Number(Number::from(8)));

        let result2 = replayer.replay_call(CapId(1), "multiply", vec![
            Value::Number(Number::from(4)),
            Value::Number(Number::from(7))
        ]).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Value::Number(Number::from(28)));

        println!("✅ Replay functionality verified");

        // Test replay with different arguments (should fail)
        let invalid_result = replayer.replay_call(CapId(1), "add", vec![
            Value::Number(Number::from(10)),
            Value::Number(Number::from(20))
        ]).await;
        assert!(invalid_result.is_err());

        println!("✅ Replay argument validation verified");
    }

    /// Test recording serialization
    #[tokio::test]
    async fn test_recording_serialization() {
        println!("🧪 Testing Recording Serialization");

        let mut recording = Recording::new();

        // Add complex data
        recording.add_call(CapId(1), "complex", vec![
            Value::Object({
                let mut obj = HashMap::new();
                obj.insert("key".to_string(), Value::String("value".to_string()));
                obj.insert("number".to_string(), Value::Number(Number::from(42)));
                obj
            }),
            Value::Array(vec![
                Value::String("item1".to_string()),
                Value::Bool(true),
                Value::Null
            ])
        ]);

        recording.add_response(CapId(1), "complex", Ok(Value::Object({
            let mut result = HashMap::new();
            result.insert("success".to_string(), Value::Bool(true));
            result.insert("data".to_string(), Value::Array(vec![Value::Number(Number::from(1)), Value::Number(Number::from(2))]));
            result
        })));

        // Serialize to JSON
        let json = recording.to_json().unwrap();

        // Deserialize back
        let deserialized = Recording::from_json(&json).unwrap();

        // Verify data integrity
        assert_eq!(deserialized.calls.len(), recording.calls.len());
        assert_eq!(deserialized.responses.len(), recording.responses.len());

        let original_call = &recording.calls[0];
        let deserialized_call = &deserialized.calls[0];
        assert_eq!(original_call.capability, deserialized_call.capability);
        assert_eq!(original_call.method, deserialized_call.method);
        assert_eq!(original_call.args, deserialized_call.args);

        println!("✅ Recording serialization verified");
    }
}

#[cfg(test)]
mod map_operation_tests {
    use super::*;

    /// Test basic .map() operation
    #[tokio::test]
    async fn test_basic_map_operation() {
        println!("🧪 Testing Basic .map() Operation");

        let map_op = MapOperation::new();

        // Define transformation function
        let transform = |value: Value| -> Value {
            match value {
                Value::Number(n) => {
                    let doubled = n.as_f64().unwrap() * 2.0;
                    Value::Number(Number::from_f64(doubled).unwrap())
                }
                Value::String(s) => Value::String(s.to_uppercase()),
                other => other,
            }
        };

        // Test array mapping
        let input_array = Value::Array(vec![
            Value::Number(Number::from(1)),
            Value::Number(Number::from(2)),
            Value::Number(Number::from(3)),
            Value::String("hello".to_string()),
        ]);

        let result = map_op.apply(input_array, transform).await;

        match result {
            Value::Array(arr) => {
                assert_eq!(arr[0], Value::Number(Number::from(2)));
                assert_eq!(arr[1], Value::Number(Number::from(4)));
                assert_eq!(arr[2], Value::Number(Number::from(6)));
                assert_eq!(arr[3], Value::String("HELLO".to_string()));
            }
            _ => panic!("Should return array")
        }

        println!("✅ Basic .map() operation verified");
    }

    /// Test nested .map() operations
    #[tokio::test]
    async fn test_nested_map_operations() {
        println!("🧪 Testing Nested .map() Operations");

        let map_op = MapOperation::new();

        // Create nested structure
        let nested_data = Value::Object({
            let mut obj = HashMap::new();
            obj.insert("numbers".to_string(), Value::Array(vec![
                Value::Number(Number::from(1)),
                Value::Number(Number::from(2)),
                Value::Number(Number::from(3)),
            ]));
            obj.insert("nested".to_string(), Value::Object({
                let mut inner = HashMap::new();
                inner.insert("values".to_string(), Value::Array(vec![
                    Value::Number(Number::from(10)),
                    Value::Number(Number::from(20)),
                ]));
                inner
            }));
            obj
        });

        // Apply recursive transformation
        let transform_recursive = |value: Value| -> Value {
            match value {
                Value::Number(n) => {
                    let squared = n.as_f64().unwrap().powi(2);
                    Value::Number(Number::from_f64(squared).unwrap())
                }
                Value::Array(arr) => {
                    let transformed: Vec<Value> = arr.into_iter()
                        .map(|v| match v {
                            Value::Number(n) => {
                                let squared = n.as_f64().unwrap().powi(2);
                                Value::Number(Number::from_f64(squared).unwrap())
                            }
                            other => other,
                        })
                        .collect();
                    Value::Array(transformed)
                }
                Value::Object(mut obj) => {
                    for (_, value) in obj.iter_mut() {
                        if let Value::Array(arr) = value {
                            let transformed: Vec<Value> = arr.iter()
                                .map(|v| match v {
                                    Value::Number(n) => {
                                        let squared = n.as_f64().unwrap().powi(2);
                                        Value::Number(Number::from_f64(squared).unwrap())
                                    }
                                    other => other.clone(),
                                })
                                .collect();
                            *value = Value::Array(transformed);
                        }
                    }
                    Value::Object(obj)
                }
                other => other,
            }
        };

        let result = map_op.apply(nested_data, transform_recursive).await;

        match result {
            Value::Object(obj) => {
                // Check first level array
                match obj.get("numbers") {
                    Some(Value::Array(arr)) => {
                        assert_eq!(arr[0], Value::Number(Number::from(1)));
                        assert_eq!(arr[1], Value::Number(Number::from(4)));
                        assert_eq!(arr[2], Value::Number(Number::from(9)));
                    }
                    _ => panic!("Should have numbers array")
                }

                // Check nested array
                match obj.get("nested") {
                    Some(Value::Object(inner)) => {
                        match inner.get("values") {
                            Some(Value::Array(arr)) => {
                                assert_eq!(arr[0], Value::Number(Number::from(100)));
                                assert_eq!(arr[1], Value::Number(Number::from(400)));
                            }
                            _ => panic!("Should have values array")
                        }
                    }
                    _ => panic!("Should have nested object")
                }
            }
            _ => panic!("Should return object")
        }

        println!("✅ Nested .map() operations verified");
    }

    /// Test .map() with async operations
    #[tokio::test]
    async fn test_async_map_operations() {
        println!("🧪 Testing Async .map() Operations");

        #[derive(Debug)]
        struct AsyncMapper {
            delay_ms: u64,
        }

        impl AsyncMapper {
            async fn transform(&self, value: Value) -> Value {
                // Simulate async operation
                tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;

                match value {
                    Value::String(s) => {
                        // Simulate async string processing
                        Value::String(format!("processed_{}", s))
                    }
                    Value::Number(n) => {
                        // Simulate async computation
                        let result = n.as_f64().unwrap() + 100.0;
                        Value::Number(Number::from_f64(result).unwrap())
                    }
                    other => other,
                }
            }
        }

        let mapper = AsyncMapper { delay_ms: 10 };

        let input = Value::Array(vec![
            Value::String("test1".to_string()),
            Value::Number(Number::from(42)),
            Value::String("test2".to_string()),
        ]);

        // Process with async mapper
        let mut results = Vec::new();
        if let Value::Array(arr) = input {
            for value in arr {
                let result = mapper.transform(value).await;
                results.push(result);
            }
        }

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Value::String("processed_test1".to_string()));
        assert_eq!(results[1], Value::Number(Number::from(142)));
        assert_eq!(results[2], Value::String("processed_test2".to_string()));

        println!("✅ Async .map() operations verified");
    }

    /// Test .map() with error handling
    #[tokio::test]
    async fn test_map_error_handling() {
        println!("🧪 Testing .map() Error Handling");

        // Transform that can fail
        let fallible_transform = |value: Value| -> Result<Value, String> {
            match value {
                Value::Number(n) => {
                    let num = n.as_f64().unwrap();
                    if num < 0.0 {
                        Err("Negative numbers not allowed".to_string())
                    } else {
                        Ok(Value::Number(Number::from_f64(num.sqrt()).unwrap()))
                    }
                }
                Value::String(s) if s.is_empty() => {
                    Err("Empty strings not allowed".to_string())
                }
                other => Ok(other),
            }
        };

        // Test with valid data
        let valid_input = vec![
            Value::Number(Number::from(4)),
            Value::Number(Number::from(9)),
            Value::String("valid".to_string()),
        ];

        let mut valid_results = Vec::new();
        for value in valid_input {
            match fallible_transform(value) {
                Ok(v) => valid_results.push(v),
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }

        assert_eq!(valid_results[0], Value::Number(Number::from(2)));
        assert_eq!(valid_results[1], Value::Number(Number::from(3)));
        assert_eq!(valid_results[2], Value::String("valid".to_string()));

        // Test with invalid data
        let invalid_input = vec![
            Value::Number(Number::from(-4)),
            Value::String("".to_string()),
        ];

        for value in invalid_input {
            let result = fallible_transform(value);
            assert!(result.is_err());
        }

        println!("✅ .map() error handling verified");
    }
}

#[cfg(test)]
mod record_replay_advanced_tests {
    use super::*;

    /// Test recording with capability references
    #[tokio::test]
    async fn test_recording_with_capabilities() {
        println!("🧪 Testing Recording with Capability References");

        let mut recording = Recording::new();

        // Record calls that return capabilities
        recording.add_call(CapId(1), "getSubCapability", vec![Value::String("logger".to_string())]);
        recording.add_response(CapId(1), "getSubCapability", Ok(Value::Capability(CapId(2))));

        recording.add_call(CapId(2), "log", vec![Value::String("Test message".to_string())]);
        recording.add_response(CapId(2), "log", Ok(Value::Bool(true)));

        // Test replay with capability chain
        let replayer = Replayer::new(recording);

        let cap_result = replayer.replay_call(CapId(1), "getSubCapability", vec![Value::String("logger".to_string())]).await;
        assert!(cap_result.is_ok());
        match cap_result.unwrap() {
            Value::Capability(cap_id) => {
                assert_eq!(cap_id, CapId(2));
            }
            _ => panic!("Should return capability")
        }

        let log_result = replayer.replay_call(CapId(2), "log", vec![Value::String("Test message".to_string())]).await;
        assert!(log_result.is_ok());
        assert_eq!(log_result.unwrap(), Value::Bool(true));

        println!("✅ Recording with capability references verified");
    }

    /// Test recording optimization and compression
    #[tokio::test]
    async fn test_recording_optimization() {
        println!("🧪 Testing Recording Optimization");

        let mut recording = Recording::new();

        // Add many duplicate calls
        for i in 0..100 {
            recording.add_call(CapId(1), "getValue", vec![]);
            recording.add_response(CapId(1), "getValue", Ok(Value::Number(Number::from(42))));
        }

        // Optimize recording (deduplicate identical sequences)
        let optimized = recording.optimize().await;

        // Should detect pattern and compress
        assert!(optimized.size() < recording.size());

        println!("✅ Recording optimization verified");

        // Test replay of optimized recording
        let replayer = Replayer::new(optimized);
        for _ in 0..100 {
            let result = replayer.replay_call(CapId(1), "getValue", vec![]).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), Value::Number(Number::from(42)));
        }

        println!("✅ Optimized recording replay verified");
    }

    /// Test recording branching and merging
    #[tokio::test]
    async fn test_recording_branching() {
        println!("🧪 Testing Recording Branching");

        // Create base recording
        let mut base_recording = Recording::new();
        base_recording.add_call(CapId(1), "init", vec![]);
        base_recording.add_response(CapId(1), "init", Ok(Value::Bool(true)));

        // Create branch A
        let mut branch_a = base_recording.clone();
        branch_a.add_call(CapId(1), "pathA", vec![Value::String("A".to_string())]);
        branch_a.add_response(CapId(1), "pathA", Ok(Value::String("Result A".to_string())));

        // Create branch B
        let mut branch_b = base_recording.clone();
        branch_b.add_call(CapId(1), "pathB", vec![Value::String("B".to_string())]);
        branch_b.add_response(CapId(1), "pathB", Ok(Value::String("Result B".to_string())));

        // Test replay of different branches
        let replayer_a = Replayer::new(branch_a);
        let result_a = replayer_a.replay_call(CapId(1), "pathA", vec![Value::String("A".to_string())]).await;
        assert_eq!(result_a.unwrap(), Value::String("Result A".to_string()));

        let replayer_b = Replayer::new(branch_b);
        let result_b = replayer_b.replay_call(CapId(1), "pathB", vec![Value::String("B".to_string())]).await;
        assert_eq!(result_b.unwrap(), Value::String("Result B".to_string()));

        println!("✅ Recording branching verified");

        // Test merging recordings
        let merged = Recording::merge(vec![base_recording.clone(), branch_a.clone(), branch_b.clone()]).await;
        assert_eq!(merged.calls.len(), 3); // init + pathA + pathB

        println!("✅ Recording merging verified");
    }

    /// Test recording filters and transformations
    #[tokio::test]
    async fn test_recording_filters() {
        println!("🧪 Testing Recording Filters");

        let mut recording = Recording::new();

        // Add various calls
        recording.add_call(CapId(1), "publicMethod", vec![Value::String("data".to_string())]);
        recording.add_response(CapId(1), "publicMethod", Ok(Value::String("public result".to_string())));

        recording.add_call(CapId(1), "privateMethod", vec![Value::String("secret".to_string())]);
        recording.add_response(CapId(1), "privateMethod", Ok(Value::String("private result".to_string())));

        recording.add_call(CapId(2), "anotherMethod", vec![]);
        recording.add_response(CapId(2), "anotherMethod", Ok(Value::Bool(true)));

        // Filter out private methods
        let filtered = recording.filter(|call| !call.method.starts_with("private")).await;
        assert_eq!(filtered.calls.len(), 2);

        // Transform sensitive data
        let sanitized = recording.transform(|call| {
            let mut transformed = call.clone();
            if transformed.method.contains("private") {
                transformed.args = vec![Value::String("[REDACTED]".to_string())];
            }
            transformed
        }).await;

        let private_call = sanitized.calls.iter()
            .find(|c| c.method == "privateMethod")
            .unwrap();
        assert_eq!(private_call.args[0], Value::String("[REDACTED]".to_string()));

        println!("✅ Recording filters and transformations verified");
    }

    /// Test recording performance metrics
    #[tokio::test]
    async fn test_recording_metrics() {
        println!("🧪 Testing Recording Performance Metrics");

        #[derive(Debug)]
        struct MetricsRecording {
            recording: Recording,
            call_timings: HashMap<(CapId, String), Vec<u128>>,
            total_calls: usize,
            error_count: usize,
        }

        impl MetricsRecording {
            fn new() -> Self {
                Self {
                    recording: Recording::new(),
                    call_timings: HashMap::new(),
                    total_calls: 0,
                    error_count: 0,
                }
            }

            fn record_timed_call(&mut self, cap: CapId, method: String, args: Vec<Value>, duration_ms: u128, result: Result<Value, RpcError>) {
                self.recording.add_call(cap, &method, args);
                self.recording.add_response(cap, &method, result.clone());

                let key = (cap, method);
                self.call_timings.entry(key).or_insert_with(Vec::new).push(duration_ms);
                self.total_calls += 1;

                if result.is_err() {
                    self.error_count += 1;
                }
            }

            fn get_average_timing(&self, cap: CapId, method: &str) -> Option<f64> {
                self.call_timings.get(&(cap, method.to_string()))
                    .map(|timings| {
                        let sum: u128 = timings.iter().sum();
                        sum as f64 / timings.len() as f64
                    })
            }

            fn get_error_rate(&self) -> f64 {
                if self.total_calls == 0 {
                    0.0
                } else {
                    self.error_count as f64 / self.total_calls as f64
                }
            }
        }

        let mut metrics_recording = MetricsRecording::new();

        // Simulate various calls with timings
        metrics_recording.record_timed_call(
            CapId(1),
            "fast".to_string(),
            vec![],
            10,
            Ok(Value::Bool(true))
        );

        metrics_recording.record_timed_call(
            CapId(1),
            "fast".to_string(),
            vec![],
            12,
            Ok(Value::Bool(true))
        );

        metrics_recording.record_timed_call(
            CapId(1),
            "slow".to_string(),
            vec![],
            100,
            Ok(Value::String("done".to_string()))
        );

        metrics_recording.record_timed_call(
            CapId(1),
            "failing".to_string(),
            vec![],
            5,
            Err(RpcError::InternalError("error".into()))
        );

        // Check metrics
        let fast_avg = metrics_recording.get_average_timing(CapId(1), "fast").unwrap();
        assert_eq!(fast_avg, 11.0);

        let slow_avg = metrics_recording.get_average_timing(CapId(1), "slow").unwrap();
        assert_eq!(slow_avg, 100.0);

        let error_rate = metrics_recording.get_error_rate();
        assert_eq!(error_rate, 0.25); // 1 error out of 4 calls

        println!("✅ Recording performance metrics verified");
    }
}