/**
 * Shared Vite configuration for Cliffy examples
 *
 * Usage in example vite.config.ts:
 * ```typescript
 * import { createExampleConfig } from '../vite.config.shared';
 * export default createExampleConfig(__dirname);
 * ```
 */

import { defineConfig, UserConfig } from 'vite';
import { resolve } from 'path';

export interface ExampleConfigOptions {
  /** Additional Vite plugins */
  plugins?: any[];
  /** Custom port (default: 3000) */
  port?: number;
  /** Additional path aliases */
  aliases?: Record<string, string>;
}

export function createExampleConfig(
  dirname: string,
  options: ExampleConfigOptions = {}
): UserConfig {
  const { plugins = [], port = 3000, aliases = {} } = options;

  return defineConfig({
    plugins,

    server: {
      port,
      host: '0.0.0.0',
      watch: {
        // Watch WASM package for changes
        ignored: ['!**/cliffy-wasm/pkg/**'],
      },
    },

    build: {
      target: 'esnext',
      outDir: 'dist',
      rollupOptions: {
        output: {
          manualChunks: {
            cliffy: ['@industrialalgebra/cliffy-core'],
            shared: ['@cliffy/shared'],
          },
        },
      },
    },

    optimizeDeps: {
      // Exclude WASM package from optimization (it's pre-built)
      exclude: ['@industrialalgebra/cliffy-core'],
    },

    resolve: {
      alias: {
        // Point to WASM package
        '@industrialalgebra/cliffy-core': resolve(dirname, '../../cliffy-wasm/pkg'),
        // Point to shared utilities
        '@cliffy/shared': resolve(dirname, '../shared/src'),
        // Custom aliases
        ...aliases,
      },
    },

    // Enable top-level await for WASM initialization
    esbuild: {
      supported: {
        'top-level-await': true,
      },
    },
  }) as UserConfig;
}

export default createExampleConfig;
