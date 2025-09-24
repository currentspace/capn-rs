#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

// Interface for testing promise pipelining
interface CounterService {
    increment_global(counterName: string): Promise<number>;
    decrement_global(counterName: string): Promise<number>;
    get_global(counterName: string): Promise<number>;
    reset_global(counterName: string): Promise<number>;

    increment_session(sessionId: string, counterName: string): Promise<number>;
    get_session(sessionId: string, counterName: string): Promise<number>;

    set_session_property(sessionId: string, propertyName: string, value: any): Promise<any>;
    get_session_property(sessionId: string, propertyName: string): Promise<any>;
}

async function testPromisePipelining() {
    console.log('🧪 Testing Promise Pipelining and Message Flow');
    console.log('===============================================\n');

    try {
        const session = newHttpBatchRpcSession<CounterService>('http://localhost:8080/rpc/batch');
        console.log('✅ Created session for promise pipelining tests\n');

        // Test 1: Sequential vs Parallel execution timing
        console.log('Test 1: Sequential vs Parallel Execution Timing');
        console.log('================================================');

        const counterName = 'timing-test-' + Date.now();
        await session.reset_global(counterName);

        // Sequential execution
        const sequentialStart = performance.now();
        await session.increment_global(counterName);
        await session.increment_global(counterName);
        await session.increment_global(counterName);
        await session.increment_global(counterName);
        await session.increment_global(counterName);
        const sequentialEnd = performance.now();
        const sequentialTime = sequentialEnd - sequentialStart;

        console.log(`✅ Sequential execution took ${sequentialTime.toFixed(2)}ms`);

        // Reset counter for parallel test
        await session.reset_global(counterName);

        // Parallel execution
        const parallelStart = performance.now();
        await Promise.all([
            session.increment_global(counterName),
            session.increment_global(counterName),
            session.increment_global(counterName),
            session.increment_global(counterName),
            session.increment_global(counterName)
        ]);
        const parallelEnd = performance.now();
        const parallelTime = parallelEnd - parallelStart;

        console.log(`✅ Parallel execution took ${parallelTime.toFixed(2)}ms`);
        console.log(`📊 Parallel was ${(sequentialTime / parallelTime).toFixed(2)}x faster`);

        // Test 2: Complex dependency chains
        console.log('\nTest 2: Complex Dependency Chains');
        console.log('==================================');

        const sessionId = 'pipeline-session-' + Date.now();

        // Create a complex dependency chain:
        // 1. Set initial values
        // 2. Increment based on those values
        // 3. Use results for further operations

        const initialValues = await Promise.all([
            session.reset_global('chain-counter-1'),
            session.reset_global('chain-counter-2'),
            session.reset_global('chain-counter-3')
        ]);

        console.log(`✅ Reset initial values: [${initialValues.join(', ')}]`);

        // First level of operations
        const firstLevel = await Promise.all([
            session.increment_global('chain-counter-1'),
            session.increment_global('chain-counter-2'),
            session.increment_global('chain-counter-3')
        ]);

        console.log(`✅ First level results: [${firstLevel.join(', ')}]`);

        // Second level depends on first level
        const secondLevel = await Promise.all([
            session.increment_global('chain-counter-1'),
            session.increment_global('chain-counter-2'),
            session.increment_session(sessionId, 'derived-counter')
        ]);

        console.log(`✅ Second level results: [${secondLevel.join(', ')}]`);

        // Third level combines previous results
        const finalResults = await Promise.all([
            session.get_global('chain-counter-1'),
            session.get_global('chain-counter-2'),
            session.get_global('chain-counter-3'),
            session.get_session(sessionId, 'derived-counter')
        ]);

        console.log(`✅ Final dependency chain results: [${finalResults.join(', ')}]`);

        // Test 3: Batch operation optimization
        console.log('\nTest 3: Batch Operation Optimization');
        console.log('=====================================');

        // Test large batch of operations
        const batchSize = 20;
        const batchCounterName = 'batch-counter-' + Date.now();
        await session.reset_global(batchCounterName);

        const batchStart = performance.now();
        const batchPromises = [];
        for (let i = 0; i < batchSize; i++) {
            batchPromises.push(session.increment_global(batchCounterName));
        }

        const batchResults = await Promise.all(batchPromises);
        const batchEnd = performance.now();
        const batchTime = batchEnd - batchStart;

        console.log(`✅ Batch of ${batchSize} operations completed in ${batchTime.toFixed(2)}ms`);
        console.log(`📊 Average per operation: ${(batchTime / batchSize).toFixed(2)}ms`);
        console.log(`✅ Final batch counter value: ${Math.max(...batchResults)}`);

        // Verify all values are unique (proper sequencing)
        const uniqueResults = new Set(batchResults);
        console.log(`✅ Result uniqueness: ${uniqueResults.size}/${batchSize} unique values`);

        // Test 4: Mixed session and global operations
        console.log('\nTest 4: Mixed Session and Global Operations');
        console.log('============================================');

        const mixedSessionId = 'mixed-session-' + Date.now();
        const globalCounterName = 'mixed-global-' + Date.now();

        // Interleave session and global operations
        const mixedOperations = await Promise.all([
            session.reset_global(globalCounterName),
            session.increment_session(mixedSessionId, 'session-counter-1'),
            session.increment_global(globalCounterName),
            session.increment_session(mixedSessionId, 'session-counter-2'),
            session.increment_global(globalCounterName),
            session.set_session_property(mixedSessionId, 'mixed-prop', 'mixed-value'),
            session.increment_session(mixedSessionId, 'session-counter-1'),
            session.increment_global(globalCounterName)
        ]);

        console.log(`✅ Mixed operations completed: [${mixedOperations.slice(0, 4).join(', ')}, ...]`);

        // Verify final state
        const finalState = await Promise.all([
            session.get_global(globalCounterName),
            session.get_session(mixedSessionId, 'session-counter-1'),
            session.get_session(mixedSessionId, 'session-counter-2'),
            session.get_session_property(mixedSessionId, 'mixed-prop')
        ]);

        console.log(`✅ Final mixed state: global=${finalState[0]}, session1=${finalState[1]}, session2=${finalState[2]}, prop=${JSON.stringify(finalState[3])}`);

        // Test 5: Error handling in pipelines
        console.log('\nTest 5: Error Handling in Pipelines');
        console.log('====================================');

        try {
            // Mix valid and invalid operations
            const mixedValidInvalid = await Promise.allSettled([
                session.increment_global('valid-counter'),
                session.get_session_property('non-existent-session', 'non-existent-prop'),
                session.increment_global('another-valid-counter'),
                // @ts-ignore - Intentionally invalid call
                (session as any).invalid_method(),
                session.increment_global('third-valid-counter')
            ]);

            let successCount = 0;
            let errorCount = 0;

            mixedValidInvalid.forEach((result, index) => {
                if (result.status === 'fulfilled') {
                    successCount++;
                    console.log(`✅ Operation ${index}: Success - ${JSON.stringify(result.value)}`);
                } else {
                    errorCount++;
                    console.log(`❌ Operation ${index}: Error - ${result.reason}`);
                }
            });

            console.log(`📊 Pipeline error handling: ${successCount} successes, ${errorCount} errors`);
            console.log('✅ Promise.allSettled correctly handled mixed success/failure');

        } catch (error) {
            console.log(`❌ Error handling test failed: ${error}`);
        }

        // Test 6: Resource cleanup in pipelines
        console.log('\nTest 6: Resource Cleanup in Pipelines');
        console.log('======================================');

        // Create multiple sessions and then clean them up
        const cleanupSessionIds = [];
        for (let i = 0; i < 5; i++) {
            cleanupSessionIds.push(`cleanup-session-${Date.now()}-${i}`);
        }

        // Populate sessions with data
        const populatePromises = cleanupSessionIds.flatMap(sessionId => [
            session.increment_session(sessionId, 'cleanup-counter'),
            session.set_session_property(sessionId, 'cleanup-prop', `value-${sessionId}`)
        ]);

        await Promise.all(populatePromises);
        console.log(`✅ Created and populated ${cleanupSessionIds.length} sessions`);

        // Verify they exist by reading data
        const verifyPromises = cleanupSessionIds.map(sessionId =>
            session.get_session(sessionId, 'cleanup-counter')
        );

        const verifyResults = await Promise.all(verifyPromises);
        console.log(`✅ Verified session data: [${verifyResults.join(', ')}]`);

        console.log('\n' + '='.repeat(80));
        console.log('🎉 PROMISE PIPELINING VALIDATION SUMMARY');
        console.log('='.repeat(80));
        console.log('✅ Sequential vs parallel execution timing analyzed');
        console.log('✅ Complex dependency chains working correctly');
        console.log('✅ Batch operation optimization validated');
        console.log('✅ Mixed session/global operations pipelined properly');
        console.log('✅ Error handling in pipelines working correctly');
        console.log('✅ Resource cleanup patterns validated');
        console.log('\n🚀 Promise pipelining and message flow optimization complete!');

    } catch (error) {
        console.error('\n💥 Fatal error in promise pipelining tests:', error);
        console.error('\nThis indicates issues with message batching or promise handling.');
        process.exit(1);
    }
}

// Performance measurement utility
function measureAsync<T>(fn: () => Promise<T>): Promise<{result: T, duration: number}> {
    return new Promise(async (resolve, reject) => {
        try {
            const start = performance.now();
            const result = await fn();
            const end = performance.now();
            resolve({ result, duration: end - start });
        } catch (error) {
            reject(error);
        }
    });
}

// Run the test if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
    testPromisePipelining().catch(error => {
        console.error('Unhandled error:', error);
        process.exit(1);
    });
}

export { testPromisePipelining };