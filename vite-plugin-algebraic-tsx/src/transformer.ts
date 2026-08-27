/**
 * Algebraic TSX Transformer
 * Converts TSX syntax to jsx() function calls using Babel
 */

import { parse } from '@babel/parser';
import traverse from '@babel/traverse';
import generate from '@babel/generator';
import * as t from '@babel/types';
import MagicString from 'magic-string';

export interface TransformOptions {
  filename: string;
  jsxFactory: string;
  jsxFragment: string;
  jsxImportSource: string;
  algebraicCombinators: string[];
  debug: boolean;
}

export interface TransformResult {
  code: string;
  map?: any;
  transformed: boolean;
}

export function transformAlgebraicTSX(
  code: string, 
  options: TransformOptions
): TransformResult {
  const {
    filename,
    jsxFactory,
    jsxFragment,
    jsxImportSource,
    algebraicCombinators,
    debug
  } = options;

  let transformed = false;
  const magicString = new MagicString(code);

  try {
    // Parse the code into an AST
    const ast = parse(code, {
      sourceType: 'module',
      allowImportExportEverywhere: true,
      allowReturnOutsideFunction: true,
      plugins: [
        'typescript',
        'jsx',
        'decorators-legacy',
        // babel 8: classProperties/objectRestSpread/asyncGenerators/
        // dynamicImport/nullishCoalescingOperator/optionalChaining/
        // exportNamespaceFrom are standard and enabled by default —
        // removed. functionBind/exportDefaultFrom remain proposals.
        'functionBind',
        'exportDefaultFrom'
      ]
    });

    // Track imports to ensure jsx is available
    let hasJsxImport = false;
    let jsxImportSpecifiers: string[] = [];

    // First pass: Check existing imports
    traverse(ast, {
      ImportDeclaration(path) {
        if (path.node.source.value === jsxImportSource) {
          hasJsxImport = true;
          
          // Extract existing import specifiers
          path.node.specifiers.forEach(spec => {
            if (t.isImportSpecifier(spec) && t.isIdentifier(spec.imported)) {
              jsxImportSpecifiers.push(spec.imported.name);
            }
          });
        }
      }
    });

    // Track what jsx functions we need to import
    const neededImports = new Set<string>();
    const transformations: Array<{
      start: number;
      end: number;
      replacement: string;
    }> = [];

    // Second pass: Transform JSX elements
    traverse(ast, {
      JSXElement(path) {
        const element = path.node;
        const openingElement = element.openingElement;
        
        if (t.isJSXIdentifier(openingElement.name)) {
          const elementName = openingElement.name.name;
          
          // Check if this is an algebraic combinator
          if (algebraicCombinators.includes(elementName)) {
            neededImports.add(elementName);
            
            const transformation = transformAlgebraicCombinator(element, elementName);
            if (transformation) {
              transformations.push({
                start: element.start!,
                end: element.end!,
                replacement: transformation
              });
              transformed = true;
            }
          } else {
            // Regular element - transform to jsx() call
            neededImports.add(jsxFactory);
            
            const transformation = transformRegularElement(element, jsxFactory);
            if (transformation) {
              transformations.push({
                start: element.start!,
                end: element.end!,
                replacement: transformation
              });
              transformed = true;
            }
          }
        }
      },

      JSXFragment(path) {
        // Transform fragments to jsx(Fragment, ...)
        neededImports.add(jsxFragment);
        neededImports.add(jsxFactory);
        
        const transformation = transformFragment(path.node, jsxFactory, jsxFragment);
        if (transformation) {
          transformations.push({
            start: path.node.start!,
            end: path.node.end!,
            replacement: transformation
          });
          transformed = true;
        }
      }
    });

    // Apply transformations in reverse order (to maintain positions)
    transformations
      .sort((a, b) => b.start - a.start)
      .forEach(({ start, end, replacement }) => {
        magicString.overwrite(start, end, replacement);
      });

    // Add missing imports if we transformed anything
    if (transformed && !hasJsxImport) {
      const importsToAdd = Array.from(neededImports).filter(
        imp => !jsxImportSpecifiers.includes(imp)
      );
      
      if (importsToAdd.length > 0) {
        const importStatement = `import { ${importsToAdd.join(', ')} } from '${jsxImportSource}';\n`;
        magicString.prepend(importStatement);
      }
    } else if (transformed && hasJsxImport) {
      // Check if we need to add imports to existing import statement
      const newImports = Array.from(neededImports).filter(
        imp => !jsxImportSpecifiers.includes(imp)
      );
      
      if (newImports.length > 0) {
        // Find the existing import and add to it
        // This is a simplified approach - in production, you'd want more robust import handling
        traverse(ast, {
          ImportDeclaration(path) {
            if (path.node.source.value === jsxImportSource) {
              const existingSpecifiers = path.node.specifiers
                .map(spec => t.isImportSpecifier(spec) && t.isIdentifier(spec.imported) 
                  ? spec.imported.name : null)
                .filter(Boolean) as string[];
              
              const allImports = [...existingSpecifiers, ...newImports];
              const start = path.node.start!;
              const end = path.node.end!;
              
              magicString.overwrite(
                start, 
                end, 
                `import { ${allImports.join(', ')} } from '${jsxImportSource}';`
              );
            }
          }
        });
      }
    }

    if (debug && transformed) {
      console.log(`[Algebraic TSX] Applied ${transformations.length} transformations`);
      console.log(`[Algebraic TSX] Added imports: ${Array.from(neededImports).join(', ')}`);
    }

    return {
      code: magicString.toString(),
      map: magicString.generateMap({ 
        source: filename,
        includeContent: true 
      }),
      transformed
    };

  } catch (error) {
    if (debug) {
      console.error('[Algebraic TSX] Parse error:', error);
    }
    throw new Error(`Failed to parse ${filename}: ${error}`);
  }
}

