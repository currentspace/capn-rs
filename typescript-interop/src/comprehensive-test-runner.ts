#!/usr/bin/env node

import { spawn, ChildProcess } from 'child_process';

/**
 * Comprehensive Test Runner for Cap'n Web Rust Implementation
 *
 * This runs all protocol-compliant test tiers in sequence, providing
 * a complete validation of the server implementation against the
 * TypeScript reference client.
 *
 * All tests respect the official Cap'n Web wire protocol:
 * - HTTP batch sessions end after sending their batch
 * - Sequential operations require new sessions or Promise.all()
 * - WebSocket allows persistent sessions
 */

interface TestTier {
    name: string;
    script: string;
    port: number;
    critical: boolean;  // If true, failure stops all testing
    transport?: 'http' | 'websocket';
}

const testTiers: TestTier[] = [
    {
        name: 'TIER 1: Protocol Compliance',
        script: './dist/tier1-protocol-compliance.js',
        port: 9000,
        critical: true,
        transport: 'http'
    },
    {
        name: 'TIER 2: HTTP Batch (Corrected)',
        script: './dist/tier2-http-batch-corrected.js',
        port: 9000,
        critical: true,
        transport: 'http'
    },
    {
        name: 'TIER 2: WebSocket Sessions',
        script: './dist/tier2-websocket-tests.js',
        port: 9000,
        critical: false, // WebSocket might not be implemented yet
        transport: 'websocket'
    },
    {
        name: 'TIER 3: Capability Composition',
        script: './dist/tier3-capability-composition.js',
        port: 9000,
        critical: false,
        transport: 'http'
    },
    {
        name: 'TIER 3: Complex Applications',
        script: './dist/tier3-complex-applications.js',
        port: 9000,
        critical: false,
        transport: 'http'
    }
];

class ComprehensiveTestRunner {
    private totalTests = 0;
    private passedTests = 0;
    private failedTests = 0;
    private results: Map<string, { passed: number; failed: number; exitCode: number }> = new Map();

    async runTest(tier: TestTier): Promise<boolean> {
        return new Promise((resolve) => {
            console.log('\n' + '='.repeat(60));
            console.log(`🚀 Running ${tier.name}`);
            console.log(`📍 Port: ${tier.port}, Transport: ${tier.transport || 'default'}`);
            console.log('='.repeat(60));

            const child = spawn('node', [tier.script, String(tier.port)], {
                cwd: process.cwd(),
                stdio: 'inherit',
                env: { ...process.env }
            });

            child.on('exit', (code) => {
                const success = code === 0;

                if (success) {
                    console.log(`\n✅ ${tier.name}: PASSED`);
                } else if (code === 1 && !tier.critical) {
                    console.log(`\n⚠️  ${tier.name}: PARTIAL PASS (non-critical)`);
                } else {
                    console.log(`\n❌ ${tier.name}: FAILED with exit code ${code}`);
                }

                this.results.set(tier.name, {
                    passed: success ? 1 : 0,
                    failed: success ? 0 : 1,
                    exitCode: code || 0
                });

                resolve(success || !tier.critical);
            });

            child.on('error', (err) => {
                console.error(`\n💥 Failed to run ${tier.name}:`, err);
                this.results.set(tier.name, {
                    passed: 0,
                    failed: 1,
                    exitCode: -1
                });
                resolve(!tier.critical);
            });
        });
    }

    async runAllTests(): Promise<void> {
        console.log('🏁 CAP\'N WEB RUST IMPLEMENTATION - COMPREHENSIVE TEST SUITE');
        console.log('============================================================');
        console.log('📋 Protocol Compliance Testing with TypeScript Reference Client');
        console.log('🎯 Testing official Cap\'n Web wire protocol (newline-delimited)');
        console.log('');

        let shouldContinue = true;

        for (const tier of testTiers) {
            if (!shouldContinue) {
                console.log(`\n⏩ Skipping ${tier.name} due to critical failure`);
                this.results.set(tier.name, {
                    passed: 0,
                    failed: 0,
                    exitCode: -2  // Skipped
                });
                continue;
            }

            const success = await this.runTest(tier);

            if (!success && tier.critical) {
                shouldContinue = false;
                console.log('\n🛑 Critical test failed - stopping test execution');
            }
        }

        this.printSummary();
    }

