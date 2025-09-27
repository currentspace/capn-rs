#!/bin/bash

# Full Coverage Analysis Script for Cap'n Web Rust
# This script runs all tests and generates a comprehensive coverage report

echo "🔍 Cap'n Web Rust - Full Coverage Analysis"
echo "==========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Create coverage directory
mkdir -p target/coverage
mkdir -p target/coverage/reports

# Function to run tests for a specific crate
run_crate_tests() {
    local crate=$1
    echo -e "${BLUE}Testing $crate...${NC}"

    cargo test --package $crate --lib --quiet 2>&1 | \
        grep -E "test result:|running" | \
        tee target/coverage/reports/$crate-tests.txt

    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        echo -e "${GREEN}✓ $crate tests passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $crate tests failed${NC}"
        return 1
    fi
}

# Function to analyze coverage for a module
analyze_module_coverage() {
    local module_path=$1
    local module_name=$(basename $module_path .rs)

    echo -e "${CYAN}Analyzing $module_name...${NC}"

    # Count functions
    local total_fns=$(grep -c "^\s*pub\s\+\(async\s\+\)\?fn" "$module_path" 2>/dev/null || echo 0)
    local tested_fns=$(grep -c "#\[test\]" "$module_path" 2>/dev/null || echo 0)

    # Count error paths
    local error_paths=$(grep -c "Err\(\|\.map_err\|return Err\|panic!\|unreachable!\|todo!\|unimplemented!" "$module_path" 2>/dev/null || echo 0)

    # Count match arms (for exhaustiveness)
    local match_arms=$(grep -c "^\s*.*=>" "$module_path" 2>/dev/null || echo 0)

    echo "  Functions: $total_fns (Tests: $tested_fns)"
    echo "  Error paths: $error_paths"
    echo "  Match arms: $match_arms"

    # Store in report
    echo "$module_name,$total_fns,$tested_fns,$error_paths,$match_arms" >> target/coverage/module-analysis.csv
}

# Step 1: Build all crates
echo -e "${BLUE}📦 Building all crates...${NC}"
cargo build --workspace --all-features --quiet

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Step 2: Run tests for each crate
echo -e "${BLUE}🧪 Running tests by crate...${NC}"
echo ""

CRATES=("capnweb-core" "capnweb-transport" "capnweb-server" "capnweb-client")
FAILED_CRATES=()

for crate in "${CRATES[@]}"; do
    if ! run_crate_tests $crate; then
        FAILED_CRATES+=($crate)
    fi
done

echo ""

# Step 3: Analyze code coverage gaps
echo -e "${BLUE}📊 Analyzing code coverage...${NC}"
echo ""

# Initialize CSV report
echo "Module,Total Functions,Tested Functions,Error Paths,Match Arms" > target/coverage/module-analysis.csv

# Analyze key modules
MODULES=(
    "capnweb-core/src/protocol/resume_tokens.rs"
    "capnweb-core/src/protocol/nested_capabilities.rs"
    "capnweb-core/src/protocol/il_runner.rs"
    "capnweb-core/src/protocol/session.rs"
    "capnweb-core/src/protocol/pipeline.rs"
    "capnweb-core/src/protocol/remap_engine.rs"
    "capnweb-transport/src/http3.rs"
    "capnweb-transport/src/websocket.rs"
    "capnweb-transport/src/webtransport.rs"
    "capnweb-server/src/capnweb_server.rs"
)

for module in "${MODULES[@]}"; do
    if [ -f "$module" ]; then
        analyze_module_coverage "$module"
    fi
done

echo ""

# Step 4: Run coverage with llvm-cov (faster alternative to tarpaulin)
echo -e "${BLUE}🔬 Running LLVM coverage analysis...${NC}"
echo ""

