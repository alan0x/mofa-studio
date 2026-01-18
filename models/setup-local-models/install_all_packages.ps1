# Install All Packages Script for MoFA Studio (Windows PowerShell)
# This script reinstalls required Python packages and builds Rust components
# Use after the conda environment (mofa-studio) already exists.

#Requires -Version 5.1

$ErrorActionPreference = "Stop"

# Configuration
$ENV_NAME = "mofa-studio"
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$PROJECT_ROOT = (Get-Item (Join-Path $SCRIPT_DIR "..\..")).FullName

# Helper Functions
function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "=======================================================" -ForegroundColor Blue
    Write-Host "   $Message" -ForegroundColor Blue
    Write-Host "=======================================================" -ForegroundColor Blue
    Write-Host ""
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check conda environment exists
Write-Header "Checking Conda Environment"

$envList = & conda env list 2>&1 | Out-String
if ($envList -match $ENV_NAME) {
    Write-Success "Conda environment '$ENV_NAME' found"
} else {
    Write-Error-Custom "Conda environment '$ENV_NAME' not found. Please run setup_isolated_env.ps1 first."
    exit 1
}

# Test environment works
try {
    $pythonVersion = & conda run -n $ENV_NAME python --version 2>&1
    Write-Success "Python version: $pythonVersion"
} catch {
    Write-Error-Custom "Failed to access conda environment"
    exit 1
}

Write-Info "Project root: $PROJECT_ROOT"

# Install all Dora packages in editable mode
Write-Header "Installing Dora Python Packages"

Set-Location $PROJECT_ROOT

# Pin dora-rs version to match dora-cli
Write-Info "Pinning dora-rs version to 0.3.12..."
& conda run -n $ENV_NAME pip install dora-rs==0.3.12
Write-Success "dora-rs pinned successfully"

# Install dora-common
$doraCommonPath = Join-Path $PROJECT_ROOT "libs\dora-common"
if (Test-Path $doraCommonPath) {
    Write-Info "Installing dora-common (shared library)..."
    & conda run -n $ENV_NAME pip install -e $doraCommonPath
    Write-Success "dora-common installed"
} else {
    Write-Info "dora-common not found, skipping..."
}

# Install dora-primespeech
$primespeechPath = Join-Path $PROJECT_ROOT "node-hub\dora-primespeech"
if (Test-Path $primespeechPath) {
    Write-Info "Installing dora-primespeech..."
    & conda run -n $ENV_NAME pip install -e $primespeechPath
    Write-Success "dora-primespeech installed"
} else {
    Write-Info "dora-primespeech not found, skipping..."
}

# Install dora-asr
$asrPath = Join-Path $PROJECT_ROOT "node-hub\dora-asr"
if (Test-Path $asrPath) {
    Write-Info "Installing dora-asr..."
    & conda run -n $ENV_NAME pip install -e $asrPath
    Write-Success "dora-asr installed"
} else {
    Write-Info "dora-asr not found, skipping..."
}

# Install dora-speechmonitor
$speechmonitorPath = Join-Path $PROJECT_ROOT "node-hub\dora-speechmonitor"
if (Test-Path $speechmonitorPath) {
    Write-Info "Installing dora-speechmonitor..."
    & conda run -n $ENV_NAME pip install -e $speechmonitorPath
    Write-Success "dora-speechmonitor installed"
} else {
    Write-Info "dora-speechmonitor not found, skipping..."
}

# Install dora-text-segmenter
$segmenterPath = Join-Path $PROJECT_ROOT "node-hub\dora-text-segmenter"
if (Test-Path $segmenterPath) {
    Write-Info "Installing dora-text-segmenter..."
    & conda run -n $ENV_NAME pip install -e $segmenterPath
    Write-Success "dora-text-segmenter installed"
} else {
    Write-Info "dora-text-segmenter not found, skipping..."
}

# Install dora-kokoro-tts
$kokoroPath = Join-Path $PROJECT_ROOT "node-hub\dora-kokoro-tts"
if (Test-Path $kokoroPath) {
    Write-Info "Installing dora-kokoro-tts..."
    & conda run -n $ENV_NAME pip install -e $kokoroPath
    Write-Success "dora-kokoro-tts installed"
} else {
    Write-Info "dora-kokoro-tts not found, skipping..."
}

# Check Rust installation
Write-Header "Checking Rust Installation"

$hasRust = $false
try {
    $rustVersion = & rustc --version 2>&1
    $cargoVersion = & cargo --version 2>&1
    Write-Success "Rust installed: $rustVersion"
    Write-Success "Cargo installed: $cargoVersion"
    $hasRust = $true
} catch {
    Write-Info "Rust not found. Please install from https://rustup.rs/"
    Write-Info "After installing Rust, restart PowerShell and run this script again to build Rust components."
}

if ($hasRust) {
    # Install Dora CLI
    Write-Header "Installing Dora CLI"
    
    $doraPath = Join-Path $env:USERPROFILE ".cargo\bin\dora.exe"
    if (Test-Path $doraPath) {
        $doraVersion = & $doraPath --version 2>&1
        Write-Info "Dora CLI already installed: $doraVersion"
        $response = Read-Host "Do you want to reinstall/update Dora CLI? (y/n)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Write-Info "Reinstalling Dora CLI..."
            & cargo install dora-cli --version 0.3.12 --locked --force
            Write-Success "Dora CLI updated"
        }
    } else {
        Write-Info "Installing Dora CLI..."
        & cargo install dora-cli --version 0.3.12 --locked
        Write-Success "Dora CLI installed"
    }
    
    # Build Rust-based nodes
    Write-Header "Building Rust Components"
    
    # Build dora-maas-client
    $maasCargoPath = Join-Path $PROJECT_ROOT "node-hub\dora-maas-client\Cargo.toml"
    if (Test-Path $maasCargoPath) {
        Write-Info "Building dora-maas-client..."
        & cargo build --release --manifest-path $maasCargoPath
        Write-Success "dora-maas-client built"
    }
    
    # Build dora-conference-bridge
    $bridgeCargoPath = Join-Path $PROJECT_ROOT "node-hub\dora-conference-bridge\Cargo.toml"
    if (Test-Path $bridgeCargoPath) {
        Write-Info "Building dora-conference-bridge..."
        & cargo build --release --manifest-path $bridgeCargoPath
        Write-Success "dora-conference-bridge built"
    }
    
    # Build dora-conference-controller
    $controllerCargoPath = Join-Path $PROJECT_ROOT "node-hub\dora-conference-controller\Cargo.toml"
    if (Test-Path $controllerCargoPath) {
        Write-Info "Building dora-conference-controller..."
        & cargo build --release --manifest-path $controllerCargoPath
        Write-Success "dora-conference-controller built"
    }
}

# Summary
Write-Header "Installation Complete!"

Write-Host "All packages have been successfully installed!" -ForegroundColor Green
Write-Host ""
Write-Host "Summary:"
Write-Host "  [OK] Python packages installed in editable mode" -ForegroundColor Green
if ($hasRust) {
    Write-Host "  [OK] Rust and Dora CLI installed" -ForegroundColor Green
    Write-Host "  [OK] Rust components built" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Rust not installed - Rust components skipped" -ForegroundColor Yellow
}
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Download models: cd models\model-manager; python download_models.py --download primespeech"
Write-Host "  2. Download additional models (funasr, kokoro, qwen) as needed"
Write-Host "  3. Configure any required API keys (e.g. OpenAI)"
Write-Host "  4. Run voice-chat examples"
Write-Host ""
Write-Success "Ready to use Dora Voice Chat!"
