[CmdletBinding()]
param(
    [string]$SourceDirectory,
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
if (-not $SourceDirectory) { $SourceDirectory = Join-Path $PSScriptRoot '..\assets\official-blizzard' }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $PSScriptRoot '..\assets\generated' }
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

function Export-TrimmedLogo {
    param([string]$InputPath, [string]$OutputPath, [int]$MaxWidth = 480, [int]$MaxHeight = 180)

    $source = [System.Drawing.Bitmap]::new($InputPath)
    try {
        $rect = [System.Drawing.Rectangle]::new(0, 0, $source.Width, $source.Height)
        $data = $source.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            $stride = [Math]::Abs($data.Stride)
            $bytes = [byte[]]::new($stride * $source.Height)
            [Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
            $left = $source.Width; $top = $source.Height; $right = -1; $bottom = -1
            for ($y = 0; $y -lt $source.Height; $y++) {
                for ($x = 0; $x -lt $source.Width; $x++) {
                    if ($bytes[$y * $stride + $x * 4 + 3] -gt 8) {
                        if ($x -lt $left) { $left = $x }; if ($x -gt $right) { $right = $x }
                        if ($y -lt $top) { $top = $y }; if ($y -gt $bottom) { $bottom = $y }
                    }
                }
            }
        }
        finally { $source.UnlockBits($data) }

        if ($right -lt $left -or $bottom -lt $top) { throw "No visible pixels in $InputPath" }
        $crop = [System.Drawing.Rectangle]::new($left, $top, $right - $left + 1, $bottom - $top + 1)
        $scale = [Math]::Min($MaxWidth / $crop.Width, $MaxHeight / $crop.Height)
        $width = [Math]::Max(1, [int][Math]::Round($crop.Width * $scale))
        $height = [Math]::Max(1, [int][Math]::Round($crop.Height * $scale))
        $output = [System.Drawing.Bitmap]::new($width, $height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($output)
            try {
                $graphics.Clear([System.Drawing.Color]::Transparent)
                $graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
                $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.DrawImage($source, [System.Drawing.Rectangle]::new(0, 0, $width, $height), $crop, [System.Drawing.GraphicsUnit]::Pixel)
            }
            finally { $graphics.Dispose() }
            $output.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        finally { $output.Dispose() }
        Write-Host "Created $OutputPath ($width x $height)"
    }
    finally { $source.Dispose() }
}

Export-TrimmedLogo (Join-Path $SourceDirectory 'WoW_Midnight_Eclipse_Logo.png') (Join-Path $OutputDirectory 'midnight-logo.png')
Export-TrimmedLogo (Join-Path $SourceDirectory 'WoW_Forever_Logo.png') (Join-Path $OutputDirectory 'forever-logo.png')
