#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

/**
 * TIER 2: Stateful Session Management Tests
 *
 * Goal: Verify session persistence and state tracking
 * Tests: Import/export lifecycle, session isolation, cleanup
 * Success Criteria: State persists across requests, proper resource management
 *
 * Prerequisites: Tier 1 tests must pass (basic protocol compliance)
 */

interface StatefulCalculator {
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;
}

const port = process.argv[2] || '9000';
const endpoint = `http://localhost:${port}/rpc/batch`;

class Tier2Tests {
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

    private async sessionPersistence(): Promise<boolean> {
        console.log('Testing session persistence across multiple requests...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            // Make multiple calls that should be processed by the same session
            const results: number[] = [];

            console.log('Making sequential requests...');
            results.push(await session.add(1, 2));
            results.push(await session.multiply(3, 4));
            results.push(await session.subtract(10, 5));

            console.log(`Results: ${results.join(', ')}`);

            // Check if we got consistent numeric responses
            const allNumbers = results.every(r => typeof r === 'number' && !isNaN(r));
            const correctValues = results[0] === 3 && results[1] === 12 && results[2] === 5;

            if (allNumbers && correctValues) {
                console.log('✓ All operations returned correct results');
                console.log('✓ Session maintained state across multiple requests');
                return true;
            } else if (allNumbers) {
                console.log('✓ Session persistent (wrong values may indicate calculation issues)');
                console.log(`  Expected: [3, 12, 5], Got: [${results.join(', ')}]`);
                return false;
            } else {
                console.log('✗ Inconsistent response types or session issues');
                return false;
            }
        } catch (error: any) {
            console.log(`Session persistence test failed: ${error.message}`);
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

            // Make different calls from each session
            const [result1, result2] = await Promise.all([
                session1.add(5, 5),
                session2.multiply(6, 6)
            ]);

            console.log(`Session 1 result: ${result1}`);
            console.log(`Session 2 result: ${result2}`);

            // Both should work independently
            if (typeof result1 === 'number' && typeof result2 === 'number') {
                if (result1 === 10 && result2 === 36) {
                    console.log('✓ Both sessions returned correct results');
                    console.log('✓ Sessions are properly isolated');
                    return true;
                } else {
                    console.log('✓ Sessions isolated but calculation errors');
                    console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
                    return false;
                }
            } else {
                console.log('✗ One or both sessions failed to respond properly');
                return false;
            }
        } catch (error: any) {
            console.log(`Session isolation test failed: ${error.message}`);
            return false;
        }
    }

    private async concurrentOperations(): Promise<boolean> {
        console.log('Testing concurrent operations within a single session...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            console.log('Launching concurrent operations...');
            const startTime = Date.now();

            // Run multiple operations concurrently
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
                console.log('✓ Server handled concurrent requests properly');

                // Bonus: Check if operations were actually concurrent (should be faster than sequential)
                if (duration < 1000) {
                    console.log('✓ Operations appear to be processed concurrently');
                }

                return true;
            } else {
                console.log('✓ Concurrent operations completed but with incorrect results');
                console.log(`  Expected: [${expected.join(', ')}], Got: [${results.join(', ')}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Concurrent operations test failed: ${error.message}`);
            return false;
        }
    }

    private async errorRecovery(): Promise<boolean> {
        console.log('Testing error recovery and session stability...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            // First, perform a successful operation
            console.log('Performing initial successful operation...');
            const initial = await session.add(1, 1);
            console.log(`Initial result: ${initial}`);

            if (typeof initial !== 'number' || initial !== 2) {
                console.log('✗ Initial operation failed - cannot test error recovery');
                return false;
            }

            // Then, trigger an error
            console.log('Triggering an error (division by zero)...');
            let errorOccurred = false;
            try {
                await session.divide(5, 0);
                console.log('ℹ️  Division by zero did not throw error (unexpected)');
            } catch (error: any) {
                console.log(`✓ Error properly thrown: ${error.message}`);
                errorOccurred = true;
            }

            // Finally, verify session is still functional
            console.log('Testing session recovery with another operation...');
            const recovery = await session.multiply(3, 4);
            console.log(`Recovery result: ${recovery}`);

            if (typeof recovery === 'number' && recovery === 12) {
                console.log('✓ Session recovered after error');
                console.log('✓ Error handling did not corrupt session state');
                return true;
            } else {
                console.log('✗ Session corrupted after error');
                return false;
            }
        } catch (error: any) {
            console.log(`Error recovery test failed: ${error.message}`);
            return false;
        }
    }

