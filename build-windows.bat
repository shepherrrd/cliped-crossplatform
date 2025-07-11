@echo off
REM Windows Build Script for Cliped Cross-Platform

echo 🪟 Building Cliped Cross-Platform for Windows...

REM Check prerequisites
echo 📋 Checking prerequisites...

where npm >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ npm is not installed. Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ cargo is not installed. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Install dependencies
echo 📦 Installing dependencies...
npm install

REM Build the app
echo 🔨 Building Cliped Cross-Platform...
npm run tauri build

REM Check if build succeeded
if not exist "src-tauri\target\release\bundle\msi" (
    echo ❌ Build failed - no MSI installer found
    pause
    exit /b 1
)

echo ✅ Build successful!

REM Create distribution directory
if not exist "dist" mkdir dist

REM Copy installer
copy "src-tauri\target\release\bundle\msi\*.msi" dist\

echo ✅ Windows build complete!
echo.
echo 📁 Distribution files:
dir dist\
echo.
echo 📦 Installation:
echo    Run the .msi installer in the dist\ folder
echo    The installed program will be available as 'cliped'
echo.
echo 🚀 To test locally:
echo    Install the .msi file and run Cliped from Start Menu

pause
