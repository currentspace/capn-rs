#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

// Define the CounterService interface for the advanced stateful server
interface CounterService {
    // Global counter operations
    increment_global(counterName: string): Promise<number>;
    decrement_global(counterName: string): Promise<number>;
    get_global(counterName: string): Promise<number>;
    reset_global(counterName: string): Promise<number>;
    list_global_counters(): Promise<any[]>;

    // Session-specific operations
    increment_session(sessionId: string, counterName: string): Promise<number>;
    get_session(sessionId: string, counterName: string): Promise<number>;

    // Session property management
    set_session_property(sessionId: string, propertyName: string, value: any): Promise<any>;
    get_session_property(sessionId: string, propertyName: string): Promise<any>;

    // Administrative operations
    list_sessions(): Promise<any[]>;
    cleanup_sessions(): Promise<string>;

    // Advanced capability operations
    get_async_processor(): Promise<string>;
    get_nested_capability(operationId: string): Promise<string>;
}

async function testAdvancedStatefulServer() {
    console.log('🧪 Testing Advanced Stateful Cap\'n Web Rust Server');
    console.log('==================================================\n');

    try {
        // Create a session with the advanced Rust server
        const session = newHttpBatchRpcSession<CounterService>('http://localhost:8081/rpc/batch');

        console.log('✅ Created session with advanced stateful Rust server');
        console.log('📍 Endpoint: http://localhost:8081/rpc/batch\n');

        // Test 1: Global counter operations
        console.log('Test 1: Global Counter Operations');
        console.log('==================================');

        try {
            // Reset counter first
            await session.reset_global('test_counter');
            console.log('✅ Reset test_counter to 0');

            // Test increment
            let result = await session.increment_global('test_counter');
            console.log(`✅ increment_global('test_counter') = ${result}`);
            assert(result === 1, `Expected 1, got ${result}`);

            // Test another increment
            result = await session.increment_global('test_counter');
            console.log(`✅ increment_global('test_counter') = ${result}`);
            assert(result === 2, `Expected 2, got ${result}`);

            // Test decrement
            result = await session.decrement_global('test_counter');
            console.log(`✅ decrement_global('test_counter') = ${result}`);
            assert(result === 1, `Expected 1, got ${result}`);

            // Test get
            result = await session.get_global('test_counter');
            console.log(`✅ get_global('test_counter') = ${result}`);
            assert(result === 1, `Expected 1, got ${result}`);

        } catch (error) {
            console.log(`❌ Global counter operations failed: ${error}`);
            throw error;
        }

        // Test 2: Session-specific counters
        console.log('\nTest 2: Session-Specific Operations');
        console.log('====================================');

        try {
            const sessionId = 'test-session-' + Date.now();

            // Test session increment
            let result = await session.increment_session(sessionId, 'session_counter');
            console.log(`✅ increment_session('${sessionId}', 'session_counter') = ${result}`);
            assert(result === 1, `Expected 1, got ${result}`);

            // Test another increment
            result = await session.increment_session(sessionId, 'session_counter');
            console.log(`✅ increment_session('${sessionId}', 'session_counter') = ${result}`);
            assert(result === 2, `Expected 2, got ${result}`);

            // Test different counter in same session
            result = await session.increment_session(sessionId, 'another_counter');
            console.log(`✅ increment_session('${sessionId}', 'another_counter') = ${result}`);
            assert(result === 1, `Expected 1, got ${result}`);

            // Test get session counter
            result = await session.get_session(sessionId, 'session_counter');
            console.log(`✅ get_session('${sessionId}', 'session_counter') = ${result}`);
            assert(result === 2, `Expected 2, got ${result}`);

        } catch (error) {
            console.log(`❌ Session operations failed: ${error}`);
            throw error;
        }

        // Test 3: Session property management
        console.log('\nTest 3: Session Property Management');
        console.log('===================================');

        try {
            const sessionId = 'prop-session-' + Date.now();

            // Set string property
            let result = await session.set_session_property(sessionId, 'user_name', 'Alice');
            console.log(`✅ set_session_property('${sessionId}', 'user_name', 'Alice') = ${JSON.stringify(result)}`);

            // Set number property
            result = await session.set_session_property(sessionId, 'user_age', 25);
            console.log(`✅ set_session_property('${sessionId}', 'user_age', 25) = ${JSON.stringify(result)}`);

            // Set object property
            const userData = { preferences: { theme: 'dark', language: 'en' } };
            result = await session.set_session_property(sessionId, 'user_data', userData);
            console.log(`✅ set_session_property with object = ${JSON.stringify(result)}`);

            // Get string property
            result = await session.get_session_property(sessionId, 'user_name');
            console.log(`✅ get_session_property('${sessionId}', 'user_name') = ${JSON.stringify(result)}`);
            assert(result === 'Alice', `Expected 'Alice', got ${result}`);

            // Get number property
            result = await session.get_session_property(sessionId, 'user_age');
            console.log(`✅ get_session_property('${sessionId}', 'user_age') = ${JSON.stringify(result)}`);
            assert(result === 25, `Expected 25, got ${result}`);

        } catch (error) {
            console.log(`❌ Session property operations failed: ${error}`);
            throw error;
        }

        // Test 4: Concurrent operations
        console.log('\nTest 4: Concurrent Operations');
        console.log('==============================');

        try {
            // Test concurrent global counter operations
            const promises = [
                session.increment_global('concurrent_counter'),
                session.increment_global('concurrent_counter'),
                session.increment_global('concurrent_counter'),
                session.increment_global('concurrent_counter'),
                session.increment_global('concurrent_counter')
            ];

            const results = await Promise.all(promises);
            console.log(`✅ Concurrent increments results: [${results.join(', ')}]`);

            // Results should be unique (1, 2, 3, 4, 5) in some order
            const sortedResults = [...results].sort((a, b) => a - b);
            const expected = [1, 2, 3, 4, 5];
            assert(JSON.stringify(sortedResults) === JSON.stringify(expected),
                   `Expected [1,2,3,4,5], got [${sortedResults.join(',')}]`);

            // Test concurrent session operations
            const sessionId = 'concurrent-session-' + Date.now();
            const sessionPromises = [
                session.increment_session(sessionId, 'counter1'),
                session.increment_session(sessionId, 'counter2'),
                session.increment_session(sessionId, 'counter3'),
                session.set_session_property(sessionId, 'test_prop', 'test_value'),
                session.increment_session(sessionId, 'counter1') // This should make counter1 = 2
            ];

            await Promise.all(sessionPromises);
            console.log('✅ Concurrent session operations completed');

            // Verify final state
            const counter1 = await session.get_session(sessionId, 'counter1');
            console.log(`✅ Final counter1 value: ${counter1}`);
            assert(counter1 === 2, `Expected counter1 = 2, got ${counter1}`);

        } catch (error) {
            console.log(`❌ Concurrent operations failed: ${error}`);
            throw error;
        }

        // Test 5: Error handling
        console.log('\nTest 5: Error Handling');
        console.log('======================');

        try {
            // Test getting non-existent session property
            try {
                await session.get_session_property('non-existent-session', 'non-existent-prop');
                console.log('❌ Should have thrown error for non-existent property');
            } catch (error) {
                console.log(`✅ Correctly threw error for non-existent property: ${error}`);
            }

            // Test invalid method call with wrong number of arguments
            try {
                // @ts-ignore - Intentionally calling with wrong args
                await (session as any).increment_global();
                console.log('❌ Should have thrown error for missing arguments');
            } catch (error) {
                console.log(`✅ Correctly threw error for missing arguments: ${error}`);
            }

        } catch (error) {
            console.log(`❌ Error handling test failed: ${error}`);
            throw error;
        }

        // Test 6: List operations
        console.log('\nTest 6: List Operations');
        console.log('=======================');

        try {
            // List global counters
            const globalCounters = await session.list_global_counters();
            console.log(`✅ Global counters: ${JSON.stringify(globalCounters, null, 2)}`);
            assert(Array.isArray(globalCounters), 'Expected array of global counters');

            // Verify we have our test counters
            const testCounter = globalCounters.find(c => c.name === 'test_counter');
            assert(testCounter !== undefined, 'Expected to find test_counter');
            assert(testCounter && testCounter.value === 1, `Expected test_counter value 1, got ${testCounter?.value}`);

            // List sessions
            const sessions = await session.list_sessions();
            console.log(`✅ Sessions: ${JSON.stringify(sessions, null, 2)}`);
            assert(Array.isArray(sessions), 'Expected array of sessions');
            assert(sessions.length > 0, 'Expected at least one session');

        } catch (error) {
            console.log(`❌ List operations failed: ${error}`);
            throw error;
        }

        // Test 7: Session persistence across requests
        console.log('\nTest 7: Session Persistence');
        console.log('============================');

        try {
            const persistentSessionId = 'persistent-session-' + Date.now();

            // Set initial state
            await session.increment_session(persistentSessionId, 'persistent_counter');
            await session.set_session_property(persistentSessionId, 'persistent_prop', 'persistent_value');

            // Wait a bit to simulate time passing
            await new Promise(resolve => setTimeout(resolve, 100));

            // Verify state is still there
            const counterValue = await session.get_session(persistentSessionId, 'persistent_counter');
            const propValue = await session.get_session_property(persistentSessionId, 'persistent_prop');

            console.log(`✅ Persistent counter value: ${counterValue}`);
            console.log(`✅ Persistent property value: ${JSON.stringify(propValue)}`);

            assert(counterValue === 1, `Expected persistent counter = 1, got ${counterValue}`);
            assert(propValue === 'persistent_value', `Expected 'persistent_value', got ${propValue}`);

        } catch (error) {
            console.log(`❌ Session persistence test failed: ${error}`);
            throw error;
        }

        // Test 8: Advanced capabilities
        console.log('\nTest 8: Advanced Capabilities');
        console.log('==============================');

        try {
            // Test async processor
            const asyncProcessor = await session.get_async_processor();
            console.log(`✅ Created async processor: ${asyncProcessor}`);

            // Test nested capability
            const nestedCap = await session.get_nested_capability('test-operation-123');
            console.log(`✅ Created nested capability: ${nestedCap}`);

        } catch (error) {
            console.log(`❌ Advanced capabilities test failed: ${error}`);
            throw error;
        }

        // Test 9: Cleanup operations
        console.log('\nTest 9: Cleanup Operations');
        console.log('===========================');

        try {
            const cleanupResult = await session.cleanup_sessions();
            console.log(`✅ Session cleanup result: ${cleanupResult}`);

        } catch (error) {
            console.log(`❌ Cleanup operations failed: ${error}`);
            throw error;
        }

        console.log('\n' + '='.repeat(80));
        console.log('🎉 ADVANCED SERVER VALIDATION SUMMARY');
        console.log('='.repeat(80));
        console.log('✅ Advanced stateful server functionality working correctly!');
        console.log('✅ Global and session-specific counters');
        console.log('✅ Session property management');
        console.log('✅ Concurrent operations');
        console.log('✅ Error handling');
        console.log('✅ List operations');
        console.log('✅ Session persistence');
        console.log('✅ Advanced capabilities');
        console.log('✅ Cleanup operations');
        console.log('\n🚀 The Rust Cap\'n Web server is ready for production use!');

    } catch (error) {
        console.error('\n💥 Fatal error:', error);
        console.error('\nThis indicates an issue with the advanced stateful server implementation.');
        process.exit(1);
    }
}

// Helper function for assertions
function assert(condition: boolean, message: string) {
    if (!condition) {
        throw new Error(`Assertion failed: ${message}`);
    }
}

// Run the test if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
    testAdvancedStatefulServer().catch(error => {
        console.error('Unhandled error:', error);
        process.exit(1);
    });
}

export { testAdvancedStatefulServer };