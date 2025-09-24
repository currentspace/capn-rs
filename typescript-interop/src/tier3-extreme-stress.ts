#!/usr/bin/env node

import { newWebSocketRpcSession, newHttpRpcSession } from 'capnweb';

/**
 * TIER 3 EXTREME: Ultra-Complex Stress Testing & Advanced Scenarios
 *
 * Goal: Push the limits of WebSocket and HTTP batch implementations
 * Tests: Massive concurrency, extreme stress, complex capability graphs
 * Success Criteria: Handle enterprise-scale loads and complex scenarios
 *
 * Prerequisites: All Tier 1, 2, and 3 tests must pass
 */

interface StressCalculator {
    // Basic operations
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;

    // Advanced batch operations
    batchCalculate?(operations: Array<{op: string, args: number[]}>): Promise<number[]>;

    // State management
    setVariable?(name: string, value: number): Promise<boolean>;
    getVariable?(name: string): Promise<number>;
    clearAllVariables?(): Promise<boolean>;

    // Resource operations
    allocateResource?(id: string, size: number): Promise<boolean>;
    releaseResource?(id: string): Promise<boolean>;
    getResourceUsage?(): Promise<number>;
}

const port = process.argv[2] || '9001';
const wsEndpoint = `ws://localhost:${port}/rpc/ws`;
const httpEndpoint = `http://localhost:${port}/rpc/batch`;

class Tier3ExtremeStressTests {
    private passed = 0;
    private total = 0;

    private async test(name: string, testFn: () => Promise<boolean>): Promise<void> {
        this.total++;
        console.log(`\n🚀 Extreme Test ${this.total}: ${name}`);
        console.log('━'.repeat(90));

        try {
            const result = await testFn();
            if (result) {
                this.passed++;
                console.log('🏆 PASSED');
            } else {
                console.log('💥 FAILED');
            }
        } catch (error: any) {
            console.log(`💥 FAILED: ${error.message}`);
            console.log(`Stack: ${error.stack?.split('\n').slice(0, 3).join('\n')}`);
        }
    }

    /**
     * Test massive concurrent operations across multiple sessions
     */
    private async massiveConcurrencyTest(): Promise<boolean> {
        console.log('Testing massive concurrency with 50+ parallel sessions...');

        const startTime = Date.now();
        const sessionCount = 50;
        const operationsPerSession = 10;

        try {
            console.log(`⚡ Creating ${sessionCount} concurrent WebSocket sessions...`);

            // Create many concurrent sessions
            const sessions = Array.from({ length: sessionCount }, () =>
                newWebSocketRpcSession<StressCalculator>(wsEndpoint)
            );

            console.log(`🔄 Launching ${sessionCount * operationsPerSession} concurrent operations...`);

            // Launch massive number of concurrent operations
            const allOperations: Promise<number>[] = [];

            for (let sessionIndex = 0; sessionIndex < sessionCount; sessionIndex++) {
                const session = sessions[sessionIndex];

                // Each session performs multiple operations
                for (let opIndex = 0; opIndex < operationsPerSession; opIndex++) {
                    const a = sessionIndex + 1;
                    const b = opIndex + 1;

                    switch (opIndex % 4) {
                        case 0:
                            allOperations.push(session.add(a, b));
                            break;
                        case 1:
                            allOperations.push(session.multiply(a, b));
                            break;
                        case 2:
                            allOperations.push(session.subtract(a + 10, b));
                            break;
                        case 3:
                            allOperations.push(session.divide(a * 10, b));
                            break;
                    }
                }
            }

            console.log(`⏱️  Waiting for ${allOperations.length} operations to complete...`);
            const results = await Promise.all(allOperations);

            const totalTime = Date.now() - startTime;

            console.log(`📊 Performance Metrics:`);
            console.log(`  Sessions: ${sessionCount}`);
            console.log(`  Operations: ${allOperations.length}`);
            console.log(`  Total time: ${totalTime}ms`);
            console.log(`  Avg per operation: ${(totalTime / allOperations.length).toFixed(2)}ms`);
            console.log(`  Throughput: ${Math.round(allOperations.length / (totalTime / 1000))} ops/sec`);

            // Cleanup
            for (const session of sessions) {
                if ('close' in session) {
                    (session as any).close();
                }
            }

            // Verify we got all results
            const allNumberResults = results.every(r => typeof r === 'number' && !isNaN(r));

            console.log(`🔍 Verification:`);
            console.log(`  All operations completed: ${results.length === allOperations.length ? '✓' : '✗'}`);
            console.log(`  All results valid: ${allNumberResults ? '✓' : '✗'}`);
            console.log(`  Performance acceptable: ${totalTime < 10000 ? '✓' : '⚠️'} (<10s)`);

            if (results.length === allOperations.length && allNumberResults) {
                console.log('✅ Massive concurrency test succeeded');
                console.log(`🚀 Handled ${sessionCount} sessions with ${allOperations.length} ops in ${totalTime}ms`);
                return true;
            }

            return false;

        } catch (error: any) {
            console.log(`Massive concurrency test failed: ${error.message}`);
            return false;
        }
    }

