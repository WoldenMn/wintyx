@echo off
REM Wintyx — Script de release automatise
REM Lance lint + tests + build + copie installers dans Install/
REM Usage: release.bat
REM
REM Prerequis :
REM   - bun installe (https://bun.sh)
REM   - Rust + cargo installes
REM   - Tauri CLI : bun add -D @tauri-apps/cli@latest

setlocal enabledelayedexpansion

echo ========================================
echo Wintyx Release Build
echo ========================================
echo.

REM 1. Lint frontend
echo [1/7] ESLint...
call bun run lint
if errorlevel 1 (
    echo ERREUR : ESLint a echoue. Corriger les warnings avant de relancer.
    exit /b 1
)
echo.

REM 2. Lint backend
echo [2/7] Clippy...
pushd src-tauri
call cargo clippy --all-targets -- -D warnings
if errorlevel 1 (
    popd
    echo ERREUR : Clippy a echoue. Corriger les warnings avant de relancer.
    exit /b 1
)
popd
echo.

REM 3. Tests frontend
echo [3/7] Vitest...
call bun run test
if errorlevel 1 (
    echo ERREUR : Tests Vitest ont echoue.
    exit /b 1
)
echo.

REM 4. Tests backend
echo [4/7] Cargo test...
pushd src-tauri
call cargo test
if errorlevel 1 (
    popd
    echo ERREUR : Tests Rust ont echoue.
    exit /b 1
)
popd
echo.

REM 5. Build frontend
echo [5/7] Vite build...
call bun run build
if errorlevel 1 (
    echo ERREUR : bun run build a echoue.
    exit /b 1
)
echo.

REM 6. Build Tauri release (MSI + NSIS)
echo [6/7] Tauri build (release, ~3 min)...
call bun tauri build
if errorlevel 1 (
    echo ERREUR : bun tauri build a echoue.
    exit /b 1
)
echo.

REM 7. Lire la version depuis package.json (champ "version")
echo [7/7] Copie des installers vers Install\...
for /f "tokens=2 delims=:," %%a in ('findstr /c:"\"version\"" package.json') do (
    set VERSION=%%~a
    set VERSION=!VERSION: =!
    set VERSION=!VERSION:"=!
)
echo Version detectee : !VERSION!

if not exist Install mkdir Install

copy /Y "src-tauri\target\release\bundle\msi\Wintyx_!VERSION!_x64_*.msi" "Install\" >nul
if errorlevel 1 (
    echo AVERTISSEMENT : MSI introuvable au chemin attendu.
)

copy /Y "src-tauri\target\release\bundle\nsis\Wintyx_!VERSION!_x64-setup.exe" "Install\" >nul
if errorlevel 1 (
    echo AVERTISSEMENT : NSIS setup introuvable au chemin attendu.
)

copy /Y "src-tauri\target\release\wintyx.exe" "Install\Wintyx.exe" >nul
if errorlevel 1 (
    echo AVERTISSEMENT : binaire portable introuvable au chemin attendu.
)

echo.
echo ========================================
echo Release v!VERSION! prete dans Install\
echo ========================================
dir Install\Wintyx_!VERSION!_*

endlocal
