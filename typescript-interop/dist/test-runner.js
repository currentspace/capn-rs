#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const interop_tests_1 = require("./interop-tests");
async function main() {
    console.log('🧪 Cap\'n Web Rust ↔ TypeScript Interoperability Test Runner');
    console.log('================================================================\n');
    const runner = new interop_tests_1.InteropTestRunner();
    try {
        await runner.run();
    }
    catch (error) {
        console.error('💥 Test runner encountered a fatal error:', error);
        process.exit(1);
    }
}
if (require.main === module) {
    main();
}
//# sourceMappingURL=test-runner.js.map