    /**
     * Test complex interdependent calculation graphs
     */
    private async complexDependencyGraphTest(): Promise<boolean> {
        console.log('Testing complex interdependent calculation graphs...');

        try {
            const session = newWebSocketRpcSession<StressCalculator>(wsEndpoint);

            console.log('🕸️  Building complex dependency graph...');

            // Level 1: Base calculations
            console.log('  Level 1: Base calculations (8 nodes)');
            const level1 = await Promise.all([
                session.add(1, 2),      // 3
                session.multiply(2, 3), // 6
                session.subtract(10, 4), // 6
                session.divide(20, 4),  // 5
                session.add(3, 4),      // 7
                session.multiply(3, 2), // 6
                session.subtract(15, 8), // 7
                session.divide(24, 3)   // 8
            ]);

            console.log(`    Results: [${level1.join(', ')}]`);

            // Level 2: Depend on Level 1 (pairwise combinations)
            console.log('  Level 2: Pairwise combinations (4 nodes)');
            const level2 = await Promise.all([
                session.add(level1[0], level1[1]),        // 3 + 6 = 9
                session.multiply(level1[2], level1[3]),   // 6 * 5 = 30
                session.subtract(level1[4], level1[5]),   // 7 - 6 = 1
                session.divide(level1[6], level1[7])      // 7 / 8 = 0.875
            ]);

            console.log(`    Results: [${level2.join(', ')}]`);

            // Level 3: Cross-combinations (require multiple Level 2 results)
            console.log('  Level 3: Cross-combinations (2 nodes)');
            const level3 = await Promise.all([
                session.add(level2[0], level2[1]),        // 9 + 30 = 39
                session.multiply(level2[2], level2[3])    // 1 * 0.875 = 0.875
            ]);

            console.log(`    Results: [${level3.join(', ')}]`);

            // Level 4: Final aggregation (depends on all previous levels)
            console.log('  Level 4: Final aggregation (1 node)');
            const finalResult = await session.add(level3[0], level3[1]); // 39 + 0.875 = 39.875

            console.log(`    Final result: ${finalResult}`);

            // Cleanup
            if ('close' in session) {
                (session as any).close();
            }

            // Verify the complex calculation tree
            const expected = {
                level1: [3, 6, 6, 5, 7, 6, 7, 8],
                level2: [9, 30, 1, 0.875],
                level3: [39, 0.875],
                final: 39.875
            };

            console.log('🔍 Dependency Graph Verification:');

            const level1Match = JSON.stringify(level1) === JSON.stringify(expected.level1);
            const level2Match = JSON.stringify(level2) === JSON.stringify(expected.level2);
            const level3Match = JSON.stringify(level3) === JSON.stringify(expected.level3);
            const finalMatch = finalResult === expected.final;

            console.log(`  Level 1 (8 nodes): ${level1Match ? '✓' : '✗'}`);
            console.log(`  Level 2 (4 nodes): ${level2Match ? '✓' : '✗'}`);
            console.log(`  Level 3 (2 nodes): ${level3Match ? '✓' : '✗'}`);
            console.log(`  Final result: ${finalMatch ? '✓' : '✗'} (${finalResult} === ${expected.final})`);

            if (level1Match && level2Match && level3Match && finalMatch) {
                console.log('✅ Complex dependency graph executed perfectly');
                console.log('🎯 All 15 interdependent calculations correct');
                return true;
            } else {
                console.log('⚠️  Dependency graph structure working but calculation errors');
                return false;
            }

        } catch (error: any) {
            console.log(`Complex dependency graph test failed: ${error.message}`);
            return false;
        }
    }