function transformAlgebraicCombinator(
  element: t.JSXElement,
  elementName: string
): string | null {
  // Transform <When condition={x}>{children}</When> 
  // to When({ condition: x, children: [...] })
  
  const props: string[] = [];
  
  // Process attributes
  element.openingElement.attributes.forEach(attr => {
    if (t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name)) {
      const propName = attr.name.name;
      let propValue = 'true'; // Default for boolean props
      
      if (attr.value) {
        if (t.isStringLiteral(attr.value)) {
          propValue = JSON.stringify(attr.value.value);
        } else if (t.isJSXExpressionContainer(attr.value)) {
          const generated = generate(attr.value.expression);
          propValue = generated.code;
        }
      }
      
      props.push(`${propName}: ${propValue}`);
    }
  });
  
  // Process children
  if (element.children.length > 0) {
    const childrenCode = element.children
      .map(child => {
        if (t.isJSXText(child)) {
          const trimmed = child.value.trim();
          return trimmed ? JSON.stringify(trimmed) : null;
        } else if (t.isJSXElement(child) || t.isJSXFragment(child)) {
          return generate(child).code;
        } else if (t.isJSXExpressionContainer(child)) {
          return generate(child.expression).code;
        }
        return null;
      })
      .filter(Boolean);
    
    if (childrenCode.length === 1) {
      props.push(`children: ${childrenCode[0]}`);
    } else if (childrenCode.length > 1) {
      props.push(`children: [${childrenCode.join(', ')}]`);
    }
  }
  
  return `${elementName}({ ${props.join(', ')} })`;
}

function transformRegularElement(
  element: t.JSXElement,
  jsxFactory: string
): string | null {
  // Transform <div prop={x}>{children}</div> 
  // to jsx('div', { prop: x }, children...)
  
  const openingElement = element.openingElement;
  let elementName = '';
  
  if (t.isJSXIdentifier(openingElement.name)) {
    elementName = `'${openingElement.name.name}'`;
  } else if (t.isJSXMemberExpression(openingElement.name)) {
    // Handle cases like <Component.SubComponent>
    elementName = generate(openingElement.name).code;
  }
  
  // Build props object
  const propEntries: string[] = [];
  
  openingElement.attributes.forEach(attr => {
    if (t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name)) {
      const propName = attr.name.name;
      let propValue = 'true';
      
      if (attr.value) {
        if (t.isStringLiteral(attr.value)) {
          propValue = JSON.stringify(attr.value.value);
        } else if (t.isJSXExpressionContainer(attr.value)) {
          propValue = generate(attr.value.expression).code;
        }
      }
      
      propEntries.push(`${propName}: ${propValue}`);
    } else if (t.isJSXSpreadAttribute(attr)) {
      propEntries.push(`...${generate(attr.argument).code}`);
    }
  });
  
  const propsObject = propEntries.length > 0 
    ? `{ ${propEntries.join(', ')} }` 
    : 'null';
  
  // Process children
  const children = element.children
    .map(child => {
      if (t.isJSXText(child)) {
        const trimmed = child.value.trim();
        return trimmed ? JSON.stringify(trimmed) : null;
      } else if (t.isJSXElement(child) || t.isJSXFragment(child)) {
        return generate(child).code;
      } else if (t.isJSXExpressionContainer(child)) {
        return generate(child.expression).code;
      }
      return null;
    })
    .filter(Boolean);
  
  // Build jsx call
  if (children.length === 0) {
    return `${jsxFactory}(${elementName}, ${propsObject})`;
  } else if (children.length === 1) {
    return `${jsxFactory}(${elementName}, { ...${propsObject === 'null' ? '{}' : propsObject.slice(1, -1)}, children: ${children[0]} })`;
  } else {
    return `${jsxFactory}(${elementName}, { ...${propsObject === 'null' ? '{}' : propsObject.slice(1, -1)}, children: [${children.join(', ')}] })`;
  }
}

function transformFragment(
  fragment: t.JSXFragment,
  jsxFactory: string,
  jsxFragment: string
): string | null {
  // Transform <>{children}</> to jsx(Fragment, null, ...children)
  
  const children = fragment.children
    .map(child => {
      if (t.isJSXText(child)) {
        const trimmed = child.value.trim();
        return trimmed ? JSON.stringify(trimmed) : null;
      } else if (t.isJSXElement(child) || t.isJSXFragment(child)) {
        return generate(child).code;
      } else if (t.isJSXExpressionContainer(child)) {
        return generate(child.expression).code;
      }
      return null;
    })
    .filter(Boolean);
  
  if (children.length === 0) {
    return `${jsxFactory}(${jsxFragment}, null)`;
  } else {
    return `${jsxFactory}(${jsxFragment}, { children: [${children.join(', ')}] })`;
  }
}