#!/bin/bash

# Run Advanced Features Validation

echo "🚀 Building Cap'n Web with all advanced features..."
cargo build --workspace --quiet

echo "✅ Build complete"
echo ""

# Run the validation test
echo "🧪 Running Advanced Features Validation..."
echo ""

cd tests

rustc --edition 2021 validate_advanced_features.rs \
    -L ../target/debug/deps \
    --extern capnweb_core=../target/debug/libcapnweb_core.rlib \
    --extern capnweb_transport=../target/debug/libcapnweb_transport.rlib \
    --extern serde_json=../target/debug/deps/libserde_json*.rlib \
    --extern async_trait=../target/debug/deps/libasync_trait*.dylib \
    --extern tokio=../target/debug/deps/libtokio*.rlib \
    --extern tracing_subscriber=../target/debug/deps/libtracing_subscriber*.rlib \
    -o ../target/debug/validate_advanced_features 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ Validation test compiled successfully"
    echo ""
    ../target/debug/validate_advanced_features
else
    echo "❌ Failed to compile validation test"
    echo "Running cargo test instead..."
    cd ..
    cargo test --package capnweb-core --lib -- --nocapture test_resume_token 2>&1 | head -30
fi