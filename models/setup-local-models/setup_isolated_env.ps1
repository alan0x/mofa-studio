# MoFA Studio - Isolated Environment Setup (PowerShell)
# Creates a fresh Python environment with all required Dora nodes
# Uses standardized dependency versions to avoid conflicts
# See DEPENDENCIES.md for detailed dependency specifications

#Requires -Version 5.1

# Set error action preference
$ErrorActionPreference = "Stop"

# Configuration
$ENV_NAME = "mofa-studio"
$PYTHON_VERSION = "3.12"
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$PROJECT_ROOT = Join-Path $SCRIPT_DIR "..\..\"
$NODE_HUB_DIR = Join-Path $PROJECT_ROOT "node-hub"

# Helper Functions
function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Blue
    Write-Host $Message -ForegroundColor Blue
    Write-Host "============================================" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

# Check prerequisites
function Test-Prerequisites {
    Write-Header "Checking Prerequisites"
    
    # Check conda
    try {
        $condaVersion = & conda --version 2>&1
        Write-Success "Conda found: $condaVersion"
    } catch {
        Write-Error-Custom "Conda not found. Please install Miniconda or Anaconda"
        Write-Host ""
        Write-Host "============================================"
        Write-Host "CONDA INSTALLATION INSTRUCTIONS"
        Write-Host "============================================"
        Write-Host ""
        Write-Host "OPTION A: Install Miniconda (RECOMMENDED - lightweight)"
        Write-Host "Download from: https://repo.anaconda.com/miniconda/Miniconda3-latest-Windows-x86_64.exe"
        Write-Host ""
        Write-Host "After installation, restart PowerShell and run this script again."
        Write-Host "============================================"
        exit 1
    }
    
    # Check git
    try {
        $gitVersion = & git --version 2>&1
        Write-Success "Git found: $gitVersion"
    } catch {
        Write-Error-Custom "Git not found. Please install git from https://git-scm.com/download/win"
        exit 1
    }
    
    # Check cargo (optional, for Rust nodes)
    try {
        $cargoVersion = & cargo --version 2>&1
        Write-Success "Cargo found: $cargoVersion"
    } catch {
        Write-Warning-Custom "Cargo not found. Rust nodes will not be built"
        Write-Info "Install from: https://rustup.rs/"
    }
}

# Create conda environment
function New-CondaEnvironment {
    Write-Header "Creating Conda Environment: $ENV_NAME"
    
    # Check if environment already exists
    $envList = & conda env list 2>&1 | Out-String
    if ($envList -match $ENV_NAME) {
        Write-Warning-Custom "Environment '$ENV_NAME' already exists"
        $response = Read-Host "Do you want to remove and recreate it? (y/n)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Write-Info "Removing existing environment..."
            & conda env remove -n $ENV_NAME -y
        } else {
            Write-Info "Using existing environment"
            return
        }
    }
    
    Write-Info "Creating new conda environment with Python $PYTHON_VERSION..."
    & conda create -n $ENV_NAME python=$PYTHON_VERSION -y
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create conda environment"
    }
    Write-Success "Environment created successfully"
}

# Setup conda initialization
function Initialize-CondaSetup {
    Write-Header "Setting up Conda Initialization"
    
    try {
        $condaBasePath = & conda info --base 2>&1
        if ($condaBasePath) {
            Write-Info "Conda base path: $condaBasePath"
            Write-Success "Conda is ready"
        }
    } catch {
        Write-Warning-Custom "Could not get conda info"
    }
}

# Activate environment and install dependencies
function Install-Dependencies {
    Write-Header "Installing Dependencies"
    
    Write-Info "Using conda environment: $ENV_NAME"
    
    try {
        $pythonExe = & conda run -n $ENV_NAME python -c "import sys; print(sys.executable)" 2>&1
        Write-Info "Active Python: $pythonExe"
        $pythonVersion = & conda run -n $ENV_NAME python --version 2>&1
        Write-Info "Python version: $pythonVersion"
    } catch {
        Write-Warning-Custom "Could not get Python info"
    }
    
    # Upgrade pip
    Write-Info "Upgrading pip..."
    & conda run -n $ENV_NAME python -m pip install --upgrade pip
    
    # Install critical dependencies with specific versions
    Write-Info "Installing core dependencies..."
    & conda run -n $ENV_NAME pip install numpy==1.26.4
    & conda run -n $ENV_NAME pip install torch==2.2.0 torchvision==0.17.0 torchaudio==2.2.0 --index-url https://download.pytorch.org/whl/cpu
    
    # Install transformers and related packages
    Write-Info "Installing ML libraries..."
    & conda run -n $ENV_NAME pip install transformers==4.45.0
    & conda run -n $ENV_NAME pip install huggingface-hub==0.34.4
    & conda run -n $ENV_NAME pip install datasets accelerate sentencepiece protobuf
    
    # Install dora-rs
    Write-Info "Installing dora-rs..."
    & conda run -n $ENV_NAME pip install dora-rs==0.3.12
    
    # Install other dependencies
    Write-Info "Installing additional dependencies..."
    & conda run -n $ENV_NAME pip install pyarrow scipy librosa soundfile webrtcvad
    & conda run -n $ENV_NAME pip install openai websockets aiohttp requests
    & conda run -n $ENV_NAME pip install pyyaml toml python-dotenv
    & conda run -n $ENV_NAME pip install sounddevice
    & conda run -n $ENV_NAME pip install nltk
    
    # Install llama-cpp-python from conda-forge (avoids build issues)
    Write-Info "Installing llama-cpp-python from conda-forge..."
    & conda install -n $ENV_NAME -c conda-forge llama-cpp-python -y

    # Install TTS backends
    Write-Info "Installing TTS backends..."
    & conda run -n $ENV_NAME pip install kokoro

    Write-Info "Using CPU backend for TTS (Windows platform)"

    # Download NLTK data for TTS text processing
    Write-Info "Downloading NLTK data for text processing..."
    & conda run -n $ENV_NAME python -c "import nltk; nltk.download('averaged_perceptron_tagger_eng', quiet=True); nltk.download('averaged_perceptron_tagger', quiet=True); nltk.download('cmudict', quiet=True)"
    Write-Success "NLTK data downloaded"

    Write-Success "Core dependencies installed"
}

