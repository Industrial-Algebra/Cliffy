#!/usr/bin/env node

/**
 * Post-build script for cliffy-wasm package.
 *
 * This script runs after wasm-pack to:
 * 1. Copy html.ts to the pkg directory
 * 2. Update package.json with proper exports for html module
 * 3. Ensure TypeScript users can import { html, mount } from 'cliffy-wasm'
 */

import { copyFileSync, readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const wasmDir = join(projectRoot, 'cliffy-wasm');
const pkgDir = join(wasmDir, 'pkg');

console.log('Running post-wasm-build...');

// 1. Copy html.ts to pkg directory
const srcHtml = join(wasmDir, 'src', 'html.ts');
const destHtml = join(pkgDir, 'html.ts');

try {
    copyFileSync(srcHtml, destHtml);
    console.log('  Copied html.ts to pkg/');
} catch (error) {
    console.error('  Failed to copy html.ts:', error.message);
    process.exit(1);
}

// 2. Update package.json with exports
const pkgJsonPath = join(pkgDir, 'package.json');

try {
    const pkgJson = JSON.parse(readFileSync(pkgJsonPath, 'utf-8'));

    // Set the npm package name (different from crate name)
    pkgJson.name = '@industrialalgebra/cliffy-core';

    // Add html.ts to files array
    if (!pkgJson.files.includes('html.ts')) {
        pkgJson.files.push('html.ts');
    }

    // Add exports field for proper ESM support
    pkgJson.exports = {
        '.': {
            types: './cliffy_wasm.d.ts',
            import: './cliffy_wasm.js',
            default: './cliffy_wasm.js'
        },
        './html': {
            types: './html.ts',
            import: './html.ts',
            default: './html.ts'
        },
        // Allow direct .ts import for bundlers that support it
        './html.ts': './html.ts'
    };

    // Update description
    pkgJson.description = 'WebAssembly bindings for Cliffy reactive framework with Algebraic TSX';

    // Add TypeScript as peer dependency for html.ts
    pkgJson.peerDependencies = {
        typescript: '>=4.7.0'
    };
    pkgJson.peerDependenciesMeta = {
        typescript: { optional: true }
    };

    writeFileSync(pkgJsonPath, JSON.stringify(pkgJson, null, 2) + '\n');
    console.log('  Updated package.json with exports');
} catch (error) {
    console.error('  Failed to update package.json:', error.message);
    process.exit(1);
}

// 3. Create an index.ts that re-exports everything for convenience
const indexContent = `/**
 * Cliffy WASM Package
 *
 * Re-exports both WASM bindings and Algebraic TSX utilities.
 *
 * @example
 * \`\`\`typescript
 * import { behavior, event } from 'cliffy-wasm';
 * import { html, mount } from 'cliffy-wasm/html';
 *
 * const count = behavior(0);
 * const app = html\`<button onclick=\${() => count.update(n => n + 1)}>\${count}</button>\`;
 * mount(app, '#app');
 * \`\`\`
 */

// Re-export WASM bindings
export * from './cliffy_wasm.js';

// Note: html and mount are available from 'cliffy-wasm/html'
// We don't re-export them here to avoid TypeScript compilation in the main entry
`;

try {
    writeFileSync(join(pkgDir, 'index.js'), indexContent);
    console.log('  Created index.js');
} catch (error) {
    console.error('  Failed to create index.js:', error.message);
    // Non-fatal, continue
}

console.log('Post-wasm-build complete!');
