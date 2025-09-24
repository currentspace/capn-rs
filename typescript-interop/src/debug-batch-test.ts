#!/usr/bin/env node

import { newHttpBatchRpcSession } from 'capnweb';

async function debugBatchTest() {
    const port = process.argv[2] || '9006';
    const endpoint = `http://localhost:${port}/rpc/batch`;

    console.log('🔍 Debug: Batch Request Test');
    console.log('Endpoint:', endpoint);
    console.log('');

    try {
        // Create session
        console.log('Creating batch session...');
        const session = newHttpBatchRpcSession(endpoint);

        // Inspect the session internals
        console.log('Session created:', typeof session);
        console.log('Session properties:', Object.keys(session));

        // Test 1: Single operation
        console.log('\n📊 Test 1: Single operation');
        const result1 = await session.add(5, 3);
        console.log('Result 1:', result1);

        // Test 2: Second operation on same session
        console.log('\n📊 Test 2: Second operation');
        try {
            const result2 = await session.multiply(4, 2);
            console.log('Result 2:', result2);
        } catch (e: any) {
            console.log('❌ Second operation failed:', e.message);
            console.log('Error details:', e);
        }

        // Test 3: Try creating a new session
        console.log('\n📊 Test 3: New session');
        const session2 = newHttpBatchRpcSession(endpoint);
        const result3 = await session2.subtract(10, 3);
        console.log('Result 3:', result3);

    } catch (error: any) {
        console.error('Error:', error.message);
        console.error('Stack:', error.stack);
    }
}

// Add request/response debugging
if (typeof global !== 'undefined' && typeof fetch !== 'undefined') {
    const originalFetch = global.fetch || fetch;
    (global as any).fetch = async (url: any, options: any) => {
        console.log('🔄 HTTP Request:', {
            url,
            method: options?.method,
            headers: options?.headers,
            body: options?.body?.toString()
        });

        const response = await originalFetch(url, options);
        const responseText = await response.text();

        console.log('📦 HTTP Response:', {
            status: response.status,
            headers: Object.fromEntries(response.headers.entries()),
            body: responseText
        });

        // Return a new response with the same text
        return new Response(responseText, {
            status: response.status,
            headers: response.headers
        });
    };
}

debugBatchTest().catch(console.error);