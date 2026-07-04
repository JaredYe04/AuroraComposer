@echo off
setlocal EnableExtensions
cd /d "%~dp0"

echo ========================================
echo  Aurora Composer - Build and Run
echo ========================================
echo.

echo [1/3] Installing UI dependencies...
cd ui
call npm install
if errorlevel 1 (
  echo npm install failed.
  exit /b 1
)

echo.
echo [2/3] Building UI...
call npm run build
if errorlevel 1 (
  echo UI build failed.
  exit /b 1
)

echo.
echo [3/3] Starting desktop app...
cd ..\src-tauri
cargo run
if errorlevel 1 (
  echo cargo run failed.
  exit /b 1
)

endlocal
