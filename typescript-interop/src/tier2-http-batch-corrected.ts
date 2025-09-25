#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

/**
 * TIER 2: HTTP Batch-Appropriate Tests (Corrected)
 *
 * Goal: Test batch semantics correctly for HTTP transport
 * Tests: Batch operations, session isolation, error handling
 * Success Criteria: Operations work within single batch constraints
 *
 * NOTE: HTTP batch sessions END after sending their batch.
 * Sequential operations require new sessions or Promise.all()
 */

interface StatefulCalculator {
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;
}

const port = process.argv[2] || '9000';
const endpoint = `http://localhost:${port}/rpc/batch`;

class Tier2BatchTests {
    private passed = 0;
    private total = 0;

    private async test(name: string, testFn: () => Promise<boolean>): Promise<void> {
        this.total++;
        console.log(`\n🧪 Test ${this.total}: ${name}`);
        console.log('─'.repeat(60));

        try {
            const result = await testFn();
            if (result) {
                this.passed++;
                console.log('✅ PASSED');
            } else {
                console.log('❌ FAILED');
            }
        } catch (error: any) {
            console.log(`❌ FAILED: ${error.message}`);
        }
    }

    private async batchOperations(): Promise<boolean> {
        console.log('Testing batch operations (all in single request)...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            console.log('Sending all operations in a single batch...');

            // Use Promise.all to batch all operations together
            const results = await Promise.all([
                session.add(1, 2),
                session.multiply(3, 4),
                session.subtract(10, 5)
            ]);

            console.log(`Results: ${results.join(', ')}`);

            // Check if we got correct results
            const expected = [3, 12, 5];
            const allCorrect = results.every((r, i) => r === expected[i]);

            if (allCorrect) {
                console.log('✓ All batch operations returned correct results');
                console.log('✓ Batch processing working correctly');
                return true;
            } else {
                console.log('✗ Batch operations returned incorrect results');
                console.log(`  Expected: [${expected.join(', ')}], Got: [${results.join(', ')}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Batch operations test failed: ${error.message}`);
            return false;
        }
    }

    private async sessionIsolation(): Promise<boolean> {
        console.log('Testing session isolation between different clients...');

        try {
            // Create two separate sessions
            const session1 = newHttpBatchRpcSession<StatefulCalculator>(endpoint);
            const session2 = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

            console.log('Creating two separate client sessions...');

            // Each session sends its own batch
            const [result1, result2] = await Promise.all([
                session1.add(5, 5),
                session2.multiply(6, 6)
            ]);

            console.log(`Session 1 result: ${result1}`);
            console.log(`Session 2 result: ${result2}`);

            if (result1 === 10 && result2 === 36) {
                console.log('✓ Both sessions returned correct results');
                console.log('✓ Sessions are properly isolated');
                return true;
            } else {
                console.log('✗ Incorrect results from isolated sessions');
                console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Session isolation test failed: ${error.message}`);
            return false;
        }
    }

    private async concurrentBatchOperations(): Promise<boolean> {
        console.log('Testing concurrent operations within a single batch...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            console.log('Launching concurrent operations in single batch...');
            const startTime = Date.now();

            // Run multiple operations concurrently in one batch
            const results = await Promise.all([
                session.add(2, 3),
                session.multiply(4, 5),
                session.divide(20, 4),
                session.subtract(15, 7)
            ]);

            const duration = Date.now() - startTime;
            console.log(`All operations completed in ${duration}ms`);
            console.log(`Results: ${results.join(', ')}`);

            // Check results
            const expected = [5, 20, 5, 8];
            const allCorrect = results.every((r, i) => r === expected[i]);

            if (allCorrect) {
                console.log('✓ All concurrent operations returned correct results');
                console.log('✓ Server handled batch request properly');
                return true;
            } else {
                console.log('✗ Batch operations returned incorrect results');
                console.log(`  Expected: [${expected.join(', ')}], Got: [${results.join(', ')}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Concurrent batch operations test failed: ${error.message}`);
            return false;
        }
    }

    private async errorHandlingInBatch(): Promise<boolean> {
        console.log('Testing error handling within batch...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            console.log('Sending batch with valid and error-triggering operations...');

            // Send a batch with mixed operations
            const results = await Promise.allSettled([
                session.add(1, 1),
                session.divide(5, 0),  // This should error
                session.multiply(3, 4)
            ]);

            console.log('Batch completed, analyzing results...');

            // Check that we got expected outcomes
            const successCount = results.filter(r => r.status === 'fulfilled').length;
            const errorCount = results.filter(r => r.status === 'rejected').length;

            console.log(`Success: ${successCount}, Errors: ${errorCount}`);

            // First operation should succeed
            if (results[0].status === 'fulfilled' && (results[0] as any).value === 2) {
                console.log('✓ First operation succeeded as expected');
            }

            // Second operation should error
            if (results[1].status === 'rejected') {
                console.log('✓ Division by zero properly rejected');
            }

            // Third operation should succeed despite error in batch
            if (results[2].status === 'fulfilled' && (results[2] as any).value === 12) {
                console.log('✓ Third operation succeeded despite error in batch');
            }

            // If we handled both success and error cases properly
            if (successCount === 2 && errorCount === 1) {
                console.log('✓ Batch properly handled mixed success/error cases');
                return true;
            } else {
                console.log('✗ Unexpected batch error handling behavior');
                return false;
            }
        } catch (error: any) {
            console.log(`Error handling test failed: ${error.message}`);
            return false;
        }
    }

    private async multipleBatchRequests(): Promise<boolean> {
        console.log('Testing multiple batch requests (new sessions)...');

        try {
            console.log('Creating new session for each batch...');

            // Each batch needs a new session
            const batch1 = newHttpBatchRpcSession<StatefulCalculator>(endpoint);
            const result1 = await Promise.all([
                batch1.add(1, 2),
                batch1.multiply(2, 3)
            ]);
            console.log(`Batch 1 results: ${result1.join(', ')}`);

            const batch2 = newHttpBatchRpcSession<StatefulCalculator>(endpoint);
            const result2 = await Promise.all([
                batch2.subtract(10, 4),
                batch2.divide(15, 3)
            ]);
            console.log(`Batch 2 results: ${result2.join(', ')}`);

            const batch3 = newHttpBatchRpcSession<StatefulCalculator>(endpoint);
            const result3 = await batch3.add(100, 200);
            console.log(`Batch 3 result: ${result3}`);

            // Check all results
            const allCorrect =
                result1[0] === 3 && result1[1] === 6 &&
                result2[0] === 6 && result2[1] === 5 &&
                result3 === 300;

            if (allCorrect) {
                console.log('✓ All batches processed correctly');
                console.log('✓ Multiple sequential batches work with new sessions');
                return true;
            } else {
                console.log('✗ Some batch results incorrect');
                return false;
            }
        } catch (error: any) {
            console.log(`Multiple batch requests test failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🏁 TIER 2: HTTP Batch Transport Tests (Corrected)');
        console.log('==================================================');
        console.log(`📍 Testing endpoint: ${endpoint}`);
        console.log('🎯 Goal: Verify proper HTTP batch semantics');
        console.log('⚠️  Note: HTTP batch sessions END after sending');
        console.log('');

        // Test 1: Batch operations in single request
        await this.test('Batch Operations', () => this.batchOperations());

        // Test 2: Session isolation (each gets own batch)
        await this.test('Session Isolation', () => this.sessionIsolation());

        // Test 3: Concurrent operations in single batch
        await this.test('Concurrent Batch Operations', () => this.concurrentBatchOperations());

        // Test 4: Error handling within batch
        await this.test('Error Handling in Batch', () => this.errorHandlingInBatch());

        // Test 5: Multiple sequential batches (new sessions)
        await this.test('Multiple Batch Requests', () => this.multipleBatchRequests());

        // Results
        console.log('\n' + '='.repeat(60));
        console.log('🏁 TIER 2 (HTTP BATCH) RESULTS');
        console.log('='.repeat(60));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`✅ Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🎉 TIER 2 COMPLETE: HTTP batch semantics working correctly!');
            console.log('📈 For persistent sessions, use WebSocket transport');
            process.exit(0);
        } else if (this.passed >= this.total * 0.6) {
            console.log('⚠️  TIER 2 PARTIAL: Some batch handling issues');
            process.exit(1);
        } else {
            console.log('💥 TIER 2 FAILED: Batch handling not working');
            process.exit(2);
        }
    }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    process.exit(3);
});

// Run tests
const tier2 = new Tier2BatchTests();
tier2.run();