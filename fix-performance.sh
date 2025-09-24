#!/bin/bash

# Performance and Code Quality Improvement Script for Cap'n Web Rust

echo "🔍 Cap'n Web Performance Analysis & Fix Helper"
echo "=============================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to count pattern occurrences
count_pattern() {
    local pattern=$1
    local description=$2
    count=$(grep -r "$pattern" --include="*.rs" capnweb-* 2>/dev/null | wc -l)
    if [ $count -gt 0 ]; then
        echo -e "${YELLOW}⚠️  $description: ${RED}$count occurrences${NC}"
    else
        echo -e "${GREEN}✅ $description: None found${NC}"
    fi
    return $count
}

# Function to show examples of a pattern
show_examples() {
    local pattern=$1
    local description=$2
    local limit=${3:-5}

    echo -e "\n${BLUE}Examples of $description:${NC}"
    grep -r "$pattern" --include="*.rs" capnweb-* -n 2>/dev/null | head -$limit
}

echo "📊 Analyzing Code Patterns..."
echo "------------------------------"

# Critical Issues
echo -e "\n${RED}CRITICAL ISSUES:${NC}"
count_pattern '\.unwrap()' "unwrap() calls (panic risk)"
count_pattern 'panic!(' "Direct panic! calls"
count_pattern '\.expect(' "expect() calls"

# Performance Issues
echo -e "\n${YELLOW}PERFORMANCE ISSUES:${NC}"
count_pattern '\.to_string()' "to_string() allocations"
count_pattern '\.clone()' "clone() operations"
count_pattern 'format!(' "format! macro usage"
count_pattern '\.collect::<String>' "String collection in hot paths"
count_pattern 'Number::from_f64.*unwrap' "Unsafe number conversions"

# Code Clarity Issues
echo -e "\n${BLUE}CODE CLARITY:${NC}"
count_pattern 'as f64' "Type casts to f64"
count_pattern 'as usize' "Type casts to usize"
count_pattern '// TODO' "TODO comments"
count_pattern '// FIXME' "FIXME comments"

# Find specific anti-patterns
echo -e "\n${RED}🔥 Hot Path Allocations:${NC}"
echo "Checking WebSocket handler for allocations..."
grep -n "collect::<String>" capnweb-server/src/capnweb_server.rs 2>/dev/null || echo "None found"

echo -e "\n${RED}🔥 Most Unwrap-Heavy Files:${NC}"
for file in $(find capnweb-* -name "*.rs" -type f); do
    count=$(grep -c "\.unwrap()" "$file" 2>/dev/null || echo 0)
    if [ $count -gt 10 ]; then
        echo -e "  ${YELLOW}$file: $count unwraps${NC}"
    fi
done

# Suggest fixes
echo -e "\n${GREEN}📝 Automated Fix Suggestions:${NC}"
echo "================================"

echo -e "\n1. ${BLUE}Replace unwrap() with ? operator:${NC}"
echo "   Run: ${YELLOW}cargo clippy --fix -- -W clippy::unwrap_used${NC}"

echo -e "\n2. ${BLUE}Find unnecessary clones:${NC}"
echo "   Run: ${YELLOW}cargo clippy -- -W clippy::redundant_clone${NC}"

echo -e "\n3. ${BLUE}Find inefficient string operations:${NC}"
echo "   Run: ${YELLOW}cargo clippy -- -W clippy::inefficient_to_string${NC}"

echo -e "\n4. ${BLUE}General performance lints:${NC}"
echo "   Run: ${YELLOW}cargo clippy -- -W clippy::perf${NC}"

# Generate fix commands
echo -e "\n${GREEN}🔧 Quick Fix Commands:${NC}"
echo "======================"

echo -e "\n# Add to Cargo.toml for stricter lints:"
cat << 'EOF'
[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "warn"
redundant_clone = "warn"
inefficient_to_string = "warn"
unnecessary_to_owned = "warn"
EOF

echo -e "\n# Run comprehensive clippy check:"
echo "cargo clippy --all-targets --all-features -- -D warnings"

echo -e "\n# Check for unsafe code:"
echo "grep -r 'unsafe ' --include='*.rs' capnweb-*"

echo -e "\n# Find large functions that might need refactoring:"
echo "find capnweb-* -name '*.rs' -exec wc -l {} + | sort -rn | head -20"

# Create a todo list
echo -e "\n${GREEN}📋 Priority Fix Order:${NC}"
echo "===================="
echo "1. [ ] Fix all unwrap() in capnweb-server (production critical)"
echo "2. [ ] Fix all unwrap() in capnweb-core (core logic)"
echo "3. [ ] Remove string allocations from WebSocket hot path"
echo "4. [ ] Replace format! with Display implementations"
echo "5. [ ] Reduce clone() operations on Arc types"
echo "6. [ ] Add error context to all expect() calls"
echo "7. [ ] Profile with flamegraph to verify improvements"

echo -e "\n${BLUE}📈 Next Steps:${NC}"
echo "1. Run: ${YELLOW}./fix-performance.sh > performance-report.txt${NC}"
echo "2. Fix critical issues first (unwraps in production code)"
echo "3. Run benchmarks before and after changes"
echo "4. Use cargo-flamegraph to profile actual bottlenecks"

echo -e "\n✨ Done! Check PERFORMANCE_IMPROVEMENTS.md for detailed analysis."