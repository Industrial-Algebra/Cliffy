/**
 * Vitest setup file - initializes WASM before tests run.
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

// Get the directory of this file
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load the WASM module synchronously
const wasmPath = join(__dirname, '../../cliffy-wasm/pkg/cliffy_wasm_bg.wasm');
const wasmBuffer = readFileSync(wasmPath);

// Import and initialize WASM
import { initSync, init } from '@industrialalgebra/cliffy-core';

// Initialize with the WASM buffer (using object format to avoid deprecation warning)
initSync({ module: wasmBuffer });

// Call the Cliffy init function
init();

console.log('cliffy-wasm initialized for tests');
