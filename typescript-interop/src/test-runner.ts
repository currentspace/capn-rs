#!/usr/bin/env node

import { InteropTestRunner } from './interop-tests';

async function main() {
  console.log('🧪 Cap\'n Web Rust ↔ TypeScript Interoperability Test Runner');
  console.log('================================================================\n');

  const runner = new InteropTestRunner();

  try {
    await runner.run();
  } catch (error) {
    console.error('💥 Test runner encountered a fatal error:', error);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}