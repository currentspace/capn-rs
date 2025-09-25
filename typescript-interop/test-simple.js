#!/usr/bin/env node

// Simple test to debug TypeScript client <-> Rust server communication

import { newHttpBatchRpcSession } from 'capnweb';

async function simpleTest() {
    console.log('=== SIMPLE TEST ===\n');

    try {
        // Create session
        const session = newHttpBatchRpcSession('http://localhost:8080/rpc/batch');
        console.log('Created session\n');

        // Make a simple call
        console.log('Calling add(5, 3)...');
        const result = await session.add(5, 3);
        console.log('Result:', result);

    } catch (error) {
        console.error('Error:', error.message);

        // Try a raw fetch to see what happens
        console.log('\n=== Trying raw fetch ===');
        try {
            const response = await fetch('http://localhost:8080/rpc/batch', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify([])
            });
            console.log('Status:', response.status);
            console.log('Response:', await response.text());
        } catch (e) {
            console.error('Raw fetch error:', e);
        }
    }
}

simpleTest();