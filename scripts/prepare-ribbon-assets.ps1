[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$projectRoot = Split-Path -Parent $PSScriptRoot
$sourceRoot = Join-Path $projectRoot 'assets\ribbon-bar'
$outputRoot = Join-Path $projectRoot 'assets\generated\ribbon-bar'
New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null

Get-ChildItem -LiteralPath $sourceRoot -Filter '*.png' | ForEach-Object {
    $source = [System.Drawing.Image]::FromFile($_.FullName)
    try {
        $bitmap = New-Object System.Drawing.Bitmap 64, 64
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
                $graphics.DrawImage($source, 0, 0, 64, 64)
            }
            finally { $graphics.Dispose() }
            $bitmap.Save((Join-Path $outputRoot $_.Name), [System.Drawing.Imaging.ImageFormat]::Png)
        }
        finally { $bitmap.Dispose() }
    }
    finally { $source.Dispose() }
}

Write-Host "Prepared ribbon icons in $outputRoot"