# Check if llvm-cov is available
if command -v cargo-llvm-cov &> /dev/null; then
    echo "Using cargo-llvm-cov for coverage..."

    RUSTFLAGS="-C instrument-coverage" \
    LLVM_PROFILE_FILE="target/coverage/profile-%p-%m.profraw" \
    cargo test --workspace --quiet 2>/dev/null

    # Generate coverage report
    cargo llvm-cov report --workspace --output-path target/coverage/llvm-report.txt 2>/dev/null

    if [ -f "target/coverage/llvm-report.txt" ]; then
        echo -e "${GREEN}✓ LLVM coverage report generated${NC}"
    fi
else
    echo -e "${YELLOW}cargo-llvm-cov not found, using basic analysis${NC}"
fi

# Step 5: Identify uncovered code patterns
echo -e "${BLUE}🔎 Identifying uncovered code patterns...${NC}"
echo ""

python3 << 'PYTHON_SCRIPT'
import os
import re
from pathlib import Path
from collections import defaultdict

def find_uncovered_patterns():
    """Identify common patterns that likely lack coverage"""

    patterns = {
        'error_recovery': [],
        'timeout_handling': [],
        'resource_cleanup': [],
        'concurrent_code': [],
        'unsafe_code': [],
        'deprecated_code': [],
    }

    # Scan Rust files
    for rust_file in Path('.').rglob('*.rs'):
        if 'target' in str(rust_file) or 'tests' in str(rust_file):
            continue

        try:
            with open(rust_file, 'r') as f:
                content = f.read()
                lines = content.split('\n')

                for i, line in enumerate(lines, 1):
                    # Error recovery patterns
                    if re.search(r'catch_unwind|recover|retry|backoff', line):
                        patterns['error_recovery'].append((str(rust_file), i, line.strip()[:80]))

                    # Timeout handling
                    if re.search(r'timeout|deadline|Duration::from', line):
                        patterns['timeout_handling'].append((str(rust_file), i, line.strip()[:80]))

                    # Resource cleanup
                    if re.search(r'Drop|drop\(|cleanup|dispose|close\(', line):
                        patterns['resource_cleanup'].append((str(rust_file), i, line.strip()[:80]))

                    # Concurrent code
                    if re.search(r'Arc::new|Mutex|RwLock|spawn|JoinHandle|atomic', line):
                        patterns['concurrent_code'].append((str(rust_file), i, line.strip()[:80]))

                    # Unsafe code
                    if 'unsafe' in line:
                        patterns['unsafe_code'].append((str(rust_file), i, line.strip()[:80]))

                    # Deprecated code
                    if re.search(r'#\[deprecated|TODO|FIXME|XXX', line):
                        patterns['deprecated_code'].append((str(rust_file), i, line.strip()[:80]))
        except:
            pass

    # Generate report
    print("\n📋 Uncovered Pattern Analysis:")
    print("=" * 60)

    for pattern_type, occurrences in patterns.items():
        if occurrences:
            print(f"\n{pattern_type.replace('_', ' ').title()}:")
            print(f"  Found {len(occurrences)} occurrences")

            # Show top 3 examples
            for file, line, code in occurrences[:3]:
                file_short = '/'.join(file.split('/')[-2:])
                print(f"    {file_short}:{line} - {code[:50]}")

    # Save detailed report
    with open('target/coverage/uncovered-patterns.txt', 'w') as f:
        for pattern_type, occurrences in patterns.items():
            f.write(f"\n{pattern_type}:\n")
            for file, line, code in occurrences:
                f.write(f"  {file}:{line} - {code}\n")

find_uncovered_patterns()
PYTHON_SCRIPT

# Step 6: Generate summary report
echo ""
echo -e "${BLUE}📝 Generating summary report...${NC}"
echo ""

cat > target/coverage/coverage-summary.md << 'EOF'
# Cap'n Web Rust - Coverage Analysis Report

## Summary
Date: $(date)
Status: Analysis Complete

## Test Results

### By Crate
EOF

