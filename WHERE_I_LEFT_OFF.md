# Where I Left Off

## Current state

WinWam is a native Rust/WinUI 3 addon manager prototype built with `windows-reactor`.
The project still has no initial Git commit. Most files are untracked; only `.gitattributes`
and `LICENSE` are staged.

The application currently builds and tests successfully:

```powershell
cargo check
cargo test
```

There are 16 unit tests covering TOC scanning, loadout diffs, settings persistence, and pagination.

## Working features

- SKU selection for Retail, MoP Classic, Classic, BC Anniversary, and Forever
- SKU-specific, softened color themes
- Live addon-directory loading with local development fallback
- Search, category filtering, sorting, and responsive pagination
- Category ribbon with generated icon artwork
- Expandable addon cards with repository, tag, author, and version details
- Debug-harness addon installation with progress feedback
- Installed-addon detection via `.toc` discovery plus `.winwam-id` markers
- SKU-specific loadouts persisted with settings
- Confirmation and result reporting for loadout apply/delete
- Configurable in-app telemetry and the **Open the Hood** diagnostics panel
- WoW folder detection and validation
- Release build/deployment script

## Browse layout

Browse uses paginated native wrapping grids rather than rendering the entire directory.
The number of cards per page changes at layout breakpoints and is limited to 6–20.
Cards are approximately 340×200 and the results area is centered with a maximum width
of 1800px to remain readable on 4K displays.

The category ribbon has no **All** option. Each category is intended to correspond to a
separately manageable directory list. Supplied ribbon artwork is generated into compact
64×64 assets.

Regenerate ribbon assets with:

```powershell
.\scripts\prepare-ribbon-assets.ps1
```

## Local development harness

`local-test/` is ignored by Git and acts as a fake World of Warcraft installation and
local addon directory. Debug builds automatically prefer it.

Regenerate the normal harness with 500 browse addons per SKU:

```powershell
.\scripts\setup-dev-harness.ps1 -Reset
```

Include three preinstalled addons per SKU with:

```powershell
.\scripts\setup-dev-harness.ps1 -Reset -WithInstalledAddons
```

Gratwurst is the first realistic benchmark fixture and uses metadata based on
`../Gratwurst`.

## Loadouts

Loadouts are SKU-specific. A loadout stores a name, flavor slug, and a set of addon IDs
in `%LOCALAPPDATA%\WinWam\settings.json` with the rest of the settings. Applying one
installs missing selected addons and removes extras through the debug-harness
install/uninstall implementation.

Apply and delete ask for confirmation first. Apply then reports installed, removed, and
failed addons. Unmanaged `.toc` addons are shown on Installed but are not removed by a
loadout.

## Telemetry

Settings contains **Open the Hood**, which reveals a right-side live event panel.
Telemetry levels are:

1. Off
2. Errors only
3. Standard
4. Verbose

The selected telemetry level is persisted. The event stream itself remains in-memory
(latest 250 events). File logging remains in `src/logging.rs` and is separate from the
in-app telemetry stream.

## Important limitations

- Production addon download/install/update/uninstall is not implemented.
- Debug installation creates synthetic `.toc`, `.lua`, and `.winwam-id` files.
- Update checks and real release/version resolution are not implemented.
- The directory is kept in memory, but cards are paginated because rich arbitrary views
  are not truly virtualized by the current `windows-reactor` usage.

## Key files

- `src/main.rs` — application state, pages, directory loading, harness install workflows
- `src/scan.rs` — `.toc` parsing, catalog matching, installed-folder discovery
- `src/loadout.rs` — SKU-scoped loadout diffs and apply/delete prompts
- `src/persist.rs` — `%LOCALAPPDATA%\WinWam\settings.json`
- `src/theme.rs` — palettes and reusable themed controls
- `src/logging.rs` — file logging and panic hook
- `data/addon-sources.schema.json` — directory schema
- `scripts/setup-dev-harness.ps1` — local WoW and directory fixtures
- `scripts/prepare-ribbon-assets.ps1` — ribbon icon generation
- `release.ps1` — release build and local deployment

## Suggested next steps

1. Test the latest card dimensions and category ribbon at 1400px, 1440p, and 4K.
2. Design the production download, extraction, backup, and rollback pipeline.
3. Add real GitHub/GitLab release resolution and update checks.
4. Review the working tree, stage intended assets, fill in the license holder, and create
   the initial commit.
