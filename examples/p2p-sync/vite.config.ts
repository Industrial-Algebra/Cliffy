import { defineConfig } from 'vite';
import { resolve } from 'path';
import { existsSync } from 'fs';

// Check if local WASM build exists
const localWasmPkg = resolve(__dirname, '../../cliffy-wasm/pkg/cliffy_wasm.js');
const hasLocalWasm = existsSync(localWasmPkg);

// Use npm package on Netlify, CI, or if local WASM build doesn't exist
const useNpmPackage = !!process.env.NETLIFY || !!process.env.CI || !hasLocalWasm;

// Only set base path for Netlify builds, not for CI dev servers (E2E tests)
const base = process.env.NETLIFY ? '/p2p-sync/' : '/';

export default defineConfig({
  base,
  resolve: useNpmPackage ? {} : {
    alias: {
      '@industrialalgebra/cliffy-core': resolve(__dirname, '../../cliffy-wasm/pkg'),
    },
  },
  // Don't pre-bundle WASM packages - they need import.meta.url to work correctly
  optimizeDeps: {
    exclude: ['@industrialalgebra/cliffy-core'],
  },
  server: {
    port: 3010,
    // Ensure .wasm files are served with correct MIME type
    fs: {
      allow: ['..'],
    },
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
