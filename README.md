# V Enhanced — Language Support for Zed

A comprehensive V language extension for [Zed](https://zed.dev/), powered by [velvet](https://github.com/DaZhi-the-Revelator/velvet) with bug fixes, enhanced hover documentation, and correct symbol renaming.

**Supports V 0.5.3.**

---

## Table of Contents

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
- [Project Structure](#project-structure)
- [Troubleshooting](#troubleshooting)
  - [velvet not found](#velvet-not-found)
  - [Server crashes on enum hover](#server-crashes-on-enum-hover)
  - [Rename only updates one occurrence](#rename-only-updates-one-occurrence)
  - [No diagnostics / formatting not working](#no-diagnostics--formatting-not-working)
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
# Should print: velvet version 0.0.6
```

### Staying Up to Date

Pull the latest fixes and rebuild:

```sh
cd velvet
git pull
v run build.vsh release
# then copy the binary to PATH as above
```

---

## V 0.5.3 — Breaking Change Notice

> **`x.ttf` rendering module has moved.**
>
> In V 0.5.1, `vlib/x/ttf/render_sokol_cpu.v` was extracted into a separate module. If your
> project imports `x.ttf` for rendering functions, update your import:
>
> ```v
> // Before (V < 0.5.1)
> import x.ttf
>
> // After (V 0.5.1+)
> import x.ttf.render_sokol
> ```
>
> The base `x.ttf` module remains available for non-rendering TTF utilities. Only the sokol
> CPU-rendering surface has moved to `x.ttf.render_sokol`. After updating V to 0.5.1, re-run
> `velvet` against your project to refresh the stub index.

---

## Features

All LSP intelligence is provided by velvet. This extension wires every capability into Zed natively and adds the full Zed-specific layer on top.

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
- **Signature Help** — Real-time parameter hints as you type function calls:
  - Active parameter highlighted as you move through arguments
  - Retrigger support (`,` and ` ` re-trigger the hint)
  - Resolves the actual function declaration via PSI — always accurate
  - Works for all functions including stdlib and user-defined
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
  - Structs (with their fields nested inside)
  - Interfaces (with their methods and fields nested inside)
  - Enums (with their values nested inside, showing implicit values)
  - Constants (with type and value)
  - Type aliases
- **Inlay Hints** — 6 types of inline annotations:
  - **Type hints** — inferred type shown after `:=` assignments: `x: int := 10`
  - **Parameter name hints** — parameter names shown before arguments in function calls
  - **Range operator hints** — `≤` and `<` shown on `..` range operators to clarify inclusivity
  - **Implicit `err →` hints** — shown inside `or { }` blocks and `else` branches of result unwrapping
  - **Enum field value hints** — implicit enum field values shown inline next to each field
  - **Constant type hints** — type shown after constant declarations
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

All tasks run from the directory containing the source file (`$ZED_DIRNAME`) or the project root, as appropriate.

#### Test functions

Functions named `test_*` get their own gutter arrow and appear individually in the Runnables panel. When you click the gutter arrow next to a specific test function, V Enhanced runs only that test using `v test -run test_name $ZED_FILE`. This enables true TDD workflows where you can iterate on a single test without running the entire file's test suite.

---

### ✅ Jupyter Kernel & REPL Integration

V Enhanced ships a complete Jupyter kernel (`v-kernel`) that integrates with Zed's built-in REPL.

#### Installing the Kernel

The kernel is in the `kernel/` subdirectory of this extension. It is a separate Rust project that must be built and installed independently.

**Requirements:**

- Rust (install from [rustup.rs](https://rustup.rs/))
- V compiler on your `PATH`
- Jupyter installed (`pip install jupyter` or via conda)
- On Linux/macOS: system ZeroMQ library

```bash
# macOS
brew install zeromq

# Ubuntu / Debian
sudo apt install libzmq3-dev

# Fedora / RHEL
sudo dnf install zeromq-devel
```

On Windows, ZeroMQ is bundled — no extra steps needed.

#### Windows — Additional Requirements

On Windows, the kernel build requires both **Microsoft Visual Studio Build Tools** and **Rust** before running `install.bat`.

##### Step 1 — Install Visual Studio Build Tools

Download the installer from the [Visual Studio downloads page](https://visualstudio.microsoft.com/downloads/) (scroll to *Tools for Visual Studio* → **Build Tools for Visual Studio**). Run the installer and, on the *Workloads* tab, select:

- ✅ **Desktop development with C++**

This installs the MSVC compiler, Windows SDK, and the C/C++ linker that Rust uses on Windows. The full download is roughly 4–6 GB.

> **Alternative — winget:**
>
> ```bat
> winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
> ```

##### Step 2 — Install Rust

Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs/). Accept the defaults. When prompted to select an installation type, choose **1) Proceed with standard installation**.

After the installer finishes, open a **new** Command Prompt or PowerShell window so that `cargo` and `rustc` are on your `PATH`.

Verify:

```bat
rustc --version
cargo --version
```

> **If you installed Build Tools *after* Rust:** rustup links the MSVC toolchain automatically, but only if Build Tools were present at install time. If you see a linker error like `link.exe not found`, run:
>
> ```bat
> rustup toolchain install stable-x86_64-pc-windows-msvc
> rustup default stable-x86_64-pc-windows-msvc
> ```

Once both prerequisites are in place, proceed with the **Build and install** steps below.

**Build and install:**

```bat
:: Windows
cd kernel
install.bat
```

```bash
# Linux / macOS
cd kernel
chmod +x install.sh
./install.sh
```

The install scripts build the kernel with `cargo build --release`, copy the binary to `~/.cargo/bin/`, and register the Jupyter kernelspec automatically.

**Verify:**

```sh
jupyter kernelspec list
# Should show:
#   v    /path/to/jupyter/kernels/v
```

#### Using the REPL in Zed

1. Open any `.v` file
2. Optionally divide it into cells using `// %%` comment separators
3. Place your cursor anywhere in the cell you want to run
4. Press `Ctrl+Shift+Enter` (Windows/Linux) or `Cmd+Shift+Enter` (macOS) to execute the cell

With no `// %%` separators the entire file is treated as a single cell. With separators, only the cell containing your cursor is sent to the kernel.

If the V kernel doesn't appear in Zed's kernel picker, run **"REPL: Refresh Kernelspecs"** from the command palette (`Ctrl+Shift+P`).

#### How It Works

`v-kernel` implements the [Jupyter messaging protocol v5.3](https://jupyter-client.readthedocs.io/en/stable/messaging.html) over ZeroMQ.

**Stateful execution across cells:** top-level declarations (`fn`, `struct`, `enum`, `const`, `import`, `type`, `interface`) accumulate across cells for the duration of the session — later cells can reference structs and functions defined in earlier cells. Bare statements and expressions are wrapped in `fn main()` for the **current cell only** and are not accumulated, so re-running or editing a cell never causes "already defined" / redeclaration errors from stale earlier runs.

```v
import math

// %%

// Cell 1 — declarations accumulate across cells
struct Point {
    x f64
    y f64
}

fn distance(a Point, b Point) f64 {
    return math.sqrt((b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - a.y))
}

// %%

// Cell 2 — statements run inside fn main() automatically
p1 := Point{0, 0}
p2 := Point{3, 4}
println(distance(p1, p2))  // → 5.0

// %%

// Cell 3 — prior declarations are still in scope
p3 := Point{6, 8}
println(distance(p1, p3))  // → 10.0
```

#### Kernel Architecture

| Component | Purpose |
|-----------|---------|
| ZeroMQ sockets | Jupyter wire transport (shell, iopub, stdin, control, heartbeat) |
| HMAC-SHA256 | Message signing per the Jupyter protocol |
| Session state | Accumulated declarations tracked across cell executions |
| V compiler | Each cell invokes `v run` on the assembled program |

#### Kernel Limitations

- **No autocomplete in notebooks** — completion comes from velvet via LSP, not the kernel; works in `.v` files, not in `.ipynb` notebooks
- **Recompilation on every cell** — the full accumulated program is recompiled each time; V is fast, but long sessions accumulate more code to compile
- **Interrupt support** — `Ctrl+C` sends `interrupt_request`; the kernel forwards SIGINT (Unix) or `TerminateProcess` (Windows) to the running `v run` child process and returns the kernel to idle
- **No arbitrary rich display** — only `dump()` output is rendered as HTML; for general rich output, V has no equivalent of IPython's `display()` machinery
- **dump() table is render-only** — Zed's "copy output" and "open in buffer" actions work on plain stream output only; `display_data` messages (which is what the HTML table uses) are not supported by those actions in Zed's REPL frontend. This is a Zed limitation, not a kernel limitation. A `text/plain` fallback is included in the message for non-HTML frontends.

---

### ✅ Rich dump() Output in REPL

`dump()` calls are automatically intercepted by the kernel and rendered as a styled HTML table in the Zed REPL output panel.

V's `dump()` already returns structured information on each call:

```log
[main.v:8] x = int(42)
```

The kernel parses this output format and emits a `display_data` Jupyter message with `text/html` MIME data. The result is a colour-coded table with columns **location · name · type · value**, styled to match Catppuccin Mocha. A `text/plain` fallback is included for non-HTML frontends.

All other output (`println`, `print`, `eprintln`, etc.) continues to appear as plain stream text — only `dump()` lines are intercepted.

```v
// Cell — mix of dump() and plain output
x := 42
name := 'world'
println('hello')  // → plain stream text: "hello"
dump(x)           // → HTML table row: main.v:4 | x | int | 42
dump(name)        // → HTML table row: main.v:5 | name | string | world
```

No changes to your V code are needed — `dump()` works exactly as before; the kernel makes it look better in the REPL.

---

### ✅ Automatic velvet Update Check

Every time the extension activates (i.e. when you open a `.v` file and the language server starts), the extension silently:

1. Runs `velvet --version` to read the local binary's version string.
2. Fetches the latest release tag from the velvet repo via the GitHub API.
3. Compares the two version strings.

If they differ, a notice appears in the Zed language-server status bar:

> velvet is out of date (local: `0.0.6`, latest release: `0.1.0`). Run: `cd velvet && git pull && v run build.vsh release`, then copy `bin/velvet` to your PATH and restart Zed.

If the versions already match, or if the check fails for any reason (no network, API rate limit, etc.), nothing is shown. The check runs at most once per session and never blocks the language server from starting.

This addresses the silent breakage that can occur when Zed updates and the locally installed velvet binary lags behind.

---

### ✅ Syntax Highlighting

Powered by `tree_sitter_v` — velvet's battle-tested grammar — with comprehensive Zed-specific highlight queries covering:

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
- Enum fetch expressions (`Direction.north`)
- Global variable definitions
- Compile-time constructs (`$if`, `$for`, `$else`)
- Labels
- All operators (arithmetic, bitwise, comparison, assignment, channel, range)
- Comments: line (`//`) and block (`/* */`)
- Shebang (`#!/usr/bin/env v`)

**Variable scoping** via `locals.scm` prevents local variable names from bleeding across function boundaries in syntax-only highlighting mode (files over 1000 lines). Scopes are defined for function bodies, blocks, `if`/`else`, `for`, `match`, `lock`, `unsafe`, and `defer`. Parameter declarations, receiver names, loop variables, and short variable declarations are all tracked as definitions.

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

50 built-in snippets for common V patterns. Type the prefix and press Tab.

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

Double-click selects complete V identifiers including underscores — `snake_case_identifier` is selected in full, not word-by-word.

---

### ✅ Block Comment Toggle

`Ctrl+/` toggles `//` line comments. `/* */` block comments are also supported for the block comment toggle command.

---

### ✅ Feature Toggles

All velvet features can be individually enabled or disabled via your Zed `settings.json`. Changes take effect after a full Zed restart. The below settings are the **default**. You do not need to input these unless you want to change them from shown.

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
        "enable_enum_field_value_hints": true
      },
      "enable_semantic_tokens": "full",
      "code_lens": {
        "enable": true,
        "enable_run_lens": true,
        "enable_inheritors_lens": true,
        "enable_super_interfaces_lens": true,
        "enable_run_tests_lens": true
      }
    }
  }
}
```

**`enable_semantic_tokens` values:**

| Value | Behavior |
|-------|-----------|
| `"full"` | Two-pass: accurate semantic + syntax highlighting (default) |
| `"syntax"` | Syntax-only pass — faster, recommended for very large files |
| `"none"` | Semantic tokens disabled entirely |

---

## Requirements

### velvet (Required)

See the [Installing velvet](#installing-velvet) section above.

> **Do not use the upstream v-analyzer binary.** It will crash on enum hover and produce incorrect rename results.

### V Compiler

velvet uses the V compiler for diagnostics and formatting. This extension targets **V 0.5.3**. Install V from [vlang.io](https://vlang.io/).

If velvet cannot find your V installation automatically, create a project-local config:

```sh
velvet init
```

Then set `custom_vroot` in the generated `.velvet/config.toml`.

### Jupyter Kernel (Optional)

Required only if you want REPL/notebook support. See [Jupyter Kernel & REPL Integration](#-jupyter-kernel--repl-integration) above.

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
```

A global config also exists at `~/.config/velvet/config.toml` and applies to all projects.

---

## Project Structure

```txt
v-enhanced/
├── extension.toml              # Extension metadata, grammar reference, default settings
├── Cargo.toml                  # Rust extension dependency (zed_extension_api)
├── build.bat                   # Windows build script
├── build.sh                    # Linux / macOS build script
├── src/
│   └── lib.rs                  # Extension entry point — locates and launches velvet
├── languages/
│   └── v/
│       ├── config.toml         # Language settings (brackets, indent, comments, word chars)
│       ├── highlights.scm      # Comprehensive syntax highlighting queries
│       ├── brackets.scm        # Rainbow bracket pairs ({ } [ ] ( ))
│       ├── folds.scm           # Code folding queries
│       ├── injections.scm      # Embedded language injections (V interp, SQL, ASM)
│       ├── locals.scm          # Variable scope definitions for syntax-only highlighting
│       ├── outline.scm         # Breadcrumb / outline panel queries
│       ├── tags.scm            # Symbol search queries (Ctrl+T)
│       └── snippets.json       # 50 code snippets
├── languages/
│   └── vmod/
│       ├── config.toml         # VModManifest language settings
│       └── highlights.scm      # v.mod syntax highlighting (reuses V grammar)
└── kernel/                     # Jupyter kernel (separate Rust project)
    ├── src/
    │   └── main.rs             # Full kernel implementation
    ├── kernelspec/
    │   └── kernel.json         # Jupyter kernelspec descriptor
    ├── Cargo.toml              # Kernel Rust dependencies
    ├── install.bat             # Windows build + install script
    └── install.sh              # Linux / macOS build + install script
```

---

## Troubleshooting

### velvet not found

- Confirm it is in your PATH: `where velvet` (Windows) / `which velvet` (Linux/Mac)
- Build and install from: `https://github.com/DaZhi-the-Revelator/velvet`
- Restart Zed after installing

### Server crashes on enum hover

- You are using the upstream v-analyzer binary instead of velvet — install velvet instead (see above)

### Rename only updates one occurrence

- You are using the upstream v-analyzer binary instead of velvet — install velvet instead (see above)

### No diagnostics / formatting not working

- velvet needs the V compiler: confirm `v` is in PATH or set `custom_vroot` in config
- Run `velvet init` in your project root and set `custom_vroot` in the generated config

### Indexing is slow on first open

- velvet indexes your workspace and the V stdlib on first use — this is normal
- Progress is shown in the Zed status bar
- Subsequent opens use the cached index and are fast

### Jupyter kernel not appearing in Zed

- Run `jupyter kernelspec list` to confirm the kernel is installed
- If missing, re-run `install.bat` / `install.sh` from the `kernel/` directory
- Run **"REPL: Refresh Kernelspecs"** from the Zed command palette (`Ctrl+Shift+P`)
- Make sure `jupyter` is installed and on your PATH

### Build script says "Cargo.toml or src\lib.rs has error" / WASM file not produced

This message can appear even when the Rust compilation actually succeeded, if the `rustup target list` check in an older version of the script produced a false negative. The real cause is usually that the WASM copy step was never reached.

**Fix:** Run the build command directly, then copy the output manually:

```bat
:: Windows
cargo build --release --target wasm32-wasip1
copy /Y target\wasm32-wasip1\release\zed_v_enhanced.wasm extension.wasm
```

```sh
# Linux / macOS
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/zed_v_enhanced.wasm extension.wasm
```

If `rustup target add wasm32-wasip1` reports *"component 'Rust-std' for target 'wasm32-wasip1' is up to date"*, the target is already installed — the script was just detecting it incorrectly. The updated `build.bat` / `build.sh` use an idempotent `rustup target add` call instead of fragile string matching, so this is fixed in the current scripts.

### Features stopped working after a Zed update

- Rebuild the extension with `build.bat` / `build.sh` and reinstall
- The automatic update check will show a notification in the status bar if your velvet binary is also out of date — follow its instructions

### velvet update notification keeps appearing

- The notification means your local `velvet` binary is behind the main branch on GitHub
- Pull and rebuild: `cd velvet && git pull && v run build.vsh release`
- Copy the new binary to your PATH and do a full Zed restart
- If you intentionally want to stay on an older build, you can ignore the notification — it appears at most once per session and never prevents the language server from starting

### Settings don't seem to apply

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
