#!/bin/bash
# ==============================================================================
# V ENHANCED ZED EXTENSION - BUILD SCRIPT (Linux / macOS)
# ==============================================================================

set -e  # Exit on error

echo ""
echo "================================================================================"
echo "                  V ENHANCED ZED EXTENSION - BUILD SCRIPT"
echo "================================================================================"
echo ""
echo "This script will:"
echo "  1. Verify prerequisites (Rust, wasm32-wasip1 target)"
echo "  2. Check for v-analyzer in PATH"
echo "  3. Clean old build artifacts"
echo "  4. Build the Rust extension for WASM"
echo "  5. Copy extension.wasm to the project root"
echo "  6. Clear Zed's extension cache"
echo ""
echo "Prerequisites:"
echo "  - Rust toolchain (rustup)"
echo "  - wasm32-wasip1 target (will be installed if missing)"
echo "  - v-analyzer installed and in PATH"
echo "  - Zed fully closed"
echo ""
read -p "Press Enter to continue..."
echo ""

# ==============================================================================
# STEP 1: Verify we are in the right directory
# ==============================================================================
echo "[1/6] Verifying directory structure..."

if [ ! -f "extension.toml" ]; then
    echo "ERROR: Cannot find extension.toml"
    echo "Please run this script from the zed-v-enhanced directory."
    echo "Current directory: $(pwd)"
    exit 1
fi

if [ ! -f "src/lib.rs" ]; then
    echo "ERROR: Cannot find src/lib.rs"
    echo "Please run this script from the zed-v-enhanced directory."
    exit 1
fi

if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Cannot find Cargo.toml"
    echo "Please run this script from the zed-v-enhanced directory."
    exit 1
fi

echo "  Directory structure verified OK"
echo ""

# ==============================================================================
# STEP 2: Verify Rust WASM target
# ==============================================================================
echo "[2/6] Verifying Rust WASM target..."

if ! rustup target list | grep -q "wasm32-wasip1 (installed)"; then
    echo "  WASM target not found, installing..."
    rustup target add wasm32-wasip1 || {
        echo "ERROR: Failed to install wasm32-wasip1 target"
        echo "Try manually: rustup target add wasm32-wasip1"
        exit 1
    }
    echo "  WASM target installed"
else
    echo "  WASM target already installed"
fi
echo ""

# ==============================================================================
# STEP 3: Check for v-analyzer
# ==============================================================================
echo "[3/6] Checking for v-analyzer..."

if ! command -v v-analyzer &>/dev/null; then
    echo ""
    echo "  WARNING: v-analyzer not found in PATH"
    echo "  The extension will still build, but users will need to install v-analyzer."
    echo ""
    echo "  To install v-analyzer:"
    echo "    v download -RD https://raw.githubusercontent.com/vlang/v-analyzer/main/install.vsh"
    echo ""
else
    echo "  Found: $(which v-analyzer)"
fi
echo ""

# ==============================================================================
# STEP 4: Clean old build artifacts
# ==============================================================================
echo "[4/6] Cleaning old build artifacts..."

if [ -f extension.wasm ]; then
    rm extension.wasm
    echo "  - Deleted old extension.wasm"
fi

if [ -d grammars ]; then
    rm -rf grammars
    echo "  - Deleted grammars directory (will be rebuilt by Zed)"
fi

if [ -d target/wasm32-wasip1/release ]; then
    rm -rf target/wasm32-wasip1/release
    echo "  - Cleaned target/wasm32-wasip1/release"
fi

cargo clean >/dev/null 2>&1 && echo "  - Cargo cache cleaned" || echo "  - cargo clean skipped"
echo ""

# ==============================================================================
# STEP 5: Build Rust extension for WASM
# ==============================================================================
echo "[5/6] Building Rust extension for WASM..."
echo "  Target: wasm32-wasip1"
echo "  This may take a few minutes on first build..."
echo ""

cargo build --release --target wasm32-wasip1 || {
    echo ""
    echo "ERROR: Cargo build failed"
    echo ""
    echo "Common causes:"
    echo "  - Rust toolchain outdated     -> run: rustup update"
    echo "  - WASM target missing         -> run: rustup target add wasm32-wasip1"
    echo "  - Cargo.toml or lib.rs errors -> check for syntax issues"
    echo ""
    echo "WASM targets currently installed:"
    rustup target list | grep wasm32 || echo "  (none found)"
    exit 1
}

# Find the compiled WASM — crate name is zed-v-enhanced so output is zed_v_enhanced.wasm
WASM_PATH="target/wasm32-wasip1/release/zed_v_enhanced.wasm"

if [ ! -f "$WASM_PATH" ]; then
    echo ""
    echo "ERROR: Expected $WASM_PATH not found after build."
    echo ""
    echo "WASM files found in target:"
    find target -name "*.wasm" 2>/dev/null || echo "  (none)"
    exit 1
fi

cp "$WASM_PATH" extension.wasm || {
    echo "ERROR: Failed to copy to extension.wasm"
    exit 1
}

WASM_SIZE=$(wc -c < extension.wasm | tr -d ' ')
echo ""
echo "  Extension built: $WASM_SIZE bytes"
echo ""

# ==============================================================================
# STEP 6: Clear Zed extension cache
# ==============================================================================
echo "[6/6] Clearing Zed extension cache..."

# Determine cache path by OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    ZED_CACHE="$HOME/Library/Application Support/Zed/extensions/work/v-enhanced"
else
    ZED_CACHE="$HOME/.local/share/zed/extensions/work/v-enhanced"
fi

if [ -d "$ZED_CACHE" ]; then
    rm -rf "$ZED_CACHE" && echo "  Zed cache cleared" || echo "  WARNING: Could not clear cache — is Zed still running?"
else
    echo "  No existing cache to clear"
fi
echo ""

# ==============================================================================
# DONE
# ==============================================================================
echo "================================================================================"
echo "                              BUILD COMPLETE!"
echo "================================================================================"
echo ""
echo "  extension.wasm  — $WASM_SIZE bytes"
echo ""
echo "NEXT STEPS:"
echo ""
echo "1. CLOSE ZED COMPLETELY"
echo "   Mac:   check Activity Monitor"
echo "   Linux: check System Monitor or run: pkill zed"
echo ""
echo "2. REOPEN ZED"
echo ""
echo "3. INSTALL DEV EXTENSION"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "   - Press Cmd+Shift+X (Extensions)"
else
    echo "   - Press Ctrl+Shift+X (Extensions)"
fi
echo "   - Click 'Install Dev Extension'"
echo "   - Browse to: $(pwd)"
echo "   - Click Open / Select Folder"
echo ""
echo "4. OPEN A .v FILE and verify:"
echo "   - Syntax highlighting works"
echo "   - Hover shows documentation"
echo "   - Completions appear"
echo "   - Inlay hints show types and parameter names"
echo "   - Code lens shows Run / Run test buttons"
echo ""
echo "5. IF SOMETHING LOOKS WRONG:"
echo "   - View -> Zed Log — look for v-analyzer entries"
echo "   - Confirm v-analyzer is in PATH: which v-analyzer"
echo "   - Confirm V compiler is in PATH: which v"
echo ""
echo "================================================================================"
echo ""
