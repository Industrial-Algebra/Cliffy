# create-cliffy

CLI tool for scaffolding Cliffy projects - both applications and reusable component libraries.

## Quick Start

```bash
# Create a new Cliffy app (interactive)
npx create-cliffy my-app

# Create with a specific template
npx create-cliffy my-app --template typescript-vite

# Create a component library
npx create-cliffy my-lib --library
```

## Installation

You can use `create-cliffy` directly with npx (recommended) or install it globally:

```bash
# Use directly with npx (recommended)
npx create-cliffy my-project

# Or install globally
npm install -g create-cliffy
create-cliffy my-project
```

## Usage

### Interactive Mode

Simply run with a project name:

```bash
npx create-cliffy my-project
```

You'll be prompted to select:
1. **Project type** - Application or Component Library
2. **Template** - TypeScript+Vite, Bun, or PureScript (for apps)

### Command Line Options

```bash
npx create-cliffy <project-name> [options]

Options:
  --template, -t <template>  Template to use (see Templates below)
  --library, -l              Create a component library instead of an app
  --help, -h                 Show help
  --version, -v              Show version
```

### Examples

```bash
# Create a TypeScript + Vite application
npx create-cliffy my-app --template typescript-vite

# Create a Bun application
npx create-cliffy my-app --template bun

# Create a PureScript application
npx create-cliffy my-app --template purescript

# Create a component library
npx create-cliffy my-components --library
```

## Templates

### Application Templates

| Template | Description |
|----------|-------------|
| `typescript-vite` | TypeScript app with Vite bundler (recommended) |
| `bun` | TypeScript app optimized for Bun runtime |
| `purescript` | PureScript app with type-safe Html DSL |

### Library Template

| Template | Description |
|----------|-------------|
| `typescript-vite-library` | TypeScript component library with Vite |

## Project Types

### Application

Applications are standalone Cliffy projects that compile to a runnable web app:

- Bundled output in `dist/`
- Development server with HMR
- Ready to deploy

```bash
npx create-cliffy my-app
cd my-app
npm install
npm run dev
```

### Component Library

Component libraries are publishable packages containing reusable Cliffy components:

- ESM output with preserved module structure
- TypeScript declarations generated
- `@industrialalgebra/cliffy-core` as peer dependency
- Package exports configured for npm publishing

```bash
npx create-cliffy my-lib --library
cd my-lib
npm install
npm run build
npm publish  # when ready
```

## App vs Library Differences

| Aspect | Application | Library |
|--------|-------------|---------|
| Output | Bundled `dist/` | Unbundled ESM modules |
| Entry point | `src/main.ts` | `src/index.ts` |
| Build mode | Standard Vite | `build.lib` mode |
| TypeScript | Standard | Declarations enabled |
| Dependencies | `@industrialalgebra/cliffy-core` in deps | `@industrialalgebra/cliffy-core` as peer dep |
| package.json | `private: true` | `exports` map configured |

## Generated Project Structure

### Application (typescript-vite)

```
my-app/
├── src/
│   ├── main.ts        # Application entry point
│   └── style.css      # Global styles
├── index.html         # HTML template
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### Component Library (typescript-vite-library)

```
my-lib/
├── src/
│   ├── index.ts              # Library exports
│   └── components/
│       └── Counter.ts        # Example component
├── package.json              # With exports map
├── tsconfig.json             # With declaration generation
└── vite.config.ts            # Library build config
```

## Working with Libraries

### Creating Components

Components are functions that use Cliffy's FRP primitives:

```typescript
// src/components/Toggle.ts
import type { Behavior } from '@industrialalgebra/cliffy-core';

export interface ToggleState {
  value: Behavior<boolean>;
  toggle: () => void;
  on: () => void;
  off: () => void;
}

export function createToggle(
  BehaviorClass: new (value: boolean) => Behavior<boolean>,
  initial = false
): ToggleState {
  const value = new BehaviorClass(initial);

  return {
    value,
    toggle: () => value.update(v => !v),
    on: () => value.set(true),
    off: () => value.set(false),
  };
}
```

### Exporting Components

Add your components to `src/index.ts`:

```typescript
export { createToggle } from './components/Toggle';
export type { ToggleState } from './components/Toggle';
```

### Building

```bash
npm run build
```

This generates:
- `dist/index.js` - ESM bundle
- `dist/index.d.ts` - TypeScript declarations

### Publishing

1. Update `package.json` with your package name and description
2. Build: `npm run build`
3. Publish: `npm publish`

### Using Your Library

Consumers install your library alongside `@industrialalgebra/cliffy-core`:

```bash
npm install my-components @industrialalgebra/cliffy-core
```

```typescript
import { Behavior } from '@industrialalgebra/cliffy-core';
import { createToggle } from 'my-components';

const darkMode = createToggle(Behavior, false);
darkMode.toggle();
```

## Development

### Building create-cliffy

```bash
cd tools/create-cliffy
npm install
npm run build
```

### Testing Locally

```bash
# Link globally
npm link

# Test creating a project
create-cliffy test-project

# Or use npx from parent directory
npx ./tools/create-cliffy test-project
```

### Adding New Templates

1. Create a new directory in `src/templates/`
2. Add `.template` files (use `{{projectName}}` and `{{cliffyVersion}}` placeholders)
3. Update `TEMPLATES` array in `src/scaffold.ts`
4. Update the prompts in `src/index.ts` if needed

## Troubleshooting

### "Cannot find module '@industrialalgebra/cliffy-core'"

Make sure you've installed dependencies:

```bash
npm install
```

### Build errors in library mode

Ensure `vite-plugin-dts` is installed:

```bash
npm install -D vite-plugin-dts
```

### TypeScript declaration errors

Check your `tsconfig.json` has the correct settings:

```json
{
  "compilerOptions": {
    "declaration": true,
    "declarationDir": "dist",
    "emitDeclarationOnly": true
  }
}
```

## Related

- [Cliffy Documentation](https://cliffy.dev)
- [@industrialalgebra/cliffy-core](https://www.npmjs.com/package/@industrialalgebra/cliffy-core) - Core FRP primitives
- [Cliffy GitHub](https://github.com/Industrial-Algebra/cliffy)

## License

MIT
