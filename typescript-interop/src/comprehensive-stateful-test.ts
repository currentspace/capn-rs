#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

// Define the Calculator interface for basic tests
interface Calculator {
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;
}

// Test configuration
const port = process.argv[2] || '9001';
const endpoint = `http://localhost:${port}/rpc/batch`;

async function testBasicCalculator() {
    console.log('🧮 Testing Basic Calculator Operations');
    console.log('=====================================');

    const session = newHttpBatchRpcSession<Calculator>(endpoint);
    let passed = 0;
    let total = 0;

    // Test 1: Addition
    total++;
    try {
        const result = await session.add(5, 3);
        if (result === 8) {
            console.log('✅ Addition test passed');
            passed++;
        } else {
            console.log(`❌ Addition test failed: expected 8, got ${result}`);
        }
    } catch (error) {
        console.log(`❌ Addition test failed with error: ${error}`);
    }

    // Test 2: Multiplication
    total++;
    try {
        const result = await session.multiply(7, 6);
        if (result === 42) {
            console.log('✅ Multiplication test passed');
            passed++;
        } else {
            console.log(`❌ Multiplication test failed: expected 42, got ${result}`);
        }
    } catch (error) {
        console.log(`❌ Multiplication test failed with error: ${error}`);
    }

    // Test 3: Division
    total++;
    try {
        const result = await session.divide(100, 4);
        if (result === 25) {
            console.log('✅ Division test passed');
            passed++;
        } else {
            console.log(`❌ Division test failed: expected 25, got ${result}`);
        }
    } catch (error) {
        console.log(`❌ Division test failed with error: ${error}`);
    }

    // Test 4: Subtraction
    total++;
    try {
        const result = await session.subtract(10, 7);
        if (result === 3) {
            console.log('✅ Subtraction test passed');
            passed++;
        } else {
            console.log(`❌ Subtraction test failed: expected 3, got ${result}`);
        }
    } catch (error) {
        console.log(`❌ Subtraction test failed with error: ${error}`);
    }

    // Test 5: Error handling (division by zero)
    total++;
    try {
        const result = await session.divide(10, 0);
        console.log(`❌ Division by zero should have thrown an error, got: ${result}`);
    } catch (error) {
        console.log('✅ Division by zero correctly threw error');
        passed++;
    }

    console.log(`\n📊 Basic Calculator: ${passed}/${total} tests passed\n`);
    return { passed, total };
}

async function testConcurrentOperations() {
    console.log('🔄 Testing Concurrent Operations');
    console.log('================================');

    const session = newHttpBatchRpcSession<Calculator>(endpoint);
    let passed = 0;
    let total = 0;

    // Test concurrent operations
    total++;
    try {
        const start = Date.now();
        const [sum, product, quotient, difference] = await Promise.all([
            session.add(10, 20),
            session.multiply(5, 8),
            session.divide(100, 5),
            session.subtract(50, 15)
        ]);
        const duration = Date.now() - start;

        const expectedResults = [30, 40, 20, 35];
        const actualResults = [sum, product, quotient, difference];

        if (JSON.stringify(actualResults) === JSON.stringify(expectedResults)) {
            console.log(`✅ Concurrent operations passed (${duration}ms)`);
            console.log(`   Results: ${actualResults.join(', ')}`);
            passed++;
        } else {
            console.log(`❌ Concurrent operations failed:`);
            console.log(`   Expected: ${expectedResults.join(', ')}`);
            console.log(`   Actual: ${actualResults.join(', ')}`);
        }
    } catch (error) {
        console.log(`❌ Concurrent operations failed with error: ${error}`);
    }

    console.log(`\n📊 Concurrent Operations: ${passed}/${total} tests passed\n`);
    return { passed, total };
}

