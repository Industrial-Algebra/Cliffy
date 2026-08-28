# cliffy-purescript

PureScript bindings for the Cliffy reactive framework, providing type-safe FRP primitives and a declarative Html DSL.

## Installation

Add to your `packages.dhall`:

```dhall
let additions =
      { cliffy-purescript =
          { dependencies =
              [ "arrays", "effect", "prelude", "web-dom", "web-events", "web-html" ]
          , repo = "https://github.com/Industrial-Algebra/Cliffy.git"
          , version = "main"
          , subdir = "cliffy-purescript"
          }
      }

in  upstream // additions
```

Then add to your `spago.dhall`:

```dhall
{ dependencies =
  [ "cliffy-purescript"
  , -- other deps
  ]
}
```

## Quick Start

```purescript
module Main where

import Prelude
import Effect (Effect)
import Cliffy (behavior, update, subscribe)
import Cliffy.Html (div, button, text, behaviorText, mount)
import Cliffy.Html.Attributes (className)
import Cliffy.Html.Events (onClick)

main :: Effect Unit
main = do
  -- Create reactive state
  count <- behavior 0

  -- Build reactive UI
  let app = div [ className "counter" ]
        [ text "Count: "
        , behaviorText count
        , button [ onClick \_ -> update (_ + 1) count ] [ text "+" ]
        ]

  -- Mount to DOM
  _ <- mount app "#app"
  pure unit
```

## Module Structure

### Cliffy (FRP Primitives)

The main module exports polymorphic FRP types and functions:

```purescript
import Cliffy
  ( Behavior        -- Time-varying value
  , Event           -- Discrete occurrences
  , behavior        -- Create a Behavior with initial value
  , sample          -- Get current value of a Behavior
  , set             -- Set Behavior to a new value
  , update          -- Transform Behavior with a function
  , subscribe       -- React to Behavior changes
  , mapBehavior     -- Transform values (like Functor.map)
  , event           -- Create an Event stream
  , emit            -- Fire an Event
  , subscribeEvent  -- React to Events
  , fold            -- Accumulate Events into a Behavior
  )
```

#### Behavior Example

```purescript
counter :: Effect Unit
counter = do
  count <- behavior 0

  -- Subscribe to changes
  _ <- subscribe (\n -> log $ "Count is: " <> show n) count

  -- Update the value
  update (_ + 1) count  -- logs "Count is: 1"
  update (_ + 1) count  -- logs "Count is: 2"

  -- Get current value
  current <- sample count
  log $ "Current: " <> show current  -- "Current: 2"
```

#### Event Example

```purescript
clickTracker :: Effect Unit
clickTracker = do
  clicks <- event

  -- Fold clicks into a count
  clickCount <- fold 0 (\acc _ -> acc + 1) clicks

  -- Subscribe to the count
  _ <- subscribe (\n -> log $ "Clicks: " <> show n) clickCount

  -- Emit events
  emit unit clicks  -- logs "Clicks: 1"
  emit unit clicks  -- logs "Clicks: 2"
```

### Cliffy.Html (Type-Safe DSL)

A declarative DSL for building reactive DOM elements:

```purescript
import Cliffy.Html
  ( Html, Element    -- Types
  , div, div_        -- Container elements
  , span, span_
  , button, button_
  , p, p_, h1_, h2_  -- Text elements
  , input, form      -- Form elements
  , ul, ol, li       -- Lists
  , text             -- Static text
  , behaviorText     -- Reactive text (updates automatically)
  , behaviorHtml     -- Reactive HTML content
  , mount            -- Attach to DOM
  )
```

Elements with `_` suffix take no attributes (e.g., `div_` vs `div`).

### Cliffy.Html.Attributes

```purescript
import Cliffy.Html.Attributes
  ( className, id, style
  , href, src, alt
  , type_, value, placeholder
  , disabled, checked, hidden
  , dataAttr  -- data-* attributes
  )
```

### Cliffy.Html.Events

```purescript
import Cliffy.Html.Events
  ( onClick, onChange, onInput
  , onSubmit, onFocus, onBlur
  , onKeyDown, onKeyUp, onKeyPress
  , onMouseEnter, onMouseLeave
  )
```

## Patterns

### Reactive Lists

```purescript
todoList :: Effect Html
todoList = do
  items <- behavior ["Buy milk", "Write code", "Sleep"]

  pure $ ul_ $ map (\item -> li_ [ text item ]) <$> sample items
```

### Form Handling

```purescript
loginForm :: Effect Html
loginForm = do
  username <- behavior ""
  password <- behavior ""

  pure $ form [ onSubmit \e -> do
      preventDefault e
      u <- sample username
      p <- sample password
      log $ "Login: " <> u
    ]
    [ input [ type_ "text"
            , placeholder "Username"
            , onInput \e -> set (targetValue e) username
            ]
    , input [ type_ "password"
            , placeholder "Password"
            , onInput \e -> set (targetValue e) password
            ]
    , button [ type_ "submit" ] [ text "Login" ]
    ]
```

### Conditional Rendering

```purescript
toggle :: Effect Html
toggle = do
  visible <- behavior true

  pure $ div_
    [ button [ onClick \_ -> update not visible ]
        [ text "Toggle" ]
    , behaviorHtml visible \v ->
        if v
        then div_ [ text "Now you see me!" ]
        else div_ []
    ]
```

## API Comparison: TypeScript vs PureScript

| Operation | TypeScript | PureScript |
|-----------|------------|------------|
| Create | `behavior(0)` | `behavior 0` |
| Update | `count.update(n => n + 1)` | `update (_ + 1) count` |
| Sample | `count.sample()` | `sample count` |
| Subscribe | `count.subscribe(cb)` | `subscribe cb count` |
| Map | `count.map(f)` | `mapBehavior f count` |

The PureScript API uses curried functions with the Behavior as the last argument, enabling point-free style:

```purescript
-- Point-free increment
increment :: Behavior Int -> Effect Unit
increment = update (_ + 1)
```

## FFI Details

The package uses JavaScript FFI (`Cliffy/Foreign.js`) to bridge PureScript with the browser. The FFI provides:

- `createBehaviorImpl` - Creates a Behavior with subscriber management
- `updateImpl`, `sampleImpl`, `setImpl` - Behavior operations
- `subscribeImpl` - Returns an unsubscribe effect
- `createEventImpl`, `emitImpl` - Event primitives
- `foldImpl` - Event-to-Behavior accumulation

All DOM operations are effectful and return `Effect` types.

## Development

```bash
# Build
spago build

# Test
spago test

# Generate docs
spago docs
```

## License

MIT
