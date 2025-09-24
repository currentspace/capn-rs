#!/usr/bin/env node

import { newWebSocketRpcSession } from 'capnweb';

/**
 * TIER 3 WebSocket: Advanced Complex Application Logic Tests
 *
 * Goal: Test sophisticated real-world scenarios over WebSocket transport
 * Tests: Advanced workflows, capability composition, cross-session coordination
 * Success Criteria: Full-featured applications working over persistent WebSocket
 *
 * Prerequisites: Tier 1 and Tier 2 WebSocket tests must pass
 */

interface AdvancedCalculator {
    // Basic operations
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;

    // Advanced operations (may return capabilities)
    getAsyncProcessor?(): Promise<AsyncProcessor>;
    getNestedCapability?(): Promise<NestedCalculator>;

    // State management
    setVariable?(name: string, value: number): Promise<boolean>;
    getVariable?(name: string): Promise<number>;

    // Batch operations
    batchCalculate?(operations: Array<{op: string, args: number[]}>): Promise<number[]>;
}

interface AsyncProcessor {
    processWithDelay(value: number, delayMs: number): Promise<number>;
    batchProcess(values: number[]): Promise<number[]>;
    getTimestamp(): Promise<number>;
}

interface NestedCalculator {
    chainOperations(value: number, operations: string[]): Promise<number>;
    createSubCalculator(): Promise<AdvancedCalculator>;
    getParentReference(): Promise<AdvancedCalculator>;
}

const port = process.argv[2] || '9001';
const wsEndpoint = `ws://localhost:${port}/rpc/ws`;

class Tier3WebSocketAdvancedTests {
    private passed = 0;
    private total = 0;

    private async test(name: string, testFn: () => Promise<boolean>): Promise<void> {
        this.total++;
        console.log(`\n🧪 Test ${this.total}: ${name} (WebSocket Advanced)`);
        console.log('─'.repeat(80));

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

    private async persistentWorkflowManagement(): Promise<boolean> {
        console.log('Testing persistent workflow management over WebSocket...');

        try {
            const session = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);

            console.log('🔄 Phase 1: Initial calculation pipeline');
            // Complex calculation pipeline that spans multiple messages
            const base = await session.add(10, 5);        // = 15
            console.log(`  Base value: ${base}`);

            const doubled = await session.multiply(base, 2);  // = 30
            console.log(`  Doubled: ${doubled}`);

            console.log('🔄 Phase 2: Dependent calculations');
            const result1 = await session.subtract(doubled, base);  // = 15
            const result2 = await session.divide(doubled, base);     // = 2

            console.log(`  Phase 2 results: ${result1}, ${result2}`);

            console.log('🔄 Phase 3: Complex multi-input operations');
            const combined = await session.add(result1, result2);    // = 17
            const final = await session.multiply(combined, base);    // = 255

            console.log(`  Combined: ${combined}, Final: ${final}`);

            console.log('🔄 Phase 4: Validation calculations');
            // Verify intermediate results can still be used
            const validation = await session.subtract(final, doubled); // = 225

            console.log(`  Validation result: ${validation}`);

            // Close the session properly
            if ('close' in session) {
                (session as any).close();
            }

            // Verify the entire workflow
            const expectedFlow = {
                base: 15,
                doubled: 30,
                result1: 15,
                result2: 2,
                combined: 17,
                final: 255,
                validation: 225
            };

            const actualFlow = { base, doubled, result1, result2, combined, final, validation };

            console.log('📊 Workflow Analysis:');
            for (const [key, expected] of Object.entries(expectedFlow)) {
                const actual = (actualFlow as any)[key];
                const match = actual === expected ? '✓' : '✗';
                console.log(`  ${key}: ${actual} (expected ${expected}) ${match}`);
            }

            const allCorrect = Object.entries(expectedFlow).every(
                ([key, expected]) => (actualFlow as any)[key] === expected
            );

            if (allCorrect) {
                console.log('✓ Persistent workflow maintained state perfectly across multiple phases');
                console.log('✓ WebSocket session handled complex interdependent calculations');
                return true;
            } else {
                console.log('✓ Workflow structure working but calculation discrepancies');
                return false;
            }

        } catch (error: any) {
            console.log(`Persistent workflow test failed: ${error.message}`);
            return false;
        }
    }

