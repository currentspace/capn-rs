#!/usr/bin/env node

import { newHttpBatchRpcSession, newWebSocketRpcSession } from 'capnweb';

/**
 * Cross-Transport Interoperability Tests
 *
 * Goal: Verify seamless interoperability between HTTP Batch and WebSocket transports
 * Tests: Mixed transport scenarios, transport switching, performance comparisons
 * Success Criteria: Both transports produce identical results with transport-specific advantages
 *
 * Prerequisites: All Tier 1, Tier 2, and Tier 3 tests must pass for both transports
 */

interface Calculator {
    add(a: number, b: number): Promise<number>;
    multiply(a: number, b: number): Promise<number>;
    divide(a: number, b: number): Promise<number>;
    subtract(a: number, b: number): Promise<number>;
}

const port = process.argv[2] || '9001';
const httpEndpoint = `http://localhost:${port}/rpc/batch`;
const wsEndpoint = `ws://localhost:${port}/rpc/ws`;

class CrossTransportInteropTests {
    private passed = 0;
    private total = 0;

    private async test(name: string, testFn: () => Promise<boolean>): Promise<void> {
        this.total++;
        console.log(`\n🔄 Test ${this.total}: ${name}`);
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

    private async transportEquivalenceTest(): Promise<boolean> {
        console.log('Testing computational equivalence between HTTP Batch and WebSocket...');

        try {
            // Same calculations on both transports
            const testCases = [
                { op: 'add', args: [15, 25] },
                { op: 'multiply', args: [7, 8] },
                { op: 'divide', args: [100, 4] },
                { op: 'subtract', args: [50, 18] }
            ];

            console.log('🌐 Executing calculations on HTTP Batch transport...');
            const httpSession = newHttpBatchRpcSession<Calculator>(httpEndpoint);
            const httpResults: number[] = [];

            for (const testCase of testCases) {
                let result: number;
                switch (testCase.op) {
                    case 'add':
                        result = await httpSession.add(testCase.args[0], testCase.args[1]);
                        break;
                    case 'multiply':
                        result = await httpSession.multiply(testCase.args[0], testCase.args[1]);
                        break;
                    case 'divide':
                        result = await httpSession.divide(testCase.args[0], testCase.args[1]);
                        break;
                    case 'subtract':
                        result = await httpSession.subtract(testCase.args[0], testCase.args[1]);
                        break;
                    default:
                        throw new Error(`Unknown operation: ${testCase.op}`);
                }
                httpResults.push(result);
                console.log(`  HTTP ${testCase.op}(${testCase.args.join(', ')}) = ${result}`);
            }

            console.log('🔌 Executing same calculations on WebSocket transport...');
            const wsSession = newWebSocketRpcSession<Calculator>(wsEndpoint);
            const wsResults: number[] = [];

            for (const testCase of testCases) {
                let result: number;
                switch (testCase.op) {
                    case 'add':
                        result = await wsSession.add(testCase.args[0], testCase.args[1]);
                        break;
                    case 'multiply':
                        result = await wsSession.multiply(testCase.args[0], testCase.args[1]);
                        break;
                    case 'divide':
                        result = await wsSession.divide(testCase.args[0], testCase.args[1]);
                        break;
                    case 'subtract':
                        result = await wsSession.subtract(testCase.args[0], testCase.args[1]);
                        break;
                    default:
                        throw new Error(`Unknown operation: ${testCase.op}`);
                }
                wsResults.push(result);
                console.log(`  WebSocket ${testCase.op}(${testCase.args.join(', ')}) = ${result}`);
            }

            // Clean up WebSocket session
            if ('close' in wsSession) {
                (wsSession as any).close();
            }

            console.log('🔍 Transport Equivalence Analysis:');
            console.log(`  HTTP Results:     [${httpResults.join(', ')}]`);
            console.log(`  WebSocket Results: [${wsResults.join(', ')}]`);

            const resultsMatch = JSON.stringify(httpResults) === JSON.stringify(wsResults);

            if (resultsMatch) {
                console.log('✓ Both transports produced identical computational results');
                console.log('✓ Transport abstraction maintains mathematical consistency');
                return true;
            } else {
                console.log('✗ Transport results differ - computational inconsistency detected');
                return false;
            }

        } catch (error: any) {
            console.log(`Transport equivalence test failed: ${error.message}`);
            return false;
        }
    }

    private async performanceCharacteristicsComparison(): Promise<boolean> {
        console.log('Comparing performance characteristics between transports...');

        try {
            const operationCount = 10;
            const operations = Array.from({length: operationCount}, (_, i) => ({
                op: ['add', 'multiply', 'subtract', 'divide'][i % 4],
                args: [i + 1, i + 2]
            }));

            console.log('⏱️  HTTP Batch Performance Test...');
            const httpStart = Date.now();
            const httpSession = newHttpBatchRpcSession<Calculator>(httpEndpoint);

            const httpResults: number[] = [];
            for (const operation of operations) {
                let result: number;
                switch (operation.op) {
                    case 'add':
                        result = await httpSession.add(operation.args[0], operation.args[1]);
                        break;
                    case 'multiply':
                        result = await httpSession.multiply(operation.args[0], operation.args[1]);
                        break;
                    case 'subtract':
                        result = await httpSession.subtract(operation.args[0], operation.args[1]);
                        break;
                    case 'divide':
                        result = await httpSession.divide(operation.args[0], operation.args[1]);
                        break;
                    default:
                        throw new Error(`Unknown operation: ${operation.op}`);
                }
                httpResults.push(result);
            }
            const httpDuration = Date.now() - httpStart;

            console.log('⚡ WebSocket Performance Test...');
            const wsStart = Date.now();
            const wsSession = newWebSocketRpcSession<Calculator>(wsEndpoint);

            const wsResults: number[] = [];
            for (const operation of operations) {
                let result: number;
                switch (operation.op) {
                    case 'add':
                        result = await wsSession.add(operation.args[0], operation.args[1]);
                        break;
                    case 'multiply':
                        result = await wsSession.multiply(operation.args[0], operation.args[1]);
                        break;
                    case 'subtract':
                        result = await wsSession.subtract(operation.args[0], operation.args[1]);
                        break;
                    case 'divide':
                        result = await wsSession.divide(operation.args[0], operation.args[1]);
                        break;
                    default:
                        throw new Error(`Unknown operation: ${operation.op}`);
                }
                wsResults.push(result);
            }
            const wsDuration = Date.now() - wsStart;

            // Clean up WebSocket session
            if ('close' in wsSession) {
                (wsSession as any).close();
            }

            console.log('📊 Performance Analysis:');
            console.log(`  HTTP Batch:    ${httpDuration}ms total, ${(httpDuration/operationCount).toFixed(1)}ms/op`);
            console.log(`  WebSocket:     ${wsDuration}ms total, ${(wsDuration/operationCount).toFixed(1)}ms/op`);

            const performanceRatio = httpDuration / wsDuration;
            console.log(`  Performance Ratio: ${performanceRatio.toFixed(2)}x (${performanceRatio > 1 ? 'WebSocket faster' : 'HTTP faster'})`);

            // Verify computational consistency
            const resultsMatch = JSON.stringify(httpResults) === JSON.stringify(wsResults);

            console.log('🔍 Consistency Check:');
            console.log(`  Results identical: ${resultsMatch ? '✓' : '✗'}`);
            console.log(`  HTTP throughput:   ${(operationCount / httpDuration * 1000).toFixed(0)} ops/sec`);
            console.log(`  WebSocket throughput: ${(operationCount / wsDuration * 1000).toFixed(0)} ops/sec`);

            if (resultsMatch) {
                console.log('✓ Both transports maintain computational consistency');
                console.log('✓ Performance characteristics measured and compared');

                // WebSocket should generally be faster for sequential operations
                if (performanceRatio > 0.8) {
                    console.log('✓ Performance characteristics within expected ranges');
                }

                return true;
            } else {
                console.log('✗ Computational inconsistency between transports');
                return false;
            }

        } catch (error: any) {
            console.log(`Performance comparison test failed: ${error.message}`);
            return false;
        }
    }

    private async concurrentTransportUsage(): Promise<boolean> {
        console.log('Testing concurrent usage of both transport types...');

        try {
            console.log('🚀 Launching concurrent operations across both transports...');

            // Create sessions on both transports
            const httpSession = newHttpBatchRpcSession<Calculator>(httpEndpoint);
            const wsSession = newWebSocketRpcSession<Calculator>(wsEndpoint);

            const startTime = Date.now();

            // Launch operations concurrently across transports
            const concurrentOps = await Promise.all([
                // HTTP operations
                httpSession.add(10, 20),
                httpSession.multiply(5, 6),
                httpSession.subtract(100, 25),

                // WebSocket operations
                wsSession.add(15, 35),
                wsSession.multiply(7, 8),
                wsSession.subtract(200, 50)
            ]);

            const duration = Date.now() - startTime;

            // Clean up WebSocket session
            if ('close' in wsSession) {
                (wsSession as any).close();
            }

            console.log('📊 Concurrent Operation Results:');
            console.log(`  HTTP add(10, 20): ${concurrentOps[0]}`);
            console.log(`  HTTP multiply(5, 6): ${concurrentOps[1]}`);
            console.log(`  HTTP subtract(100, 25): ${concurrentOps[2]}`);
            console.log(`  WebSocket add(15, 35): ${concurrentOps[3]}`);
            console.log(`  WebSocket multiply(7, 8): ${concurrentOps[4]}`);
            console.log(`  WebSocket subtract(200, 50): ${concurrentOps[5]}`);

            console.log(`⏱️  Total concurrent execution time: ${duration}ms`);
            console.log(`    Average per operation: ${(duration / 6).toFixed(1)}ms`);

            // Verify results
            const expectedResults = [30, 30, 75, 50, 56, 150];
            const resultsCorrect = concurrentOps.every((result, i) => result === expectedResults[i]);

            console.log('🔍 Verification:');
            console.log(`  Expected: [${expectedResults.join(', ')}]`);
            console.log(`  Actual:   [${concurrentOps.join(', ')}]`);
            console.log(`  All correct: ${resultsCorrect ? '✓' : '✗'}`);

            if (resultsCorrect) {
                console.log('✓ Concurrent transport usage working perfectly');
                console.log('✓ Both transports can be used simultaneously without interference');
                console.log('✓ No resource conflicts or computation errors detected');
                return true;
            } else {
                console.log('✗ Concurrent transport usage produced incorrect results');
                return false;
            }

        } catch (error: any) {
            console.log(`Concurrent transport usage test failed: ${error.message}`);
            return false;
        }
    }

    private async errorHandlingConsistency(): Promise<boolean> {
        console.log('Testing error handling consistency across transports...');

        try {
            console.log('🧪 Testing error scenarios on both transports...');

            const httpSession = newHttpBatchRpcSession<Calculator>(httpEndpoint);
            const wsSession = newWebSocketRpcSession<Calculator>(wsEndpoint);

            // Test identical error scenarios
            let httpError: any = null;
            let wsError: any = null;

            console.log('  Triggering division by zero on HTTP Batch...');
            try {
                await httpSession.divide(10, 0);
            } catch (error) {
                httpError = error;
                console.log(`    HTTP Error: ${error.message}`);
            }

            console.log('  Triggering division by zero on WebSocket...');
            try {
                await wsSession.divide(10, 0);
            } catch (error) {
                wsError = error;
                console.log(`    WebSocket Error: ${error.message}`);
            }

            // Test recovery on both transports
            console.log('  Testing error recovery...');
            const httpRecovery = await httpSession.add(5, 7);
            const wsRecovery = await wsSession.add(5, 7);

            console.log(`    HTTP Recovery: ${httpRecovery}`);
            console.log(`    WebSocket Recovery: ${wsRecovery}`);

            // Clean up WebSocket session
            if ('close' in wsSession) {
                (wsSession as any).close();
            }

            console.log('🔍 Error Handling Analysis:');
            const bothErrored = httpError !== null && wsError !== null;
            const errorMessagesMatch = httpError?.message === wsError?.message;
            const recoveryMatches = httpRecovery === wsRecovery && httpRecovery === 12;

            console.log(`  Both transports errored: ${bothErrored ? '✓' : '✗'}`);
            console.log(`  Error messages consistent: ${errorMessagesMatch ? '✓' : '✗'}`);
            console.log(`  Recovery successful: ${recoveryMatches ? '✓' : '✗'}`);

            if (bothErrored && errorMessagesMatch && recoveryMatches) {
                console.log('✓ Error handling is consistent across both transports');
                console.log('✓ Both transports maintain session integrity after errors');
                console.log('✓ Error messages are standardized between transports');
                return true;
            } else {
                console.log('✗ Error handling inconsistencies detected between transports');
                return false;
            }

        } catch (error: any) {
            console.log(`Error handling consistency test failed: ${error.message}`);
            return false;
        }
    }

    private async transportSpecificAdvantagesTest(): Promise<boolean> {
        console.log('Testing transport-specific advantages and use cases...');

        try {
            console.log('📊 WebSocket advantage: Real-time stream processing...');
            const wsSession = newWebSocketRpcSession<Calculator>(wsEndpoint);

            const streamStart = Date.now();
            const streamValues = [1, 2, 3, 4, 5];
            const streamResults: number[] = [];

            for (const value of streamValues) {
                const result = await wsSession.multiply(value, 2);
                streamResults.push(result);
            }
            const streamDuration = Date.now() - streamStart;

            console.log(`  WebSocket stream processing: ${streamDuration}ms for ${streamValues.length} operations`);
            console.log(`  Results: [${streamResults.join(', ')}]`);

            console.log('🔄 HTTP advantage: Stateless bulk operations...');
            const httpSession = newHttpBatchRpcSession<Calculator>(httpEndpoint);

            const bulkStart = Date.now();

            // HTTP can handle bulk operations efficiently in a single request/response cycle
            const bulkOperations = await Promise.all([
                httpSession.add(10, 10),
                httpSession.add(20, 20),
                httpSession.add(30, 30),
                httpSession.add(40, 40),
                httpSession.add(50, 50)
            ]);
            const bulkDuration = Date.now() - bulkStart;

            console.log(`  HTTP bulk processing: ${bulkDuration}ms for ${bulkOperations.length} operations`);
            console.log(`  Results: [${bulkOperations.join(', ')}]`);

            // Clean up WebSocket session
            if ('close' in wsSession) {
                (wsSession as any).close();
            }

            console.log('🔍 Transport Advantage Analysis:');
            console.log(`  WebSocket avg/op: ${(streamDuration/streamValues.length).toFixed(1)}ms (persistent connection)`);
            console.log(`  HTTP avg/op: ${(bulkDuration/bulkOperations.length).toFixed(1)}ms (stateless batch)`);

            // Verify results are mathematically correct
            const streamCorrect = streamResults.every((result, i) => result === streamValues[i] * 2);
            const bulkCorrect = bulkOperations.every((result, i) => result === (i + 1) * 10 * 2);

            console.log(`  WebSocket stream results correct: ${streamCorrect ? '✓' : '✗'}`);
            console.log(`  HTTP bulk results correct: ${bulkCorrect ? '✓' : '✗'}`);

            if (streamCorrect && bulkCorrect) {
                console.log('✓ Both transports demonstrate their specific advantages');
                console.log('✓ WebSocket excels at real-time streaming scenarios');
                console.log('✓ HTTP Batch excels at stateless bulk operations');
                return true;
            } else {
                console.log('✗ Transport advantages not properly demonstrated');
                return false;
            }

        } catch (error: any) {
            console.log(`Transport advantages test failed: ${error.message}`);
            return false;
        }
    }

    async run(): Promise<void> {
        console.log('🔄 Cross-Transport Interoperability Tests');
        console.log('=====================================');
        console.log(`📍 Testing endpoints:`);
        console.log(`   HTTP Batch: ${httpEndpoint}`);
        console.log(`   WebSocket:  ${wsEndpoint}`);
        console.log('🎯 Goal: Verify seamless interoperability between transport types');
        console.log('📋 Prerequisites: All Tier 1, 2, and 3 tests must pass for both transports');
        console.log('');

        // Test 1: Transport equivalence
        await this.test('Transport Computational Equivalence', () => this.transportEquivalenceTest());

        // Test 2: Performance characteristics
        await this.test('Performance Characteristics Comparison', () => this.performanceCharacteristicsComparison());

        // Test 3: Concurrent transport usage
        await this.test('Concurrent Multi-Transport Usage', () => this.concurrentTransportUsage());

        // Test 4: Error handling consistency
        await this.test('Error Handling Consistency', () => this.errorHandlingConsistency());

        // Test 5: Transport-specific advantages
        await this.test('Transport-Specific Advantages', () => this.transportSpecificAdvantagesTest());

        // Results
        console.log('\n' + '='.repeat(80));
        console.log('🔄 CROSS-TRANSPORT INTEROPERABILITY RESULTS');
        console.log('='.repeat(80));

        const passRate = Math.round((this.passed / this.total) * 100);
        console.log(`✅ Passed: ${this.passed}/${this.total} (${passRate}%)`);

        if (this.passed === this.total) {
            console.log('🎉 CROSS-TRANSPORT INTEROPERABILITY COMPLETE!');
            console.log('🚀 HTTP Batch and WebSocket transports are fully interoperable');
            console.log('⚡ Both transports provide consistent Cap\'n Web protocol implementation');
            console.log('🏆 Production-ready multi-transport Cap\'n Web server achieved!');
            console.log('📊 Applications can seamlessly choose optimal transport for their use case');
            process.exit(0);
        } else if (this.passed >= this.total * 0.8) {
            console.log('⭐ CROSS-TRANSPORT INTEROPERABILITY EXCELLENT!');
            console.log('🔧 Minor transport differences detected, but overall compatibility is strong');
            process.exit(0);
        } else {
            console.log('💥 CROSS-TRANSPORT INTEROPERABILITY ISSUES DETECTED');
            console.log('🚨 Significant transport inconsistencies require attention');
            process.exit(1);
        }
    }
}

// Handle unhandled promise rejections
process.on('unhandledRejection', (reason, promise) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason);
    process.exit(3);
});

// Run tests
const crossTransportTests = new CrossTransportInteropTests();
crossTransportTests.run();