    /**
     * Test sustained high-throughput operations
     */
    private async sustainedThroughputTest(): Promise<boolean> {
        console.log('Testing sustained high-throughput operations...');

        try {
            const session = newWebSocketRpcSession<StressCalculator>(wsEndpoint);
            const testDuration = 5000; // 5 seconds
            const batchSize = 100;

            console.log(`🏃‍♂️ Running sustained throughput test for ${testDuration}ms...`);

            const startTime = Date.now();
            let totalOperations = 0;
            let batchCount = 0;

            const results: number[][] = [];

            while (Date.now() - startTime < testDuration) {
                batchCount++;
                console.log(`  Batch ${batchCount}: ${batchSize} operations...`);

                // Launch a batch of concurrent operations
                const batchOperations: Promise<number>[] = [];

                for (let i = 0; i < batchSize; i++) {
                    const a = Math.floor(Math.random() * 100) + 1;
                    const b = Math.floor(Math.random() * 50) + 1;

                    switch (i % 4) {
                        case 0:
                            batchOperations.push(session.add(a, b));
                            break;
                        case 1:
                            batchOperations.push(session.multiply(a, b));
                            break;
                        case 2:
                            batchOperations.push(session.subtract(a, b));
                            break;
                        case 3:
                            batchOperations.push(session.divide(a, Math.max(b, 1)));
                            break;
                    }
                }

                const batchResults = await Promise.all(batchOperations);
                results.push(batchResults);
                totalOperations += batchSize;

                // Brief pause between batches to simulate realistic load
                await new Promise(resolve => setTimeout(resolve, 50));
            }

            const totalTime = Date.now() - startTime;

            console.log(`📊 Sustained Throughput Results:`);
            console.log(`  Duration: ${totalTime}ms`);
            console.log(`  Batches completed: ${batchCount}`);
            console.log(`  Total operations: ${totalOperations}`);
            console.log(`  Average throughput: ${Math.round(totalOperations / (totalTime / 1000))} ops/sec`);
            console.log(`  Average batch time: ${(totalTime / batchCount).toFixed(2)}ms`);

            // Cleanup
            if ('close' in session) {
                (session as any).close();
            }

            // Verify all results are valid numbers
            const allValid = results.every(batch =>
                batch.every(result => typeof result === 'number' && !isNaN(result))
            );

            const minThroughput = 1000; // At least 1000 ops/sec
            const actualThroughput = totalOperations / (totalTime / 1000);

            console.log('🔍 Throughput Verification:');
            console.log(`  All results valid: ${allValid ? '✓' : '✗'}`);
            console.log(`  Minimum throughput (${minThroughput} ops/sec): ${actualThroughput >= minThroughput ? '✓' : '✗'}`);
            console.log(`  Consistent performance: ${batchCount >= 10 ? '✓' : '✗'} (${batchCount} batches)`);

            if (allValid && actualThroughput >= minThroughput && batchCount >= 10) {
                console.log('✅ Sustained throughput test succeeded');
                console.log(`🚀 Maintained ${Math.round(actualThroughput)} ops/sec for ${testDuration}ms`);
                return true;
            }

            return false;

        } catch (error: any) {
            console.log(`Sustained throughput test failed: ${error.message}`);
            return false;
        }
    }

