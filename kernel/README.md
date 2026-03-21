# v-kernel

A Jupyter kernel for the [V programming language](https://vlang.io/), written in Rust.  
Integrates with [Zed's REPL](https://zed.dev/docs/repl) — press `Ctrl+Shift+Enter` (or `Cmd+Shift+Enter` on macOS) on any `.v` file to evaluate a cell.

Part of the **V Enhanced** suite: [v-enhanced](https://github.com/DaZhi-the-Revelator/zed-v-enhanced) · [velvet](https://github.com/DaZhi-the-Revelator/velvet) · [tree-sitter-v](https://github.com/DaZhi-the-Revelator/tree-sitter-v)

---

## How it works

`v-kernel` implements the [Jupyter messaging protocol v5.3](https://jupyter-client.readthedocs.io/en/stable/messaging.html) over ZeroMQ.  
Zed detects it automatically once the kernelspec is installed — no configuration needed.

**Stateful execution across cells:** top-level declarations (`fn`, `struct`, `enum`, `const`, `import`, `type`, `interface`) accumulate across cells in a session — later cells can reference structs and functions defined earlier. Bare statements and expressions are wrapped in `fn main()` for the **current cell only** and are not accumulated, so re-running or editing a cell never causes redeclaration errors.

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

// Cell 2 — statements (wrapped in fn main automatically)
p1 := Point{0, 0}
p2 := Point{3, 4}
println(distance(p1, p2))  // → 5.0
```

---

## Features

### Rich `dump()` output

`dump()` calls are intercepted and rendered as a styled HTML table in the Zed REPL panel instead of raw text.

V's `dump()` already returns structured data — name, type, and value — on a single line:

```txt
[main.v:8] x = int(42)
```

The kernel parses this format and emits a `display_data` message with `text/html` mime data containing a colour-coded table, taking advantage of Jupyter's rich display protocol. A `text/plain` fallback is also included for non-HTML frontends.

```v
// Cell
x := 42
name := 'world'
pt := Point{3.0, 4.0}
dump(x)
dump(name)
dump(pt)
```

Renders as a table with columns: **location · name · type · value** — styled to match Catppuccin Mocha.

Non-`dump()` output (regular `println`, etc.) continues to appear as plain stream text as before.

---

## Requirements

- [Rust](https://rustup.rs/) (to build the kernel)
- [V](https://vlang.io/) installed and `v` on your `PATH`
- [Jupyter](https://jupyter.org/) installed (`pip install jupyter` or via conda)
- [Zed](https://zed.dev/) with the **v-enhanced** extension installed

### System ZeroMQ

The `zmq` crate links against the system `libzmq` on all platforms.

**Linux / macOS** — install before building:

```bash
# macOS
brew install zeromq

# Ubuntu / Debian
sudo apt install libzmq3-dev

# Fedora / RHEL
sudo dnf install zeromq-devel
```

**Windows** — install [vcpkg](https://github.com/microsoft/vcpkg) and then run:

```bat
vcpkg install zeromq:x64-windows
vcpkg integrate install
```

The `zmq` crate does **not** bundle `libzmq`. You also need **Microsoft Visual Studio Build Tools** with the **Desktop development with C++** workload installed before running `install.bat` — see below.

---

## Build & Install

### Windows — Prerequisites

Before running `install.bat` you need:

1. **Visual Studio Build Tools** with the **Desktop development with C++** workload. Download from the [Visual Studio downloads page](https://visualstudio.microsoft.com/downloads/) (scroll to *Tools for Visual Studio* → **Build Tools for Visual Studio**), or via winget:

   ```bat
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```

2. **Rust** — download `rustup-init.exe` from [rustup.rs](https://rustup.rs/) and proceed with the standard installation. Open a new terminal after installing so `cargo` and `rustc` are on your `PATH`.

   > If you installed Build Tools *after* Rust and see `link.exe not found`, run:
>
   > ```bat
   > rustup toolchain install stable-x86_64-pc-windows-msvc
   > rustup default stable-x86_64-pc-windows-msvc
   > ```

1. **libzmq via vcpkg** — see [System ZeroMQ](#system-zeromq) above.

### Windows

```bat
cd kernel
install.bat
```

### macOS / Linux

```bash
cd kernel
chmod +x install.sh
./install.sh
```

Both scripts:

1. Run `cargo build --release`
2. Copy the `v-kernel` binary to `~/.cargo/bin/`
3. Install the Jupyter kernelspec to the correct location

### Verify

```bash
jupyter kernelspec list
# Should show:
#   v    /path/to/jupyter/kernels/v
```

---

## Using in Zed

1. Open any `.v` file
2. Add a cell separator comment: `// %%`
3. Place your cursor in a cell
4. Press `Ctrl+Shift+Enter` (Windows/Linux) or `Cmd+Shift+Enter` (macOS)

If the V kernel doesn't appear in Zed's kernel picker, run **"REPL: Refresh Kernelspecs"** from the command palette (`Ctrl+Shift+P`).

---

## Example Session

The session below covers the main patterns a learner will hit. Create any `.v` file, paste the cells in order, and execute each one with `Ctrl+Shift+Enter` / `Cmd+Shift+Enter`.

```v
import math

// %%

// ---- Cell 1: Declarations -----------------------------------------------
// Top-level declarations accumulate for the whole session.
// Re-running this cell is safe — the kernel guards against redeclaration errors.

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

// Output: (none — declarations produce no output)

// %%

// ---- Cell 2: Exploration with dump() ------------------------------------
// Bare statements run in an isolated fn main() for this cell only.
// dump() renders as a styled HTML table instead of raw text.

a := Vec2{ x: 3.0, y: 4.0 }
b := Vec2{ x: 1.0, y: 2.0 }
dump(a)            // table row → location | a | Vec2 | Vec2{x: 3.0, y: 4.0}
dump(a.length())   // table row → location | a.length() | f64 | 5.0
dump(a.add(b))     // table row → location | a.add(b) | Vec2 | Vec2{x: 4.0, y: 6.0}

// %%

// ---- Cell 3: Scope boundary (important!) --------------------------------
// 'a' and 'b' from Cell 2 are gone — they were local to that cell's fn main().
// Declarations from Cell 1 (Vec2, length, add) are still available.

c := Vec2{ x: 3.0, y: 4.0 }.add(Vec2{ x: 1.0, y: 2.0 })
dump(c)                              // Vec2{x: 4.0, y: 6.0}
println('length: ${c.length():.4f}') // length: 7.2111

// %%

// ---- Cell 4: Error handling ---------------------------------------------
// Functions that return result/option types are fully supported.

fn safe_sqrt(x f64) !f64 {
    if x < 0 {
        return error('cannot take sqrt of negative number: ${x}')
    }
    return math.sqrt(x)
}

// Successful call:
println(safe_sqrt(9.0)!)    // 3.0

// Failed call — handled with or {}:
result := safe_sqrt(-1.0) or {
    println('caught: ${err}')  // caught: cannot take sqrt of negative number: -1.0
    0.0
}
dump(result)  // 0.0
```

**What to observe in each cell:**

| Cell | What to notice |
|------|---------------|
| 1 | No output — declarations are silently registered for the session |
| 2 | Three `dump()` calls render as a single HTML table with four columns |
| 3 | `a` from Cell 2 is undefined here; `Vec2` from Cell 1 is still available |
| 4 | `!` and `or {}` work normally; the kernel reports the error message as stream text |

---

## Cell separator

Zed uses `// %%` to delimit REPL cells in non-notebook files:

```v
import math

// %%

x := math.sqrt(2.0)
println(x)  // → 1.4142135623730951

// %%

y := x * x
println(y)  // → 2.0
```

---

## Architecture

```txt
v-kernel/
├── src/
│   └── main.rs       # Full kernel implementation
├── kernelspec/
│   └── kernel.json   # Jupyter kernelspec descriptor
├── Cargo.toml        # Rust dependencies
├── install.bat       # Windows installer
└── install.sh        # macOS / Linux installer
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `zmq` | ZeroMQ sockets (Jupyter transport) |
| `serde` / `serde_json` | Jupyter wire protocol JSON |
| `hmac` + `sha2` + `hex` | Message signing (HMAC-SHA256) |
| `uuid` | Message and session IDs |
| `chrono` | ISO 8601 timestamps in message headers |

---

## Limitations

- **No autocomplete / introspection** — the kernel runs code but does not expose completion or inspection endpoints (those come from velvet via the LSP, which works independently)
- **Re-execution overhead** — the full accumulated program is recompiled on every cell execution; V is fast, but deep sessions will accumulate latency
- **Interrupt support** — `Ctrl+C` sends an `interrupt_request` message; the kernel forwards SIGINT (Unix) or `TerminateProcess` (Windows) to the running `v run` child process and returns the kernel to idle. `kernel.json` uses `interrupt_mode: "message"`, which works on all platforms
- **dump() table is render-only** — Zed's "copy output" and "open in buffer" actions apply to plain stream messages only; the HTML table uses `display_data` which Zed does not currently expose those actions for. A `text/plain` fallback is included for non-HTML frontends. This is a Zed frontend limitation.
- **No arbitrary rich display** — only `dump()` is rendered as HTML; V has no equivalent of IPython's `display()` machinery