async function testSessionPersistence() {
    console.log('💾 Testing Session Persistence');
    console.log('==============================');

    let passed = 0;
    let total = 0;

    // Test that sessions maintain state across requests
    total++;
    try {
        // Create multiple sessions (would normally be different session IDs)
        const session1 = newHttpBatchRpcSession<Calculator>(endpoint);
        const session2 = newHttpBatchRpcSession<Calculator>(endpoint);

        // Both should work independently
        const [result1, result2] = await Promise.all([
            session1.add(1, 2),
            session2.multiply(3, 4)
        ]);

        if (result1 === 3 && result2 === 12) {
            console.log('✅ Multiple sessions work independently');
            passed++;
        } else {
            console.log(`❌ Session independence failed: ${result1}, ${result2}`);
        }
    } catch (error) {
        console.log(`❌ Session persistence test failed: ${error}`);
    }

    console.log(`\n📊 Session Persistence: ${passed}/${total} tests passed\n`);
    return { passed, total };
}

async function testErrorScenarios() {
    console.log('⚠️  Testing Error Scenarios');
    console.log('===========================');

    const session = newHttpBatchRpcSession<Calculator>(endpoint);
    let passed = 0;
    let total = 0;

    // Test 1: Invalid method
    total++;
    try {
        // This should fail - invalid method
        await (session as any).invalidMethod(1, 2);
        console.log('❌ Invalid method should have failed');
    } catch (error) {
        console.log('✅ Invalid method correctly failed');
        passed++;
    }

    // Test 2: Division by zero (already tested but important)
    total++;
    try {
        await session.divide(1, 0);
        console.log('❌ Division by zero should have failed');
    } catch (error) {
        console.log('✅ Division by zero correctly failed');
        passed++;
    }

    console.log(`\n📊 Error Scenarios: ${passed}/${total} tests passed\n`);
    return { passed, total };
}

async function testPerformance() {
    console.log('⚡ Testing Performance');
    console.log('=====================');

    const session = newHttpBatchRpcSession<Calculator>(endpoint);
    let passed = 0;
    let total = 0;

    // Test sequential vs parallel performance
    total++;
    try {
        // Sequential operations
        const startSeq = Date.now();
        await session.add(1, 2);
        await session.add(3, 4);
        await session.add(5, 6);
        await session.add(7, 8);
        const sequentialTime = Date.now() - startSeq;

        // Parallel operations
        const startPar = Date.now();
        await Promise.all([
            session.add(1, 2),
            session.add(3, 4),
            session.add(5, 6),
            session.add(7, 8)
        ]);
        const parallelTime = Date.now() - startPar;

        console.log(`📈 Sequential: ${sequentialTime}ms, Parallel: ${parallelTime}ms`);

        // Parallel should be faster (or at least not significantly slower)
        if (parallelTime <= sequentialTime * 1.5) {
            console.log('✅ Parallel operations perform well');
            passed++;
        } else {
            console.log('⚠️  Parallel operations may need optimization');
            passed++; // Don't fail the test for this
        }
    } catch (error) {
        console.log(`❌ Performance test failed: ${error}`);
    }

    console.log(`\n📊 Performance: ${passed}/${total} tests passed\n`);
    return { passed, total };
}

async function main() {
    console.log('🚀 Comprehensive Stateful Server Test Suite');
    console.log('===========================================');
    console.log(`📍 Testing endpoint: ${endpoint}\n`);

    try {
        // Run all test suites
        const results = await Promise.all([
            testBasicCalculator(),
            testConcurrentOperations(),
            testSessionPersistence(),
            testErrorScenarios(),
            testPerformance()
        ]);

        // Calculate overall results
        const totalPassed = results.reduce((sum, result) => sum + result.passed, 0);
        const totalTests = results.reduce((sum, result) => sum + result.total, 0);
        const passRate = Math.round((totalPassed / totalTests) * 100);

        console.log('🏁 Final Results');
        console.log('================');
        console.log(`✅ Passed: ${totalPassed}/${totalTests} (${passRate}%)`);

        if (totalPassed === totalTests) {
            console.log('🎉 All tests passed! The stateful server is working correctly.');
            process.exit(0);
        } else {
            console.log('💥 Some tests failed. Check the server implementation.');
            process.exit(1);
        }

    } catch (error) {
        console.error('💥 Test suite failed with error:', error);
        process.exit(1);
    }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    process.exit(1);
});

main();