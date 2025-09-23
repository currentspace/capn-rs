//! Procedural macros for ergonomic recorder usage

/// Macro for creating parameter sources with less boilerplate
///
/// # Example
///
/// ```rust
/// use capnweb_client::params;
/// let args = params![5, "hello", true];
/// ```
#[macro_export]
macro_rules! params {
    [$($expr:expr),* $(,)?] => {
        vec![$(
            $crate::recorder::Param::value(serde_json::json!($expr))
        ),*]
    };
}

/// Macro for object construction with natural syntax
///
/// # Example
///
/// ```rust
/// use capnweb_client::{record_object, Recorder};
/// use capnweb_core::CapId;
/// let recorder = Recorder::new();
/// let cap = recorder.capture("api", CapId::new(1));
/// let name_result = cap.call("getName", vec![]);
/// let age_result = cap.call("getAge", vec![]);
/// let obj = record_object!(recorder; {
///     "name" => name_result,
///     "age" => age_result,
/// });
/// ```
#[macro_export]
macro_rules! record_object {
    ($recorder:expr; { $($key:expr => $value:expr),* $(,)? }) => {{
        let mut fields = std::collections::BTreeMap::new();
        $(
            fields.insert($key.to_string(), $value.as_source());
        )*
        $recorder.object(fields)
    }};
}

/// Macro for array construction with natural syntax
///
/// # Example
///
/// ```rust
/// use capnweb_client::{record_array, Recorder};
/// use capnweb_core::CapId;
/// let recorder = Recorder::new();
/// let cap = recorder.capture("api", CapId::new(1));
/// let item1 = cap.call("getValue", vec![]);
/// let item2 = cap.call("getValue", vec![]);
/// let arr = record_array!(recorder; [item1, item2]);
/// ```
#[macro_export]
macro_rules! record_array {
    ($recorder:expr; [$($item:expr),* $(,)?]) => {{
        let items = vec![$($item.as_source()),*];
        $recorder.array(items)
    }};
}

/// Macro for creating recorded plans with fluent syntax
///
/// # Example
///
/// ```rust
/// use capnweb_client::{record_plan, Recorder, params};
/// use capnweb_core::CapId;
/// let cap_id = CapId::new(1);
/// let plan = record_plan! {
///     {
///         let recorder = Recorder::new();
///         let calc = recorder.capture("calculator", cap_id);
///         let sum = calc.call("add", params![5, 3]);
///         recorder.build(sum.as_source())
///     }
/// };
/// ```
#[macro_export]
macro_rules! record_plan {
    ($body:expr) => {
        $body
    };
}

#[cfg(test)]
mod tests {
    use crate::recorder::Recorder;
    use capnweb_core::CapId;

    #[test]
    fn test_params_macro() {
        let args = params![5, "hello", true];
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn test_record_object_macro() {
        let recorder = Recorder::new();
        let cap = recorder.capture("api", CapId::new(1));
        let name = cap.call("getName", vec![]);
        let age = cap.call("getAge", vec![]);

        let obj = record_object!(recorder; {
            "name" => name,
            "age" => age,
        });

        // Verify the object was created
        let plan = recorder.build(obj.as_source());
        assert_eq!(plan.ops.len(), 3); // 2 calls + 1 object
    }

    #[test]
    fn test_record_array_macro() {
        let recorder = Recorder::new();
        let cap = recorder.capture("api", CapId::new(1));
        let item1 = cap.call("getValue", params![1]);
        let item2 = cap.call("getValue", params![2]);

        let arr = record_array!(recorder; [item1, item2]);

        let plan = recorder.build(arr.as_source());
        assert_eq!(plan.ops.len(), 3); // 2 calls + 1 array
    }

    #[test]
    fn test_complex_recording() {
        let recorder = Recorder::new();

        // Simulate a complex plan
        let calc = recorder.capture("calculator", CapId::new(1));
        let api = recorder.capture("api", CapId::new(2));

        let sum = calc.call("add", params![5, 3]);
        let user = api.call("getUser", params![123]);
        let name = user.call("getName", vec![]);

        let result = record_object!(recorder; {
            "sum" => sum,
            "userName" => name,
        });

        let plan = recorder.build(result.as_source());
        assert_eq!(plan.captures.len(), 2);
        assert_eq!(plan.ops.len(), 4); // 3 calls + 1 object
    }
}