[CmdletBinding()]
param(
    [string]$Source,
    [string]$Destination
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
if (-not $Source) { $Source = Join-Path $projectRoot 'assets\icon.png' }
if (-not $Destination) { $Destination = Join-Path $projectRoot 'assets\icon.ico' }
Add-Type -AssemblyName System.Drawing

$sourceImage = [System.Drawing.Image]::FromFile($Source)
$streams = [System.Collections.Generic.List[System.IO.MemoryStream]]::new()
try {
    foreach ($size in @(16, 24, 32, 48, 64, 128, 256)) {
        $bitmap = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        try {
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $graphics.Clear([System.Drawing.Color]::Transparent)
                $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
                $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
                $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
                $graphics.DrawImage($sourceImage, 0, 0, $size, $size)
            }
            finally { $graphics.Dispose() }

            $stream = [System.IO.MemoryStream]::new()
            $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
            $streams.Add($stream)
        }
        finally { $bitmap.Dispose() }
    }

    $file = [System.IO.File]::Create($Destination)
    $writer = [System.IO.BinaryWriter]::new($file)
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]$streams.Count)
        $offset = 6 + 16 * $streams.Count
        for ($index = 0; $index -lt $streams.Count; $index++) {
            $size = @(16, 24, 32, 48, 64, 128, 256)[$index]
            $writer.Write([byte]($(if ($size -eq 256) { 0 } else { $size })))
            $writer.Write([byte]($(if ($size -eq 256) { 0 } else { $size })))
            $writer.Write([byte]0)
            $writer.Write([byte]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$streams[$index].Length)
            $writer.Write([uint32]$offset)
            $offset += $streams[$index].Length
        }
        foreach ($stream in $streams) { $writer.Write($stream.ToArray()) }
    }
    finally {
        $writer.Dispose()
        $file.Dispose()
    }
}
finally {
    foreach ($stream in $streams) { $stream.Dispose() }
    $sourceImage.Dispose()
}

Write-Host "Created multi-resolution Windows icon: $Destination"
