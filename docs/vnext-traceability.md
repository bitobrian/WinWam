# vNext mock-to-native traceability

Capture environment: client 1440×856, 100% scale, dark, Segoe UI, SKU Retail, `data/fixtures/visual-catalog.json`. Seed with `.\scripts\setup-dev-harness.ps1 -Reset -VisualParity`. windows-reactor 0.100 cannot render to a bitmap; this table plus `src/layout_contract.rs` is the automated gate. Manual capture remains the pixel check.

Keyboard order follows the visual tree: top nav → SKU combo → Check for Updates → Settings → ribbon → search → cards → overlay. WinUI system focus visuals stand in for the 2px accent ring (`theme::FOCUS_RING`).

| Mock selector / region | Native symbol | State field | Notes |
|---|---|---|---|
| `.topbar` / `.brand` | `WinWam::view` TitleBar `"title-bar"` | — | `assets/icon.png` + `TextBlock` “WinWAM”; OS caption buttons, not mock glyphs |
| `.window-controls` | documented omission | — | OS chrome; do not draw ─□× |
| `.topnav[data-view=discover]` | `theme::topnav_button` | `page: Page::Discover` | Accessible name `Discover` |
| `.topnav[data-view=installed]` | `theme::topnav_button` | `page: Page::Installed` | Accessible name `My Addons`; default route |
| `.topnav[data-view=loadouts]` | `theme::topnav_button` | `page: Page::Loadouts` | Hosts current native Loadouts screen |
| `.topnav[data-view=workshop]` | `theme::topnav_button` | `page: Page::Workshop` | Accessible name `Addon Workshop` |
| `.game-panel` | `game_panel_view` | `selected_flavor` | Width `GAME_PANEL_WIDTH` 280 |
| `.game-art` / `#skuArtwork` | `theme::sku_flair` | `selected_flavor` | Retail + Forever art; SKUs 1–3 gradient only |
| `#release` / `GAME VERSION` | `ComboBox` | `selected_flavor` | Accessible name `Game version` |
| `#updateButton` | `theme::accent_button` | `update_status` | `Check for Updates` / `Checking…` / `Update All` |
| `#settingsButton` | `theme::icon_outline_button` | `page: Page::Settings` | Accessible name `Settings`; hosts current Settings screen |
| `#updateStatus` | `update_status_text` | `update_status` | Failed copy includes `failed` |
| `em` availability | `game_panel_view` availability | `source_list.addons` | `{n} addons available.` in `status_ok`; omitted when `directory_error` is set |
| `.gamebar` / `.category-rail` | `theme::ribbon_button` | `CatalogSurfaceState.category` | No All chip; schema enum order |
| `.support-banner` | `support_banner_view` | `support_banner_dismissed` | Heart is `Symbol::Favorite`, not ♥ |
| `#learnSupport` | `ShowSupportAuthors` | `support_authors_open` | Accessible name `Support authors` |
| `#sectionEyebrow` / `#sectionTitle` | `theme::catalog_header` | `page` | Discover `COMMUNITY PICKS` / `Featured Addons`; Installed `YOUR COLLECTION` / `My Addons` |
| `#search` | `TextBox` | `CatalogSurfaceState.query` | Placeholder `Search addons` |
| `#addonGrid` / `.addon-card` | `addon_card` | `source_list` / `installed_addons` | Three columns, `item_width` 362 |
| `.card-art` install/remove | `InstallAddon` / `UninstallAddon` | `installing_addon_ids`, `fs_busy` | Named `Install {name}` / `Remove {name}` |
| source SVG | `open_https_url` | `resolved_source_url` | Named `Open {name} source project` |
| `View details` | `ToggleAddonDetails` | `expanded_addon_id` | Named `View details for {name}` |
| `#detailsDialog` | `addon_details_overlay` | `expanded_addon_id` | Dim `palette.overlay`; width `MODAL_WIDTH` 760 |
| `.dialog-close` | `CloseAddonDetails` | `expanded_addon_id` | Accessible name `Close`; no key-down API for Escape |
| `#dialogDownloads` / `#dialogVersion` | overlay sidebar | `download_count`, versions | Rows omitted when schema fields missing |
| `#toast` | `shell_overlay` toast | `toast` | `Opening {host}` / copy status |
| `#workshopView` nav | `workshop_nav` | `workshop_screen` | Overview / Anatomy / APIs / Hello World |
| `.ai-assistance` | `workshop_ai_banner` | workshop content | Same copy on every Workshop screen |
| `[data-workshop-panel=overview]` | `workshop_overview` | `workshop::content().overview` | `Start learning` → Anatomy |
| `[data-workshop-panel=anatomy]` | `workshop_anatomy` | `workshop_file` | TOC/Lua/XML/SavedVariables |
| `[data-workshop-panel=apis]` | `workshop_apis` | `workshop_hosts` | Warcraft Wiki + UI & Macro; not Battle.net web API |
| `[data-workshop-panel=examples]` | `workshop_examples` | `workshop_example` | Copy code via `clipboard-win` |
| Loadouts launcher visuals | documented omission | `loadouts_view` | Current native editor hosted until a dedicated mock exists |
| Settings launcher visuals | documented omission | `settings_view` | Current native form hosted |

## Manual capture script

No WinApp SDK UI-test runner is wired in CI. Walk this on the VisualParity harness:

1. Launch: My Addons selected, Retail palette, two managed cards, support banner, `{n} addons available.`
2. Switch SKU to Forever and back: palette/art change; last tab restored.
3. Open details overlay: Close named, versions split, missing download/updated rows omitted.
4. Source and Support authors: toast `Opening {host}`; support overlay lists static channels.
5. Addon Workshop: Overview → Anatomy → APIs → Hello World; Copy code toast Copied/Denied/Failed.

## Destructive-path review

Walk on `.\scripts\setup-dev-harness.ps1 -Reset -WithInstalledAddons` (and `-VisualParity` for Retail cards). Per-addon errors stay in `operation_results` and on cards/modal (M5); they are not cleared by the next scan.

| Path | Native entry | Error surface |
|---|---|---|
| Install | `FsOp::Install` → `install::install_addon` | `operation_results[id]` |
| Update / Update All | `InstallAddon` / `UpdateAll` queue | same; one failure does not skip the rest |
| Uninstall | `FsOp::Uninstall` | `Unmanaged` when no marker |
| Loadout apply | `FsOp::ApplyLoadout` | `LoadoutFailure` list |
| Folder relocation | Settings browse + `refresh_installed` | missing-AddOns banner keeps last scan |
