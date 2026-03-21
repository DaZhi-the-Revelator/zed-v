# V Enhanced — Language Support for Zed

A comprehensive V language extension for [Zed](https://zed.dev/), powered by [velvet](https://github.com/DaZhi-the-Revelator/velvet) with bug fixes, enhanced hover documentation, and correct symbol renaming.

**Supports V 0.5.1. Extension version 0.6.9. Requires velvet 0.4.0+.**

---

## Table of Contents

- [Quick Start](#quick-start)
- [⚠️ Important: velvet Required](#%EF%B8%8F-important-velvet-required)
  - [Why velvet?](#why-velvet)
  - [Installing velvet](#installing-velvet)
  - [Staying Up to Date](#staying-up-to-date)
- [Features](#features)
  - [✅ Core Language Intelligence](#-core-language-intelligence)
  - [✅ Advanced Code Editing](#-advanced-code-editing)
  - [✅ v.mod Manifest Support](#-vmod-manifest-support)
  - [✅ Running Programs (Runnables)](#-running-programs-runnables)
  - [✅ Jupyter Kernel & REPL Integration](#-jupyter-kernel--repl-integration)
  - [✅ Rich dump() Output in REPL](#-rich-dump-output-in-repl)
  - [✅ Automatic velvet Update Check](#-automatic-velvet-update-check)
  - [✅ Syntax Highlighting](#-syntax-highlighting)
  - [✅ Rainbow Brackets (Optional)](#-rainbow-brackets-optional)
  - [✅ Code Snippets](#-code-snippets)
  - [✅ Smart Auto-Closing](#-smart-auto-closing)
  - [✅ Intelligent Word Selection](#-intelligent-word-selection)
  - [✅ Block Comment Toggle](#-block-comment-toggle)
  - [✅ Feature Toggles](#-feature-toggles)
- [Requirements](#requirements)
  - [velvet (Required)](#velvet-required)
  - [V Compiler](#v-compiler)
  - [Jupyter Kernel (Optional)](#jupyter-kernel-optional)
- [Migrating from the Standard V Extension](#migrating-from-the-standard-v-extension)
  - [Step 1 — Stop and Remove VLS](#step-1--stop-and-remove-vls)
  - [Step 2 — Uninstall the V Extension](#step-2--uninstall-the-v-extension)
  - [Step 3 — Clean Up Residual Settings](#step-3--clean-up-residual-settings)
  - [Step 4 — Install V Enhanced](#step-4--install-v-enhanced)
- [Installation](#installation)
  - [Development Installation](#development-installation)
- [Configuration](#configuration)
  - [Per-Project velvet Config](#per-project-velvet-config)
- [Troubleshooting](#troubleshooting)
  - [velvet not found](#velvet-not-found)
  - [Running velvet check in CI produces no output / exits 0 on a dirty project](#running-velvet-check-in-ci-produces-no-output--exits-0-on-a-dirty-project)
  - [Server crashes on enum hover](#server-crashes-on-enum-hover)
  - [Rename only updates one occurrence](#rename-only-updates-one-occurrence)
  - [No diagnostics / formatting not working](#no-diagnostics--formatting-not-working)
  - [Diagnostics appear delayed or show a timeout warning in logs](#diagnostics-appear-delayed-or-show-a-timeout-warning-in-logs)
  - [Indexing is slow on first open](#indexing-is-slow-on-first-open)
  - [Jupyter kernel not appearing in Zed](#jupyter-kernel-not-appearing-in-zed)
  - [Build script says "Cargo.toml or src\lib.rs has error"](#build-script-says-cargotoml-or-srclibrs-has-error--wasm-file-not-produced)
  - [Features stopped working after a Zed update](#features-stopped-working-after-a-zed-update)
  - [velvet update notification keeps appearing](#velvet-update-notification-keeps-appearing)
  - [Settings don't seem to apply](#settings-dont-seem-to-apply)
  - [Checking logs](#checking-logs)
- [Links](#links)
- [License](#license)

---

## Repository Structure

```txt
v-enhanced/
  extension/     ← Zed extension source (Rust/WASM) — point Zed here
    src/
    languages/
    grammars/
    extension.toml
    Cargo.toml
    build.bat / build.sh
  kernel/        ← Jupyter kernel for Zed REPL integration (separate Rust project)
  README.md
  LICENSE
```

The `extension/` directory is the Zed extension itself. When installing as a dev extension, select the `extension/` folder, **not** the repo root. The `kernel/` directory is a separate Rust project — see [kernel/README.md](kernel/README.md) for its own build and install instructions.

---

## Quick Start

This section shows what working in V Enhanced actually looks like. Paste the program below into a new `shapes.v` file, then read the annotations to see which extension feature each line exercises.

```v
module main

import math

// Hover over 'Shape' — velvet renders the full interface body inline.
interface Shape {
    area() f64
    name() string
}

// @[required] is a compile-time guard: forgetting 'radius' in a Circle{}
// literal is a compile error, not a runtime panic.
struct Circle {
    @[required]
    radius f64
}

struct Rect {
    @[required]
    width f64
    @[required]
    height f64
}

// Hover over 'c' — velvet shows '(c Circle) area() f64' with the doc comment.
// The outline panel lists this as 'Circle.area', not just 'area'.
fn (c Circle) area() f64  { return math.pi * c.radius * c.radius }
fn (c Circle) name() string { return 'circle' }
fn (r Rect)   area() f64  { return r.width * r.height }
fn (r Rect)   name() string { return 'rect' }

// Type a new variant here and the interface-compliance warning fires immediately
// if you forget to add area() or name() — before you even save.

// Generic function — hover over 'T' to see the type parameter.
// velvet resolves generics through instantiation: T becomes f64 at the call site.
fn largest[T](a T, b T) T {
    return if a > b { a } else { b }
}

// Function returning a result type. Callers must handle the error.
fn parse_radius(s string) !f64 {
    r := s.f64()
    if r <= 0 {
        return error('radius must be positive, got: ${s}')
    }
    return r
}

fn print_shape(s Shape) {
    // Inlay hint: velvet shows the inferred return type of the anonymous fn.
    describe := fn(label string, val f64) string {
        return '${label}: ${val:.2f}'
    }
    println('${s.name()} — ${describe('area', s.area())}')
}

fn main() {
    shapes := [Circle{ radius: 3.0 }, Rect{ width: 4.0, height: 5.0 }] as []Shape

    for s in shapes {
        print_shape(s)
    }

    // Error propagation with !: if parse_radius returns an error, main exits.
    // Remove the or{} block and change the return type of main to ! to use ! instead.
    r := parse_radius('7.5') or {
        eprintln('bad input: ${err}')
        return
    }
    println('parsed radius: ${r}')
    println('largest of 3.0 and ${r}: ${largest(3.0, r)}')

    // $if is erased at compile time — only one branch is compiled into the binary.
    \$if windows {
        println('running on Windows')
    } \$else {
        println('running on Linux / macOS')
    }
}
```

**What lights up as you work:**

| Action | Feature |
|--------|--------|
| Hover `Shape` / `Circle` / `Rect` | Full interface or struct body rendered in the popup |
| Hover a method name at its call site | Signature + doc comment |
| Place cursor after `Circle{` | Signature help lists remaining required fields |
| Add a new struct that partially implements `Shape` | Interface compliance warning appears instantly, listing the missing method |
| `F2` on `radius` | Safe rename updates every reference across the file |
| `Alt+Shift+→` inside `for s in shapes` | Selection expands: identifier → expression → statement → block → function |
| Select the body of `print_shape`, invoke code-action light-bulb | **Extract Function** offered |
| Cursor on `parse_radius`, light-bulb | **Generate Constructor** is not offered (no struct), but on `Circle` it is |
| `F4` | Task picker: `v run`, `v build`, `v vet`, `v doc`, `v watch run` |

**To run it:**

```sh
# From Zed: press F4 and choose 'v run <file>'
# Or from the terminal:
v run shapes.v
```

Expected output:

```
circle — area: 28.27
rect — area: 20.00
parsed radius: 7.5
largest of 3.0 and 7.5: 7.5
running on Linux / macOS
```

---

## ⚠️ Important: velvet Required

This extension requires **velvet** as its language server. velvet contains critical fixes and feature additions that are not in the upstream v-analyzer binary.

### Why velvet?

velvet is a purpose-built language server for V. It is a maintained fork of the upstream v-analyzer, which is missing features and has bugs that prevent this extension from working correctly.

### Installing velvet

Clone and build from source:

```sh
git clone --recurse-submodules https://github.com/DaZhi-the-Revelator/velvet
cd velvet
v run build.vsh release
```

The build script places the binary at `./bin/velvet` (or `./bin/velvet.exe` on Windows). Copy it to a location on your `PATH`, for example:

```sh
# Linux / macOS
cp bin/velvet ~/.local/bin/velvet

# Windows (PowerShell — run from the velvet directory)
Copy-Item .\bin\velvet.exe "$env:USERPROFILE\.config\velvet\bin\velvet.exe"
# Ensure that directory is on your PATH
```

**Verify:**

```sh
velvet --version
# Should print: velvet version 0.4.0
```

### Staying Up to Date

Pull the latest fixes and rebuild:

```sh
cd velvet
git pull
v run build.vsh release
# then copy the binary to PATH as above
```

To check whether a newer release is available without rebuilding:

```sh
velvet check-updates
```

To run `v vet` across your whole V project from the command line (useful in CI or pre-commit hooks):

```sh
velvet check             # human-readable output, exits 0/1/2
velvet check --json      # JSON array — pipe into jq, reviewdog, etc.
velvet check --no-color  # plain text for log files
```

---

> **Migrating from V < 0.5.1?** `vlib/x/ttf/render_sokol_cpu.v` was extracted into a separate module in 0.5.1. Change `import x.ttf` to `import x.ttf.render_sokol` for rendering functions, then re-run `velvet` to refresh the stub index.

## Features

All LSP intelligence is provided by velvet. This extension wires every capability into Zed natively and adds the full Zed-specific layer on top.

---

### ✅ Core Language Intelligence

- **Diagnostics** — Real V compiler errors, warnings, and notices with:
  - `unused` variables and imports tagged with strikethrough (`DiagnosticTag.unnecessary`)
  - `deprecated` symbols tagged with strikethrough (`DiagnosticTag.deprecated`)
  - All errors, warnings, and notices from the actual V compiler — not heuristics
  - **Unused parameter warning** (velvet-native) — flags parameters never referenced in the function body; parameters prefixed with `_` and `test_*` functions are excluded; **enabled by default**, toggleable via `enable_unused_parameter_warning` (see [Feature Toggles](#-feature-toggles))
  - **Unused variable warning** (velvet-native) — real-time PSI-based warning when a local `:=` variable is declared but never referenced; `_`-prefixed names and `test_*` functions excluded; **enabled by default**, disable via `enable_unused_variable_warning: false`
  - **Unused import warning** (velvet-native) — real-time PSI-based warning when a module is imported but never referenced as `module.symbol`; selective imports (`import os { getenv }`) are excluded; **enabled by default**, disable via `enable_unused_import_warning: false`
  - **Dead / unreachable code** (velvet-native) — flags any statement following an unconditional `return`, `break`, `continue`, `goto`, or `panic()`/`exit()` call in the same block; rendered greyed-out via `DiagnosticTag.unnecessary`; **always enabled**.
  - **Interface compliance check** (velvet-native) — warns at edit time when a struct has started implementing an interface (already provides at least one required method) but is still missing others; the warning appears on the struct name and lists every missing method, e.g. `struct 'Dog' partially implements 'Animal' but is missing: move`; **always enabled**; structs with no methods are never flagged, preventing false positives from coincidental name matches
  - **Incremental text sync** — velvet uses `TextDocumentSyncKind.incremental`, sending only per-keystroke diffs instead of the full file on every change; reduces memory and CPU on large files
  - **Crash protection** — velvet wraps every `v -check`, `v vet`, and `v fmt` invocation in a hard timeout (30 s for compiler passes, 15 s for formatting); if V hangs or crashes and leaves an orphaned `v.exe` process behind, velvet kills it, discards the result, and continues serving requests without freezing Zed; the background diagnostic thread is also monitored by a watchdog that automatically restarts it if a task exceeds 60 seconds

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
  - **Structs** — renders the full struct body with fields grouped by access modifier (`pub mut`, `pub`, `mut`, private)
  - **Interfaces** — renders the full interface body with all methods, fields, and embedded interface names
  - **Enums** — renders all fields with their computed values (implicit auto-increment, explicit values, and `[flag]` bitfield binary representations)
  - Type aliases and sum types (full `type A = B | C` signature)
  - Constants (with value and type)
  - Variables (with inferred type)
  - Parameters and receivers
  - Struct fields (with type and mutability)
  - Enum fields (with computed numeric value)
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
- **Signature Help** — Real-time parameter hints as you type:
  - Active parameter highlighted as you move through arguments
  - Retrigger support (`,` and ` ` re-trigger the hint)
  - Resolves the actual function declaration via PSI — always accurate
  - Works for all functions including stdlib and user-defined
  - **Struct literal field hints** — typing `StructName{` triggers a persistent popup listing all remaining (unfilled) fields with their types; as you fill each field the hint shrinks; the field under the cursor is highlighted as the active parameter; triggered by `{` in addition to `(` and `,`
- **Find All References** — PSI-based cross-file reference search:
  - Accurate — not text search, uses the program structure index
  - Works across modules and into the stdlib
  - Skips false positives in comments and strings automatically
- **Rename Symbol** (`F2`) — Safe, complete symbol renaming:
  - `prepareRename` validation before any changes are made
  - Finds every occurrence in the live open file using its in-memory parse tree — no stale positions
  - Correctly handles all reference types (declaration, usage, field access, method calls)
  - Works for: local variables, parameters, functions, methods, structs, enums, constants, type aliases
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
- **Selection Range** — Structural selection expansion via **Alt+Shift+→** (Expand Selection):
  - Each press grows the selection one syntactic level outward
  - Follows the actual V parse tree: identifier → expression → argument list → call → statement → block → function body → file
  - Implemented in velvet via `textDocument/selectionRange`
- **Document Symbols** — Full nested symbol tree in the outline panel:
  - Functions and methods (with signature as detail)
  - Methods show receiver type as context prefix — e.g. `Rect.area` instead of `area` — in both the outline panel and the breadcrumb bar
  - Structs (with their fields nested inside)
  - Interfaces (with their methods and fields nested inside)
  - Enums (with their values nested inside, showing implicit values)
  - Constants (with type and value)
  - Type aliases
- **Inlay Hints** — 9 types of inline annotations:
  - **Type hints** — inferred type shown after `:=` assignments: `x: int := 10`
  - **Parameter name hints** — parameter names shown before arguments in function calls
  - **Range operator hints** — `≤` and `<` shown on `..` range operators to clarify inclusivity
  - **Implicit `err →` hints** — shown inside `or { }` blocks and `else` branches of result unwrapping
  - **Enum field value hints** — implicit enum field values shown inline next to each field
  - **Constant type hints** — type shown after constant declarations
  - **Anonymous function return type hints** — inferred return type shown on the closing `}` of anonymous functions with no explicit return type
  - **Struct field name hints** — when a struct is initialised using positional syntax (no field names), each field name is shown as an inlay hint before its value: `Config{5000, 3}` renders as `Config{ timeout: 5000, retries: 3 }`. Mirrors Rust-analyzer's tuple-struct hints and catches field-order mistakes before the compiler does. Configurable via `enable_struct_field_name_hints`.
  - **Struct field order hints** — when a fully-keyed struct literal writes fields in a different order from their declaration order, a `⚠` hint appears before the out-of-order field. V's compiler accepts any field order in keyed literals, so this class of mistake is otherwise invisible. Configurable via `enable_struct_field_order_hints`.
- **Semantic Tokens** — Enhanced syntax highlighting from the LSP layer:
  - Two-pass system for accuracy and performance:
    - Fast syntax-based pass for files over 1000 lines
    - Accurate resolve-based pass for full semantic colouring on smaller files
  - Distinguishes user-defined functions from built-in functions
  - Correctly colours struct names, interface names, enum names vs. primitive types
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
  - **Import Module** — detects an `undefined ident` compiler error and automatically inserts the correct `import` statement
  - **Remove Unused Import** — automatically removes import statements that the V compiler reports as unused
  - **Extract Variable** — replaces a compound expression with a fresh `name := expr` declaration inserted on the line above; the variable name is inferred from the expression where possible; if the suggested name already exists in the file, velvet appends an incrementing number automatically (`extracted`, `extracted2`, `extracted3`, …)
  - **Inline Variable** — the inverse of Extract Variable; cursor on `x := <expr>` → replaces every reference to `x` in the enclosing block with `expr` and removes the declaration; only offered when `x` is referenced at least once.
  - **Convert `if`/`else` to `match`** — converts an `if` / `else if` chain whose every branch compares the same subject with `==` into an idiomatic V `match` block; a trailing `else` is preserved as the `match else` arm
  - **Sort imports** — sorts the import block at the top of the file alphabetically without re-formatting anything else; useful as an "organize imports on save" action distinct from running `v fmt`; offered only when two or more imports are present and not already sorted
  - **Extract Function** — wraps the selected statement(s) into a new `fn` inserted immediately after the enclosing function. velvet infers parameters (outer-scope variables read by the selection) and return values (variables defined inside the selection and used after it). A single return value is returned directly; multiple values are returned as a tuple and unpacked at the call site. Types are resolved from PSI type inference; when a type cannot be determined a `/* T */` placeholder is emitted so the code still compiles after manual fixup. Trigger: select one or more statements and invoke the code-action light-bulb.
  - **Generate Constructor** — when the cursor is on a struct declaration, generates a `new_<struct_name>(field1 Type1, ...) StructName` factory function inserted directly after the struct's closing brace. Fields with declared default values are omitted from the parameter list. The constructor visibility matches the struct (`pub` struct → `pub fn`). PascalCase struct names are converted to snake_case (e.g. `MyHttpServer` → `new_my_http_server`). Suppressed if a constructor with that name already exists. Trigger: cursor on the struct name, invoke the light-bulb.
  - **Implement Interface** — when the cursor is on a struct declaration, generates stub method bodies for every method of every interface in the workspace that the struct does not yet implement. Methods the struct already satisfies are skipped. Each stub contains `// TODO: implement`. Trigger: cursor on the struct name, invoke the light-bulb. (Disabled by default in CLion to avoid duplication with the intellij-v plugin — see `enable_implement_interface` under [Feature Toggles](#-feature-toggles).)
  - **Add Missing Match Arms** — when the cursor is inside a `match` expression whose subject is an enum type, detects which variants are not yet covered and inserts stub arms with `// TODO: implement` bodies for each missing one; suppressed when an `else` arm is already present

---

### ✅ Running Programs (Runnables)

V Enhanced wires Zed's Runnables system to the V compiler so you can run, build, and test without leaving the editor.

#### Gutter arrow (fn main)

When your cursor is inside or near `fn main()`, a ▶ arrow appears in the gutter. Clicking it (or using the Runnables panel) runs the file with `v run`. The dropdown also exposes build and test tasks.

#### `F4` — task picker

Press `F4` to open the full task list for the current file. Available tasks:

| Task | Command | When shown |
|------|---------|------------|
| `v run <file>` | `v run $ZED_FILE` | Any `.v` file |
| `v build <file>` | `v build $ZED_FILE` | Any `.v` file |
| `v test <file>` | `v test $ZED_FILE` | Any `.v` file |
| `v run <project>` | `v run $ZED_WORKTREE_ROOT` | When a worktree is open |
| `v build <project>` | `v build $ZED_WORKTREE_ROOT` | When a worktree is open |
| `v missdoc <project>` | `v missdoc $ZED_WORKTREE_ROOT` | When a worktree is open |
| `v vet <file>` | `v vet $ZED_FILE` | Any `.v` file |
| `v vet <project>` | `v vet $ZED_WORKTREE_ROOT` | When a worktree is open |
| `v doc <project>` | `v doc $ZED_WORKTREE_ROOT` | When a worktree is open |
| `v watch run <file>` | `v watch run $ZED_FILE` | Any `.v` file |

All tasks run from the directory containing the source file (`$ZED_DIRNAME`) or the project root, as appropriate.

#### Test functions

Functions named `test_*` get their own gutter arrow and appear individually in the Runnables panel. When you click the gutter arrow next to a specific test function, V Enhanced runs only that test using `v test -run test_name $ZED_FILE`. This enables true TDD workflows where you can iterate on a single test without running the entire file's test suite.

---

### ✅ Jupyter Kernel & REPL Integration

V Enhanced ships a complete Jupyter kernel (`v-kernel`) that integrates with Zed's built-in REPL. The kernel is a separate Rust project in the `kernel/` subdirectory with its own full documentation.

**[→ Full kernel documentation and installation instructions](kernel/README.md)**

Quick summary:

- Cells are delimited with `// %%` comment separators
- Top-level declarations (`fn`, `struct`, `enum`, `const`, `import`, etc.) accumulate across cells; bare statements run in an isolated `fn main()` per cell
- `dump()` output is rendered as a styled HTML table (columns: location · name · type · value)
- Press `Ctrl+Shift+Enter` (Windows/Linux) or `Cmd+Shift+Enter` (macOS) to execute the current cell
- If the kernel doesn't appear in Zed's picker, run **"REPL: Refresh Kernelspecs"** from the command palette (`Ctrl+Shift+P`)

### Example REPL session

Create a new `.v` file, add `// %%` separators between cells, and press `Ctrl+Shift+Enter` (or `Cmd+Shift+Enter`) to execute each one:

```v
import math

// %%

// Cell 1 — top-level declarations accumulate for the rest of the session.
// Re-running this cell is safe: velvet wraps it in a guard so you never
// get 'struct already declared' errors.
struct Vec2 {
    x f64
    y f64
}

fn (v Vec2) length() f64 {
    return math.sqrt(v.x * v.x + v.y * v.y)
}

fn (v Vec2) add(other Vec2) Vec2 {
    return Vec2{ x: v.x + other.x, y: v.y + other.y }
}

// %%

// Cell 2 — statements run in an isolated fn main().
// dump() renders as a styled table: location · name · type · value
a := Vec2{ x: 3.0, y: 4.0 }
b := Vec2{ x: 1.0, y: 2.0 }
dump(a)           // → table row: a  Vec2  Vec2{x: 3.0, y: 4.0}
dump(a.length())  // → table row: a.length()  f64  5.0

// %%

// Cell 3 — builds on declarations from Cell 1; 'a' from Cell 2 is gone
// (bare statements don’t leak across cells).
result := a.add(b)   // ← compile error: 'a' is not defined here
// Fix: re-declare a in this cell, or promote it to a const in Cell 1.
c := Vec2{ x: 3.0, y: 4.0 }.add(Vec2{ x: 1.0, y: 2.0 })
dump(c)           // → table row: c  Vec2  Vec2{x: 4.0, y: 6.0}
println('length: ${c.length():.4f}')
```

Key things to notice:
- **Declarations in Cell 1** (`struct Vec2`, `fn length`, `fn add`) are available in every later cell without re-running them.
- **`dump()` rows** appear as a formatted HTML table instead of raw `[main.v:N] x = Type(value)` text.
- **Bare variables** (`a`, `b` in Cell 2) are local to that cell's `fn main()` and do not carry over — Cell 3 demonstrates this with an intentional error and the fix.

See [kernel/README.md](kernel/README.md) for full details on how it works, architecture, and limitations.

---

### ✅ Rich dump() Output in REPL

`dump()` calls are automatically intercepted and rendered as a styled HTML table in the Zed REPL output panel — columns: **location · name · type · value**, styled to match Catppuccin Mocha. All other output (`println`, `print`, `eprintln`) appears as plain stream text.

No changes to your V code are needed. See [kernel/README.md](kernel/README.md) for full details.

---

### ✅ Automatic velvet Update Check

Every time the extension activates (i.e. when you open a `.v` file and the language server starts), the extension silently:

1. Runs `velvet --version` to read the local binary's version string.
2. Fetches the latest release tag from the velvet repo via the GitHub API.
3. Compares the two version strings.

If they differ, a notice appears in the Zed language-server status bar:

> velvet is out of date (local: `0.3.9`, latest release: `0.4.0`). Run: `cd velvet && git pull && v run build.vsh release`, then copy `bin/velvet` to your PATH and restart Zed.

If the versions already match, or if the check fails for any reason (no network, API rate limit, etc.), nothing is shown. The check runs at most once per session and never blocks the language server from starting.

This addresses the silent breakage that can occur when Zed updates and the locally installed velvet binary lags behind.

---

### ✅ Syntax Highlighting

Powered by `tree_sitter_v` — velvet's battle-tested grammar — with comprehensive Zed-specific highlight queries covering:

- Functions, methods (including static methods declared as `fn Foo.bar()`), and function calls (distinguished)
- Struct, interface, enum, and type declarations
- All keyword categories: storage modifiers (`mut`, `pub`, `const`, `static`), control flow (`if`, `for`, `match`, `return`), type definitions (`struct`, `interface`, `enum`, `type`)
- String literals: interpreted, raw, C strings, string interpolation
- Escape sequences within strings
- Rune literals
- Integer and float literals
- Boolean literals (`true`, `false`)
- Built-in constants (`nil`, `none`)
- Attributes (`@[...]`)
- Modules and import paths
- Struct fields and selector expressions
- Enum fetch expressions (`Direction.north`)
- Global variable definitions
- Compile-time constructs (`$if`, `$for`, `$else`)
- Labels
- All operators (arithmetic, bitwise, comparison, assignment, channel, range)
- Comments: line (`//`) and block (`/* */`)
- Shebang (`#!/usr/bin/env v`)

**Variable scoping** via `locals.scm` prevents local variable names from bleeding across function boundaries in syntax-only highlighting mode (files over 1000 lines). Scopes are defined for function bodies, blocks, `if`/`else`, `for`, `match`, `lock`, `unsafe`, and `defer`. Parameter declarations, receiver names, loop variables (including both the index and value in `for i, v in` style loops), and short variable declarations are all tracked as definitions.

**Embedded language injection** via `injections.scm` gives sub-languages their own highlighting inside V source:

| Context | Injected language |
|---------|------------------|
| `${ ... }` inside string interpolation | V |
| `sql db { ... }` ORM blocks | SQL |
| `asm x64 { ... }` inline assembly | ASM |

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

### ✅ v.mod Manifest Support

Files named `v.mod` are recognised as a distinct language (**VModManifest**) and get:

- Syntax highlighting: manifest type name, field keys, string values, brackets, and comments
- Correct bracket auto-closing (`{`, `[`, `'`)
- Comment toggling (`//` and `/* */`)
- Proper indentation (4-space tabs, matching V style)

No language server is attached — v.mod files are static manifests and do not need LSP features.

---

### ✅ Code Snippets

63 built-in snippets for common V patterns. Type the prefix and press Tab.

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
| `select` | Select with send arm |
| `selectrecv` | Select with receive arm |
| `selectelse` | Select with non-blocking `else` branch |
| `selecttimeout` | Select with timeout branch |

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
| `sqljoin` | SQL ORM query with explicit JOIN (V 0.5.1+) |
| `sqltx` | SQL ORM transaction block (V 0.5.1+) |
| `ormselect` | Low-level `orm.select()` call with `orm.SelectConfig` (V 0.5+) |
| `dbexec` | Raw SQL via `db.exec()` (V 0.5+) |
| `dbexecparams` | Parameterized SQL via `db.exec_param()` (V 0.5+) |
| `route` | Vweb route handler |
| `header` | Section comment header |

#### Generics

| Prefix | Description |
|--------|-------------|
| `fng` | Generic function with a type parameter (`fn name[T](val T) T`) |
| `structg` | Generic struct with a type parameter (`struct Name[T]`) |

#### Concurrency — shared state

| Prefix | Description |
|--------|-------------|
| `shared` | `shared` variable declaration — must be accessed inside `lock`/`rlock` blocks |
| `rlock` | `rlock` block — read-only lock (multiple readers allowed simultaneously) |

#### Error / option propagation

| Prefix | Description |
|--------|-------------|
| `prop` | `result := fn_call()!` — propagate a result error; equivalent to `or { return err }` |
| `propopt` | `result := fn_call()?` — propagate an option; equivalent to `or { return none }` |

#### Attributes

| Prefix | Description |
|--------|-------------|
| `required` | `@[required]` on a struct field — omitting it in a literal is a compile error |

#### Compile-time

| Prefix | Description |
|--------|-------------|
| `compfor` | `$for field in Type.fields { }` — iterate struct fields at compile time |
| `compformethod` | `$for method in Type.methods { }` — iterate struct methods at compile time |
| `compileerr` | `$compile_error('…')` — emit a compile-time error |
| `compilewarn` | `$compile_warn('…')` — emit a compile-time warning |
| `tmpl` | `$tmpl('path.html')` — embed and compile an HTML template at compile time |
| `compifos` | `$if windows/linux/macos … $else` — OS-specific compile-time branch (erased for other platforms) |
| `compif` | `$if <condition>` — generic compile-time conditional (os, arch, compiler flags) |

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

Double-click selects complete V identifiers including underscores — `snake_case_identifier` is selected in full, not word-by-word.

---

### ✅ Block Comment Toggle

`Ctrl+/` toggles `//` line comments. `/* */` block comments are also supported for the block comment toggle command.

---

### ✅ Feature Toggles

All velvet features can be individually enabled or disabled via your Zed `settings.json`. Changes take effect after a full Zed restart. The settings below show the **defaults** — you only need to include a key if you want to change it.

```json
"lsp": {
  "velvet": {
    "initialization_options": {
      "inlay_hints": {
        "enable": true,
        "enable_type_hints": true,
        "enable_parameter_name_hints": true,
        "enable_range_hints": true,
        "enable_implicit_err_hints": true,
        "enable_constant_type_hints": true,
        "enable_enum_field_value_hints": true,
        "enable_anon_fn_return_type_hints": true,
        "enable_struct_field_name_hints": true,
        "enable_struct_field_order_hints": true
      },
      "enable_semantic_tokens": "full",
      "code_lens": {
        "enable": true,
        "enable_run_lens": true,
        "enable_inheritors_lens": true,
        "enable_super_interfaces_lens": true,
        "enable_run_tests_lens": true
      },
      "inspections": {
        "enable_unused_parameter_warning": true,
        "enable_unused_variable_warning": true,
        "enable_unused_import_warning": true
      },
      "code_actions": {
        "enable_make_public": true,
        "enable_implement_interface": true
      }
    }
  }
}
```

> **Note:** You only need to include the keys you want to change. User-supplied values are deep-merged on top of the extension defaults, so setting a single nested key (e.g. `inspections.enable_unused_parameter_warning`) does not affect any other settings.

**`enable_semantic_tokens` values:**

| Value | Behavior |
|-------|-----------|
| `"full"` | Two-pass: accurate semantic + syntax highlighting (default) |
| `"syntax"` | Syntax-only pass — faster, recommended for very large files |
| `"none"` | Semantic tokens disabled entirely |

**`inlay_hints` keys:**

| Key | Default | Description |
|-----|---------|-------------|
| `enable` | `true` | Master switch — disabling this turns off all inlay hints |
| `enable_type_hints` | `true` | Inferred type after `:=` assignments |
| `enable_parameter_name_hints` | `true` | Parameter names before arguments in function calls |
| `enable_range_hints` | `true` | `≤` / `<` on `..` range operators |
| `enable_implicit_err_hints` | `true` | `err →` inside `or { }` and result `else` branches |
| `enable_constant_type_hints` | `true` | Type shown after constant declarations |
| `enable_enum_field_value_hints` | `true` | Implicit numeric values shown next to enum fields |
| `enable_anon_fn_return_type_hints` | `true` | Inferred return type on closing `}` of anonymous functions |
| `enable_struct_field_name_hints` | `true` | Field names shown on positional (un-keyed) struct literals |
| `enable_struct_field_order_hints` | `true` | `⚠` before fields written out of declaration order in keyed literals |

**`inspections` keys:**

| Key | Default | Description |
|-----|---------|-------------|
| `enable_unused_parameter_warning` | `true` | Warn when a parameter is declared but never used in the function body. Parameters prefixed with `_` and all parameters in `test_*` functions are excluded. |
| `enable_unused_variable_warning` | `true` | Real-time PSI warning when a local `:=` variable is never referenced. `_`-prefixed names and `test_*` functions excluded. |
| `enable_unused_import_warning` | `true` | Real-time PSI warning when an import is never used as `module.symbol`. Selective imports excluded. |

**`code_actions` keys:**

| Key | Default | Description |
|-----|---------|-------------|
| `enable_make_public` | `true` | Offer the **Make public** refactoring in the code-action light-bulb. Disable in CLion if the intellij-v plugin already provides this natively to avoid a duplicate entry in the menu. |
| `enable_implement_interface` | `true` | Offer the **Implement interface** code action. Disable in CLion for the same reason as `enable_make_public`. |

Also configurable in `config.toml` under `[inspections]` and `[code_actions]` — see the [velvet configuration docs](https://github.com/DaZhi-the-Revelator/velvet#configuration). Settings supplied via `initialization_options` take precedence over the TOML file.

---

## Requirements

### velvet (Required)

See the [Installing velvet](#installing-velvet) section above.

> **Do not use the upstream v-analyzer binary.** It will crash on enum hover and produce incorrect rename results.

### V Compiler

velvet uses the V compiler for diagnostics and formatting. This extension targets **V 0.5.1**. Install V from [vlang.io](https://vlang.io/).

If velvet cannot find your V installation automatically, create a project-local config:

```sh
velvet init
```

This creates `.velvet/config.toml`. If the file already exists, `velvet init` **refuses to overwrite it** and prints its current contents instead. Delete the file first if you want a fresh default. Then set `custom_vroot` in the config.

### Jupyter Kernel (Optional)

Required only if you want REPL/notebook support. See [kernel/README.md](kernel/README.md) for full build and install instructions.

---

## Migrating from the Standard V Extension

If you previously used the **V** extension (the one backed by [VLS — the official V Language Server](https://github.com/vlang/vls)), follow these steps to switch cleanly to V Enhanced. Running both extensions or both language servers at the same time will cause conflicts.

### Step 1 — Stop and Remove VLS

VLS is a separate binary that the standard V extension downloads and manages. Remove it before installing V Enhanced so it cannot start and interfere with velvet.

**Locate the VLS binary:**

| Platform | Default VLS location |
|----------|----------------------|
| Windows | `%USERPROFILE%\.vls\bin\vls.exe` |
| Linux / macOS | `~/.vls/bin/vls` |

Delete the binary (and the entire `~/.vls` directory if you no longer need it):

```powershell
# Windows (PowerShell)
Remove-Item -Recurse -Force "$env:USERPROFILE\.vls"
```

```sh
# Linux / macOS
rm -rf ~/.vls
```

If you installed VLS manually to a custom location or via `v install vls`, also remove that copy:

```sh
# Installed via `v install vls`
rm -rf "$(v doctor 2>/dev/null | grep 'vmodules' | awk '{print $2}')/vls"
# Or simply locate and delete the `vls` binary from your PATH
```

### Step 2 — Uninstall the V Extension

1. Open Zed
2. Open Extensions (`Ctrl+Shift+X` on Windows/Linux, `Cmd+Shift+X` on macOS)
3. Find **V** in the Installed extensions list
4. Click **Uninstall**
5. Restart Zed fully (quit and reopen — not just a window reload)

### Step 3 — Clean Up Residual Settings

The standard V extension may have left `lsp` configuration blocks in your Zed `settings.json` that reference `vls`. These will cause Zed to attempt to launch VLS even after the extension is removed.

Open your Zed `settings.json` (`Ctrl+,` / `Cmd+,`, then click **Open Settings JSON**) and remove any block that looks like:

```json
"lsp": {
  "vls": {
    ...
  }
}
```

Also remove any `"V"` language overrides that point to VLS, for example:

```json
"languages": {
  "V": {
    "language_servers": ["vls"],
    ...
  }
}
```

Leave any other unrelated settings intact.

### Step 4 — Install V Enhanced

With the old extension and server fully removed, follow the [Installation](#installation) instructions below. Install velvet as described in [Installing velvet](#installing-velvet) — **do not reuse any VLS binary or configuration**.

After restarting Zed with V Enhanced active, open a `.v` file and confirm in **View → Zed Log** that `velvet` (not `vls`) is the language server that started.

---

## Installation

### ~~From Zed Extensions Marketplace~~

1. ~~Open Zed~~
2. ~~Go to Extensions (`Ctrl+Shift+X` / `Cmd+Shift+X`)~~
3. ~~Search for **V Enhanced**~~
4. ~~Click Install~~
5. ~~Install velvet (see above)~~

### Development Installation

1. Clone this repository
2. Build the extension (run from the `extension/` subdirectory):

   ```bat
   :: Windows
   cd extension
   build.bat
   ```

   ```sh
   # Linux / macOS
   cd extension
   chmod +x build.sh && ./build.sh
   ```

3. In Zed, open Extensions (`Ctrl+Shift+X`)
4. Click **Install Dev Extension**
5. Select the **`extension/`** folder (not the repo root)
6. Install velvet (see above)

---

## Configuration

### Per-Project velvet Config

Create a local config at your project root for project-specific settings:

```sh
velvet init
```

This creates `.velvet/config.toml`. Key settings:

```toml
# Path to your V installation (if velvet can't find it automatically)
custom_vroot = "/path/to/v"

# Custom cache directory
custom_cache_dir = ".velvet/cache"

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
enable_anon_fn_return_type_hints = true
# Field names shown before each value in positional (un-keyed) struct literals.
enable_struct_field_name_hints = true
# ⚠ hint before fields written out of declaration order in keyed struct literals.
enable_struct_field_order_hints = true

[inspections]
# Warn on parameters never referenced in the function body.
# Parameters prefixed with _ and test_* functions are excluded.
enable_unused_parameter_warning = true
# Real-time PSI-based unused variable warning. Enabled by default.
enable_unused_variable_warning = true
# Real-time PSI-based unused import warning. Enabled by default.
enable_unused_import_warning = true

[code_actions]
# Offer the "Make public" refactoring as a code action.
# Disable in CLion when the intellij-v plugin already provides this natively.
enable_make_public = true
# Offer the "Implement interface" code action.
# Disable in CLion for the same reason as enable_make_public.
enable_implement_interface = true
```

A global config also exists at `~/.config/velvet/config.toml` and applies to all projects.

---

## Troubleshooting

### velvet not found

- Confirm it is in your PATH: `where velvet` (Windows) / `which velvet` (Linux/Mac)
- Build and install from: `https://github.com/DaZhi-the-Revelator/velvet`
- Restart Zed after installing

### Running velvet check in CI produces no output / exits 0 on a dirty project

`velvet check` runs `v vet` on the path you give it. A few things to verify:

- **Working directory** — with no argument, velvet checks the current working directory. Make sure your CI job `cd`s into the project root before running it, or pass the path explicitly: `velvet check /path/to/project`.
- **v not on PATH** — velvet shells out to `v vet`. If `v` is not on the CI agent's PATH, velvet exits with `[ERROR] Cannot find the V executable`. Install V before calling velvet check, or set `VROOT` in the environment.
- **Exit code** — `velvet check` exits `0` when there are no issues, `1` when there are warnings only, and `2` when there is at least one error. Configure your CI step to fail on exit code ≥ 1 if you want warnings to block the build, or ≥ 2 if you only care about errors.
- **JSON mode** — for richer CI integration, run `velvet check --json` and parse the output with `jq` or a tool like [reviewdog](https://github.com/reviewdog/reviewdog).

### Server crashes on enum hover

- You are using the upstream v-analyzer binary instead of velvet — install velvet instead (see above)

### Rename only updates one occurrence

- You are using the upstream v-analyzer binary instead of velvet — install velvet instead (see above)

### No diagnostics / formatting not working

- velvet needs the V compiler: confirm `v` is in PATH or set `custom_vroot` in config
- Run `velvet init` in your project root and set `custom_vroot` in the generated config

### Diagnostics appear delayed or show a timeout warning in logs

If the Zed log shows a line like `velvet: v -check timed out; orphaned v.exe processes were killed`, it means the V compiler hung on that particular save cycle. velvet automatically killed the orphan and will retry on your next save — no action required. This typically happens on large projects where V's type-checker takes longer than 30 seconds, or after a V compiler crash leaves an orphaned process that blocks the next run. If timeouts happen consistently, try setting a per-project `custom_vroot` pointing to a fast local V build, and check that no stale `v.exe` processes are lingering in Task Manager.

### Indexing is slow on first open

- velvet indexes your workspace and the V stdlib on first use — this is normal
- Progress is shown in the Zed status bar
- Subsequent opens use the cached index and are fast
- The cache key includes a **CRC32 content hash** (not just the file modification timestamp), so cache hits are reliable on Windows FAT32 volumes, network drives, and other environments where `mtime` is unreliable

### Jupyter kernel not appearing in Zed

- Run `jupyter kernelspec list` to confirm the kernel is installed
- If missing, re-run `install.bat` / `install.sh` from the `kernel/` directory
- Run **"REPL: Refresh Kernelspecs"** from the Zed command palette (`Ctrl+Shift+P`)
- Make sure `jupyter` is installed and on your PATH

### Build script says "Cargo.toml or src\lib.rs has error" / WASM file not produced

Make sure you are running the build script from inside the `extension/` subdirectory, not the repo root. The script checks for `extension.toml` in the current directory and will exit with an error if it is not found.

This message can also appear if the `rustup target list` check produced a false negative. The real cause is usually that the WASM copy step was never reached.

**Fix:** Run the build command directly from `extension/`, then copy the output manually:

```bat
:: Windows — run from extension\
cargo build --release --target wasm32-wasip1
copy /Y target\wasm32-wasip1\release\zed_v_enhanced.wasm extension.wasm
```

```sh
# Linux / macOS — run from extension/
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/zed_v_enhanced.wasm extension.wasm
```

If `rustup target add wasm32-wasip1` reports *"component 'Rust-std' for target 'wasm32-wasip1' is up to date"*, the target is already installed — the script was just detecting it incorrectly. The updated `build.bat` / `build.sh` use an idempotent `rustup target add` call instead of fragile string matching, so this is fixed in the current scripts.

### Features stopped working after a Zed update

- Rebuild the extension (`cd extension && build.bat` / `./build.sh`) and reinstall
- The automatic update check will show a notification in the status bar if your velvet binary is also out of date — follow its instructions

### velvet update notification keeps appearing

- The notification means your local `velvet` binary is behind the main branch on GitHub
- Pull and rebuild: `cd velvet && git pull && v run build.vsh release`
- Copy the new binary to your PATH and do a full Zed restart
- If you intentionally want to stay on an older build, you can ignore the notification — it appears at most once per session and never prevents the language server from starting

### Settings don't seem to apply

- Settings must be placed under `lsp.velvet.initialization_options` in your Zed `settings.json` — **not** at the top level of `settings.json`
- For example, to disable the unused parameter warning: `"lsp": { "velvet": { "initialization_options": { "inspections": { "enable_unused_parameter_warning": false } } } }`
- Settings changes require a **full Zed restart** — not just closing and reopening a file

### Checking logs

- Zed menu → View → Zed Log
- Look for `velvet` entries to see initialization and request details

---

## Links

- [V Language](https://vlang.io/)
- [velvet (language server)](https://github.com/DaZhi-the-Revelator/velvet)
- [tree-sitter-v (V grammar)](https://github.com/DaZhi-the-Revelator/tree-sitter-v)
- [Upstream v-analyzer](https://github.com/vlang/v-analyzer)
- [Zed Editor](https://zed.dev/)
- [Zed REPL Docs](https://zed.dev/docs/repl)
- [Jupyter Kernel Protocol](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

---

## License

MIT
