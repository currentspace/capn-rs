#!/usr/bin/env node

// Simple test to see what the official Cap'n Web client actually sends

async function debugRequest() {
    console.log('🔍 Debugging Cap\'n Web Client Request\n');

    // Manually construct what we think the client should send
    const requestBody = [
        ["push", ["import", 0, ["add"], [5, 3]]]
    ];

    console.log('📤 Sending request:');
    console.log(JSON.stringify(requestBody, null, 2));

    try {
        const response = await fetch('http://localhost:8080/rpc/batch', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(requestBody),
        });

        console.log('\n📥 Response:');
        console.log(`Status: ${response.status} ${response.statusText}`);
        console.log('Headers:', Object.fromEntries(response.headers.entries()));

        const responseBody = await response.text();
        console.log('Body:', responseBody);

        if (response.ok) {
            try {
                const parsed = JSON.parse(responseBody);
                console.log('\nParsed response:', JSON.stringify(parsed, null, 2));
            } catch (e) {
                console.log('Could not parse as JSON');
            }
        }

    } catch (error) {
        console.error('Request failed:', error);
    }
}

debugRequest();