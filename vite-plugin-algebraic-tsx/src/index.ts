/**
 * Vite Plugin for Algebraic TSX Transformation
 * Transforms TSX syntax into Cliffy jsx() function calls
 */

import type { Plugin } from 'vite';
import { transformAlgebraicTSX } from './transformer';

export interface AlgebraicTSXOptions {
  /**
   * File extensions to transform
   * @default ['.tsx', '.jsx']
   */
  extensions?: string[];
  
  /**
   * Include patterns for files to transform
   * @default /\.(tsx?|jsx?)$/
   */
  include?: RegExp | RegExp[];
  
  /**
   * Exclude patterns for files to skip
   * @default /node_modules/
   */
  exclude?: RegExp | RegExp[];
  
  /**
   * JSX factory function name
   * @default 'jsx'
   */
  jsxFactory?: string;
  
  /**
   * JSX fragment factory function name
   * @default 'Fragment'
   */
  jsxFragment?: string;
  
  /**
   * Import source for jsx functions
   * @default '@cliffy/typescript'
   */
  jsxImportSource?: string;
  
  /**
   * Enable debugging output
   * @default false
   */
  debug?: boolean;
  
  /**
   * Algebraic combinators that should be treated as function calls
   * @default ['When', 'For', 'Map', 'Switch', 'Case', 'Default', 'Else', 'Filter', 'Combine', 'Memoize']
   */
  algebraicCombinators?: string[];
}

const DEFAULT_COMBINATORS = [
  'When', 'For', 'Map', 'Switch', 'Case', 'Default', 'Else', 
  'Filter', 'Combine', 'Memoize', 'FlatMap'
];

export default function algebraicTSX(options: AlgebraicTSXOptions = {}): Plugin {
  const {
    extensions = ['.tsx', '.jsx'],
    include = /\.(tsx?|jsx?)$/,
    exclude = /node_modules/,
    jsxFactory = 'jsx',
    jsxFragment = 'Fragment',
    jsxImportSource = '@cliffy/typescript',
    debug = false,
    algebraicCombinators = DEFAULT_COMBINATORS
  } = options;

  return {
    name: 'vite:algebraic-tsx',
    enforce: 'pre',
    
    configResolved(config) {
      if (debug) {
        console.log('[Algebraic TSX] Plugin loaded with options:', {
          extensions,
          jsxFactory,
          jsxFragment,
          jsxImportSource,
          algebraicCombinators
        });
      }
    },

    transform(code, id) {
      // Skip files that don't match our criteria
      if (!shouldTransform(id, include, exclude, extensions)) {
        return null;
      }

      if (debug) {
        console.log(`[Algebraic TSX] Transforming: ${id}`);
      }

      try {
        const result = transformAlgebraicTSX(code, {
          filename: id,
          jsxFactory,
          jsxFragment,
          jsxImportSource,
          algebraicCombinators,
          debug
        });

        if (result.transformed) {
          if (debug) {
            console.log(`[Algebraic TSX] Transformed ${id}:`);
            console.log('Before:', code.slice(0, 200) + '...');
            console.log('After:', result.code.slice(0, 200) + '...');
          }

          return {
            code: result.code,
            map: result.map
          };
        }
      } catch (error) {
        this.error(`[Algebraic TSX] Transform failed for ${id}: ${error}`);
      }

      return null;
    }
  };
}

function shouldTransform(
  id: string,
  include: RegExp | RegExp[],
  exclude: RegExp | RegExp[],
  extensions: string[]
): boolean {
  // Check exclude patterns first
  const excludePatterns = Array.isArray(exclude) ? exclude : [exclude];
  if (excludePatterns.some(pattern => pattern.test(id))) {
    return false;
  }

  // Check include patterns
  const includePatterns = Array.isArray(include) ? include : [include];
  if (!includePatterns.some(pattern => pattern.test(id))) {
    return false;
  }

  // Check file extensions
  return extensions.some(ext => id.endsWith(ext));
}

// Export transformer for external use (AlgebraicTSXOptions is exported
// at its declaration above).
export { transformAlgebraicTSX } from './transformer';