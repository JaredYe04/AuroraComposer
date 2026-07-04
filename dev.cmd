@echo off
setlocal EnableExtensions
cd /d "%~dp0"

echo ========================================
echo  Aurora Composer - Dev (hot reload)
echo ========================================
echo.
echo Frontend changes reload automatically via Vite HMR.
echo Rust/backend changes still require restarting this script.
echo.

echo [1/2] Installing UI dependencies...
cd ui
call npm install
if errorlevel 1 (
  echo npm install failed.
  exit /b 1
)

echo.
echo [2/2] Starting Tauri dev (Vite + desktop shell)...
cd ..
call ui\node_modules\.bin\tauri dev
if errorlevel 1 (
  echo tauri dev failed.
  exit /b 1
)

endlocal
