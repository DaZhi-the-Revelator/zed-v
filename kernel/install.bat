@echo off
:: install.bat — Build and install the V Jupyter kernel on Windows
setlocal

echo [v-kernel] Building release binary...
cargo build --release
if errorlevel 1 (
    echo [v-kernel] Build failed.
    exit /b 1
)

:: Copy binary to a location on PATH — use %USERPROFILE%\.cargo\bin
echo [v-kernel] Installing binary to %USERPROFILE%\.cargo\bin\v-kernel.exe
copy /Y "target\release\v-kernel.exe" "%USERPROFILE%\.cargo\bin\v-kernel.exe"
if errorlevel 1 (
    echo [v-kernel] Failed to copy binary. Make sure %%USERPROFILE%%\.cargo\bin is on your PATH.
    exit /b 1
)

:: Install kernelspec
set KERNELSPEC_DIR=%APPDATA%\jupyter\kernels\v
echo [v-kernel] Installing kernelspec to %KERNELSPEC_DIR%
if not exist "%KERNELSPEC_DIR%" mkdir "%KERNELSPEC_DIR%"
copy /Y "kernelspec\kernel.json" "%KERNELSPEC_DIR%\kernel.json"

echo.
echo [v-kernel] Installation complete!
echo.
echo To verify, run:
echo   jupyter kernelspec list
echo.
echo Then in Zed, open a .v file and press Ctrl+Shift+Enter.
echo Run "repl: refresh kernelspecs" in Zed command palette if V does not appear.
