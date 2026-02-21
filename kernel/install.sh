#!/usr/bin/env bash
# install.sh — Build and install the V Jupyter kernel on macOS / Linux
set -e

echo "[v-kernel] Building release binary..."
cargo build --release

# Determine install destination — prefer ~/.cargo/bin (already on PATH for Rust users)
INSTALL_DIR="$HOME/.cargo/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    INSTALL_DIR="/usr/local/bin"
fi

echo "[v-kernel] Installing binary to $INSTALL_DIR/v-kernel"
cp target/release/v-kernel "$INSTALL_DIR/v-kernel"
chmod +x "$INSTALL_DIR/v-kernel"

# Install kernelspec
# jupyter --data-dir gives the base; kernels go in kernels/v/
JUPYTER_DATA=$(jupyter --data-dir 2>/dev/null || echo "$HOME/.local/share/jupyter")
KERNELSPEC_DIR="$JUPYTER_DATA/kernels/v"

echo "[v-kernel] Installing kernelspec to $KERNELSPEC_DIR"
mkdir -p "$KERNELSPEC_DIR"
cp kernelspec/kernel.json "$KERNELSPEC_DIR/kernel.json"

echo ""
echo "[v-kernel] Installation complete!"
echo ""
echo "To verify:"
echo "  jupyter kernelspec list"
echo ""
echo "Then in Zed, open a .v file and press Ctrl+Shift+Enter (macOS: Cmd+Shift+Enter)."
echo "Run 'repl: refresh kernelspecs' in the Zed command palette if V does not appear."
