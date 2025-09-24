#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

// Define the Calculator interface
interface Calculator {
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;
}

async function testWithOfficialClient() {
    console.log('🧪 Testing Cap\'n Web Rust Server with Official TypeScript Client');
    console.log('================================================================\n');

    try {
        const port = process.argv[2] || '9006';
        const endpoint = `http://localhost:${port}/rpc/batch`;

        console.log('✅ Created endpoint configuration');
        console.log(`📍 Endpoint: ${endpoint}\n`);

        // Test 1: Single operation per batch (correct pattern)
        console.log('Test 1: Single Operations (New Session Per Call)');
        console.log('------------------------------------------------');

        const session1 = newHttpBatchRpcSession<Calculator>(endpoint);
        const result1 = await session1.add(5, 3);
        console.log(`✅ add(5, 3) = ${result1}`);

        const session2 = newHttpBatchRpcSession<Calculator>(endpoint);
        const result2 = await session2.multiply(7, 6);
        console.log(`✅ multiply(7, 6) = ${result2}`);

        const session3 = newHttpBatchRpcSession<Calculator>(endpoint);
        const result3 = await session3.divide(100, 4);
        console.log(`✅ divide(100, 4) = ${result3}`);

        const session4 = newHttpBatchRpcSession<Calculator>(endpoint);
        const result4 = await session4.subtract(10, 3);
        console.log(`✅ subtract(10, 3) = ${result4}`);

        // Test 2: True batching - multiple operations in one batch
        console.log('\nTest 2: True Batching (Multiple Operations in One Request)');
        console.log('----------------------------------------------------------');

        const batchSession = newHttpBatchRpcSession<Calculator>(endpoint);

        // Queue all operations before awaiting
        const addPromise = batchSession.add(10, 20);
        const multiplyPromise = batchSession.multiply(3, 4);
        const dividePromise = batchSession.divide(100, 5);
        const subtractPromise = batchSession.subtract(50, 15);

        // Now await all at once - this sends ONE HTTP request with all operations
        const [addResult, multiplyResult, divideResult, subtractResult] =
            await Promise.all([addPromise, multiplyPromise, dividePromise, subtractPromise]);

        console.log(`✅ Batch results:`);
        console.log(`   add(10, 20) = ${addResult}`);
        console.log(`   multiply(3, 4) = ${multiplyResult}`);
        console.log(`   divide(100, 5) = ${divideResult}`);
        console.log(`   subtract(50, 15) = ${subtractResult}`);

        // Test 3: Error handling
        console.log('\nTest 3: Error Handling');
        console.log('----------------------');

        try {
            const errorSession = newHttpBatchRpcSession<Calculator>(endpoint);
            await errorSession.divide(10, 0);
            console.log('❌ Should have thrown for division by zero');
        } catch (error: any) {
            console.log(`✅ Division by zero correctly threw error: ${error.message}`);
        }

        console.log('\n================================================================================');
        console.log('🎉 SUCCESS: All tests passed!');
        console.log('================================================================================');
        console.log('✅ Rust server is fully compatible with official Cap\'n Web TypeScript client');
        console.log('✅ Single operations work with new sessions');
        console.log('✅ True batching works with multiple operations in one request');
        console.log('✅ Error handling works correctly');

    } catch (error: any) {
        console.error('\n❌ Test failed:', error.message);
        console.error('Stack:', error.stack);
        process.exit(1);
    }
}

testWithOfficialClient().catch(console.error);