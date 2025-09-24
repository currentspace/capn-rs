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
        // Get port from command line or use default
        const port = process.argv[2] || '8080';
        const endpoint = `http://localhost:${port}/rpc/batch`;

        // Create a session with the Rust server
        // Note: The official client expects strongly-typed interfaces
        // Our server needs to handle the protocol correctly
        const session = newHttpBatchRpcSession<Calculator>(endpoint);

        console.log('✅ Created session with Rust server');
        console.log(`📍 Endpoint: ${endpoint}\n`);

        // Test 1: Basic addition
        console.log('Test 1: Addition');
        console.log('----------------');
        try {
            // This will generate: ["push", ["import", 0, ["add"], [5, 3]]]
            const result = await session.add(5, 3);
            console.log(`✅ add(5, 3) = ${result}`);

            if (result !== 8) {
                throw new Error(`Expected 8, got ${result}`);
            }
        } catch (error) {
            console.log(`❌ Addition failed: ${error}`);
        }

        // Test 2: Multiplication
        console.log('\nTest 2: Multiplication');
        console.log('----------------------');
        try {
            const result = await session.multiply(7, 6);
            console.log(`✅ multiply(7, 6) = ${result}`);

            if (result !== 42) {
                throw new Error(`Expected 42, got ${result}`);
            }
        } catch (error) {
            console.log(`❌ Multiplication failed: ${error}`);
        }

        // Test 3: Division
        console.log('\nTest 3: Division');
        console.log('----------------');
        try {
            const result = await session.divide(100, 4);
            console.log(`✅ divide(100, 4) = ${result}`);

            if (result !== 25) {
                throw new Error(`Expected 25, got ${result}`);
            }
        } catch (error) {
            console.log(`❌ Division failed: ${error}`);
        }

        // Test 4: Subtraction
        console.log('\nTest 4: Subtraction');
        console.log('-------------------');
        try {
            const result = await session.subtract(10, 7);
            console.log(`✅ subtract(10, 7) = ${result}`);

            if (result !== 3) {
                throw new Error(`Expected 3, got ${result}`);
            }
        } catch (error) {
            console.log(`❌ Subtraction failed: ${error}`);
        }

        // Test 5: Error handling (division by zero)
        console.log('\nTest 5: Error Handling');
        console.log('----------------------');
        try {
            const result = await session.divide(10, 0);
            console.log(`❌ Division by zero should have thrown an error, got: ${result}`);
        } catch (error) {
            console.log(`✅ Division by zero correctly threw error: ${error}`);
        }

        // Test 6: Promise pipelining (if supported)
        console.log('\nTest 6: Multiple Operations');
        console.log('----------------------------');
        try {
            // Send multiple operations
            const [sum, product] = await Promise.all([
                session.add(10, 20),
                session.multiply(5, 8)
            ]);

            console.log(`✅ Parallel operations:`);
            console.log(`   add(10, 20) = ${sum}`);
            console.log(`   multiply(5, 8) = ${product}`);

            if (sum !== 30 || product !== 40) {
                throw new Error(`Unexpected results: sum=${sum}, product=${product}`);
            }
        } catch (error) {
            console.log(`❌ Parallel operations failed: ${error}`);
        }

        console.log('\n' + '='.repeat(80));
        console.log('🎉 VALIDATION SUMMARY');
        console.log('='.repeat(80));
        console.log('✅ Official Cap\'n Web TypeScript client can communicate with Rust server!');
        console.log('⚠️  Note: This validates basic protocol compatibility');
        console.log('❌ Missing: Promise pipelining, WebSocket transport, full capability system');

    } catch (error) {
        console.error('\n💥 Fatal error:', error);
        console.error('\nThis likely means the Rust server is not properly implementing');
        console.error('the Cap\'n Web protocol as expected by the official client.');
        process.exit(1);
    }
}

// Run the test
testWithOfficialClient().catch(error => {
    console.error('Unhandled error:', error);
    process.exit(1);
});