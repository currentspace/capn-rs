#!/bin/bash

# Cap'n Web Rust Code Coverage Analysis

echo "🔍 Cap'n Web Code Coverage Analysis"
echo "===================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
    cargo install cargo-tarpaulin
fi

# Create coverage directory
mkdir -p target/coverage

echo -e "${BLUE}Running code coverage analysis...${NC}"
echo "This may take a few minutes..."
echo ""

# Run tarpaulin with all features
cargo tarpaulin \
    --all-features \
    --workspace \
    --timeout 300 \
    --exclude-files "*/tests/*" \
    --exclude-files "*/examples/*" \
    --exclude-files "*/benches/*" \
    --ignore-panics \
    --ignore-tests \
    --out Html \
    --out Json \
    --output-dir target/coverage \
    2>&1 | tee target/coverage/coverage.log

# Extract coverage percentage
COVERAGE=$(grep "Coverage Results" target/coverage/coverage.log -A 1 | tail -1 | grep -oE '[0-9]+\.[0-9]+%' | head -1)

echo ""
echo -e "${GREEN}Coverage analysis complete!${NC}"
echo -e "Overall coverage: ${YELLOW}${COVERAGE}${NC}"

# Generate detailed uncovered lines report
echo ""
echo -e "${BLUE}Analyzing uncovered code...${NC}"

# Parse the JSON output to find uncovered areas
if [ -f "target/coverage/tarpaulin-report.json" ]; then
    python3 << 'PYTHON_SCRIPT'
import json
import sys
from collections import defaultdict

# Load coverage data
with open('target/coverage/tarpaulin-report.json', 'r') as f:
    data = json.load(f)

# Analyze uncovered lines per file
uncovered_by_file = defaultdict(list)
total_lines = 0
covered_lines = 0

for file_path, file_data in data.get('files', {}).items():
    if 'target' in file_path or 'tests' in file_path:
        continue

    for line_num, coverage in file_data.get('lines', {}).items():
        total_lines += 1
        if coverage == 0:
            uncovered_by_file[file_path].append(int(line_num))
        else:
            covered_lines += 1

# Sort files by number of uncovered lines
sorted_files = sorted(uncovered_by_file.items(), key=lambda x: len(x[1]), reverse=True)

# Print summary
print("\n📊 Coverage Summary by Module:")
print("=" * 60)

modules = defaultdict(lambda: {'total': 0, 'covered': 0})

for file_path, uncovered in sorted_files[:20]:  # Top 20 files
    module = file_path.split('/src/')[0].split('/')[-1] if '/src/' in file_path else 'unknown'
    file_name = file_path.split('/')[-1]

    # Estimate total lines (uncovered + some covered estimate)
    file_total = len(uncovered) * 2  # Rough estimate
    modules[module]['total'] += file_total
    modules[module]['covered'] += file_total - len(uncovered)

    if len(uncovered) > 5:  # Only show files with significant uncovered code
        print(f"\n📁 {file_name} ({module})")
        print(f"   Uncovered lines: {len(uncovered)}")

        # Show line ranges
        if uncovered:
            ranges = []
            start = uncovered[0]
            end = uncovered[0]

            for line in uncovered[1:]:
                if line == end + 1:
                    end = line
                else:
                    if start == end:
                        ranges.append(str(start))
                    else:
                        ranges.append(f"{start}-{end}")
                    start = end = line

            if start == end:
                ranges.append(str(start))
            else:
                ranges.append(f"{start}-{end}")

            print(f"   Line ranges: {', '.join(ranges[:10])}")
            if len(ranges) > 10:
                print(f"   ... and {len(ranges) - 10} more ranges")

print("\n📈 Module Coverage:")
print("-" * 40)
for module, stats in sorted(modules.items()):
    if stats['total'] > 0:
        coverage = (stats['covered'] / stats['total']) * 100
        print(f"  {module:20} {coverage:5.1f}%")

PYTHON_SCRIPT
fi

# Create a detailed report
echo ""
echo -e "${BLUE}Creating detailed coverage report...${NC}"

cat > target/coverage/coverage-summary.md << EOF
# Cap'n Web Rust Code Coverage Report

## Summary
- **Date**: $(date)
- **Overall Coverage**: ${COVERAGE}
- **Report Location**: target/coverage/tarpaulin-report.html

## Coverage by Crate

| Crate | Coverage | Status |
|-------|----------|--------|
| capnweb-core | TBD | 🔍 |
| capnweb-transport | TBD | 🔍 |
| capnweb-server | TBD | 🔍 |
| capnweb-client | TBD | 🔍 |

## Uncovered Code Areas

### Priority 1: Core Protocol
- [ ] Error handling paths
- [ ] Edge cases in IL runner
- [ ] Resume token error scenarios

### Priority 2: Transport Layer
- [ ] HTTP/3 error handling
- [ ] WebTransport edge cases
- [ ] Connection pool cleanup

### Priority 3: Server Implementation
- [ ] Concurrent request handling
- [ ] Resource cleanup
- [ ] Rate limiting edge cases

## Next Steps
1. Add tests for uncovered error paths
2. Test edge cases in advanced features
3. Add property-based tests for complex scenarios

EOF

echo -e "${GREEN}✅ Coverage report generated${NC}"
echo ""
echo "📁 Reports available at:"
echo "  - HTML: target/coverage/tarpaulin-report.html"
echo "  - JSON: target/coverage/tarpaulin-report.json"
echo "  - Summary: target/coverage/coverage-summary.md"
echo ""

# Open HTML report if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}Opening HTML report in browser...${NC}"
    open target/coverage/tarpaulin-report.html
fi