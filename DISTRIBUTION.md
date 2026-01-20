# MoFA Studio Distribution Guide (Phase 1: Developer/Beta)

This guide explains how to build, package, and distribute the beta version of MoFA Studio.

## Table of Contents

1. [Distribution Package Structure](#distribution-package-structure)
2. [Build Steps](#build-steps)
3. [Packaging Scripts](#packaging-scripts)
4. [User Installation Guide](#user-installation-guide)
5. [FAQ](#faq)

---

## Distribution Package Structure

Final distribution package structure:

```
mofa-studio-v0.1.0-win64/
├── mofa-studio.exe           # Main executable
├── install.ps1               # Windows install script
├── install.sh                # macOS/Linux install script
├── start.bat                 # Windows launch script
├── start.sh                  # macOS/Linux launch script
├── pixi.toml                 # Python environment config
├── pixi.lock                 # Locked versions
├── node-hub/                 # Dora node source code
│   ├── dora-asr/
│   ├── dora-kokoro-tts/
│   ├── dora-qwen3/
│   └── ...
├── apps/                     # Dataflow definitions
│   └── mofa-fm/
│       └── dataflow/
│           └── voice-chat.yml
├── libs/                     # Shared libraries
│   └── dora-common/
└── README.txt                # User instructions
```

---

## Build Steps

### Prerequisites (Build Machine)

- Rust toolchain (rustup)
- Git

### Step 1: Build Release Version

```bash
# Navigate to project directory
cd mofa-studio

# Build main executable (optimized)
cargo build --release -p mofa-studio-shell

# Build Rust Dora nodes
cargo build --release -p dora-maas-client
cargo build --release -p dora-conference-bridge
cargo build --release -p dora-conference-controller
```

**Output locations**:
- Windows: `target/release/mofa-studio.exe`
- macOS: `target/release/mofa-studio`
- Linux: `target/release/mofa-studio`

### Step 2: Optimize Build Configuration (Optional)

Add the following to the end of `Cargo.toml` to reduce binary size:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"
opt-level = "z"  # Optimize for size, or use "3" for speed
```

Recompiling can reduce size by 30-50%.

### Step 3: Prepare Distribution Directory

```bash
# Create distribution directory
mkdir -p dist/mofa-studio-v0.1.0-win64

# Copy main executable
cp target/release/mofa-studio.exe dist/mofa-studio-v0.1.0-win64/

# Copy Rust nodes
mkdir -p dist/mofa-studio-v0.1.0-win64/bin
cp target/release/dora-maas-client.exe dist/mofa-studio-v0.1.0-win64/bin/
cp target/release/dora-conference-bridge.exe dist/mofa-studio-v0.1.0-win64/bin/
cp target/release/dora-conference-controller.exe dist/mofa-studio-v0.1.0-win64/bin/

# Copy Python environment config
cp pixi.toml dist/mofa-studio-v0.1.0-win64/
cp pixi.lock dist/mofa-studio-v0.1.0-win64/

# Copy Python nodes
cp -r node-hub dist/mofa-studio-v0.1.0-win64/
cp -r libs dist/mofa-studio-v0.1.0-win64/

# Copy dataflow definitions
mkdir -p dist/mofa-studio-v0.1.0-win64/apps/mofa-fm/dataflow
cp apps/mofa-fm/dataflow/voice-chat.yml dist/mofa-studio-v0.1.0-win64/apps/mofa-fm/dataflow/

# Copy model download scripts
cp -r models/setup-local-models dist/mofa-studio-v0.1.0-win64/models/
```

---

## Packaging Scripts

### Windows Install Script (install.ps1)

Create `install.ps1` in the distribution directory:

```powershell
# MoFA Studio Install Script (Windows)
# Usage: Right-click -> Run with PowerShell

$ErrorActionPreference = "Stop"
$MOFA_HOME = $PSScriptRoot

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  MoFA Studio Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check Pixi
Write-Host "[1/4] Checking Pixi..." -ForegroundColor Yellow
if (!(Get-Command pixi -ErrorAction SilentlyContinue)) {
    Write-Host "  Pixi not found, installing..." -ForegroundColor Gray
    irm https://pixi.sh/install.ps1 | iex
    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}
Write-Host "  Pixi ready: $(pixi --version)" -ForegroundColor Green

# Step 2: Install Python environment
Write-Host "[2/4] Installing Python environment (first run takes a few minutes)..." -ForegroundColor Yellow
Push-Location $MOFA_HOME
pixi install
Pop-Location
Write-Host "  Python environment ready" -ForegroundColor Green

# Step 3: Check Dora
Write-Host "[3/4] Checking Dora..." -ForegroundColor Yellow
if (!(Get-Command dora -ErrorAction SilentlyContinue)) {
    Write-Host "  Dora not found, installing..." -ForegroundColor Gray
    cargo install dora-cli --version 0.3.12 --locked
}
Write-Host "  Dora ready: $(dora --version)" -ForegroundColor Green

# Step 4: Download models (optional)
Write-Host "[4/4] Model download (optional)" -ForegroundColor Yellow
$downloadModels = Read-Host "  Download AI models? This requires ~5GB disk space (y/N)"
if ($downloadModels -eq "y" -or $downloadModels -eq "Y") {
    Push-Location "$MOFA_HOME\models\setup-local-models"
    pixi run python download_models.py --download funasr
    pixi run python download_models.py --download kokoro
    Pop-Location
    Write-Host "  Model download complete" -ForegroundColor Green
} else {
    Write-Host "  Skipping model download. Run models\setup-local-models\download_models.py later" -ForegroundColor Gray
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Installation complete!" -ForegroundColor Green
Write-Host "  Run start.bat to launch the application" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
```

### macOS/Linux Install Script (install.sh)

Create `install.sh`:

```bash
#!/bin/bash
# MoFA Studio Install Script (macOS/Linux)
# Usage: chmod +x install.sh && ./install.sh

set -e
MOFA_HOME="$(cd "$(dirname "$0")" && pwd)"

echo "========================================"
echo "  MoFA Studio Installer"
echo "========================================"
echo ""

# Step 1: Check Pixi
echo -e "\033[33m[1/4] Checking Pixi...\033[0m"
if ! command -v pixi &> /dev/null; then
    echo "  Pixi not found, installing..."
    curl -fsSL https://pixi.sh/install.sh | bash
    export PATH="$HOME/.pixi/bin:$PATH"
fi
echo -e "\033[32m  Pixi ready: $(pixi --version)\033[0m"

# Step 2: Install Python environment
echo -e "\033[33m[2/4] Installing Python environment (first run takes a few minutes)...\033[0m"
cd "$MOFA_HOME"
pixi install
echo -e "\033[32m  Python environment ready\033[0m"

# Step 3: Check Dora
echo -e "\033[33m[3/4] Checking Dora...\033[0m"
if ! command -v dora &> /dev/null; then
    echo "  Dora not found, installing..."
    cargo install dora-cli --version 0.3.12 --locked
fi
echo -e "\033[32m  Dora ready: $(dora --version)\033[0m"

# Step 4: Download models (optional)
echo -e "\033[33m[4/4] Model download (optional)\033[0m"
read -p "  Download AI models? This requires ~5GB disk space (y/N): " download_models
if [[ "$download_models" == "y" || "$download_models" == "Y" ]]; then
    cd "$MOFA_HOME/models/setup-local-models"
    pixi run python download_models.py --download funasr
    pixi run python download_models.py --download kokoro
    echo -e "\033[32m  Model download complete\033[0m"
else
    echo "  Skipping model download. Run models/setup-local-models/download_models.py later"
fi

echo ""
echo "========================================"
echo -e "\033[32m  Installation complete!\033[0m"
echo "  Run ./start.sh to launch the application"
echo "========================================"
```

### Windows Launch Script (start.bat)

Create `start.bat`:

```batch
@echo off
setlocal

set MOFA_HOME=%~dp0
cd /d "%MOFA_HOME%"

echo ========================================
echo   MoFA Studio
echo ========================================
echo.

REM Check if voice chat should be started
set /p VOICE_CHAT="Start voice chat feature? (y/N): "
if /i "%VOICE_CHAT%"=="y" (
    echo Starting Dora dataflow...
    start /B dora up
    timeout /t 2 >nul
    start /B dora start apps\mofa-fm\dataflow\voice-chat.yml
    timeout /t 3 >nul
)

echo Starting MoFA Studio...
mofa-studio.exe

REM Cleanup Dora if started
if /i "%VOICE_CHAT%"=="y" (
    echo Shutting down Dora...
    dora destroy
)

endlocal
```

### macOS/Linux Launch Script (start.sh)

Create `start.sh`:

```bash
#!/bin/bash
MOFA_HOME="$(cd "$(dirname "$0")" && pwd)"
cd "$MOFA_HOME"

echo "========================================"
echo "  MoFA Studio"
echo "========================================"
echo ""

# Check if voice chat should be started
read -p "Start voice chat feature? (y/N): " VOICE_CHAT
if [[ "$VOICE_CHAT" == "y" || "$VOICE_CHAT" == "Y" ]]; then
    echo "Starting Dora dataflow..."
    dora up &
    sleep 2
    dora start apps/mofa-fm/dataflow/voice-chat.yml &
    sleep 3
fi

echo "Starting MoFA Studio..."
./mofa-studio

# Cleanup
if [[ "$VOICE_CHAT" == "y" || "$VOICE_CHAT" == "Y" ]]; then
    echo "Shutting down Dora..."
    dora destroy
fi
```

### User Instructions (README.txt)

Create `README.txt`:

```
MoFA Studio v0.1.0 (Beta)
=========================

System Requirements
-------------------
- Windows 10/11 (64-bit) or macOS 12+ or Linux (Ubuntu 22.04+)
- 8GB+ RAM
- 5GB+ disk space (including models)
- Internet connection required (first-time setup)

Installation Steps
------------------
1. Extract to any directory (avoid paths with spaces or special characters)

2. Run the install script:
   - Windows: Right-click install.ps1 -> Run with PowerShell
   - macOS/Linux: Run ./install.sh in terminal

3. Wait for installation to complete (first run takes ~10-20 minutes)

Launching the Application
-------------------------
- Windows: Double-click start.bat
- macOS/Linux: Run ./start.sh in terminal

UI-only mode (without voice chat):
- Windows: Double-click mofa-studio.exe directly
- macOS/Linux: Run ./mofa-studio directly

Directory Structure
-------------------
mofa-studio.exe    - Main executable
bin/               - Dora node executables
node-hub/          - Python node source code
apps/              - Dataflow configurations
models/            - Model files (generated after install)
.pixi/             - Python environment (generated after install)

Troubleshooting
---------------
Q: "pixi not found" error
A: Reopen terminal, or manually add ~/.pixi/bin to PATH

Q: "dora not found" error
A: Ensure Rust is installed, then run: cargo install dora-cli --version 0.3.12

Q: Black screen on startup
A: Check that graphics drivers are up to date. Makepad requires OpenGL 3.3+

Q: Voice chat not responding
A: Check if Dora is running: dora list

Support
-------
GitHub: https://github.com/xxx/mofa-studio/issues
```

---

## One-Click Packaging Script

Create `scripts/package.ps1` (Windows):

```powershell
# MoFA Studio Packaging Script
param(
    [string]$Version = "0.1.0",
    [string]$Platform = "win64"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$DistName = "mofa-studio-v$Version-$Platform"
$DistDir = "$ProjectRoot\dist\$DistName"

Write-Host "Packaging MoFA Studio v$Version ($Platform)" -ForegroundColor Cyan

# Clean
if (Test-Path $DistDir) { Remove-Item -Recurse -Force $DistDir }
New-Item -ItemType Directory -Path $DistDir | Out-Null

# Build
Write-Host "Building Release version..." -ForegroundColor Yellow
Push-Location $ProjectRoot
cargo build --release -p mofa-studio-shell
cargo build --release -p dora-maas-client
cargo build --release -p dora-conference-bridge
cargo build --release -p dora-conference-controller
Pop-Location

# Copy files
Write-Host "Copying files..." -ForegroundColor Yellow
Copy-Item "$ProjectRoot\target\release\mofa-studio.exe" $DistDir

New-Item -ItemType Directory -Path "$DistDir\bin" | Out-Null
Copy-Item "$ProjectRoot\target\release\dora-maas-client.exe" "$DistDir\bin\"
Copy-Item "$ProjectRoot\target\release\dora-conference-bridge.exe" "$DistDir\bin\"
Copy-Item "$ProjectRoot\target\release\dora-conference-controller.exe" "$DistDir\bin\"

Copy-Item "$ProjectRoot\pixi.toml" $DistDir
Copy-Item "$ProjectRoot\pixi.lock" $DistDir

Copy-Item -Recurse "$ProjectRoot\node-hub" $DistDir
Copy-Item -Recurse "$ProjectRoot\libs" $DistDir

New-Item -ItemType Directory -Path "$DistDir\apps\mofa-fm\dataflow" -Force | Out-Null
Copy-Item "$ProjectRoot\apps\mofa-fm\dataflow\voice-chat.yml" "$DistDir\apps\mofa-fm\dataflow\"

New-Item -ItemType Directory -Path "$DistDir\models" | Out-Null
Copy-Item -Recurse "$ProjectRoot\models\setup-local-models" "$DistDir\models\"

# Copy scripts (these need to be created first)
# Copy-Item "$ProjectRoot\scripts\install.ps1" $DistDir
# Copy-Item "$ProjectRoot\scripts\start.bat" $DistDir
# Copy-Item "$ProjectRoot\scripts\README.txt" $DistDir

# Create archive
Write-Host "Creating archive..." -ForegroundColor Yellow
$ZipPath = "$ProjectRoot\dist\$DistName.zip"
if (Test-Path $ZipPath) { Remove-Item $ZipPath }
Compress-Archive -Path $DistDir -DestinationPath $ZipPath

Write-Host "Complete: $ZipPath" -ForegroundColor Green
Write-Host "Size: $([math]::Round((Get-Item $ZipPath).Length / 1MB, 2)) MB" -ForegroundColor Green
```

---

## User Installation Guide

### Minimal Installation (UI Only)

Users only need to:
1. Download and extract
2. Double-click `mofa-studio.exe`

No dependencies required - the UI runs standalone.

### Full Installation (with Voice Chat)

Users need to:
1. Download and extract
2. Run `install.ps1` or `install.sh`
3. Wait for environment setup to complete
4. Run `start.bat` or `start.sh`

**Prerequisites**:
- Windows: PowerShell 5.1+ (included with system)
- macOS/Linux: bash, curl
- Optional: Rust toolchain (if dora-cli needs to be installed)

---

## FAQ

### Q: How to reduce distribution package size?

1. **Exclude unnecessary nodes**:
   ```bash
   # Only copy required nodes
   cp -r node-hub/dora-asr dist/node-hub/
   cp -r node-hub/dora-kokoro-tts dist/node-hub/
   ```

2. **Compress executables with UPX**:
   ```bash
   upx --best mofa-studio.exe
   ```

3. **Don't include models** - let users download on demand

### Q: How to support offline installation?

1. Pre-export pixi environment:
   ```bash
   pixi install
   # Package the .pixi directory
   ```

2. Provide offline model download package

### Q: How to sign executables?

Windows:
```powershell
signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com mofa-studio.exe
```

macOS:
```bash
codesign --sign "Developer ID Application: Your Name" mofa-studio
```

---

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Run full test suite
- [ ] Build Release version
- [ ] Test install scripts in clean environment
- [ ] Verify all features work correctly
- [ ] Create Git tag
- [ ] Upload distribution package
- [ ] Update release notes
