# Build Aeonmi Windows release executable
# Usage: powershell -ExecutionPolicy Bypass -File .\build_windows.ps1

Write-Host "Building Aeonmi (release with Windows icon)..." -ForegroundColor Cyan
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) { Write-Error "Cargo (Rust) not found in PATH"; exit 1 }

# Optional features passthrough
param(
    [string]$Features = ""
)

$featureArg = if ($Features -and $Features.Trim() -ne "") { "--features $Features" } else { "" }

$cmd = "cargo build --release $featureArg"
Write-Host $cmd -ForegroundColor Yellow
Invoke-Expression $cmd
if ($LASTEXITCODE -ne 0) { Write-Error "Cargo build failed"; exit 2 }

$exe = Join-Path -Path (Resolve-Path .) -ChildPath "target/release/Aeonmi.exe"
if (Test-Path $exe) {
    Write-Host "Built: $exe" -ForegroundColor Green
    Write-Host "If the icon did not appear, ensure assets/icon.ico is a valid ICO (multiple resolutions)." -ForegroundColor DarkYellow
} else {
    Write-Warning "Executable not found where expected: $exe"
}