# Add test results
for crate in "${CRATES[@]}"; do
    if [[ " ${FAILED_CRATES[@]} " =~ " ${crate} " ]]; then
        echo "- ❌ $crate: Failed" >> target/coverage/coverage-summary.md
    else
        echo "- ✅ $crate: Passed" >> target/coverage/coverage-summary.md
    fi
done

# Add module analysis
echo "" >> target/coverage/coverage-summary.md
echo "## Module Coverage Analysis" >> target/coverage/coverage-summary.md
echo "" >> target/coverage/coverage-summary.md
echo '```' >> target/coverage/coverage-summary.md
cat target/coverage/module-analysis.csv >> target/coverage/coverage-summary.md
echo '```' >> target/coverage/coverage-summary.md

# Step 7: Identify specific uncovered functions
echo -e "${BLUE}🎯 Identifying uncovered functions...${NC}"
echo ""

# Find public functions without corresponding tests
find capnweb-*/src -name "*.rs" -type f | while read file; do
    # Extract public function names
    pub_fns=$(grep -E "^\s*pub\s+(async\s+)?fn\s+\w+" "$file" | \
              sed -E 's/.*fn\s+([a-z_][a-z0-9_]*).*/\1/' | \
              grep -v "^new$\|^default$\|^fmt$\|^from$\|^into$")

    for fn_name in $pub_fns; do
        # Check if there's a test for this function
        if ! grep -r "test.*$fn_name\|${fn_name}.*test" --include="*.rs" . >/dev/null 2>&1; then
            echo "  Untested: $(basename $file .rs)::$fn_name"
        fi
    done
done | head -20

# Step 8: Calculate coverage percentage (rough estimate)
echo ""
echo -e "${BLUE}📈 Coverage Estimation:${NC}"
echo ""

TOTAL_FUNCTIONS=$(find capnweb-*/src -name "*.rs" -type f -exec grep -c "^\s*pub\s\+\(async\s\+\)\?fn" {} \; | paste -sd+ | bc)
TOTAL_TESTS=$(find . -name "*.rs" -type f -exec grep -c "#\[test\]" {} \; | paste -sd+ | bc)
TOTAL_FILES=$(find capnweb-*/src -name "*.rs" -type f | wc -l)

echo "  Total public functions: $TOTAL_FUNCTIONS"
echo "  Total test functions: $TOTAL_TESTS"
echo "  Total source files: $TOTAL_FILES"

if [ $TOTAL_FUNCTIONS -gt 0 ]; then
    COVERAGE_ESTIMATE=$((TOTAL_TESTS * 100 / TOTAL_FUNCTIONS))
    echo ""
    echo -e "  ${CYAN}Estimated function coverage: ~${COVERAGE_ESTIMATE}%${NC}"
fi

# Step 9: List files with no tests
echo ""
echo -e "${BLUE}📁 Files with no test modules:${NC}"
echo ""

find capnweb-*/src -name "*.rs" -type f | while read file; do
    if ! grep -q "#\[cfg(test)\]\|#\[test\]" "$file"; then
        echo "  - $(echo $file | sed 's|.*/capnweb-|capnweb-|')"
    fi
done | head -10

# Step 10: Final summary
echo ""
echo "=" * 60
echo -e "${GREEN}✅ Coverage analysis complete!${NC}"
echo ""
echo "📊 Reports generated:"
echo "  - target/coverage/coverage-summary.md"
echo "  - target/coverage/module-analysis.csv"
echo "  - target/coverage/uncovered-patterns.txt"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}⚠️  Some crates have failing tests:${NC}"
    for crate in "${FAILED_CRATES[@]}"; do
        echo "  - $crate"
    done
fi

echo ""
echo "💡 Recommendations:"
echo "  1. Add tests for untested public functions"
echo "  2. Cover error recovery and timeout paths"
echo "  3. Test resource cleanup with Drop implementations"
echo "  4. Add concurrent operation tests"
echo "  5. Document or remove deprecated code"