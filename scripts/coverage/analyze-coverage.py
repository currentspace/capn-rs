#!/usr/bin/env python3
"""
Cap'n Web Rust Code Coverage Analyzer
Analyzes which parts of the code lack test coverage
"""

import os
import re
import json
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

class CoverageAnalyzer:
    def __init__(self, project_root="."):
        self.project_root = Path(project_root)
        self.uncovered_functions = defaultdict(list)
        self.uncovered_modules = defaultdict(int)
        self.coverage_by_feature = defaultdict(lambda: {'covered': 0, 'total': 0})

    def analyze_rust_files(self) -> Dict[str, List[str]]:
        """Analyze Rust source files to identify potentially untested code"""
        untested_patterns = {
            'error_handling': [],
            'advanced_features': [],
            'edge_cases': [],
            'public_apis': []
        }

        # Key modules to analyze
        modules = [
            'capnweb-core/src/protocol/resume_tokens.rs',
            'capnweb-core/src/protocol/nested_capabilities.rs',
            'capnweb-core/src/protocol/il_runner.rs',
            'capnweb-transport/src/http3.rs',
            'capnweb-transport/src/webtransport.rs',
            'capnweb-server/src/capnweb_server.rs'
        ]

        for module_path in modules:
            full_path = self.project_root / module_path
            if full_path.exists():
                self.analyze_module(full_path, untested_patterns)

        return untested_patterns

    def analyze_module(self, file_path: Path, patterns: Dict):
        """Analyze a single module for coverage gaps"""
        with open(file_path, 'r') as f:
            content = f.read()
            lines = content.split('\n')

        module_name = file_path.stem

        # Check for error handling paths
        error_patterns = [
            r'Err\(',
            r'\.map_err\(',
            r'return Err\(',
            r'panic!\(',
            r'unreachable!\(',
            r'todo!\(',
            r'unimplemented!\('
        ]

        for i, line in enumerate(lines, 1):
            for pattern in error_patterns:
                if re.search(pattern, line):
                    # Check if this line is in a test
                    if not self.is_in_test(lines, i):
                        patterns['error_handling'].append({
                            'file': str(file_path),
                            'line': i,
                            'code': line.strip(),
                            'type': 'error_path'
                        })

        # Check for public APIs without tests
        pub_fn_pattern = r'^\s*pub\s+(async\s+)?fn\s+(\w+)'
        for i, line in enumerate(lines, 1):
            match = re.match(pub_fn_pattern, line)
            if match:
                fn_name = match.group(2)
                if not fn_name.startswith('test_') and not fn_name.startswith('new'):
                    # Check if there's a corresponding test
                    if not self.has_test_for_function(content, fn_name):
                        patterns['public_apis'].append({
                            'file': str(file_path),
                            'line': i,
                            'function': fn_name,
                            'type': 'untested_api'
                        })

        # Check advanced features
        advanced_patterns = {
            'resume_tokens': [
                r'ResumeToken',
                r'SessionSnapshot',
                r'restore_session',
                r'snapshot_session'
            ],
            'nested_capabilities': [
                r'create_sub_capability',
                r'CapabilityGraph',
                r'dispose_child',
                r'factory\.create'
            ],
            'il_runner': [
                r'execute_plan',
                r'PlanBuilder',
                r'PlanOptimizer',
                r'ExecutionContext'
            ],
            'http3': [
                r'Http3Transport',
                r'Http3Stream',
                r'ConnectionPool',
                r'LoadBalancer'
            ]
        }

        for feature, feature_patterns in advanced_patterns.items():
            for i, line in enumerate(lines, 1):
                for pattern in feature_patterns:
                    if re.search(pattern, line) and not self.is_in_test(lines, i):
                        patterns['advanced_features'].append({
                            'file': str(file_path),
                            'line': i,
                            'feature': feature,
                            'code': line.strip()[:80],
                            'type': 'advanced_feature'
                        })

        # Check for edge cases
        edge_case_patterns = [
            r'if\s+.*\s*==\s*0',  # Zero checks
            r'if\s+.*\.is_empty\(\)',  # Empty checks
            r'Option::None',  # None handling
            r'\.unwrap_or',  # Default values
            r'timeout',  # Timeout handling
            r'max_',  # Limit checks
        ]

        for i, line in enumerate(lines, 1):
            for pattern in edge_case_patterns:
                if re.search(pattern, line, re.IGNORECASE) and not self.is_in_test(lines, i):
                    patterns['edge_cases'].append({
                        'file': str(file_path),
                        'line': i,
                        'code': line.strip()[:80],
                        'type': 'edge_case'
                    })

    def is_in_test(self, lines: List[str], line_num: int) -> bool:
        """Check if a line is within a test function or module"""
        # Look backwards for #[test] or #[cfg(test)]
        for i in range(max(0, line_num - 20), line_num):
            if '#[test]' in lines[i] or '#[cfg(test)]' in lines[i]:
                return True
            if 'mod tests' in lines[i]:
                return True
        return False

    def has_test_for_function(self, content: str, fn_name: str) -> bool:
        """Check if there's a test for a specific function"""
        test_patterns = [
            f'test_{fn_name}',
            f'{fn_name}_test',
            f'test.*{fn_name}',
            f'{fn_name}.*test'
        ]

        for pattern in test_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                return True
        return False

    def generate_report(self, patterns: Dict) -> str:
        """Generate a coverage gap report"""
        report = []
        report.append("# Cap'n Web Rust Code Coverage Gap Analysis")
        report.append("=" * 60)
        report.append("")

        # Summary
        total_gaps = sum(len(items) for items in patterns.values())
        report.append(f"## Summary")
        report.append(f"Total coverage gaps identified: {total_gaps}")
        report.append("")

        # Error handling gaps
        if patterns['error_handling']:
            report.append("## 1. Error Handling Paths Missing Tests")
            report.append("-" * 40)
            by_file = defaultdict(list)
            for item in patterns['error_handling'][:20]:  # Limit to top 20
                by_file[item['file']].append(item)

            for file, items in by_file.items():
                report.append(f"\n### {Path(file).name}")
                for item in items[:5]:  # Top 5 per file
                    report.append(f"  Line {item['line']}: {item['code'][:60]}...")

        # Untested public APIs
        if patterns['public_apis']:
            report.append("\n## 2. Public APIs Without Direct Tests")
            report.append("-" * 40)
            by_file = defaultdict(list)
            for item in patterns['public_apis'][:20]:
                by_file[item['file']].append(item)

            for file, items in by_file.items():
                report.append(f"\n### {Path(file).name}")
                for item in items[:5]:
                    report.append(f"  Line {item['line']}: pub fn {item['function']}()")

        # Advanced features
        if patterns['advanced_features']:
            report.append("\n## 3. Advanced Features Needing More Tests")
            report.append("-" * 40)
            by_feature = defaultdict(list)
            for item in patterns['advanced_features'][:30]:
                by_feature[item['feature']].append(item)

            for feature, items in by_feature.items():
                report.append(f"\n### {feature.replace('_', ' ').title()}")
                unique_files = set(item['file'] for item in items)
                report.append(f"  Files: {len(unique_files)}")
                report.append(f"  Gaps: {len(items)}")

        # Edge cases
        edge_count = len(patterns['edge_cases'])
        if edge_count > 0:
            report.append("\n## 4. Edge Cases")
            report.append("-" * 40)
            report.append(f"Found {edge_count} potential edge cases without explicit tests")
            by_file = defaultdict(int)
            for item in patterns['edge_cases']:
                by_file[Path(item['file']).name] += 1

            for file, count in sorted(by_file.items(), key=lambda x: x[1], reverse=True)[:10]:
                report.append(f"  {file}: {count} edge cases")

        report.append("\n## Recommendations")
        report.append("-" * 40)
        report.append("1. Add error path tests for critical modules")
        report.append("2. Create integration tests for advanced features")
        report.append("3. Add property-based tests for edge cases")
        report.append("4. Test timeout and resource exhaustion scenarios")
        report.append("5. Add tests for concurrent operations")

        return "\n".join(report)

    def generate_test_stubs(self, patterns: Dict) -> str:
        """Generate test stubs for uncovered code"""
        stubs = []
        stubs.append("// Generated test stubs for uncovered code")
        stubs.append("")

        # Generate error handling tests
        if patterns['error_handling']:
            stubs.append("#[cfg(test)]")
            stubs.append("mod error_handling_tests {")
            stubs.append("    use super::*;")
            stubs.append("")

            seen_files = set()
            for item in patterns['error_handling'][:10]:
                file_name = Path(item['file']).stem
                if file_name not in seen_files:
                    seen_files.add(file_name)
                    stubs.append(f"    #[test]")
                    stubs.append(f"    fn test_{file_name}_error_paths() {{")
                    stubs.append(f"        // TODO: Test error handling in {file_name}")
                    stubs.append(f"        // Line {item['line']}: {item['code'][:50]}")
                    stubs.append(f"    }}")
                    stubs.append("")

            stubs.append("}")
            stubs.append("")

        # Generate API tests
        if patterns['public_apis']:
            stubs.append("#[cfg(test)]")
            stubs.append("mod api_tests {")
            stubs.append("    use super::*;")
            stubs.append("")

            for item in patterns['public_apis'][:10]:
                stubs.append(f"    #[tokio::test]")
                stubs.append(f"    async fn test_{item['function']}() {{")
                stubs.append(f"        // TODO: Test {item['function']} from {Path(item['file']).name}")
                stubs.append(f"    }}")
                stubs.append("")

            stubs.append("}")

        return "\n".join(stubs)


def main():
    analyzer = CoverageAnalyzer()

    print("🔍 Analyzing Cap'n Web Rust code for coverage gaps...")
    print("=" * 60)

    patterns = analyzer.analyze_rust_files()

    # Generate report
    report = analyzer.generate_report(patterns)
    print(report)

    # Save report
    with open('target/coverage-gaps.md', 'w') as f:
        f.write(report)

    # Generate test stubs
    stubs = analyzer.generate_test_stubs(patterns)
    with open('target/test-stubs.rs', 'w') as f:
        f.write(stubs)

    print("\n📁 Reports saved to:")
    print("  - target/coverage-gaps.md")
    print("  - target/test-stubs.rs")

    # Summary statistics
    total = sum(len(items) for items in patterns.values())
    print(f"\n📊 Total coverage gaps found: {total}")

    if total > 100:
        print("⚠️  Significant coverage gaps detected")
        return 1
    elif total > 50:
        print("⚡ Moderate coverage gaps detected")
        return 0
    else:
        print("✅ Coverage looks good!")
        return 0


if __name__ == "__main__":
    exit(main())