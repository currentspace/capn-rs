// Test to understand the exact format the TypeScript client sends
import { newHttpBatchRpcSession } from 'capnweb';

async function testFormat() {
    console.log('Testing Cap\'n Web TypeScript client format\n');

    // Intercept and log the exact request
    const originalFetch = global.fetch;
    global.fetch = async (url, options) => {
        console.log('=== REQUEST DETAILS ===');
        console.log('URL:', url);
        console.log('Method:', options?.method);
        console.log('Headers:', JSON.stringify(options?.headers, null, 2));
        console.log('\nRaw Body:');
        console.log(options?.body);

        // Split by newlines to see each message
        if (options?.body) {
            const lines = options.body.split('\n').filter(line => line.trim());
            console.log('\nBody split by lines:');
            lines.forEach((line, i) => {
                console.log(`Line ${i}:`, line);
                try {
                    const parsed = JSON.parse(line);
                    console.log(`  Parsed:`, JSON.stringify(parsed));
                } catch (e) {
                    console.log(`  Not valid JSON`);
                }
            });
        }

        // Create a mock response that the client expects
        const mockResponse = new Response('["result",1,["success",5]]', {
            status: 200,
            headers: {
                'Content-Type': 'text/plain' // Try different content types
            }
        });

        return mockResponse;
    };

    try {
        const session = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');
        console.log('\n=== Calling add(2, 3) ===');
        const result = await session.add(2, 3);
        console.log('Result:', result);
    } catch (error) {
        console.log('Error:', error.message);
    }

    // Test with different operations
    global.fetch = async (url, options) => {
        console.log('\n=== Multiple operations ===');
        console.log('Body:', options?.body);
        return new Response('', { status: 200 });
    };

    try {
        const session2 = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');

        // Make multiple calls to see batching
        const p1 = session2.multiply(4, 5);
        const p2 = session2.divide(10, 2);

        await Promise.all([p1, p2]);
    } catch (error) {
        // Expected to fail
    }
}