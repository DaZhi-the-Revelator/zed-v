@echo off
setlocal enabledelayedexpansion

echo.
echo ================================================================================
echo                    V ENHANCED ZED EXTENSION - BUILD SCRIPT
echo ================================================================================
echo.
echo This script will:
echo   1. Verify prerequisites (Rust, wasm32-wasip1 target)
echo   2. Clean old build artifacts
echo   3. Build the Rust extension for WASM
echo   4. Copy extension.wasm to the project root
echo   5. Clear Zed's extension cache
echo.
echo Prerequisites:
echo   - Rust toolchain (rustup)
echo   - wasm32-wasip1 target (will be installed if missing)
echo   - v-analyzer installed and in PATH
echo   - Zed fully closed
echo.
pause
echo.

:: ==============================================================================
:: STEP 1: Verify Rust WASM target
:: ==============================================================================
echo [1/5] Verifying Rust WASM target...

rustup target list | findstr /C:"wasm32-wasip1 (installed)" >nul
if %errorlevel% neq 0 (
    echo   WASM target not found, installing...
    rustup target add wasm32-wasip1
    if %errorlevel% neq 0 (
        echo ERROR: Failed to install wasm32-wasip1 target
        echo Try manually: rustup target add wasm32-wasip1
        pause
        exit /b 1
    )
    echo   WASM target installed
) else (
    echo   WASM target already installed
)
echo.

:: ==============================================================================
:: STEP 2: Verify v-analyzer is available
:: ==============================================================================
echo [2/5] Checking for v-analyzer...

where v-analyzer >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo   WARNING: v-analyzer not found in PATH
    echo   The extension will still build, but users will need to install v-analyzer.
    echo.
    echo   To install v-analyzer:
    echo     v download -RD https://raw.githubusercontent.com/vlang/v-analyzer/main/install.vsh
    echo.
) else (
    for /f "tokens=*" %%i in ('where v-analyzer') do echo   Found: %%i
)
echo.

:: ==============================================================================
:: STEP 3: Clean old build artifacts
:: ==============================================================================
echo [3/5] Cleaning old build artifacts...

if exist extension.wasm (
    del /Q extension.wasm
    echo   - Deleted old extension.wasm
)

if exist target\wasm32-wasip1\release (
    rmdir /S /Q target\wasm32-wasip1\release 2>nul
    echo   - Cleaned target\wasm32-wasip1\release
)

cargo clean >nul 2>&1
echo   - Cargo cache cleaned
echo.

:: ==============================================================================
:: STEP 4: Build Rust extension for WASM
:: ==============================================================================
echo [4/5] Building Rust extension for WASM...
echo   Target: wasm32-wasip1
echo   This may take a few minutes on first build...
echo.

cargo build --release --target wasm32-wasip1
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Cargo build failed
    echo.
    echo Common causes:
    echo   - Rust toolchain not installed or outdated (run: rustup update)
    echo   - wasm32-wasip1 target missing (run: rustup target add wasm32-wasip1)
    echo   - Cargo.toml or src\lib.rs has errors
    echo.
    pause
    exit /b 1
)

:: Find and copy the WASM file
if not exist target\wasm32-wasip1\release\zed_v_enhanced.wasm (
    echo ERROR: Expected zed_v_enhanced.wasm not found
    echo Listing WASM files found:
    dir target\wasm32-wasip1\release\*.wasm 2>nul
    pause
    exit /b 1
)

copy /Y target\wasm32-wasip1\release\zed_v_enhanced.wasm extension.wasm >nul
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy to extension.wasm
    pause
    exit /b 1
)

for %%A in (extension.wasm) do echo   Extension built: %%~zA bytes
echo.

:: ==============================================================================
:: STEP 5: Clear Zed extension cache
:: ==============================================================================
echo [5/5] Clearing Zed extension cache...

set ZED_CACHE=%LOCALAPPDATA%\Zed\extensions\work\v-enhanced
if exist "%ZED_CACHE%" (
    rmdir /S /Q "%ZED_CACHE%" 2>nul
    if exist "%ZED_CACHE%" (
        echo   WARNING: Could not delete Zed cache - is Zed still running?
    ) else (
        echo   Zed cache cleared
    )
) else (
    echo   No existing cache to clear
)
echo.

:: ==============================================================================
:: DONE
:: ==============================================================================
echo ================================================================================
echo                              BUILD COMPLETE!
echo ================================================================================
echo.
echo NEXT STEPS:
echo.
echo 1. CLOSE ZED COMPLETELY (check Task Manager)
echo.
echo 2. REOPEN ZED
echo.
echo 3. INSTALL DEV EXTENSION
echo    - Press Ctrl+Shift+X (Extensions)
echo    - Click "Install Dev Extension"
echo    - Browse to: %CD%
echo    - Click Select Folder
echo.
echo 4. OPEN A .v FILE and verify:
echo    - Syntax highlighting works
echo    - Hover shows documentation
echo    - Completions appear
echo    - Inlay hints show types and parameter names
echo    - Code lens shows Run / Run test buttons
echo.
echo ================================================================================
echo.
pause
