Param(
    [int]$ThresholdMB = 1,
    [int]$Top = 50
)

$ErrorActionPreference = 'Stop'
$thresholdBytes = $ThresholdMB * 1MB

Write-Host "Scanning repository history for blobs larger than $ThresholdMB MB..." -ForegroundColor Cyan

# Use batch to improve performance
$tmp = New-TemporaryFile
try {
    # Collect all object ids with paths
    git rev-list --objects --all > $tmp
    $results = @()
    Get-Content $tmp | ForEach-Object {
        $line = $_
        if (-not $line) { return }
        $first, $rest = $line -split ' ', 2
        if (-not $rest) { return }
        $type = git cat-file -t $first 2>$null
        if ($type -ne 'blob') { return }
        $size = git cat-file -s $first 2>$null
        if ($size -and [int]$size -gt $thresholdBytes) {
            $results += [pscustomobject]@{
                SizeMB = [math]::Round($size / 1MB, 2)
                Blob   = $first
                Path   = $rest
            }
        }
    }
    if ($results.Count -eq 0) {
        Write-Host "No historical blobs exceed $ThresholdMB MB." -ForegroundColor Green
        exit 0
    }
    $results | Sort-Object SizeMB -Descending | Select-Object -First $Top | Format-Table -AutoSize
    Write-Host "\nIf unintended, consider: git filter-repo --invert-paths --path <path> (after backup & coordination)." -ForegroundColor Yellow
    exit 1
}
finally {
    Remove-Item $tmp -ErrorAction SilentlyContinue
}
