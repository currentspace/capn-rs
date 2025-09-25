// Debug script to test what the TypeScript client sends
import { newHttpBatchRpcSession } from 'capnweb';

async function debugTest() {
    console.log('Debug: Testing TypeScript client request format');

    // Create a proxy to intercept the actual fetch call
    const originalFetch = global.fetch;
    global.fetch = async (url, options) => {
        console.log('=== INTERCEPTED FETCH ===');
        console.log('URL:', url);
        console.log('Method:', options?.method);
        console.log('Headers:', options?.headers);
        console.log('Body:', options?.body);

        // Try to parse the body if it's JSON
        if (options?.body) {
            try {
                const parsed = JSON.parse(options.body);
                console.log('Parsed Body:', JSON.stringify(parsed, null, 2));
            } catch (e) {
                console.log('Body is not JSON');
            }
        }

        // Call the original fetch
        return originalFetch(url, options);
    };

    try {
        const session = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');

        // Try a simple call
        console.log('\nAttempting to call add(2, 3)...');
        const result = await session.add(2, 3);
        console.log('Result:', result);
    } catch (error) {
        console.log('Error:', error.message);
        console.log('Full error:', error);
    }
}

debugTest();