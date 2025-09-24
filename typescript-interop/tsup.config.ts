import { defineConfig } from 'tsup';

export default defineConfig({
  entry: [
    'src/official-client-test.ts',
    'src/fixed-official-client-test.ts',
    'src/debug-client.ts',
    'src/test-newline-format.ts',
    'src/advanced-server-test.ts',
    'src/promise-pipelining-test.ts',
    'src/comprehensive-test-runner.ts',
    'src/comprehensive-stateful-test.ts',
    'src/tier1-protocol-compliance.ts',
    'src/tier2-stateful-sessions.ts',
    'src/tier2-websocket-tests.ts',
    'src/tier3-complex-applications.ts',
    'src/tier3-websocket-advanced.ts',
    'src/cross-transport-interop.ts',
    'src/tier3-extreme-stress.ts',
    'src/tier3-capability-composition.ts',
    'src/advanced-features-demo.ts'
  ],
  format: ['esm'],
  dts: false,
  sourcemap: true,
  clean: true,
  target: 'node18',
});