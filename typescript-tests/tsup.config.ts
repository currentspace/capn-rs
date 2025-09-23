import { defineConfig } from 'tsup'

export default defineConfig({
  entry: [
    'src/**/*.ts',
    '!src/**/*.test.ts'
  ],
  format: ['esm'],
  target: 'node24',
  outDir: 'dist',
  clean: true,
  sourcemap: true,
  splitting: false,
  dts: true,
  minify: false,
  keepNames: true,
  treeshake: true,
  bundle: false,
  platform: 'node',
  esbuildOptions: (options) => {
    options.keepNames = true
  }
})