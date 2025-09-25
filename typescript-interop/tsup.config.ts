import { defineConfig } from 'tsup';

export default defineConfig({
  entry: [
    'src/comprehensive-test-runner.ts',
    'src/tier1-protocol-compliance.ts',
    'src/tier2-http-batch-corrected.ts',
    'src/tier2-websocket-tests.ts',
    'src/tier3-capability-composition.ts',
    'src/tier3-complex-applications.ts',
    'src/tier3-extreme-stress.ts',
    'src/tier3-websocket-advanced.ts'
  ],
  format: ['esm'],
  dts: false,
  sourcemap: true,
  clean: true,
  target: 'node18',
});