    private printSummary(): void {
        console.log('\n' + '='.repeat(60));
        console.log('📊 COMPREHENSIVE TEST RESULTS SUMMARY');
        console.log('='.repeat(60));

        let totalPassed = 0;
        let totalFailed = 0;
        let skipped = 0;

        console.log('\n📋 Individual Tier Results:');
        console.log('-'.repeat(60));

        for (const [name, result] of this.results) {
            const icon = result.exitCode === 0 ? '✅' :
                        result.exitCode === -2 ? '⏩' :
                        result.exitCode === 1 ? '⚠️' : '❌';

            const status = result.exitCode === 0 ? 'PASSED' :
                          result.exitCode === -2 ? 'SKIPPED' :
                          result.exitCode === 1 ? 'PARTIAL' : 'FAILED';

            console.log(`${icon} ${name.padEnd(40)} ${status}`);

            if (result.exitCode === -2) {
                skipped++;
            } else {
                totalPassed += result.passed;
                totalFailed += result.failed;
            }
        }

        const completionRate = totalPassed + totalFailed > 0
            ? ((totalPassed / (totalPassed + totalFailed)) * 100).toFixed(1)
            : '0.0';

        console.log('\n📈 Overall Statistics:');
        console.log('-'.repeat(60));
        console.log(`   Tests Run: ${totalPassed + totalFailed}`);
        console.log(`   Passed: ${totalPassed} ✅`);
        console.log(`   Failed: ${totalFailed} ❌`);
        console.log(`   Skipped: ${skipped} ⏩`);
        console.log(`   Success Rate: ${completionRate}%`);

        console.log('\n🎯 Protocol Compliance Status:');
        console.log('-'.repeat(60));

        const tier1Result = this.results.get('TIER 1: Protocol Compliance');
        const tier2HttpResult = this.results.get('TIER 2: HTTP Batch (Corrected)');
        const tier2WsResult = this.results.get('TIER 2: WebSocket Sessions');

        if (tier1Result?.exitCode === 0) {
            console.log('✅ Basic Wire Protocol: COMPLIANT');
        } else {
            console.log('❌ Basic Wire Protocol: NON-COMPLIANT');
        }

        if (tier2HttpResult?.exitCode === 0) {
            console.log('✅ HTTP Batch Transport: COMPLIANT');
        } else {
            console.log('⚠️  HTTP Batch Transport: PARTIAL/NON-COMPLIANT');
        }

        if (tier2WsResult?.exitCode === 0) {
            console.log('✅ WebSocket Transport: COMPLIANT');
        } else if (tier2WsResult?.exitCode === -2) {
            console.log('⏩ WebSocket Transport: NOT TESTED');
        } else {
            console.log('⚠️  WebSocket Transport: NOT IMPLEMENTED/NON-COMPLIANT');
        }

        // Final verdict
        console.log('\n' + '='.repeat(60));

        const allCriticalPassed = tier1Result?.exitCode === 0 &&
                                  tier2HttpResult?.exitCode === 0;

        if (allCriticalPassed) {
            console.log('🎉 IMPLEMENTATION STATUS: PROTOCOL COMPLIANT');
            console.log('='.repeat(60));
            console.log('✅ The Rust server correctly implements the Cap\'n Web protocol');
            console.log('✅ Compatible with official TypeScript reference client');
            console.log('✅ HTTP batch transport working correctly');

            if (tier2WsResult?.exitCode === 0) {
                console.log('✅ WebSocket transport also working');
            }

            process.exit(0);
        } else {
            console.log('❌ IMPLEMENTATION STATUS: NON-COMPLIANT');
            console.log('='.repeat(60));
            console.log('⚠️  Critical protocol compliance issues detected');
            console.log('🔧 Review failed tests and fix protocol implementation');
            process.exit(1);
        }
    }
}

// Handle interrupts gracefully
process.on('SIGINT', () => {
    console.log('\n\n⚠️  Test suite interrupted by user');
    process.exit(130);
});

// Run the test suite
if (import.meta.url === `file://${process.argv[1]}`) {
    const runner = new ComprehensiveTestRunner();
    runner.runAllTests().catch(error => {
        console.error('\n💥 Fatal error in test runner:', error);
        process.exit(1);
    });
}

export { ComprehensiveTestRunner };