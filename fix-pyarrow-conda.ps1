# Fix PyArrow using Conda (precompiled binaries)

Write-Host "=== Fixing PyArrow with Conda ===" -ForegroundColor Cyan
Write-Host ""

# Check conda environment
if ($env:CONDA_DEFAULT_ENV -ne "mofa-studio") {
    Write-Host "[ERROR] Please activate conda environment first:" -ForegroundColor Red
    Write-Host "  conda activate mofa-studio" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Conda environment: mofa-studio" -ForegroundColor Green
Write-Host ""

# Verify NumPy version
Write-Host "[CHECK] NumPy version:" -ForegroundColor Yellow
$numpyVersion = conda run -n mofa-studio python -c "import numpy; print(numpy.__version__)" 2>&1
Write-Host "  $numpyVersion" -ForegroundColor Cyan
Write-Host ""

# Uninstall pip-installed PyArrow
Write-Host "[1/3] Removing pip-installed PyArrow..." -ForegroundColor Cyan
conda run -n mofa-studio pip uninstall -y pyarrow 2>&1 | Out-Null
Write-Host "[OK] Removed" -ForegroundColor Green
Write-Host ""

# Install PyArrow from conda-forge (precompiled)
Write-Host "[2/3] Installing PyArrow from conda-forge..." -ForegroundColor Cyan
Write-Host "This will use precompiled binaries (no compilation needed)" -ForegroundColor Yellow
Write-Host ""

conda install -n mofa-studio -c conda-forge "pyarrow>=10.0.0,<17.0.0" "numpy=1.26.4" -y

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[OK] PyArrow installed from conda-forge" -ForegroundColor Green
    Write-Host ""

    # Test import
    Write-Host "[3/3] Testing PyArrow import..." -ForegroundColor Yellow
    $testResult = conda run -n mofa-studio python -c "import pyarrow; import numpy; print(f'NumPy {numpy.__version__}, PyArrow {pyarrow.__version__} - OK')" 2>&1

    if ($testResult -match "OK") {
        Write-Host "[SUCCESS] $testResult" -ForegroundColor Green
        Write-Host ""
        Write-Host "=== Fix Complete ===" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Cyan
        Write-Host "  1. Run: .\start-simple.ps1" -ForegroundColor Yellow
        Write-Host "  2. Click Start in the app" -ForegroundColor Yellow
        Write-Host "  3. Test sending a message" -ForegroundColor Yellow
    } else {
        Write-Host "[ERROR] Import still fails:" -ForegroundColor Red
        Write-Host $testResult -ForegroundColor Red
    }
} else {
    Write-Host ""
    Write-Host "[ERROR] Failed to install PyArrow from conda" -ForegroundColor Red
    exit 1
}
