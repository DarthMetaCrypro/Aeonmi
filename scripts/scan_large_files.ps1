Param(
    [int]$ThresholdMB = 1,
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$thresholdBytes = $ThresholdMB * 1MB

if (-not $Quiet) { Write-Host "Scanning tracked files larger than $ThresholdMB MB..." -ForegroundColor Cyan }

$large = @()
git ls-files | ForEach-Object {
    $path = $_
    try {
        if (Test-Path -LiteralPath $path) {
            $item = Get-Item -LiteralPath $path -ErrorAction Stop
            if ($item.Length -gt $thresholdBytes) {
                $large += [pscustomobject]@{
                    SizeMB = [math]::Round($item.Length / 1MB, 2)
                    Path   = $path
                }
            }
        }
    }
    catch {
        if (-not $Quiet) { Write-Verbose "Skipped invalid path entry: $path" }
    }
}

if ($large.Count -eq 0) {
    if (-not $Quiet) { Write-Host "No tracked files exceed $ThresholdMB MB." -ForegroundColor Green }
    exit 0
}

$large | Sort-Object SizeMB -Descending | Format-Table -AutoSize
Write-Host "\nReview large files. Consider Git LFS or refactoring to reduce size." -ForegroundColor Yellow
exit 1

