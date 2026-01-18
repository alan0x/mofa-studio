# Fix NumPy Version
# Downgrade NumPy to 1.26.4 (required for PyArrow compatibility)

Write-Host "=== Fixing NumPy Version ===" -ForegroundColor Cyan
Write-Host ""

# Check conda environment
if ($env:CONDA_DEFAULT_ENV -ne "mofa-studio") {
    Write-Host "[ERROR] Please activate conda environment first:" -ForegroundColor Red
    Write-Host "  conda activate mofa-studio" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Conda environment: mofa-studio" -ForegroundColor Green
Write-Host ""

# Check current NumPy version
Write-Host "[CHECK] Current NumPy version:" -ForegroundColor Yellow
conda run -n mofa-studio python -c "import numpy; print(f'NumPy {numpy.__version__}')"
Write-Host ""

# Downgrade NumPy
Write-Host "[FIX] Downgrading NumPy to 1.26.4..." -ForegroundColor Cyan
Write-Host "This is required for PyArrow compatibility" -ForegroundColor Yellow
Write-Host ""

conda run -n mofa-studio pip install "numpy==1.26.4" --force-reinstall

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[OK] NumPy downgraded successfully" -ForegroundColor Green
    Write-Host ""

    # Verify
    Write-Host "[VERIFY] New NumPy version:" -ForegroundColor Yellow
    conda run -n mofa-studio python -c "import numpy; print(f'NumPy {numpy.__version__}')"
    Write-Host ""

    # Test PyArrow import
    Write-Host "[TEST] Testing PyArrow import..." -ForegroundColor Yellow
    $testResult = conda run -n mofa-studio python -c "import pyarrow; print('PyArrow OK')" 2>&1
    if ($testResult -match "PyArrow OK") {
        Write-Host "[OK] PyArrow can now import successfully" -ForegroundColor Green
    } else {
        Write-Host "[WARN] PyArrow import may still have issues:" -ForegroundColor Yellow
        Write-Host $testResult
    }

    Write-Host ""
    Write-Host "=== Fix Complete ===" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Run: .\start-simple.ps1" -ForegroundColor Yellow
    Write-Host "  2. In the app, click Start" -ForegroundColor Yellow
    Write-Host "  3. Wait for 'Dataflow Connected'" -ForegroundColor Yellow
    Write-Host "  4. Send a test message" -ForegroundColor Yellow

} else {
    Write-Host ""
    Write-Host "[ERROR] Failed to downgrade NumPy" -ForegroundColor Red
    Write-Host "Try manually: pip install numpy==1.26.4 --force-reinstall" -ForegroundColor Yellow
    exit 1
}
