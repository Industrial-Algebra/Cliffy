import { defineConfig } from 'vite';
import { resolve } from 'path';

// Use npm package on Netlify and CI (GitHub Actions), local pkg for local dev
const isCI = !!process.env.NETLIFY || !!process.env.CI;
// Only set base path for Netlify builds, not for CI dev servers (E2E tests)
const base = process.env.NETLIFY ? '/tsx-counter/' : '/';

export default defineConfig({
  base,
  resolve: isCI ? {} : {
    alias: {
      '@industrialalgebra/cliffy-core': resolve(__dirname, '../../cliffy-wasm/pkg'),
    },
  },
  server: {
    port: 3001,
  },
  build: {
    target: 'esnext',
  },
  esbuild: {
    supported: {
      'top-level-await': true,
    },
  },
});
