# Fix PyArrow - Reinstall to work with NumPy 1.26.4

Write-Host "=== Fixing PyArrow ===" -ForegroundColor Cyan
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

if ($numpyVersion -notmatch "1\.26\.4") {
    Write-Host "[ERROR] NumPy is not 1.26.4. Run .\fix-numpy.ps1 first" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Uninstall PyArrow completely
Write-Host "[1/3] Uninstalling PyArrow..." -ForegroundColor Cyan
conda run -n mofa-studio pip uninstall -y pyarrow 2>&1 | Out-Null
Write-Host "[OK] PyArrow uninstalled" -ForegroundColor Green
Write-Host ""

# Clear pip cache
Write-Host "[2/3] Clearing pip cache..." -ForegroundColor Cyan
conda run -n mofa-studio pip cache purge 2>&1 | Out-Null
Write-Host "[OK] Cache cleared" -ForegroundColor Green
Write-Host ""

# Reinstall PyArrow (compatible version for NumPy 1.26.4)
Write-Host "[3/3] Reinstalling PyArrow..." -ForegroundColor Cyan
Write-Host "Installing PyArrow 10.0.1 (compatible with NumPy 1.26.4)..." -ForegroundColor Yellow
conda run -n mofa-studio pip install "pyarrow==10.0.1" --no-cache-dir

if ($LASTEXITCODE -eq 0) {
    Write-Host "[OK] PyArrow reinstalled" -ForegroundColor Green
    Write-Host ""

    # Test import
    Write-Host "[TEST] Testing PyArrow import..." -ForegroundColor Yellow
    $testResult = conda run -n mofa-studio python -c "import pyarrow; print(f'PyArrow {pyarrow.__version__} OK')" 2>&1

    if ($testResult -match "OK") {
        Write-Host "[SUCCESS] $testResult" -ForegroundColor Green
        Write-Host ""
        Write-Host "=== Fix Complete ===" -ForegroundColor Green
        Write-Host ""
        Write-Host "PyArrow is now compatible with NumPy 1.26.4" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Cyan
        Write-Host "  1. Run: .\start-simple.ps1" -ForegroundColor Yellow
        Write-Host "  2. In the app, click Start" -ForegroundColor Yellow
        Write-Host "  3. All Python nodes should now work!" -ForegroundColor Yellow
    } else {
        Write-Host "[ERROR] PyArrow import still fails:" -ForegroundColor Red
        Write-Host $testResult -ForegroundColor Red
        Write-Host ""
        Write-Host "Trying alternative fix..." -ForegroundColor Yellow

        # Try different PyArrow version
        conda run -n mofa-studio pip uninstall -y pyarrow 2>&1 | Out-Null
        conda run -n mofa-studio pip install "pyarrow>=10.0.0,<11.0.0" --no-cache-dir

        $testResult2 = conda run -n mofa-studio python -c "import pyarrow; print(f'PyArrow {pyarrow.__version__} OK')" 2>&1
        if ($testResult2 -match "OK") {
            Write-Host "[SUCCESS] $testResult2" -ForegroundColor Green
        } else {
            Write-Host "[ERROR] Still failing. Manual intervention needed." -ForegroundColor Red
            Write-Host $testResult2 -ForegroundColor Red
        }
    }
} else {
    Write-Host "[ERROR] Failed to install PyArrow" -ForegroundColor Red
    exit 1
}
