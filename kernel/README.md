# v-kernel

A Jupyter kernel for the [V programming language](https://vlang.io/), written in Rust.  
Integrates with [Zed's REPL](https://zed.dev/docs/repl) — press `Ctrl+Shift+Enter` (or `Cmd+Shift+Enter` on macOS) on any `.v` file to evaluate a cell.

---

## How it works

`v-kernel` implements the [Jupyter messaging protocol v5.3](https://jupyter-client.readthedocs.io/en/stable/messaging.html) over ZeroMQ.  
Zed detects it automatically once the kernelspec is installed — no configuration needed.

**Stateful execution across cells:** top-level declarations (`fn`, `struct`, `enum`, `const`, `import`, `type`, `interface`) accumulate across cells in a session — later cells can reference structs and functions defined earlier. Bare statements and expressions are wrapped in `fn main()` for the **current cell only** and are not accumulated, so re-running or editing a cell never causes redeclaration errors.

```v
// Cell 1 — declares a struct (accumulated)
struct Point {
    x f64
    y f64
}

fn distance(a Point, b Point) f64 {
    return math.sqrt((b.x - a.x) * (b.x - a.x) + (b.y - a.y) * (b.y - a.y))
}

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

### System ZeroMQ (Linux / macOS only)

The `zmq` crate links against `libzmq`. Install it before building:

```bash
# macOS
brew install zeromq

# Ubuntu / Debian
sudo apt install libzmq3-dev

# Fedora / RHEL
sudo dnf install zeromq-devel
```

On Windows, the `zmq` crate bundles a pre-built `libzmq` — no extra steps needed.

---

## Build & Install

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
- **Interrupt support** — `Ctrl+C` sends `interrupt_request`; the kernel forwards SIGINT (Unix) or `TerminateProcess` (Windows) to the running `v run` child process and returns the kernel to idle
- **dump() table is render-only** — Zed's "copy output" and "open in buffer" actions apply to plain stream messages only; the HTML table uses `display_data` which Zed does not currently expose those actions for. A `text/plain` fallback is included for non-HTML frontends. This is a Zed frontend limitation.
- **No arbitrary rich display** — only `dump()` is rendered as HTML; V has no equivalent of IPython's `display()` machinery
