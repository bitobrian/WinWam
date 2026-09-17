[CmdletBinding()]
param(
    [switch]$Reset,
    [switch]$WithInstalledAddons
)

$AddonCount = 500

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$harnessRoot = Join-Path $projectRoot 'local-test'
$wowRoot = Join-Path $harnessRoot 'World of Warcraft'
$directoryRoot = Join-Path $harnessRoot 'directory'

if ($Reset -and (Test-Path -LiteralPath $harnessRoot)) {
    Remove-Item -LiteralPath $harnessRoot -Recurse -Force
}

New-Item -ItemType Directory -Path $directoryRoot -Force | Out-Null

$flavors = @(
    @{ Slug = 'retail'; Folder = '_retail_'; Schema = 'Retail'; Interface = '110200' },
    @{ Slug = 'mop-classic'; Folder = '_classic_'; Schema = 'MoPClassic'; Interface = '50500' },
    @{ Slug = 'classic'; Folder = '_classic_era_'; Schema = 'Classic'; Interface = '11507' },
    @{ Slug = 'bc-anniversary'; Folder = '_classic_anniversary_'; Schema = 'BCAnniversary'; Interface = '20505' },
    @{ Slug = 'forever'; Folder = '_classic_era_'; Schema = 'Forever'; Interface = '11507' }
)

$addons = @(
    @{ Id = 'gratwurst'; Name = 'Gratwurst'; Folder = 'Gratwurst'; Category = 'QualityOfLife'; Tags = @('guild', 'achievements', 'chat'); Author = 'Bitobrian'; Owner = 'bitobrian'; Repo = 'Gratwurst'; Version = '1.10.0'; Homepage = 'https://github.com/bitobrian/Gratwurst'; Summary = 'A delicious automatic congratulations messaging addon for guild achievements.' },
    @{ Id = 'test-bag-manager'; Name = 'Test Bag Manager'; Folder = 'TestBagManager'; Category = 'Bags'; Tags = @('bags', 'inventory') },
    @{ Id = 'test-map-notes'; Name = 'Test Map Notes'; Folder = 'TestMapNotes'; Category = 'Interface'; Tags = @('map', 'notes') }
)

$categories = @('Combat', 'Bags', 'Interface', 'Raiding', 'Dungeons', 'Economy', 'Collections', 'Quests', 'Development', 'QualityOfLife')
$authors = @('Northwind Labs', 'Waypoint Works', 'Pixel Foundry', 'Copper Byte', 'Open UI Guild', 'Lua Workshop')
for ($number = 4; $number -le $AddonCount; $number++) {
    $padded = $number.ToString('000')
    $category = $categories[($number - 1) % $categories.Count]
    $addons += @{
        Id = "test-addon-$padded"
        Name = "Test Addon $padded"
        Folder = "TestAddon$padded"
        Category = $category
        Tags = @('synthetic', $category.ToLowerInvariant(), "fixture-$padded")
        Author = $authors[($number - 1) % $authors.Count]
    }
}

foreach ($flavor in $flavors) {
    $addonsRoot = Join-Path $wowRoot "$($flavor.Folder)\Interface\AddOns"
    New-Item -ItemType Directory -Path $addonsRoot -Force | Out-Null

    $manifestAddons = foreach ($addon in $addons) {
        if ($WithInstalledAddons -and $addons.IndexOf($addon) -lt 3) {
            $addonRoot = Join-Path $addonsRoot $addon.Folder
            New-Item -ItemType Directory -Path $addonRoot -Force | Out-Null
            @"
## Interface: $($flavor.Interface)
## Title: $($addon.Name)
## Notes: Synthetic addon created by the WinWam development harness.
## Author: WinWam Test Harness
## Version: 1.0.$($addons.IndexOf($addon))
$($addon.Folder).lua
"@ | Set-Content -LiteralPath (Join-Path $addonRoot "$($addon.Folder).toc") -Encoding utf8
            "-- Synthetic addon for local WinWam testing.`nprint('$($addon.Name) loaded')" |
                Set-Content -LiteralPath (Join-Path $addonRoot "$($addon.Folder).lua") -Encoding utf8
            $addon.Id | Set-Content -LiteralPath (Join-Path $addonRoot '.winwam-id') -Encoding ascii -NoNewline
        }

        [ordered]@{
            id = $addon.Id
            name = $addon.Name
            summary = if ($addon.Summary) { $addon.Summary } else { "Synthetic $($addon.Category.ToLowerInvariant()) addon for local integration testing." }
            author = if ($addon.Author) { $addon.Author } else { 'WinWam Test Harness' }
            sourceKind = 'GitHub'
            host = 'github.com'
            owner = if ($addon.Owner) { $addon.Owner } else { 'winwam-test' }
            repo = if ($addon.Repo) { $addon.Repo } else { $addon.Id }
            version = if ($addon.Version) { $addon.Version } else { '1.0.0' }
            homepage = if ($addon.Homepage) { $addon.Homepage } else { "https://github.com/winwam-test/$($addon.Id)" }
            category = $addon.Category
            tags = $addon.Tags
        }
    }

    $manifest = [ordered]@{
        '$schema' = '../../../data/addon-sources.schema.json'
        schemaVersion = '1.0'
        directory = [ordered]@{
            name = 'WinWam Local Test Directory'
            repository = 'local-test/directory'
            description = 'Generated synthetic data; safe to delete and regenerate.'
        }
        flavor = $flavor.Schema
        addons = @($manifestAddons)
    }
    $manifestJson = $manifest | ConvertTo-Json -Depth 8
    $manifestPath = Join-Path $directoryRoot "addons.$($flavor.Slug).json"
    [System.IO.File]::WriteAllText(
        $manifestPath,
        $manifestJson,
        [System.Text.UTF8Encoding]::new($false)
    )
}

Write-Host "WinWam development harness created at $harnessRoot"
if ($WithInstalledAddons) {
    Write-Host 'Synthetic addons were installed in each flavor.'
} else {
    Write-Host 'Addon folders are empty. Pass -WithInstalledAddons to populate them.'
}
Write-Host 'Debug builds will automatically prefer its WoW folder and directory manifests.'
