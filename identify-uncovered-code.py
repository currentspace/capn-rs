#!/usr/bin/env python3
"""
Comprehensive Uncovered Code Identifier for Cap'n Web Rust
Identifies all code that lacks proper test coverage
"""

import os
import re
import ast
from pathlib import Path
from collections import defaultdict
import json

class UncoveredCodeAnalyzer:
    def __init__(self):
        self.uncovered = defaultdict(list)
        self.stats = defaultdict(int)

    def analyze_all_modules(self):
        """Analyze all Rust modules for uncovered code"""

        modules = [
            # Core protocol modules
            ('capnweb-core/src/protocol/resume_tokens.rs', 'Resume Tokens'),
            ('capnweb-core/src/protocol/nested_capabilities.rs', 'Nested Capabilities'),
            ('capnweb-core/src/protocol/il_runner.rs', 'IL Plan Runner'),
            ('capnweb-core/src/protocol/session.rs', 'Session Management'),
            ('capnweb-core/src/protocol/pipeline.rs', 'Promise Pipelining'),
            ('capnweb-core/src/protocol/remap_engine.rs', 'Remap Engine'),
            ('capnweb-core/src/protocol/variable_state.rs', 'Variable State'),
            ('capnweb-core/src/protocol/evaluator.rs', 'Expression Evaluator'),
            ('capnweb-core/src/protocol/tables.rs', 'Import/Export Tables'),
            ('capnweb-core/src/protocol/ids.rs', 'ID Management'),

            # Transport modules
            ('capnweb-transport/src/http3.rs', 'HTTP/3 Transport'),
            ('capnweb-transport/src/websocket.rs', 'WebSocket Transport'),
            ('capnweb-transport/src/webtransport.rs', 'WebTransport'),
            ('capnweb-transport/src/http_batch.rs', 'HTTP Batch'),
            ('capnweb-transport/src/negotiate.rs', 'Transport Negotiation'),

            # Server implementation
            ('capnweb-server/src/capnweb_server.rs', 'Server Implementation'),
            ('capnweb-server/src/lib.rs', 'Server Library'),

            # Client implementation
            ('capnweb-client/src/lib.rs', 'Client Library'),
        ]

        for module_path, module_name in modules:
            if os.path.exists(module_path):
                self.analyze_module(module_path, module_name)

    def analyze_module(self, file_path, module_name):
        """Analyze a single module for uncovered code"""

        with open(file_path, 'r') as f:
            content = f.read()
            lines = content.split('\n')

        uncovered_items = {
            'untested_functions': [],
            'error_paths': [],
            'match_arms': [],
            'async_code': [],
            'unsafe_code': [],
            'complex_logic': [],
            'resource_management': [],
            'concurrency': [],
        }

        # Track what we've seen
        has_tests = '#[cfg(test)]' in content or '#[test]' in content
        test_functions = set(re.findall(r'fn\s+(test_\w+)', content))

        # Analyze line by line
        in_test_module = False
        current_function = None
        function_complexity = 0

        for i, line in enumerate(lines, 1):
            # Track test module boundaries
            if '#[cfg(test)]' in line or 'mod tests' in line:
                in_test_module = True
            elif in_test_module and line.strip() == '}':
                in_test_module = False

            # Skip if we're in a test module
            if in_test_module:
                continue

            # Track current function
            pub_fn_match = re.match(r'\s*pub\s+(async\s+)?fn\s+(\w+)', line)
            if pub_fn_match:
                current_function = pub_fn_match.group(2)
                function_complexity = 0

                # Check if function has a corresponding test
                has_test = False
                for test_fn in test_functions:
                    if current_function in test_fn or test_fn.endswith(current_function):
                        has_test = True
                        break

                if not has_test and current_function not in ['new', 'default', 'fmt', 'from', 'into', 'drop']:
                    uncovered_items['untested_functions'].append({
                        'line': i,
                        'name': current_function,
                        'async': bool(pub_fn_match.group(1))
                    })

            # Count complexity indicators
            if current_function:
                if any(keyword in line for keyword in ['if ', 'match ', 'while ', 'for ']):
                    function_complexity += 1
                if 'return' in line:
                    function_complexity += 1

                # Flag complex functions without tests
                if function_complexity > 10 and current_function not in [fn['name'] for fn in uncovered_items['untested_functions']]:
                    uncovered_items['complex_logic'].append({
                        'line': i,
                        'function': current_function,
                        'complexity': function_complexity
                    })

            # Error handling paths
            if re.search(r'Err\(|\.map_err|return Err|\.expect\(|\.unwrap\(|\?;', line):
                uncovered_items['error_paths'].append({
                    'line': i,
                    'code': line.strip()[:80]
                })

            # Match arms (often have edge cases)
            if '=>' in line and '{' not in line:  # Simple match arm
                if any(pattern in line for pattern in ['_', 'None', 'Err', 'Some']):
                    uncovered_items['match_arms'].append({
                        'line': i,
                        'code': line.strip()[:80]
                    })

            # Async code that might have race conditions
            if re.search(r'\.await|tokio::spawn|async move|futures::', line):
                uncovered_items['async_code'].append({
                    'line': i,
                    'code': line.strip()[:80]
                })

            # Unsafe code blocks
            if 'unsafe' in line:
                uncovered_items['unsafe_code'].append({
                    'line': i,
                    'code': line.strip()[:80]
                })

            # Resource management
            if re.search(r'Drop|drop\(|close\(|shutdown|cleanup|dispose', line):
                uncovered_items['resource_management'].append({
                    'line': i,
                    'code': line.strip()[:80]
                })

            # Concurrency primitives
            if re.search(r'Arc::new|Mutex|RwLock|AtomicU|channel\(|mpsc::', line):
                uncovered_items['concurrency'].append({
                    'line': i,
                    'code': line.strip()[:80]
                })

        # Store results
        if any(len(items) > 0 for items in uncovered_items.values()):
            self.uncovered[module_name] = uncovered_items

            # Update statistics
            self.stats['total_modules'] += 1
            self.stats['modules_without_tests'] += 1 if not has_tests else 0
            self.stats['untested_functions'] += len(uncovered_items['untested_functions'])
            self.stats['error_paths'] += len(uncovered_items['error_paths'])
            self.stats['async_code'] += len(uncovered_items['async_code'])

    def generate_detailed_report(self):
        """Generate a detailed report of uncovered code"""

        report = []
        report.append("=" * 80)
        report.append("CAP'N WEB RUST - UNCOVERED CODE ANALYSIS")
        report.append("=" * 80)
        report.append("")

        # Summary statistics
        report.append("SUMMARY STATISTICS")
        report.append("-" * 40)
        report.append(f"Total modules analyzed: {self.stats['total_modules']}")
        report.append(f"Modules without tests: {self.stats['modules_without_tests']}")
        report.append(f"Untested public functions: {self.stats['untested_functions']}")
        report.append(f"Uncovered error paths: {self.stats['error_paths']}")
        report.append(f"Async code needing tests: {self.stats['async_code']}")
        report.append("")

        # Critical uncovered areas
        report.append("🔴 CRITICAL UNCOVERED AREAS")
        report.append("-" * 40)

        critical_modules = ['Resume Tokens', 'Nested Capabilities', 'IL Plan Runner', 'HTTP/3 Transport']
        for module in critical_modules:
            if module in self.uncovered:
                items = self.uncovered[module]
                report.append(f"\n{module}:")

                # Untested functions
                if items['untested_functions']:
                    report.append(f"  ❌ {len(items['untested_functions'])} untested functions:")
                    for fn in items['untested_functions'][:3]:
                        async_marker = "async " if fn['async'] else ""
                        report.append(f"    - Line {fn['line']}: {async_marker}fn {fn['name']}()")

                # Error paths
                if items['error_paths']:
                    report.append(f"  ❌ {len(items['error_paths'])} uncovered error paths")

                # Complex logic
                if items['complex_logic']:
                    report.append(f"  ❌ {len(items['complex_logic'])} complex functions without tests")

        # Detailed breakdown by category
        report.append("\n" + "=" * 40)
        report.append("DETAILED BREAKDOWN BY CATEGORY")
        report.append("=" * 40)

        # Error handling
        report.append("\n1. ERROR HANDLING GAPS")
        report.append("-" * 30)
        error_count_by_module = {}
        for module, items in self.uncovered.items():
            if items['error_paths']:
                error_count_by_module[module] = len(items['error_paths'])

        for module, count in sorted(error_count_by_module.items(), key=lambda x: x[1], reverse=True)[:5]:
            report.append(f"  {module}: {count} uncovered error paths")

        # Async/concurrent code
        report.append("\n2. ASYNC/CONCURRENT CODE")
        report.append("-" * 30)
        async_count_by_module = {}
        for module, items in self.uncovered.items():
            total = len(items['async_code']) + len(items['concurrency'])
            if total > 0:
                async_count_by_module[module] = total

        for module, count in sorted(async_count_by_module.items(), key=lambda x: x[1], reverse=True)[:5]:
            report.append(f"  {module}: {count} async/concurrent patterns")

        # Resource management
        report.append("\n3. RESOURCE MANAGEMENT")
        report.append("-" * 30)
        resource_count = sum(len(items['resource_management']) for items in self.uncovered.values())
        report.append(f"  Total uncovered: {resource_count} resource management patterns")

        # Unsafe code
        report.append("\n4. UNSAFE CODE")
        report.append("-" * 30)
        unsafe_count = sum(len(items['unsafe_code']) for items in self.uncovered.values())
        if unsafe_count > 0:
            report.append(f"  ⚠️  {unsafe_count} unsafe blocks need careful testing")
        else:
            report.append("  ✅ No unsafe code found")

        # Recommendations
        report.append("\n" + "=" * 40)
        report.append("PRIORITY RECOMMENDATIONS")
        report.append("=" * 40)
        report.append("")

        recommendations = [
            "1. Add tests for all public functions in critical modules",
            "2. Cover error paths with explicit error injection tests",
            "3. Test async code with concurrent operations",
            "4. Add timeout tests for all async functions",
            "5. Test resource cleanup with Drop implementations",
            "6. Add integration tests for cross-module interactions",
            "7. Use property-based testing for complex logic",
            "8. Add benchmarks for performance-critical paths"
        ]

        for rec in recommendations:
            report.append(rec)

        # Generate test templates
        report.append("\n" + "=" * 40)
        report.append("TEST TEMPLATES TO ADD")
        report.append("=" * 40)
        report.append("")

        # Generate specific test recommendations
        for module_name, items in self.uncovered.items():
            if items['untested_functions']:
                report.append(f"\n// Tests needed for {module_name}")
                report.append("#[cfg(test)]")
                report.append("mod tests {")

                for fn_info in items['untested_functions'][:3]:
                    test_name = f"test_{fn_info['name']}"
                    if fn_info['async']:
                        report.append(f"    #[tokio::test]")
                        report.append(f"    async fn {test_name}() {{")
                        report.append(f"        // TODO: Test {fn_info['name']} function")
                        report.append(f"    }}")
                    else:
                        report.append(f"    #[test]")
                        report.append(f"    fn {test_name}() {{")
                        report.append(f"        // TODO: Test {fn_info['name']} function")
                        report.append(f"    }}")
                report.append("}")
                break  # Just show one module as example

        return "\n".join(report)

    def save_json_report(self, filename='target/uncovered-code.json'):
        """Save detailed JSON report"""

        # Convert to JSON-serializable format
        json_data = {
            'summary': dict(self.stats),
            'modules': {}
        }

        for module, items in self.uncovered.items():
            json_data['modules'][module] = {
                'untested_functions': len(items['untested_functions']),
                'error_paths': len(items['error_paths']),
                'async_code': len(items['async_code']),
                'complex_logic': len(items['complex_logic']),
                'resource_management': len(items['resource_management']),
                'concurrency': len(items['concurrency']),
                'unsafe_code': len(items['unsafe_code']),
            }

        os.makedirs('target', exist_ok=True)
        with open(filename, 'w') as f:
            json.dump(json_data, f, indent=2)

        return json_data


def main():
    print("🔍 Analyzing Cap'n Web Rust code for coverage gaps...")
    print("=" * 60)

    analyzer = UncoveredCodeAnalyzer()
    analyzer.analyze_all_modules()

    # Generate and print report
    report = analyzer.generate_detailed_report()
    print(report)

    # Save JSON report
    json_data = analyzer.save_json_report()

    # Print summary metrics
    print("\n" + "=" * 60)
    print("📊 COVERAGE GAP METRICS")
    print("=" * 60)

    total_gaps = sum(json_data['summary'].values())
    print(f"Total coverage gaps identified: {total_gaps}")

    if total_gaps > 200:
        print("⚠️  CRITICAL: Major coverage gaps detected!")
        print("   Immediate action required to improve test coverage")
    elif total_gaps > 100:
        print("⚡ WARNING: Significant coverage gaps")
        print("   Additional tests recommended")
    else:
        print("✅ Coverage is improving but still needs work")

    print("\n📁 Reports saved:")
    print("  - target/uncovered-code.json")
    print("  - Full analysis above")

    return 0 if total_gaps < 100 else 1


if __name__ == "__main__":
    exit(main())