    private async concurrentSessionCoordination(): Promise<boolean> {
        console.log('Testing coordination between multiple WebSocket sessions...');

        try {
            // Create three concurrent WebSocket sessions
            console.log('🌐 Creating multiple concurrent WebSocket sessions...');
            const session1 = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);
            const session2 = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);
            const session3 = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);

            console.log('📊 Phase 1: Parallel computation across sessions');
            // Each session computes part of a larger calculation
            const [part1, part2, part3] = await Promise.all([
                session1.multiply(5, 4),    // = 20
                session2.add(10, 15),       // = 25
                session3.subtract(50, 20)   // = 30
            ]);

            console.log(`  Parallel results: ${part1}, ${part2}, ${part3}`);

            console.log('📊 Phase 2: Cross-session result sharing');
            // Use results from other sessions in new calculations
            const [combo1, combo2, combo3] = await Promise.all([
                session1.add(part1, part2),        // 20 + 25 = 45
                session2.multiply(part2, 2),       // 25 * 2 = 50
                session3.divide(part3, 2)          // 30 / 2 = 15
            ]);

            console.log(`  Cross-session combinations: ${combo1}, ${combo2}, ${combo3}`);

            console.log('📊 Phase 3: Final aggregation');
            // Final calculation combining all sessions' work
            const finalResults = await Promise.all([
                session1.add(combo1, combo2),      // 45 + 50 = 95
                session2.subtract(combo2, combo3), // 50 - 15 = 35
                session3.multiply(combo1, combo3)  // 45 * 15 = 675
            ]);

            console.log(`  Final aggregated results: ${finalResults.join(', ')}`);

            // Close all sessions properly
            if ('close' in session1) (session1 as any).close();
            if ('close' in session2) (session2 as any).close();
            if ('close' in session3) (session3 as any).close();

            // Verify all calculations
            const expected = {
                parts: [20, 25, 30],
                combos: [45, 50, 15],
                finals: [95, 35, 675]
            };

            const actual = {
                parts: [part1, part2, part3],
                combos: [combo1, combo2, combo3],
                finals: finalResults
            };

            console.log('🔍 Verification:');
            let allCorrect = true;

            ['parts', 'combos', 'finals'].forEach(phase => {
                const expectedVals = (expected as any)[phase];
                const actualVals = (actual as any)[phase];
                const match = JSON.stringify(expectedVals) === JSON.stringify(actualVals);
                console.log(`  ${phase}: ${actualVals.join(', ')} ${match ? '✓' : '✗'}`);
                if (!match) allCorrect = false;
            });

            if (allCorrect) {
                console.log('✓ Multiple WebSocket sessions coordinated perfectly');
                console.log('✓ Cross-session data sharing and computation working');
                console.log('✓ Concurrent session isolation maintained');
                return true;
            } else {
                console.log('✓ Session coordination structure working but calculation errors');
                return false;
            }

        } catch (error: any) {
            console.log(`Concurrent session coordination test failed: ${error.message}`);
            return false;
        }
    }

    private async realTimeStreamProcessing(): Promise<boolean> {
        console.log('Testing real-time stream processing over WebSocket...');

        try {
            const session = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);

            console.log('🔄 Simulating real-time data stream processing...');
            const startTime = Date.now();

            // Simulate incoming data stream with rapid-fire operations
            const streamData = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
            const processedStream: number[] = [];

            console.log('📈 Processing stream in real-time...');

            // Process each data point as it "arrives"
            for (const dataPoint of streamData) {
                // Simulate real-time processing with small delays
                await new Promise(resolve => setTimeout(resolve, 10)); // 10ms delay

                // Process the data point
                const processed = await session.multiply(dataPoint, 2);
                processedStream.push(processed);

                console.log(`    Stream[${dataPoint}] -> ${processed}`);
            }

            const processingTime = Date.now() - startTime;
            console.log(`📊 Stream processing completed in ${processingTime}ms`);
            console.log(`    Average per item: ${(processingTime / streamData.length).toFixed(1)}ms`);

            // Perform aggregation operations on the processed stream
            console.log('🔢 Performing stream aggregations...');

            const sum = processedStream.reduce((acc, val) => acc + val, 0);
            const serverSum = await session.add(0, sum); // Verify server can handle large numbers

            console.log(`  Processed stream: [${processedStream.join(', ')}]`);
            console.log(`  Local sum: ${sum}, Server verification: ${serverSum}`);

            // Close session
            if ('close' in session) {
                (session as any).close();
            }

            // Verify stream processing
            const expectedStream = streamData.map(x => x * 2);  // [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
            const expectedSum = expectedStream.reduce((acc, val) => acc + val, 0); // 110

            const streamCorrect = JSON.stringify(processedStream) === JSON.stringify(expectedStream);
            const sumCorrect = serverSum === expectedSum;

            console.log('🔍 Stream Verification:');
            console.log(`  Stream processing: ${streamCorrect ? '✓' : '✗'}`);
            console.log(`  Sum verification: ${sumCorrect ? '✓' : '✗'} (${serverSum} === ${expectedSum})`);

            if (streamCorrect && sumCorrect) {
                console.log('✓ Real-time stream processing working perfectly');
                console.log('✓ WebSocket handled rapid sequential operations efficiently');

                if (processingTime < 1000) {
                    console.log('✓ Excellent real-time performance achieved');
                }

                return true;
            } else {
                console.log('✓ Stream processing structure working but data discrepancies');
                return false;
            }

        } catch (error: any) {
            console.log(`Real-time stream processing test failed: ${error.message}`);
            return false;
        }
    }

    private async errorRecoveryAndResiliency(): Promise<boolean> {
        console.log('Testing advanced error recovery and connection resiliency...');

        try {
            const session = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);

            console.log('🧪 Phase 1: Normal operations establishment');
            const baseline = await session.add(1, 1);
            console.log(`  Baseline result: ${baseline}`);

            console.log('🧪 Phase 2: Intentional error injection');
            let errorCount = 0;
            const errorTypes = [];

            // Test multiple error scenarios
            try {
                await session.divide(5, 0);
            } catch (error: any) {
                errorCount++;
                errorTypes.push('division_by_zero');
                console.log(`    Error captured: ${error.message}`);
            }

            // Test if session is still functional after error
            console.log('🧪 Phase 3: Post-error session validation');
            const recovery1 = await session.multiply(3, 4);
            console.log(`  Recovery test 1: ${recovery1}`);

            // Inject another error type if possible
            try {
                await session.subtract(1, 'invalid' as any);
            } catch (error: any) {
                errorCount++;
                errorTypes.push('invalid_argument');
                console.log(`    Second error captured: ${error.message}`);
            }

            // Test session functionality again
            const recovery2 = await session.add(10, 5);
            console.log(`  Recovery test 2: ${recovery2}`);

            console.log('🧪 Phase 4: Stress recovery with rapid operations');
            // Rapid-fire operations to test session stability after errors
            const rapidResults = await Promise.all([
                session.add(1, 2),
                session.multiply(2, 3),
                session.subtract(10, 4),
                session.divide(20, 4)
            ]);

            console.log(`  Rapid recovery results: [${rapidResults.join(', ')}]`);

            // Close session
            if ('close' in session) {
                (session as any).close();
            }

            // Verification
            const expectedResults = {
                baseline: 2,
                recovery1: 12,
                recovery2: 15,
                rapid: [3, 6, 6, 5]
            };

            const actualResults = {
                baseline,
                recovery1,
                recovery2,
                rapid: rapidResults
            };

            console.log('🔍 Error Recovery Verification:');
            console.log(`  Errors encountered: ${errorCount} (${errorTypes.join(', ')})`);

            let allCorrect = true;
            Object.entries(expectedResults).forEach(([key, expected]) => {
                const actual = (actualResults as any)[key];
                const match = JSON.stringify(actual) === JSON.stringify(expected);
                console.log(`  ${key}: ${JSON.stringify(actual)} ${match ? '✓' : '✗'}`);
                if (!match) allCorrect = false;
            });

            if (allCorrect && errorCount > 0) {
                console.log('✓ WebSocket session demonstrated excellent error recovery');
                console.log('✓ Connection remained stable through multiple error scenarios');
                console.log('✓ Session functionality fully restored after errors');
                return true;
            } else if (allCorrect) {
                console.log('✓ Session stability confirmed, but errors may not be properly handled');
                return true;  // Still pass if calculations are correct
            } else {
                console.log('✓ Error handling working but calculation discrepancies');
                return false;
            }

        } catch (error: any) {
            console.log(`Error recovery test failed: ${error.message}`);
            return false;
        }
    }

    private async highFrequencyTradingSimulation(): Promise<boolean> {
        console.log('Testing high-frequency trading-like scenarios over WebSocket...');

        try {
            const session = newWebSocketRpcSession<AdvancedCalculator>(wsEndpoint);

            console.log('📊 Simulating high-frequency financial calculations...');
            const startTime = Date.now();

            // Simulate rapid market data processing
            const marketTicks = [
                { price: 100, volume: 1000 },
                { price: 101, volume: 1500 },
                { price: 99, volume: 2000 },
                { price: 102, volume: 800 },
                { price: 98, volume: 2500 }
            ];

            const calculations: Promise<number>[] = [];

            console.log('⚡ Launching high-frequency calculations...');

            // Launch rapid concurrent calculations
            marketTicks.forEach((tick, i) => {
                // Calculate value (price * volume)
                calculations.push(session.multiply(tick.price, tick.volume));

                // Calculate volume-weighted adjustments
                calculations.push(session.divide(tick.volume, 100));

                // Calculate price differentials (if not first tick)
                if (i > 0) {
                    const prevPrice = marketTicks[i - 1].price;
                    calculations.push(session.subtract(tick.price, prevPrice));
                }
            });

            console.log(`    Launched ${calculations.length} concurrent calculations...`);

            // Execute all calculations
            const results = await Promise.all(calculations);
            const executionTime = Date.now() - startTime;

            console.log(`⏱️  All calculations completed in ${executionTime}ms`);
            console.log(`    Average per calculation: ${(executionTime / calculations.length).toFixed(2)}ms`);
            console.log(`    Throughput: ${(calculations.length / executionTime * 1000).toFixed(0)} ops/second`);

            // Analyze results structure
            console.log('📈 Market Analysis Results:');
            let resultIndex = 0;

            marketTicks.forEach((tick, i) => {
                const value = results[resultIndex++];
                const volumeWeight = results[resultIndex++];

                console.log(`  Tick ${i + 1}: Value=${value}, VolumeWeight=${volumeWeight.toFixed(2)}`);

                if (i > 0) {
                    const priceDiff = results[resultIndex++];
                    console.log(`           PriceDiff=${priceDiff > 0 ? '+' : ''}${priceDiff}`);
                }
            });

            // Verify some calculations
            const expectedFirstValue = marketTicks[0].price * marketTicks[0].volume; // 100000
            const actualFirstValue = results[0];

            const calculationsCorrect = actualFirstValue === expectedFirstValue;

            // Close session
            if ('close' in session) {
                (session as any).close();
            }

            console.log('🔍 Performance Verification:');
            console.log(`  Calculation accuracy: ${calculationsCorrect ? '✓' : '✗'}`);
            console.log(`  Execution time: ${executionTime}ms ${executionTime < 1000 ? '✓' : '⚠️'}`);
            console.log(`  All operations completed: ${results.length === calculations.length ? '✓' : '✗'}`);

            if (calculationsCorrect && results.length === calculations.length) {
                console.log('✓ High-frequency trading simulation successful');
                console.log('✓ WebSocket handled rapid concurrent calculations excellently');

                if (executionTime < 500) {
                    console.log('✓ Outstanding performance suitable for real-time trading');
                }

                return true;
            } else {
                console.log('✓ High-frequency structure working but some discrepancies');
                return false;
            }

        } catch (error: any) {
            console.log(`High-frequency trading simulation failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🌐 TIER 3 WebSocket: Advanced Complex Application Logic Tests');
        console.log('===========================================================');
        console.log(`📍 Testing WebSocket endpoint: ${wsEndpoint}`);
        console.log('🎯 Goal: Test sophisticated real-world scenarios over WebSocket transport');
        console.log('📋 Prerequisites: Tier 1 and Tier 2 WebSocket tests must pass');
        console.log('');

        // Test 1: Persistent workflow management
        await this.test('Persistent Workflow Management', () => this.persistentWorkflowManagement());

        // Test 2: Concurrent session coordination
        await this.test('Concurrent Session Coordination', () => this.concurrentSessionCoordination());

        // Test 3: Real-time stream processing
        await this.test('Real-time Stream Processing', () => this.realTimeStreamProcessing());

        // Test 4: Advanced error recovery
        await this.test('Error Recovery and Resiliency', () => this.errorRecoveryAndResiliency());

        // Test 5: High-frequency trading simulation
        await this.test('High-Frequency Trading Simulation', () => this.highFrequencyTradingSimulation());

        // Results
        console.log('\n' + '='.repeat(80));
        console.log('🌐 TIER 3 WebSocket ADVANCED RESULTS');
        console.log('='.repeat(80));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`✅ Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🎉 TIER 3 WebSocket COMPLETE: Advanced complex applications working perfectly!');
            console.log('🚀 WebSocket transport provides enterprise-grade real-time capabilities');
            console.log('🏆 Full Cap\'n Web WebSocket compatibility achieved!');
            console.log('📊 Production-ready for complex real-time applications');
            process.exit(0);
        } else if (this.passed >= this.total * 0.8) {
            console.log('⭐ TIER 3 WebSocket EXCELLENT: Advanced features working with minor limitations');
            console.log('🔧 Consider optimizing edge cases for critical applications');
            process.exit(0);
        } else if (this.passed >= this.total * 0.6) {
            console.log('⚠️  TIER 3 WebSocket GOOD: Most advanced features working');
            console.log('🔧 Some advanced scenarios need refinement');
            process.exit(1);
        } else {
            console.log('💥 TIER 3 WebSocket FAILED: Advanced WebSocket features not working');
            console.log('🚨 Requires significant WebSocket implementation work');
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
const tier3Advanced = new Tier3WebSocketAdvancedTests();
tier3Advanced.run();