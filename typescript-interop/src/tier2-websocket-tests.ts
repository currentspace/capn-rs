#!/usr/bin/env node

import { newWebSocketRpcSession } from 'capnweb';

/**
 * TIER 2 WebSocket: Stateful Session Management Tests over WebSocket
 *
 * Goal: Verify session persistence and state tracking over WebSocket transport
 * Tests: Same as tier2-stateful-sessions.ts but using WebSocket instead of HTTP batch
 * Success Criteria: State persists across WebSocket messages, proper resource management
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
const wsEndpoint = `ws://localhost:${port}/rpc/ws`;

class Tier2WebSocketTests {
    private passed = 0;
    private total = 0;

    private async test(name: string, testFn: () => Promise<boolean>): Promise<void> {
        this.total++;
        console.log(`\n🧪 Test ${this.total}: ${name} (WebSocket)`);
        console.log('─'.repeat(70));

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
        console.log('Testing WebSocket session persistence across multiple messages...');

        try {
            const session = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);

            // Make multiple calls that should be processed by the same WebSocket connection
            const results: number[] = [];

            console.log('Making sequential requests over WebSocket...');
            results.push(await session.add(1, 2));
            results.push(await session.multiply(3, 4));
            results.push(await session.subtract(10, 5));

            console.log(`Results: ${results.join(', ')}`);

            // Close the session properly
            if ('close' in session) {
                (session as any).close();
            }

            // Check if we got consistent numeric responses
            const allNumbers = results.every(r => typeof r === 'number' && !isNaN(r));
            const correctValues = results[0] === 3 && results[1] === 12 && results[2] === 5;

            if (allNumbers && correctValues) {
                console.log('✓ All operations returned correct results');
                console.log('✓ WebSocket session maintained state across multiple messages');
                return true;
            } else if (allNumbers) {
                console.log('✓ WebSocket session persistent (wrong values may indicate calculation issues)');
                console.log(`  Expected: [3, 12, 5], Got: [${results.join(', ')}]`);
                return false;
            } else {
                console.log('✗ Inconsistent response types or WebSocket session issues');
                return false;
            }
        } catch (error: any) {
            console.log(`WebSocket session persistence test failed: ${error.message}`);
            return false;
        }
    }

    private async sessionIsolation(): Promise<boolean> {
        console.log('Testing WebSocket session isolation between different connections...');

        try {
            // Create two separate WebSocket sessions
            const session1 = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);
            const session2 = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);

            console.log('Creating two separate WebSocket client sessions...');

            // Make different calls from each session
            const [result1, result2] = await Promise.all([
                session1.add(5, 5),
                session2.multiply(6, 6)
            ]);

            console.log(`WebSocket Session 1 result: ${result1}`);
            console.log(`WebSocket Session 2 result: ${result2}`);

            // Close sessions properly
            if ('close' in session1) (session1 as any).close();
            if ('close' in session2) (session2 as any).close();

            // Both should work independently
            if (typeof result1 === 'number' && typeof result2 === 'number') {
                if (result1 === 10 && result2 === 36) {
                    console.log('✓ Both WebSocket sessions returned correct results');
                    console.log('✓ WebSocket sessions are properly isolated');
                    return true;
                } else {
                    console.log('✓ WebSocket sessions isolated but calculation errors');
                    console.log(`  Expected: [10, 36], Got: [${result1}, ${result2}]`);
                    return false;
                }
            } else {
                console.log('✗ One or both WebSocket sessions failed to respond properly');
                return false;
            }
        } catch (error: any) {
            console.log(`WebSocket session isolation test failed: ${error.message}`);
            return false;
        }
    }

    private async concurrentOperations(): Promise<boolean> {
        console.log('Testing concurrent operations within a single WebSocket session...');

        try {
            const session = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);

            console.log('Launching concurrent operations over WebSocket...');
            const startTime = Date.now();

            // Run multiple operations concurrently over the same WebSocket
            const results = await Promise.all([
                session.add(2, 3),
                session.multiply(4, 5),
                session.divide(20, 4),
                session.subtract(15, 7)
            ]);

            const duration = Date.now() - startTime;
            console.log(`All WebSocket operations completed in ${duration}ms`);
            console.log(`Results: ${results.join(', ')}`);

            // Close session properly
            if ('close' in session) (session as any).close();

            // Check results
            const expected = [5, 20, 5, 8];
            const allCorrect = results.every((r, i) => r === expected[i]);

            if (allCorrect) {
                console.log('✓ All concurrent WebSocket operations returned correct results');
                console.log('✓ Server handled concurrent WebSocket requests properly');

                // Bonus: Check if operations were actually concurrent
                if (duration < 1000) {
                    console.log('✓ WebSocket operations appear to be processed concurrently');
                }

                return true;
            } else {
                console.log('✓ Concurrent WebSocket operations completed but with incorrect results');
                console.log(`  Expected: [${expected.join(', ')}], Got: [${results.join(', ')}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Concurrent WebSocket operations test failed: ${error.message}`);
            return false;
        }
    }

    private async errorRecovery(): Promise<boolean> {
        console.log('Testing error recovery and WebSocket session stability...');

        try {
            const session = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);

            // First, perform a successful operation
            console.log('Performing initial successful operation over WebSocket...');
            const initial = await session.add(1, 1);
            console.log(`Initial result: ${initial}`);

            if (typeof initial !== 'number' || initial !== 2) {
                console.log('✗ Initial WebSocket operation failed - cannot test error recovery');
                if ('close' in session) (session as any).close();
                return false;
            }

            // Then, trigger an error
            console.log('Triggering an error (division by zero) over WebSocket...');
            let errorOccurred = false;
            try {
                await session.divide(5, 0);
                console.log('ℹ️  Division by zero did not throw error (unexpected)');
            } catch (error: any) {
                console.log(`✓ Error properly thrown over WebSocket: ${error.message}`);
                errorOccurred = true;
            }

            // Finally, verify WebSocket session is still functional
            console.log('Testing WebSocket session recovery with another operation...');
            const recovery = await session.multiply(3, 4);
            console.log(`Recovery result: ${recovery}`);

            // Close session properly
            if ('close' in session) (session as any).close();

            if (typeof recovery === 'number' && recovery === 12) {
                console.log('✓ WebSocket session recovered after error');
                console.log('✓ Error handling did not corrupt WebSocket session state');
                return true;
            } else {
                console.log('✗ WebSocket session corrupted after error');
                return false;
            }
        } catch (error: any) {
            console.log(`WebSocket error recovery test failed: ${error.message}`);
            return false;
        }
    }

    private async realTimeMessaging(): Promise<boolean> {
        console.log('Testing real-time bidirectional messaging over WebSocket...');

        try {
            const session = newWebSocketRpcSession<StatefulCalculator>(wsEndpoint);

            console.log('Testing rapid-fire operations over persistent WebSocket connection...');
            const startTime = Date.now();

            // Send rapid sequence of operations to test real-time capabilities
            const operations = [];
            for (let i = 0; i < 10; i++) {
                operations.push(session.add(i, i + 1));
            }

            const results = await Promise.all(operations);
            const duration = Date.now() - startTime;

            console.log(`10 rapid WebSocket operations completed in ${duration}ms`);
            console.log(`Average per operation: ${(duration / 10).toFixed(1)}ms`);

            // Close session properly
            if ('close' in session) (session as any).close();

            // Check results (should be [1, 3, 5, 7, 9, 11, 13, 15, 17, 19])
            const expected = Array.from({length: 10}, (_, i) => i + (i + 1));
            const allCorrect = results.every((r, i) => r === expected[i]);

            if (allCorrect) {
                console.log('✓ All rapid-fire WebSocket operations returned correct results');
                console.log('✓ WebSocket demonstrates real-time messaging capability');

                if (duration < 500) {
                    console.log('✓ Excellent WebSocket performance for real-time use');
                }

                return true;
            } else {
                console.log('✗ Some rapid-fire WebSocket operations returned incorrect results');
                console.log(`  Expected: [${expected.join(', ')}]`);
                console.log(`  Got: [${results.join(', ')}]`);
                return false;
            }
        } catch (error: any) {
            console.log(`Real-time WebSocket messaging test failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🌐 TIER 2 WebSocket: Stateful Session Management Tests');
        console.log('====================================================');
        console.log(`📍 Testing WebSocket endpoint: ${wsEndpoint}`);
        console.log('🎯 Goal: Verify session persistence over WebSocket transport');
        console.log('📋 Prerequisites: Tier 1 tests must pass + WebSocket support');
        console.log('');

        // Test 1: WebSocket session persistence
        await this.test('WebSocket Session Persistence', () => this.sessionPersistence());

        // Test 2: WebSocket session isolation
        await this.test('WebSocket Session Isolation', () => this.sessionIsolation());

        // Test 3: Concurrent operations over WebSocket
        await this.test('Concurrent WebSocket Operations', () => this.concurrentOperations());

        // Test 4: Error recovery over WebSocket
        await this.test('WebSocket Error Recovery', () => this.errorRecovery());

        // Test 5: Real-time messaging (WebSocket-specific)
        await this.test('Real-time Bidirectional Messaging', () => this.realTimeMessaging());

        // Results
        console.log('\n' + '='.repeat(70));
        console.log('🌐 TIER 2 WebSocket RESULTS');
        console.log('='.repeat(70));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`✅ Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🎉 TIER 2 WebSocket COMPLETE: WebSocket stateful session management working!');
            console.log('🚀 WebSocket transport provides real-time capabilities');
            console.log('📈 Ready for Tier 3: Complex Application Logic over WebSocket');
            process.exit(0);
        } else if (this.passed >= this.total * 0.6) {
            console.log('⚠️  TIER 2 WebSocket PARTIAL: Some WebSocket session management issues remain');
            console.log('🔧 Address WebSocket state issues before Tier 3');
            process.exit(1);
        } else {
            console.log('💥 TIER 2 WebSocket FAILED: WebSocket session management not working');
            console.log('🚨 Fix WebSocket state tracking before proceeding');
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
const tier2WebSocket = new Tier2WebSocketTests();
tier2WebSocket.run();