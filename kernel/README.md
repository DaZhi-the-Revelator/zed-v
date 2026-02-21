# v-kernel

A Jupyter kernel for the [V programming language](https://vlang.io/), written in Rust.  
Integrates with [Zed's REPL](https://zed.dev/docs/repl) — press `Ctrl+Shift+Enter` (or `Cmd+Shift+Enter` on macOS) on any `.v` file to evaluate a cell.

---

## How it works

`v-kernel` implements the [Jupyter messaging protocol v5.3](https://jupyter-client.readthedocs.io/en/stable/messaging.html) over ZeroMQ.  
Zed detects it automatically once the kernelspec is installed — no configuration needed.

**Stateful execution across cells:** top-level declarations (`fn`, `struct`, `enum`, `const`, `import`, `type`, `interface`) accumulate across cells in a session. Bare statements and expressions are wrapped in `fn main()` and re-executed together with all prior statements each time. This mirrors how other REPL kernels (e.g. evcxr for Rust) handle compiled languages.

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

```
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

- **No autocomplete / introspection** — the kernel runs code but does not expose completion or inspection endpoints (those come from v-analyzer via the LSP, which works independently)
- **Re-execution overhead** — the full accumulated program is recompiled on every cell execution; V is fast, but deep sessions will accumulate latency
- **No interrupt** — `Ctrl+C` in Zed will send `interrupt_request`; the kernel does not yet forward SIGINT to the running `v` process
- **No rich display** — output is plain text/stderr only; V has no equivalent of IPython's `display()` machinery
