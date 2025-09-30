use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use currentspace_capnweb_core::{Plan, Op, Source, CapId, Message, CallId, Target, Outcome};
use currentspace_capnweb_client::{Recorder, params};
use serde_json::json;
use std::collections::BTreeMap;

fn bench_plan_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan_creation");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("simple_calls", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let recorder = Recorder::new();
                    let cap = recorder.capture("test", CapId::new(1));

                    let mut results = Vec::new();
                    for i in 0..size {
                        let result = cap.call("method", params![i]);
                        results.push(result);
                    }

                    let plan = recorder.build(results.last().unwrap().as_source());
                    black_box(plan)
                })
            },
        );
    }

    group.finish();
}

fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    let messages = vec![
        Message::call(
            CallId::new(1),
            Target::cap(CapId::new(42)),
            "test".to_string(),
            vec![json!("hello"), json!(42)],
        ),
        Message::result(
            CallId::new(1),
            Outcome::Success { value: json!("result") },
        ),
        Message::cap_ref(CapId::new(1)),
    ];

    for (i, message) in messages.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("serialize", i),
            message,
            |b, message| {
                b.iter(|| {
                    let serialized = serde_json::to_value(message).unwrap();
                    black_box(serialized)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("deserialize", i),
            message,
            |b, message| {
                let json_value = serde_json::to_value(message).unwrap();
                b.iter(|| {
                    let deserialized: Message = serde_json::from_value(json_value.clone()).unwrap();
                    black_box(deserialized)
                })
            },
        );
    }

    group.finish();
}

fn bench_plan_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plan_serialization");

    for complexity in [1, 10, 100].iter() {
        let mut ops = Vec::new();
        let mut fields = BTreeMap::new();

        for i in 0..*complexity {
            ops.push(Op::call(
                Source::capture(0),
                format!("method_{}", i),
                vec![Source::by_value(json!(i))],
                i as u32,
            ));

            fields.insert(
                format!("field_{}", i),
                Source::result(i as u32),
            );
        }

        ops.push(Op::object(
            fields,
            *complexity as u32,
        ));

        let plan = Plan::new(
            vec![CapId::new(1)],
            ops,
            Source::result(*complexity as u32),
        );

        group.bench_with_input(
            BenchmarkId::new("serialize", complexity),
            &plan,
            |b, plan| {
                b.iter(|| {
                    let serialized = serde_json::to_value(plan).unwrap();
                    black_box(serialized)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("deserialize", complexity),
            &plan,
            |b, plan| {
                let json_value = serde_json::to_value(plan).unwrap();
                b.iter(|| {
                    let deserialized: Plan = serde_json::from_value(json_value.clone()).unwrap();
                    black_box(deserialized)
                })
            },
        );
    }

    group.finish();
}

fn bench_recorder_api(c: &mut Criterion) {
    let mut group = c.benchmark_group("recorder_api");

    group.bench_function("macro_usage", |b| {
        b.iter(|| {
            let recorder = Recorder::new();
            let calc = recorder.capture("calculator", CapId::new(1));
            let api = recorder.capture("api", CapId::new(2));

            let sum = calc.call("add", params![5, 3]);
            let user = api.call("getUser", params![123]);
            let name = user.call("getName", vec![]);

            use currentspace_capnweb_client::record_object;
            let result = record_object!(recorder; {
                "sum" => sum,
                "userName" => name,
            });

            let plan = recorder.build(result.as_source());
            black_box(plan)
        })
    });

    group.bench_function("direct_api", |b| {
        b.iter(|| {
            let recorder = Recorder::new();
            let calc = recorder.capture("calculator", CapId::new(1));
            let api = recorder.capture("api", CapId::new(2));

            let sum = calc.call("add", vec![json!(5), json!(3)]);
            let user = api.call("getUser", vec![json!(123)]);
            let name = user.call("getName", vec![]);

            let mut fields = BTreeMap::new();
            fields.insert("sum".to_string(), sum.as_source());
            fields.insert("userName".to_string(), name.as_source());

            let result = recorder.object(fields);
            let plan = recorder.build(result.as_source());
            black_box(plan)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_plan_creation,
    bench_message_serialization,
    bench_plan_serialization,
    bench_recorder_api
);
criterion_main!(benches);