import { resolve } from 'path';
import { defineConfig } from 'vite';

// Use npm package on Netlify and CI (GitHub Actions), local pkg for local dev
const isCI = !!process.env.NETLIFY || !!process.env.CI;

export default defineConfig({
  base: '/',

  server: {
    port: 3100,
  },

  build: {
    target: 'esnext',
    outDir: 'dist',
  },

  optimizeDeps: {
    exclude: ['@industrialalgebra/cliffy-core'],
  },

  resolve: isCI ? {} : {
    alias: {
      '@industrialalgebra/cliffy-core': resolve(__dirname, '../../cliffy-wasm/pkg'),
    },
  },

  esbuild: {
    supported: {
      'top-level-await': true,
    },
  },
});
