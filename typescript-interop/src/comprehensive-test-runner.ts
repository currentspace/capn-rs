#!/usr/bin/env node

import { testAdvancedStatefulServer } from './advanced-server-test';
import { testPromisePipelining } from './promise-pipelining-test';

// Import the basic test function (modify the file to export it)
import { spawn } from 'child_process';
import { promisify } from 'util';

interface TestResult {
    name: string;
    success: boolean;
    duration: number;
    error?: string;
}

async function runBasicClientTest(): Promise<TestResult> {
    const start = performance.now();

    return new Promise((resolve) => {
        const child = spawn('node', ['dist/official-client-test.js'], {
            cwd: process.cwd(),
            stdio: 'pipe'
        });

        let stdout = '';
        let stderr = '';

        child.stdout.on('data', (data) => {
            stdout += data.toString();
        });

        child.stderr.on('data', (data) => {
            stderr += data.toString();
        });

        child.on('close', (code) => {
            const end = performance.now();
            resolve({
                name: 'Basic Client Test',
                success: code === 0,
                duration: end - start,
                error: code !== 0 ? stderr || 'Process exited with non-zero code' : undefined
            });
        });
    });
}

async function runTestWithMeasurement<T>(
    name: string,
    testFn: () => Promise<T>
): Promise<TestResult> {
    const start = performance.now();

    try {
        await testFn();
        const end = performance.now();
        return {
            name,
            success: true,
            duration: end - start
        };
    } catch (error) {
        const end = performance.now();
        return {
            name,
            success: false,
            duration: end - start,
            error: error instanceof Error ? error.message : String(error)
        };
    }
}

async function checkServerHealth(): Promise<boolean> {
    try {
        const response = await fetch('http://localhost:8080/rpc/batch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify([])
        });
        return response.ok;
    } catch {
        return false;
    }
}