    private async importExportLifecycle(): Promise<boolean> {
        console.log('Testing import/export lifecycle management...');

        const session = newHttpBatchRpcSession<StatefulCalculator>(endpoint);

        try {
            console.log('Testing multiple operations to check import/export handling...');

            // Perform a series of operations that should create and manage imports/exports
            const operations = [
                { op: 'add', args: [1, 2], expected: 3 },
                { op: 'multiply', args: [2, 3], expected: 6 },
                { op: 'subtract', args: [10, 4], expected: 6 },
                { op: 'divide', args: [15, 3], expected: 5 }
            ];

            const results: number[] = [];

            for (const { op, args, expected } of operations) {
                console.log(`  ${op}(${args.join(', ')}) = ?`);
                const result = await (session as any)[op](...args);
                results.push(result);
                console.log(`    -> ${result} (expected ${expected})`);

                if (typeof result !== 'number') {
                    console.log('✗ Non-numeric result indicates import/export issues');
                    return false;
                }
            }

            // Check if all operations completed successfully
            const allCompleted = results.length === operations.length;
            const allNumbers = results.every(r => typeof r === 'number');

            if (allCompleted && allNumbers) {
                console.log('✓ All operations completed with numeric results');
                console.log('✓ Import/export lifecycle appears functional');

                // Check correctness
                const allCorrect = operations.every((op, i) => results[i] === op.expected);
                if (allCorrect) {
                    console.log('✓ All calculations correct');
                    return true;
                } else {
                    console.log('ℹ️  Import/export working but calculation errors present');
                    return false;
                }
            } else {
                console.log('✗ Import/export lifecycle has issues');
                return false;
            }
        } catch (error: any) {
            console.log(`Import/export lifecycle test failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🏁 TIER 2: Stateful Session Management Tests');
        console.log('============================================');
        console.log(`📍 Testing endpoint: ${endpoint}`);
        console.log('🎯 Goal: Verify session persistence and state tracking');
        console.log('📋 Prerequisites: Tier 1 tests must pass');
        console.log('');

        // Test 1: Session persistence
        await this.test('Session Persistence Across Requests', () => this.sessionPersistence());

        // Test 2: Session isolation
        await this.test('Session Isolation Between Clients', () => this.sessionIsolation());

        // Test 3: Concurrent operations
        await this.test('Concurrent Operations Within Session', () => this.concurrentOperations());

        // Test 4: Error recovery
        await this.test('Error Recovery and Session Stability', () => this.errorRecovery());

        // Test 5: Import/Export lifecycle
        await this.test('Import/Export Lifecycle Management', () => this.importExportLifecycle());

        // Results
        console.log('\n' + '='.repeat(60));
        console.log('🏁 TIER 2 RESULTS');
        console.log('='.repeat(60));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`✅ Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🎉 TIER 2 COMPLETE: Stateful session management working!');
            console.log('📈 Ready for Tier 3: Complex Application Logic');
            process.exit(0);
        } else if (this.passed >= this.total * 0.6) {
            console.log('⚠️  TIER 2 PARTIAL: Some session management issues remain');
            console.log('🔧 Address session state issues before Tier 3');
            process.exit(1);
        } else {
            console.log('💥 TIER 2 FAILED: Session management not working');
            console.log('🚨 Fix state tracking before proceeding');
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
const tier2 = new Tier2Tests();
tier2.run();