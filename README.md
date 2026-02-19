# V Enhanced — Language Support for Zed

A comprehensive V language extension for [Zed](https://zed.dev/), powered by [v-analyzer](https://github.com/vlang/v-analyzer).

## Version

**Current Version**: 0.1.0

---

## Features

All LSP intelligence is provided by v-analyzer. This extension wires every capability into Zed natively, and adds the full Zed-specific layer on top.

---

### ✅ Core Language Intelligence

- **Diagnostics** — Real V compiler errors, warnings, and notices with:
  - `unused` variables and imports tagged with strikethrough (`DiagnosticTag.unnecessary`)
  - `deprecated` symbols tagged with strikethrough (`DiagnosticTag.deprecated`)
  - All errors, warnings, and notices from the actual V compiler — not heuristics

- **Type Checking** — Full PSI-based type inference:
  - Variable type inference across assignments
  - Cross-module type resolution
  - Generic type parameter tracking
  - Embedded struct type propagation

- **Go-to-Definition** — Navigate to any symbol's declaration:
  - User-defined functions, methods, structs, interfaces, enums, constants, variables
  - Cross-file and cross-module
  - Into the V standard library itself

- **Go-to-Type-Definition** — Jump to the *type* of a variable, not just where it was declared

- **Go-to-Implementation** — Bidirectional interface navigation:
  - From an interface → all structs that implement it
  - From a struct → all interfaces it satisfies
  - From an interface method → all concrete implementations

- **Hover Information** — Rich markdown documentation for every symbol type:
  - Functions and methods (signature + doc comment + module name)
  - Structs, interfaces, enums, type aliases
  - Constants (with value and type)
  - Variables (with inferred type)
  - Parameters and receivers
  - Struct fields (with type and mutability)
  - Enum fields (with value)
  - Import paths (with README content if available)
  - Generic parameters
  - Language keywords (e.g. `chan`)

---

### ✅ Advanced Code Editing

- **Intelligent Code Completion** — 19 context-aware providers including:
  - Struct fields after `.` (knows the type at the cursor)
  - Method completions after `.` on a typed variable
  - Module members after `.` on an import
  - All V keywords in the correct context
  - Attributes (`@[inline]`, `@[heap]`, `@[deprecated]`, etc.)
  - JSON field attribute names
  - `or` block expression completions
  - Loop keyword completions (`break`, `continue`)
  - Compile-time constants (`@FILE`, `@LINE`, `@MOD`, etc.)
  - Import path completions
  - Struct literal field completions
  - Return value completions based on function signature
  - `nil` and `none` in the correct contexts
  - Module name completions
  - Top-level declaration completions

- **Signature Help** — Real-time parameter hints as you type function calls:
  - Active parameter highlighted as you move through arguments
  - Retrigger support (`,` and ` ` re-trigger the hint)
  - Resolves the actual function declaration via PSI — always accurate
  - Works for all functions including stdlib and user-defined

- **Find All References** — PSI-based cross-file reference search:
  - Accurate — not text search, uses the program structure index
  - Works across modules and into the stdlib
  - Skips false positives in comments and strings automatically

- **Rename Symbol** — Safe symbol renaming:
  - `prepareRename` validation before any changes are made
  - Renames across all files in the workspace
  - Correctly handles all reference types (declaration, usage, field access)

- **Document Formatting** — Via `v fmt`:
  - Always produces idiomatic, correctly indented V code
  - Handles the full V syntax including generics, attributes, and C interop

- **Folding Ranges** — Code folding for:
  - Function bodies
  - Struct, interface, and enum bodies
  - `if` / `else` blocks
  - `for` loops
  - `match` expressions
  - All `{ }` block constructs

- **Document Symbols** — Full nested symbol tree in the outline panel:
  - Functions and methods (with signature as detail)
  - Structs (with their fields nested inside)
  - Interfaces (with their methods and fields nested inside)
  - Enums (with their values nested inside, showing implicit values)
  - Constants (with type and value)
  - Type aliases

- **Inlay Hints** — 5 types of inline annotations:
  - **Type hints** — inferred type shown after `:=` assignments: `x: int := 10`
  - **Parameter name hints** — parameter names shown before arguments in function calls
  - **Range operator hints** — `≤` and `<` shown on `..` range operators to clarify inclusivity
  - **Implicit `err →` hints** — shown inside `or { }` blocks and `else` branches of result unwrapping, clarifying the implicit `err` variable
  - **Enum field value hints** — implicit enum field values shown inline
  - **Constant type hints** — type shown after constant declarations

- **Semantic Tokens** — Enhanced syntax highlighting from the LSP layer:
  - Two-pass system for accuracy and performance:
    - Fast syntax-based pass for files over 1000 lines
    - Accurate resolve-based pass for full semantic coloring on smaller files
  - Distinguishes user-defined functions from built-in functions
  - Correctly colors struct names, interface names, enum names vs. primitive types
  - Read vs. write access distinction for variable highlights

- **Workspace Symbols** — Global symbol search across your entire project:
  - Fully-qualified names (`module.Symbol`)
  - Searches all `.v` files via the persistent stub index
  - Fast — backed by the indexed stub cache, not a live file scan

- **Document Highlights** — All occurrences of the symbol under cursor:
  - **Read access** highlighted differently from **write access**
  - Declaration sites highlighted distinctly
  - Updates instantly as you move the cursor

- **Code Actions** — Compiler-integrated quick fixes and refactorings:
  - **Make Mutable** — adds `mut` to an immutable variable; triggered by `is immutable` compiler error
  - **Make Public** — adds `pub` to any declaration
  - **Add `[heap]` Attribute** — adds `[heap]` to a struct definition
  - **Add `[flag]` Attribute** — adds `[flag]` to an enum definition
  - **Import Module** — detects an `undefined ident` compiler error and automatically inserts the correct `import` statement at the right location in the file

- **Code Lens** — Inline action buttons above declarations:
  - `▶ Run workspace` and `single file` above `fn main()`
  - `▶ Run test` above each `fn test_*()` function
  - `all file tests` above the first test in a file
  - `N implementations` above interface declarations (click to show all)
  - `implement N interfaces` above struct declarations (click to show all)

---

### ✅ Syntax Highlighting

Powered by `tree_sitter_v` — v-analyzer's own battle-tested grammar — with comprehensive Zed-specific highlight queries covering:

- Functions, methods, and function calls (distinguished)
- Struct, interface, enum, and type declarations
- All keyword categories: storage modifiers (`mut`, `pub`, `const`, `static`), control flow (`if`, `for`, `match`, `return`), type definitions (`struct`, `interface`, `enum`, `type`)
- String literals: interpreted, raw, C strings, string interpolation
- Escape sequences within strings
- Rune literals
- Integer and float literals
- Boolean literals (`true`, `false`)
- Builtin constants (`nil`, `none`)
- Attributes (`@[...]`)
- Modules and import paths
- Struct fields and selector expressions
- Enum fetch expressions (`Direction.left`)
- Global variable definitions
- Compile-time constructs (`$if`, `$for`, `$else`)
- Labels
- All operators (arithmetic, bitwise, comparison, assignment, channel, range)
- Comments: line (`//`) and block (`/* */`)
- Shebang (`#!/usr/bin/env v`)

---

### ✅ Rainbow Brackets (Optional)

Color-coded nesting levels for `{}`, `[]`, and `()`.

Enable in your Zed `settings.json`:

```json
{
  "languages": {
    "V": {
      "colorize_brackets": true
    }
  }
}
```

Or enable globally:

```json
{
  "colorize_brackets": true
}
```

---

### ✅ Code Snippets

44 built-in snippets for common V patterns. Type the prefix and press Tab.

#### Functions

| Prefix | Description |
|--------|-------------|
| `fn` | Function definition |
| `fnpub` | Public function |
| `fnr` | Function with return type |
| `fnresult` | Function returning `!T` (result type) |
| `fnoption` | Function returning `?T` (option type) |
| `method` | Method with receiver |
| `methodpub` | Public method |
| `methodmut` | Mutable method (`mut` receiver) |

#### Types

| Prefix | Description |
|--------|-------------|
| `struct` | Struct definition |
| `structpub` | Public struct with `pub mut` fields |
| `interface` | Interface definition |
| `enum` | Enum definition |
| `typealias` | Type alias |
| `sumtype` | Sum type (`type A = B \| C`) |

#### Control Flow

| Prefix | Description |
|--------|-------------|
| `if` | If statement |
| `ifelse` | If-else |
| `iferr` | If with option/result unwrap |
| `match` | Match statement |

#### Loops

| Prefix | Description |
|--------|-------------|
| `forrange` | For over `0..n` |
| `forin` | For..in over a collection |
| `forindex` | For with index and value |
| `forc` | Classic C-style for loop |

#### Error Handling

| Prefix | Description |
|--------|-------------|
| `orblock` | `or { }` block |
| `orpanic` | `or { panic(err) }` |
| `orreterr` | `or { return error(...) }` |

#### Concurrency

| Prefix | Description |
|--------|-------------|
| `spawn` | Spawn a goroutine |
| `chan` | Channel declaration |
| `lock` | Lock block for shared variable |

#### Declarations

| Prefix | Description |
|--------|-------------|
| `const` | Const block |
| `module` | Module declaration |
| `import` | Import statement |
| `importas` | Import with alias |

#### Testing

| Prefix | Description |
|--------|-------------|
| `test` | Test function with assert |
| `assert` | Assert statement |
| `assertmsg` | Assert with custom message |

#### I/O & Debugging

| Prefix | Description |
|--------|-------------|
| `println` | println |
| `print` | print (no newline) |
| `dump` | dump() — shows name, type, and value |
| `eprintln` | eprintln to stderr |

#### Other

| Prefix | Description |
|--------|-------------|
| `defer` | Defer block |
| `structlit` | Struct literal |
| `array` | Array literal |
| `map` | Map literal |
| `interp` | String interpolation |
| `unsafe` | Unsafe block |
| `sql` | SQL ORM query |
| `route` | Vweb route handler |
| `header` | Section comment header |

---

### ✅ Smart Auto-Closing

Automatic bracket and quote pairing:

- Blocks: `{` `}`
- Arrays / expressions: `[` `]`
- Calls: `(` `)`
- Strings: `"` `"`, `'` `'`
- Raw strings / char literals: `` ` `` `` ` ``

---

### ✅ Intelligent Word Selection

Double-click selects complete V identifiers. V identifiers use letters, digits, and underscores — all handled correctly.

---

### ✅ Block Comment Toggle

`Ctrl+/` toggles `//` line comments. `/* */` block comments are also supported for the block comment toggle command.

---

### ✅ Feature Toggles

All v-analyzer features can be individually enabled or disabled via your Zed `settings.json`. Changes take effect after a full Zed restart.

```json
"lsp": {
  "v-analyzer": {
    "initialization_options": {
      "inlay_hints": {
        "enable": true,
        "enable_type_hints": true,
        "enable_parameter_name_hints": true,
        "enable_range_hints": true,
        "enable_implicit_err_hints": true,
        "enable_constant_type_hints": true,
        "enable_enum_field_value_hints": true
      },
      "enable_semantic_tokens": "full",
      "code_lens": {
        "enable": true,
        "enable_run_lens": true,
        "enable_run_tests_lens": true,
        "enable_inheritors_lens": true,
        "enable_super_interfaces_lens": true
      }
    }
  }
}
```

**`enable_semantic_tokens` values:**

| Value | Behaviour |
|-------|-----------|
| `"full"` | Two-pass: accurate semantic + syntax highlighting (default) |
| `"syntax"` | Syntax-only pass — faster, recommended for very large files |
| `"none"` | Semantic tokens disabled entirely |

**When to change settings:**

- Disable `enable_type_hints` if inferred type annotations feel cluttered
- Disable `enable_parameter_name_hints` if you find call-site labels distracting
- Set `enable_semantic_tokens` to `"syntax"` if highlighting is slow on large files
- Disable `code_lens` if you don't use the run/test buttons

---

## Requirements

### v-analyzer

This extension requires `v-analyzer` to be installed and available in your `PATH`.

**Install:**

```sh
v download -RD https://raw.githubusercontent.com/vlang/v-analyzer/main/install.vsh
```

**Update:**

```sh
v-analyzer up
```

**Verify:**

```sh
v-analyzer --version
```

### V Compiler

v-analyzer uses the V compiler for diagnostics and formatting. Install V from [vlang.io](https://vlang.io/).

If v-analyzer cannot find your V installation automatically, create a project-local config:

```sh
v-analyzer init
```

Then set `custom_vroot` in the generated `.v-analyzer/config.toml`.

---

## Installation

### From Zed Extensions Marketplace

1. Open Zed
2. Go to Extensions (`Ctrl+Shift+X` / `Cmd+Shift+X`)
3. Search for **V Enhanced**
4. Click Install

### Development Installation

1. Clone this repository
2. Build the extension:

   ```bat
   :: Windows
   build.bat
   ```

   ```sh
   # Linux / macOS
   chmod +x build.sh && ./build.sh
   ```

3. In Zed, open Extensions (`Ctrl+Shift+X`)
4. Click **Install Dev Extension**
5. Select this folder

---

## Configuration

### Per-Project v-analyzer Config

Create a local config at your project root for project-specific settings:

```sh
v-analyzer init
```

This creates `.v-analyzer/config.toml`. Key settings:

```toml
# Path to your V installation (if v-analyzer can't find it automatically)
custom_vroot = "/path/to/v"

# Custom cache directory
custom_cache_dir = ".v-analyzer/cache"

# Semantic tokens mode: "full", "syntax", or "none"
enable_semantic_tokens = "full"

[inlay_hints]
enable = true
enable_type_hints = true
enable_parameter_name_hints = true
enable_range_hints = true
enable_implicit_err_hints = true
enable_constant_type_hints = true
enable_enum_field_value_hints = true

[code_lens]
enable = true
enable_run_lens = true
enable_run_tests_lens = true
enable_inheritors_lens = true
enable_super_interfaces_lens = true
```

A global config also exists at `~/.config/v-analyzer/config.toml` and applies to all projects.

---

## Project Structure

```txt
zed-v-enhanced/
├── extension.toml              # Extension metadata, grammar reference, default settings
├── Cargo.toml                  # Rust extension dependency (zed_extension_api)
├── build.bat                   # Windows build script
├── build.sh                    # Linux / macOS build script
├── src/
│   └── lib.rs                  # Extension entry point — locates and launches v-analyzer
└── languages/
    └── v/
        ├── config.toml         # Language settings (brackets, indent, comments, word chars)
        ├── highlights.scm      # Comprehensive syntax highlighting queries
        ├── brackets.scm        # Rainbow bracket pairs ({ } [ ] ( ))
        ├── folds.scm           # Code folding queries
        ├── outline.scm         # Breadcrumb / outline panel queries
        ├── tags.scm            # Symbol search queries (Ctrl+T)
        └── snippets.json       # 44 code snippets
```

---

### Troubleshooting

#### v-analyzer not found

- Confirm it is in your PATH: `where v-analyzer` (Windows) / `which v-analyzer` (Linux/Mac)
- Install with: `v download -RD https://raw.githubusercontent.com/vlang/v-analyzer/main/install.vsh`
- Restart Zed after installing

#### No diagnostics / formatting not working

- v-analyzer needs the V compiler: confirm `v` is in PATH or set `custom_vroot` in config
- Run `v-analyzer init` in your project root and set `custom_vroot` in the generated config

#### Indexing is slow on first open

- v-analyzer indexes your workspace and the V stdlib on first use — this is normal
- Progress is shown in the Zed status bar
- Subsequent opens use the cached index and are fast

#### Features stopped working after a Zed update

- Rebuild the extension with `build.bat` / `build.sh` and reinstall

#### Settings don't seem to apply

- Settings changes require a **full Zed restart** — not just closing and reopening a file

#### Checking logs

- Zed menu → View → Zed Log
- Look for `v-analyzer` entries to see initialization and request details

---

## Links

- [V Language](https://vlang.io/)
- [v-analyzer](https://github.com/vlang/v-analyzer)
- [Zed Editor](https://zed.dev/)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

---

## License

MIT
