#!/bin/bash

set -e

echo "🔍 Cap'n Web Rust Code Coverage Analysis"
echo "========================================"

# Create coverage directory
mkdir -p target/coverage

# Clean previous coverage data
echo "🧹 Cleaning previous coverage data..."
rm -rf target/coverage/*
rm -rf target/tarpaulin

echo "📊 Running code coverage analysis with tarpaulin..."

# Run basic coverage
echo ""
echo "🧪 Phase 1: Unit Tests Coverage"
echo "--------------------------------"
cargo tarpaulin \
    --config tarpaulin.toml \
    --exclude-files "examples/*" "benches/*" "*/tests/*" "typescript-interop/*" \
    --out Html Xml Json \
    --output-dir target/coverage/unit \
    --timeout 300 \
    --verbose

# Run integration tests coverage
echo ""
echo "🔧 Phase 2: Integration Tests Coverage"
echo "---------------------------------------"
cargo tarpaulin \
    --config tarpaulin.toml \
    --test integration_test \
    --exclude-files "examples/*" "benches/*" "typescript-interop/*" \
    --out Html Xml Json \
    --output-dir target/coverage/integration \
    --timeout 300 \
    --verbose

# Combine coverage data
echo ""
echo "📈 Phase 3: Combined Coverage Analysis"
echo "---------------------------------------"
cargo tarpaulin \
    --config tarpaulin.toml \
    --exclude-files "examples/*" "benches/*" "typescript-interop/*" \
    --out Html Xml Json Lcov \
    --output-dir target/coverage/combined \
    --timeout 300 \
    --verbose

# Generate coverage summary
echo ""
echo "📋 Coverage Summary"
echo "==================="

# Extract coverage percentage from the XML report
if [ -f "target/coverage/combined/cobertura.xml" ]; then
    COVERAGE=$(grep -o 'line-rate="[^"]*"' target/coverage/combined/cobertura.xml | head -1 | sed 's/line-rate="//' | sed 's/"//' | awk '{printf "%.2f", $1 * 100}')
    echo "📊 Overall Line Coverage: ${COVERAGE}%"
else
    echo "⚠️  Could not extract coverage percentage"
fi

# Check for coverage files
echo ""
echo "📁 Generated Reports:"
for report_dir in "unit" "integration" "combined"; do
    if [ -d "target/coverage/$report_dir" ]; then
        echo "  📂 $report_dir coverage:"
        ls -la "target/coverage/$report_dir"/ | grep -E '\.(html|xml|json|lcov)$' | awk '{print "    📄 " $9}'
    fi
done

echo ""
echo "🌐 Coverage Reports:"
echo "  📊 HTML Report: file://$(pwd)/target/coverage/combined/tarpaulin-report.html"
echo "  📋 XML Report:  $(pwd)/target/coverage/combined/cobertura.xml"
echo "  📊 JSON Report: $(pwd)/target/coverage/combined/tarpaulin-report.json"
echo "  📊 LCOV Report: $(pwd)/target/coverage/combined/lcov.info"

echo ""
echo "🎯 Coverage Analysis Complete!"

# Check coverage threshold
THRESHOLD=85
if [ -f "target/coverage/combined/cobertura.xml" ]; then
    COVERAGE_NUM=$(echo $COVERAGE | cut -d'.' -f1)
    if [ "$COVERAGE_NUM" -ge "$THRESHOLD" ]; then
        echo "✅ Coverage $COVERAGE% meets threshold of $THRESHOLD%"
        exit 0
    else
        echo "⚠️  Coverage $COVERAGE% is below threshold of $THRESHOLD%"
        echo "💡 Consider adding more tests to improve coverage"
        exit 1
    fi
else
    echo "⚠️  Could not verify coverage threshold"
    exit 1
fi