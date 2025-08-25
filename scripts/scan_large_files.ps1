Param(
    [int]$ThresholdMB = 1
)

$ErrorActionPreference = 'Stop'
$thresholdBytes = $ThresholdMB * 1MB

Write-Host "Scanning tracked files larger than $ThresholdMB MB..." -ForegroundColor Cyan

$large = @()
git ls-files | ForEach-Object {
    if (Test-Path $_) {
        $item = Get-Item $_
        if ($item.Length -gt $thresholdBytes) {
            $large += [pscustomobject]@{
                SizeMB = [math]::Round($item.Length / 1MB, 2)
                Path   = $_
            }
        }
    }
}

if ($large.Count -eq 0) {
    Write-Host "No tracked files exceed $ThresholdMB MB." -ForegroundColor Green
    exit 0
}

$large | Sort-Object SizeMB -Descending | Format-Table -AutoSize
Write-Host "\nReview large files. Consider Git LFS or refactoring to reduce size." -ForegroundColor Yellow
exit 1