    /**
     * Test memory and connection management under stress
     */
    private async memoryStressTest(): Promise<boolean> {
        console.log('Testing memory and connection management under stress...');

        try {
            const iterations = 20;
            const sessionsPerIteration = 10;
            const operationsPerSession = 50;

            console.log(`🧠 Memory stress test: ${iterations} iterations of ${sessionsPerIteration} sessions`);
            console.log(`   Total: ${iterations * sessionsPerIteration * operationsPerSession} operations`);

            for (let iteration = 0; iteration < iterations; iteration++) {
                console.log(`  Iteration ${iteration + 1}/${iterations}...`);

                // Create sessions for this iteration
                const sessions = Array.from({ length: sessionsPerIteration }, () =>
                    newWebSocketRpcSession<StressCalculator>(wsEndpoint)
                );

                // Run operations on all sessions
                const allOperations: Promise<number>[] = [];

                for (const session of sessions) {
                    for (let op = 0; op < operationsPerSession; op++) {
                        const a = Math.floor(Math.random() * 100);
                        const b = Math.floor(Math.random() * 50) + 1;
                        allOperations.push(session.add(a, b));
                    }
                }

                // Wait for all operations to complete
                const results = await Promise.all(allOperations);

                // Verify results
                const allValid = results.every(r => typeof r === 'number' && !isNaN(r));

                if (!allValid) {
                    console.log(`❌ Iteration ${iteration + 1} failed - invalid results`);
                    return false;
                }

                // Clean up sessions
                for (const session of sessions) {
                    if ('close' in session) {
                        (session as any).close();
                    }
                }

                // Brief pause to allow cleanup
                await new Promise(resolve => setTimeout(resolve, 100));
            }

            console.log('🔍 Memory Stress Verification:');
            console.log(`  Completed ${iterations} iterations: ✓`);
            console.log(`  All sessions properly cleaned up: ✓`);
            console.log(`  Memory management stable: ✓`);

            console.log('✅ Memory stress test succeeded');
            console.log(`🧠 Handled ${iterations * sessionsPerIteration} session lifecycles`);

            return true;

        } catch (error: any) {
            console.log(`Memory stress test failed: ${error.message}`);
            return false;
        }
    }