# Install and check dora CLI
function Install-DoraCli {
    Write-Header "Installing Dora CLI"
    
    # Check if cargo is available
    try {
        $null = & cargo --version 2>&1
        Write-Info "Installing dora-cli v0.3.12 via cargo..."
        & cargo install dora-cli --version 0.3.12 --locked
        
        # Check if installation was successful
        $doraPath = Join-Path $env:USERPROFILE ".cargo\bin\dora.exe"
        if (Test-Path $doraPath) {
            Write-Success "Dora CLI installed"
        } else {
            Write-Warning-Custom "Dora CLI installation failed"
        }
    } catch {
        Write-Warning-Custom "Cargo not found. Cannot install dora-cli via cargo."
        Write-Info "Install Rust from https://rustup.rs/ to get the latest dora-cli"
        Write-Info "Using dora from pip installation instead"
    }
}

# Install Dora nodes
function Install-DoraNodes {
    Write-Header "Installing Dora Nodes"
    
    # List of Python nodes to install
    $NODES = @(
        "dora-asr",
        "dora-primespeech",
        "dora-kokoro-tts",
        "dora-qwen3",
        "dora-text-segmenter",
        "dora-speechmonitor"
    )
    
    foreach ($node in $NODES) {
        $nodePath = Join-Path $NODE_HUB_DIR $node
        if (Test-Path $nodePath) {
            Write-Info "Installing $node..."
            & conda run -n $ENV_NAME pip install -e $nodePath
            Write-Success "$node installed"
        } else {
            Write-Warning-Custom "$node not found at $nodePath"
        }
    }
    
    # Build Rust nodes if cargo is available
    try {
        $null = & cargo --version 2>&1
        Write-Info "Building Rust nodes..."
        
        # Build dora-maas-client
        $maasPath = Join-Path $NODE_HUB_DIR "dora-maas-client"
        if (Test-Path $maasPath) {
            Write-Info "Building dora-maas-client..."
            Push-Location $maasPath
            & cargo build --release
            Pop-Location
            Write-Success "dora-maas-client built"
        }
        
        # Build dora-openai-websocket
        $websocketPath = Join-Path $NODE_HUB_DIR "dora-openai-websocket"
        if (Test-Path $websocketPath) {
            Write-Info "Building dora-openai-websocket..."
            Push-Location $websocketPath
            & cargo build --release -p dora-openai-websocket
            Pop-Location
            Write-Success "dora-openai-websocket built"
        }
    } catch {
        Write-Warning-Custom "Skipping Rust node builds (cargo not found)"
    }
}

# Fix numpy compatibility
function Repair-NumpyCompatibility {
    Write-Header "Fixing NumPy Compatibility"
    
    Write-Info "Ensuring numpy 1.26.4 is installed..."
    & conda run -n $ENV_NAME pip install numpy==1.26.4 --force-reinstall
    
    Write-Success "NumPy compatibility fixed"
}

# Print summary
function Show-Summary {
    Write-Header "Setup Complete!"

    Write-Host ""
    Write-Host "Environment Name: $ENV_NAME"
    Write-Host "Python Version: $PYTHON_VERSION"
    Write-Host ""

    Write-Host "TTS Backends Installed:"
    Write-Host "  CPU (kokoro) - Cross-platform"
    Write-Host ""

    Write-Host "To activate the environment:"
    Write-Host "  conda activate $ENV_NAME"
    Write-Host ""
    Write-Host "To run examples:"
    Write-Host "  dora up"
    Write-Host "  dora start voice-chat-with-aec.yml"
    Write-Host ""
    Write-Success "Setup completed successfully!"
}

# Main execution
function Main {
    Write-Header "Dora Voice Chat - Isolated Environment Setup"
    
    try {
        Test-Prerequisites
        Initialize-CondaSetup
        New-CondaEnvironment
        Install-Dependencies
        Install-DoraCli
        Install-DoraNodes
        Repair-NumpyCompatibility
        
        Show-Summary
    } catch {
        $errMsg = $_.Exception.Message
        Write-Error-Custom "Setup failed: $errMsg"
        exit 1
    }
}

# Run main function
Main
