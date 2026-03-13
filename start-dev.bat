@echo off
title ArtCraft Dev Server
echo ============================================
echo   ArtCraft Desktop - Dev Mode
echo ============================================
echo.

:: Set required environment variables
set SQLX_OFFLINE=true
set LIBCLANG_PATH=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin
set PATH=%PATH%;C:\Program Files\NASM

:: Refresh PATH from system
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v Path 2^>nul') do set "SYSPATH=%%b"
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v Path 2^>nul') do set "USRPATH=%%b"
set PATH=%SYSPATH%;%USRPATH%;C:\Program Files\NASM

echo [1/2] Starting frontend dev server...
echo.

:: Start frontend in a new window
start "ArtCraft Frontend" cmd /c "cd /d F:\__PROJECTS\ArtCraft\frontend && npx nx run artcraft:dev"

:: Wait for Vite to start
echo Waiting for frontend to be ready (5 seconds)...
timeout /t 5 /nobreak >nul

echo.
echo [2/2] Starting Tauri backend...
echo.

cd /d F:\__PROJECTS\ArtCraft\crates\desktop\artcraft
cargo tauri dev --config tauri.dev.override.json

pause