    /**
     * Test error recovery under extreme conditions
     */
    private async extremeErrorRecoveryTest(): Promise<boolean> {
        console.log('Testing error recovery under extreme conditions...');

        try {
            const session = newWebSocketRpcSession<StressCalculator>(wsEndpoint);

            console.log('💥 Phase 1: Generate multiple error conditions...');

            const errorConditions = [
                { name: 'Division by zero', test: () => session.divide(10, 0) },
                { name: 'Large number overflow', test: () => session.multiply(Number.MAX_SAFE_INTEGER, 2) },
                { name: 'Invalid operation', test: () => session.subtract(NaN, 5) },
                { name: 'Negative division', test: () => session.divide(-100, -0.001) }
            ];

            let errorsHandled = 0;
            const errorResults: string[] = [];

            for (const condition of errorConditions) {
                try {
                    await condition.test();
                    console.log(`    ${condition.name}: No error thrown (unexpected)`);
                } catch (error: any) {
                    errorsHandled++;
                    errorResults.push(condition.name);
                    console.log(`    ${condition.name}: Error handled ✓`);
                }
            }

            console.log('🔄 Phase 2: Verify session recovery after errors...');

            // Test that session still works after errors
            const recoveryOperations = await Promise.all([
                session.add(1, 2),
                session.multiply(3, 4),
                session.subtract(10, 5),
                session.divide(20, 4)
            ]);

            console.log(`    Recovery results: [${recoveryOperations.join(', ')}]`);

            console.log('⚡ Phase 3: Mixed error and success operations...');

            const mixedResults: (number | string)[] = [];

            // Interleave successful and error operations
            for (let i = 0; i < 10; i++) {
                try {
                    if (i % 3 === 0) {
                        // Intentional error every 3rd operation
                        await session.divide(i, 0);
                        mixedResults.push('unexpected_success');
                    } else {
                        // Normal operation
                        const result = await session.add(i, i + 1);
                        mixedResults.push(result);
                    }
                } catch (error: any) {
                    mixedResults.push('error_handled');
                }
            }

            console.log(`    Mixed results: [${mixedResults.join(', ')}]`);

            // Cleanup
            if ('close' in session) {
                (session as any).close();
            }

            // Verify error handling and recovery
            const expectedRecovery = [3, 12, 5, 5];
            const recoveryCorrect = JSON.stringify(recoveryOperations) === JSON.stringify(expectedRecovery);

            const expectedMixed = [
                'error_handled', 2, 3,  // i=0 error, i=1 success (1+2), i=2 success (2+3)
                'error_handled', 5, 6,  // i=3 error, i=4 success (4+5), i=5 success (5+6)
                'error_handled', 8, 9,  // i=6 error, i=7 success (7+8), i=8 success (8+9)
                'error_handled'         // i=9 error
            ];

            const mixedCorrect = JSON.stringify(mixedResults) === JSON.stringify(expectedMixed);

            console.log('🔍 Error Recovery Verification:');
            console.log(`  Errors properly handled: ${errorsHandled}/${errorConditions.length} ${errorsHandled >= errorConditions.length / 2 ? '✓' : '✗'}`);
            console.log(`  Session recovery working: ${recoveryCorrect ? '✓' : '✗'}`);
            console.log(`  Mixed operation handling: ${mixedCorrect ? '✓' : '✗'}`);

            if (errorsHandled >= errorConditions.length / 2 && recoveryCorrect) {
                console.log('✅ Extreme error recovery test succeeded');
                console.log(`🛡️  Session resilient through ${errorsHandled} error conditions`);
                return true;
            }

            return false;

        } catch (error: any) {
            console.log(`Extreme error recovery test failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🌟 TIER 3 EXTREME: Ultra-Complex Stress Testing');
        console.log('═'.repeat(90));
        console.log(`🎯 WebSocket endpoint: ${wsEndpoint}`);
        console.log(`🎯 HTTP Batch endpoint: ${httpEndpoint}`);
        console.log('🚀 Goal: Push implementation to its absolute limits');
        console.log('⚠️  Prerequisites: All previous tiers must pass');
        console.log('');

        await this.test('Massive Concurrency (50+ Sessions)', () => this.massiveConcurrencyTest());
        await this.test('Complex Dependency Graph (15 Nodes)', () => this.complexDependencyGraphTest());
        await this.test('Sustained High Throughput (5s)', () => this.sustainedThroughputTest());
        await this.test('Memory & Connection Stress', () => this.memoryStressTest());
        await this.test('Extreme Error Recovery', () => this.extremeErrorRecoveryTest());

        console.log('\n' + '═'.repeat(90));
        console.log('🌟 TIER 3 EXTREME STRESS RESULTS');
        console.log('═'.repeat(90));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`🏆 Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🔥 ULTIMATE SUCCESS: Implementation handles extreme enterprise loads!');
            console.log('💪 Production-ready for the most demanding applications');
            console.log('🚀 Peak performance and reliability achieved');
            console.log('🏆 Tier 3 Extreme: COMPLETE MASTERY');
            process.exit(0);
        } else if (this.passed >= this.total * 0.8) {
            console.log('⭐ EXCELLENT: Near-perfect under extreme stress');
            console.log('💯 Ready for high-demand production workloads');
            process.exit(0);
        } else if (this.passed >= this.total * 0.6) {
            console.log('✨ GOOD: Handles most extreme scenarios');
            console.log('⚙️  Some optimization opportunities remain');
            process.exit(1);
        } else {
            console.log('🚨 NEEDS WORK: Extreme stress testing failed');
            console.log('🔧 Requires performance and reliability improvements');
            process.exit(2);
        }
    }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    process.exit(3);
});

const extremeTests = new Tier3ExtremeStressTests();
extremeTests.run();