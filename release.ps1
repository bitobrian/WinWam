[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$projectRoot = $PSScriptRoot
$destination = 'C:\apps\WinWam'
$executable = Join-Path $projectRoot 'target\release\winwam.exe'

Write-Host 'Building WinWam in release mode...'
Push-Location $projectRoot
try {
    cargo build --locked --release
    if ($LASTEXITCODE -ne 0) {
        throw "Release build failed with exit code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $destination -PathType Container)) {
    Write-Host "Creating $destination..."
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
}

Copy-Item -LiteralPath $executable -Destination (Join-Path $destination 'WinWam.exe') -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'README.md') -Destination $destination -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'LICENSE') -Destination $destination -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'PRIVACY.md') -Destination $destination -Force

Write-Host "WinWam release copied to $destination"