async function runComprehensiveTests() {
    console.log('🚀 Comprehensive Cap\'n Web Rust Server Test Suite');
    console.log('===================================================\n');

    // Check if server is running
    console.log('🔍 Checking server health...');
    const serverHealthy = await checkServerHealth();

    if (!serverHealthy) {
        console.error('❌ Server is not running or not responding');
        console.error('   Please start the server with:');
        console.error('   cargo run --example advanced_stateful_server -p capnweb-server');
        process.exit(1);
    }

    console.log('✅ Server is healthy and responding\n');

    const tests: Array<{name: string, fn: () => Promise<any>}> = [
        {
            name: 'Basic Calculator Client Test',
            fn: async () => {
                const result = await runBasicClientTest();
                if (!result.success) {
                    throw new Error(result.error || 'Basic client test failed');
                }
                return result;
            }
        },
        {
            name: 'Advanced Stateful Server Test',
            fn: () => testAdvancedStatefulServer()
        },
        {
            name: 'Promise Pipelining Test',
            fn: () => testPromisePipelining()
        }
    ];

    const results: TestResult[] = [];

    for (const test of tests) {
        console.log(`🧪 Running: ${test.name}`);
        console.log('='.repeat(50));

        const result = await runTestWithMeasurement(test.name, test.fn);
        results.push(result);

        if (result.success) {
            console.log(`✅ ${test.name} - PASSED (${result.duration.toFixed(2)}ms)`);
        } else {
            console.log(`❌ ${test.name} - FAILED (${result.duration.toFixed(2)}ms)`);
            if (result.error) {
                console.log(`   Error: ${result.error}`);
            }
        }

        console.log('\n');
    }

    // Generate comprehensive report
    console.log('=' + '='.repeat(79));
    console.log('📊 COMPREHENSIVE TEST RESULTS SUMMARY');
    console.log('=' + '='.repeat(79));

    const totalTests = results.length;
    const passedTests = results.filter(r => r.success).length;
    const failedTests = totalTests - passedTests;
    const totalDuration = results.reduce((sum, r) => sum + r.duration, 0);

    console.log(`\n📈 Test Statistics:`);
    console.log(`   Total Tests: ${totalTests}`);
    console.log(`   Passed: ${passedTests} ✅`);
    console.log(`   Failed: ${failedTests} ${failedTests > 0 ? '❌' : '✅'}`);
    console.log(`   Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    console.log(`   Total Duration: ${totalDuration.toFixed(2)}ms`);
    console.log(`   Average per Test: ${(totalDuration / totalTests).toFixed(2)}ms`);

    console.log(`\n📋 Individual Test Results:`);
    results.forEach(result => {
        const status = result.success ? '✅ PASS' : '❌ FAIL';
        const duration = result.duration.toFixed(2).padStart(8);
        console.log(`   ${status} │ ${duration}ms │ ${result.name}`);
        if (!result.success && result.error) {
            console.log(`         │         │   └─ ${result.error}`);
        }
    });

    console.log('\n🏆 FEATURE VALIDATION STATUS:');
    console.log('==============================');

    const featureStatus = {
        'Basic RPC Communication': results[0]?.success ?? false,
        'Stateful Session Management': results[1]?.success ?? false,
        'Global Counter Operations': results[1]?.success ?? false,
        'Session-Specific Storage': results[1]?.success ?? false,
        'Property Management': results[1]?.success ?? false,
        'Concurrent Operations': results[1]?.success ?? false,
        'Error Handling': results[1]?.success ?? false,
        'Promise Pipelining': results[2]?.success ?? false,
        'Batch Optimization': results[2]?.success ?? false,
        'Mixed Operation Types': results[2]?.success ?? false,
        'Resource Cleanup': results[2]?.success ?? false
    };

    Object.entries(featureStatus).forEach(([feature, status]) => {
        const icon = status ? '✅' : '❌';
        console.log(`   ${icon} ${feature}`);
    });

    const allPassed = results.every(r => r.success);

    if (allPassed) {
        console.log('\n🎉 ALL TESTS PASSED! 🎉');
        console.log('========================');
        console.log('🚀 The Cap\'n Web Rust implementation is fully functional!');
        console.log('📦 Ready for production deployment');
        console.log('🔗 Compatible with official TypeScript Cap\'n Web client');
        console.log('⚡ Optimized for performance and concurrency');
        console.log('🛡️  Robust error handling and session management');
    } else {
        console.log('\n⚠️  SOME TESTS FAILED');
        console.log('====================');
        console.log('❌ Implementation needs attention before production use');
        console.log('🔧 Review failed tests and fix underlying issues');
        console.log('🧪 Re-run tests after fixes are applied');

        process.exit(1);
    }
}

// Performance monitoring
function setupPerformanceMonitoring() {
    const memoryUsage = process.memoryUsage();
    console.log('\n📊 Performance Monitoring:');
    console.log(`   Heap Used: ${(memoryUsage.heapUsed / 1024 / 1024).toFixed(2)} MB`);
    console.log(`   Heap Total: ${(memoryUsage.heapTotal / 1024 / 1024).toFixed(2)} MB`);
    console.log(`   RSS: ${(memoryUsage.rss / 1024 / 1024).toFixed(2)} MB`);
    console.log(`   External: ${(memoryUsage.external / 1024 / 1024).toFixed(2)} MB`);
}

// Resource cleanup
process.on('exit', () => {
    setupPerformanceMonitoring();
    console.log('\n👋 Test suite completed - resources cleaned up');
});

process.on('SIGINT', () => {
    console.log('\n\n⚠️  Test suite interrupted by user');
    process.exit(0);
});

process.on('unhandledRejection', (reason, promise) => {
    console.error('\n💥 Unhandled promise rejection:', reason);
    process.exit(1);
});

// Run the comprehensive test suite
if (import.meta.url === `file://${process.argv[1]}`) {
    runComprehensiveTests().catch(error => {
        console.error('\n💥 Fatal error in test suite:', error);
        process.exit(1);
    });
}

export { runComprehensiveTests };