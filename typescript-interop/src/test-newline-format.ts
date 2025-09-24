#!/usr/bin/env node

// Test with newline-delimited format (what official client sends)

async function testNewlineFormat() {
    console.log('🔍 Testing newline-delimited format (official Cap\'n Web)\n');

    // Send exactly what the official client would send
    const body = '["push",["import",0,["add"],[5,3]]]';

    console.log('📤 Sending (newline-delimited, no Content-Type):');
    console.log(body);

    try {
        const response = await fetch('http://localhost:8080/rpc/batch', {
            method: 'POST',
            // NO Content-Type header (like official client)
            body: body,
        });

        console.log('\n📥 Response:');
        console.log(`Status: ${response.status} ${response.statusText}`);
        console.log('Headers:', Object.fromEntries(response.headers.entries()));

        const responseBody = await response.text();
        console.log('Body:', responseBody);
        console.log('Body length:', responseBody.length);

        // The official client expects empty string or newline-delimited responses
        if (responseBody === '') {
            console.log('\n✅ Got empty response (expected for Push without Pull)');
        } else {
            console.log('\n📝 Parsing newline-delimited response:');
            const lines = responseBody.split('\n').filter(line => line.trim());
            for (const line of lines) {
                console.log('  Line:', line);
                try {
                    const parsed = JSON.parse(line);
                    console.log('  Parsed:', JSON.stringify(parsed));
                } catch (e) {
                    console.log('  Parse error:', e);
                }
            }
        }

    } catch (error) {
        console.error('Request failed:', error);
    }
}

testNewlineFormat();