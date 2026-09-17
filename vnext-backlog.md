# WinWAM vNext backlog

Each checkbox is intentionally scoped as a one-line work item; mock references are the visual and interaction acceptance target.

The vNext UI must be implemented with native `windows-reactor` views, real state, and real assets; screenshots, webviews, transparent overlays, hard-coded captured pixels, and nonfunctional controls do not satisfy these items.

## Application shell and navigation

- [ ] Rebuild the app shell with the fixed desktop-window proportions, two-column content layout, and launcher-style top bar shown in the [shell mock](poc/after/index.html#L11).
- [ ] Replace the current sidebar navigation with top-level Discover, My Addons, Loadouts, and Addon Workshop tabs matching the [navigation mock](poc/after/index.html#L16).
- [ ] Make My Addons the initial route and preserve the selected route while the app is running, following the [mock view behavior](poc/after/app.js#L37).
- [ ] Implement the left game panel with SKU artwork, version selection, update controls, and compact status copy from the [game-panel mock](poc/after/index.html#L25).
- [ ] Move Settings beside the update action and reserve folder detection/rescanning for the Settings workflow, matching the [control placement](poc/after/index.html#L39).

## SKU themes and game selection

- [ ] Populate the game-version selector from the five real SKU definitions and labels shown in the [SKU selector mock](poc/after/index.html#L31).
- [ ] Apply the existing Rust palette for the selected SKU across the shell, panels, cards, dialogs, controls, and focus states as demonstrated by the [theme switcher](poc/after/app.js#L56).
- [ ] Display Midnight artwork for Retail, Forever artwork for Forever, and the atmospheric themed panel without placeholder art for the remaining SKUs, matching the [SKU artwork behavior](poc/after/app.js#L56).
- [ ] Persist the selected SKU and restore its palette, artwork, addon catalog, and installed collection on the next launch.

## Addon management

- [ ] Replace the manual scan primary action with Check for Updates and expose disabled, checking, current, updates-available, and failed states based on the [update-control mock](poc/after/index.html#L39).
- [ ] Implement Update All when one or more managed addons have updates and report per-addon outcomes without blocking unaffected updates.
- [ ] Render installed addons as launcher-style cards with author, downloads, install state, source link, and detail action matching the [catalog mock](poc/after/index.html#L61).
- [ ] Wire install, remove, and update actions to the real addon-management service while preserving the immediate state feedback shown in the [mock interaction](poc/after/app.js#L43).
- [ ] Add search over addon name, author, and category with empty, loading, error, and no-results states based on the [catalog heading](poc/after/index.html#L61).
- [ ] Move the category ribbon above the catalog content, keep labeled category artwork, and filter the current view without an All item as shown in the [ribbon mock](poc/after/index.html#L50).
- [ ] Resolve each catalog entry’s source-project URL and expose it through the source icon on its card.

## Catalog and metadata contract

- [ ] Introduce a backward-compatible directory schema revision because schema 1.0 cannot supply downloads, update timestamps, donation URLs, artwork, release data, or compatibility displayed by the mock ([current schema](data/addon-sources.schema.json#L26)).
- [ ] Add optional `sourceUrl` and `supportUrls` fields with HTTPS validation and host allow-listing so source and donation actions never rely on synthesized URLs.
- [ ] Add optional icon and banner asset metadata with declared dimensions, MIME type, integrity hash, and a deterministic initials fallback when media is unavailable.
- [ ] Add latest-release metadata containing version, published timestamp, archive URL, checksum, supported game flavors, interface versions, and changelog URL.
- [ ] Add optional aggregate download count and define its source and freshness semantics before rendering the mock’s download field.
- [ ] Add optional long description and feature bullets while retaining the existing 300-character summary for compact cards.
- [ ] Parse unknown schema fields safely, reject unsupported major schema versions, and surface a non-destructive directory compatibility error.
- [ ] Cache the last valid per-SKU directory locally and render it with an offline/stale indicator when refresh fails instead of replacing it with an empty catalog.
- [ ] Add schema fixtures and deserialization tests for minimum entries, fully populated entries, malformed URLs, incompatible schema versions, and every supported flavor.

## Native window and composition constraints

- [ ] Preserve the stable native `TitleBar` node required by the current windows-reactor crash workaround while restyling its content to match the [mock header](poc/after/index.html#L11) ([current constraint](src/main.rs#L545)).
- [ ] Use the operating system’s real minimize, maximize, close, drag, snap, DPI, and accessibility behaviors instead of drawing the mock’s window-control glyphs.
- [ ] Decide and document the production client size and minimum size from the mock’s 2164:1287 content ratio, then reconcile that contract with the current 1400×900 client and 1400×760 minimum ([current window sizing](src/main.rs#L535)).
- [ ] Keep the desktop composition fixed at two columns below the documented minimum and use content scrolling or clipping rules rather than responsive web-style reflow.
- [ ] Replace `browse_page_size` window-width heuristics with a layout-derived native card capacity that matches the approved three-column catalog at the reference window size ([current heuristic](src/main.rs#L1873)).
- [ ] Build the shell from native grids, borders, text, images, buttons, combo boxes, scroll viewers, and overlays with no embedded HTML or browser surface.
- [ ] Replace every mock Unicode placeholder icon with a Windows symbol, repository-owned vector/raster asset, or custom drawn path that has an accessible name.
- [ ] Add a native design-token layer for typography, spacing, radii, elevation, focus rings, status colors, overlays, gradients, and control heights beyond the nine existing SKU palette colors.
- [ ] Capture the reference font family, weight, size, line height, truncation, and wrapping behavior for every mock text role before implementing native typography.
- [ ] Define exact reference-window measurements for header height, left rail width, content padding, ribbon height, card size, grid gaps, modal width, and modal column widths from the mock CSS.

## Application state and routing gaps

- [ ] Add `Workshop` to the native `Page` enum and message routing because the current state model only supports Browse, Installed, Loadouts, and Settings ([current page model](src/main.rs#L91)).
- [ ] Rename the user-facing Browse route to Discover and Installed route to My Addons without changing catalog or scan domain terminology.
- [ ] Start on My Addons when the selected SKU has a valid installation and continue routing missing-installation cases to Settings as the current startup guard requires ([current startup route](src/main.rs#L225)).
- [ ] Persist the last non-Settings route separately per SKU and restore it only when its prerequisites remain valid.
- [ ] Define SKU-switch transition behavior for route, ribbon selection, search query, open modal, loadout draft, pending operations, and scroll position before changing flavor.
- [ ] Close or rebind an open addon modal when its addon does not exist in the newly selected SKU catalog.
- [ ] Preserve independent search and ribbon state for Discover and My Addons so switching tabs does not unexpectedly erase either workflow.
- [ ] Replace the mock Loadouts placeholder with an approved vNext Loadouts screen before claiming full shell parity.
- [ ] Produce an approved vNext Settings screen for folder management, directory source, update policy, telemetry, rescanning, and support-banner reset before implementing shell parity.

## Real install and update pipeline

- [ ] Replace `install_test_addon` with a production installer because release builds currently return “installation is not implemented yet” ([current release stub](src/main.rs#L1957)).
- [ ] Replace `uninstall_test_addon` with a production uninstaller that only removes folders proven to be owned by WinWAM and never recursively deletes an unresolved or unmanaged path ([current release stub](src/main.rs#L1982)).
- [ ] Resolve the correct release artifact for the selected SKU from catalog metadata rather than assuming the repository root is directly installable.
- [ ] Download archives to a unique temporary directory, enforce HTTPS, cap download and expanded sizes, validate checksums, and reject traversal, absolute, symlink, and device paths.
- [ ] Detect single-folder and multi-folder addon packages, validate TOC presence, and build an explicit install plan before mutating the AddOns directory.
- [ ] Stage installs beside the target, back up replaced managed folders, atomically swap where possible, and roll back every affected folder on failure.
- [ ] Store a managed-install manifest containing addon ID, source, installed release, owned folders, checksums, SKU, and install timestamp instead of relying only on `.winwam-id`.
- [ ] Preserve user configuration and SavedVariables by restricting install/update/uninstall operations to owned addon folders under the resolved SKU AddOns directory.
- [ ] Compare installed TOC or managed-manifest versions against normalized catalog releases with explicit handling for missing, custom, prerelease, and non-semver versions.
- [ ] Implement concurrent update checks with cancellation, timeouts, retry policy, progress, and per-addon results while serializing filesystem mutations.
- [ ] Disable SKU changes, loadout application, and conflicting addon actions only for the minimum safe duration of active filesystem operations.
- [ ] Refresh the installed scan and update model after every successful or partially successful mutation without discarding actionable failure details.
- [ ] Add integration tests using temporary synthetic WoW trees for install, update, downgrade refusal, rollback, multi-folder packages, unmanaged collisions, and safe uninstall.

## Installed-addon model gaps

- [ ] Extend installed scanning to retain TOC interface version and author already parsed by `TocMetadata` so detail and compatibility fields use local evidence ([current scanner](src/scan.rs#L14)).
- [ ] Represent one logical addon owning multiple folders instead of deduplicating the first matching folder by addon ID ([current deduplication](src/scan.rs#L137)).
- [ ] Distinguish managed-current, managed-update-available, managed-modified, unmanaged-matched, unmanaged-unknown, incompatible, and broken installations in state and UI.
- [ ] Keep unmanaged addons visible in My Addons while disabling destructive management actions and clearly explaining why, consistent with the current ownership safety rule.
- [ ] Define how catalog entries match installed folders when marker, folder name, repository name, addon title, and multi-folder package identity disagree.
- [ ] Surface the actual local version separately from the latest catalog version instead of silently falling back from one to the other.
- [ ] Detect missing or invalid SKU AddOns directories without clearing the last known installed inventory until the user confirms a new location.

## Addon detail experience

- [ ] Replace inline card expansion with the themed modal layout shown in the [detail mock](poc/after/index.html#L130).
- [ ] Populate detail identity, author, category, version, compatible SKU, download count, last-updated value, description, and installed state from real catalog data.
- [ ] Add source-project and author-support actions to the detail sidebar, hiding either action when its URL is unavailable, following the [detail action layout](poc/after/index.html#L136).
- [ ] Keep install, remove, and update actions synchronized between the modal, catalog cards, and My Addons view using the [detail interaction model](poc/after/app.js#L48).
- [ ] Add keyboard focus containment, Escape dismissal, focus restoration, and screen-reader labels to the addon detail modal.
- [ ] Implement the detail experience as a native overlay or dialog layer in the existing window rather than a second browser, screenshot, or inline card expansion.
- [ ] Specify modal loading, offline, missing metadata, unmanaged addon, update available, operation in progress, operation failed, and removed-addon states.
- [ ] Source compatibility, local version, latest version, and update timestamp from real scan/catalog data and omit unavailable values rather than inventing them.
- [ ] Open external source and support URLs through the Windows shell only after validating the resolved HTTPS URL and showing the destination host.

## Community support

- [ ] Add the dismissible author-support banner above the catalog using the respectful donation language and visual hierarchy in the [support mock](poc/after/index.html#L55).
- [ ] Route Support authors to an explainer that covers GitHub Sponsors, Ko-fi, Patreon, project pages, stars, issue reports, and non-financial appreciation.
- [ ] Store banner dismissal locally and provide a Settings control to show the message again.

## Addon Workshop

- [ ] Add the Addon Workshop route and lesson navigation for Overview, Addon Anatomy, WoW APIs, and Hello World matching the [Workshop shell](poc/after/index.html#L68).
- [ ] Implement the Workshop overview with the Windows-style code preview and three-step learning path shown in the mock.
- [ ] Build the interactive Addon Anatomy lesson for TOC, Lua, optional XML, SavedVariables, events, frames, and slash commands using the [anatomy mock](poc/after/index.html#L90).
- [ ] Build the WoW APIs lesson explaining functions, events, widgets, protected actions, SKU compatibility, `/api`, and the research workflow shown in the [API lesson](poc/after/index.html#L105).
- [ ] Link API learners to Warcraft Wiki plus Blizzard’s official in-game `/api` reference and UI & Macro forum without conflating the addon API with the Battle.net web API.
- [ ] Build the Hello World lesson with login greeting, slash command, clickable button, copy-code, `/reload`, and follow-up exercises matching the [examples mock](poc/after/index.html#L115).
- [ ] Replace hard-coded Workshop lessons with versioned content that can be updated independently from the application binary.
- [ ] Add one compact AI-assistance note across Workshop screens covering context, small explained changes, review, testing, and privacy without repeating AI prompts throughout lessons.
- [ ] Validate every Workshop sample against each supported SKU and display compatibility or required changes beside the example.
- [ ] Render Workshop code samples with native text and scroll surfaces using a monospace font, selectable content, and preserved indentation rather than a captured code image.
- [ ] Implement Copy code through the Windows clipboard with success, denial, and failure feedback and no browser clipboard dependency.
- [ ] Version Workshop TOC interface numbers and API examples per SKU so the currently selected game flavor never receives an incompatible sample.
- [ ] Treat external learning links as versioned content with HTTPS validation, offline behavior, and a visible destination host before launch.
- [ ] Add a content-review checklist covering technical accuracy, beginner readability, privacy guidance, and separation of the in-game Lua API from the Battle.net web API.

## Loadouts parity dependencies

- [ ] Retain the existing flavor-scoped loadout data model and safe unmanaged-addon exclusion while moving it into the vNext navigation shell ([current loadout plan](src/loadout.rs#L54)).
- [ ] Design and approve launcher-style loadout list, editor, apply-confirmation, partial-failure, empty, and delete-confirmation screens before replacing the current native views.
- [ ] Show the exact install and uninstall plan before applying a loadout and preserve per-addon errors in the final result instead of collapsing them into a toast.
- [ ] Prevent loadout application while another install/update/uninstall mutation is active and refresh update state after completion.

## Quality and accessibility

- [ ] Reproduce the mock’s desktop-only sizing at supported Windows scale factors without introducing responsive web reflow.
- [ ] Define keyboard navigation order, visible focus styling, accessible names, and high-contrast behavior for every new interactive control.
- [ ] Add unit tests for SKU selection, category filtering, search, install-state transitions, update-state transitions, and Workshop navigation.
- [ ] Add UI automation covering the My Addons landing route, SKU theme changes, addon details, source/support actions, and Workshop examples.
- [ ] Establish a reference capture environment with fixed window client size, Windows scale, text scale, theme, font availability, SKU, seeded catalog, and seeded installed state.
- [ ] Add deterministic fixture data matching every visible value in the approved mock so visual comparisons do not depend on live network or Git host data.
- [ ] Add screenshot-based native regression tests for each top-level route, all five SKU palettes, the detail modal, empty/error/loading states, and every Workshop screen.
- [ ] Compare native captures against the approved mock at the same client dimensions with documented tolerances for text rasterization only, not layout geometry.
- [ ] Add measurement assertions for major bounds and spacing so pixel drift is diagnosed structurally rather than hidden by increasing screenshot thresholds.
- [ ] Verify 100%, 125%, 150%, and 200% Windows scaling separately and maintain logical-pixel geometry without clipped text or blurry raster assets.
- [ ] Verify Windows text scaling, high contrast, keyboard-only use, Narrator names/roles/states, reduced motion, and color-independent status communication.
- [ ] Add explicit loading, empty, stale/offline, recoverable error, fatal configuration, operation-progress, partial-success, and retry states for every data-backed surface.
- [ ] Profile startup, SKU switching, directory refresh, installed scanning, card rendering, modal opening, and Workshop navigation against documented responsiveness budgets.
- [ ] Run `cargo fmt`, `cargo clippy --all-targets --all-features`, unit tests, integration tests, and the native visual-regression suite before marking any parity milestone complete.

## Pixel-parity acceptance gate

- [ ] Approve one canonical reference capture for every required state and store its exact mock commit, route, SKU, viewport, and fixture identifiers beside the baseline.
- [ ] Require every visible mock element to map to a native view, real asset, real state field, or intentionally documented omission in a traceability table.
- [ ] Reject implementation shortcuts that paint the whole mock as an image, host the POC in a WebView, position invisible controls over a screenshot, or hard-code data that should come from application state.
- [ ] Require interactive parity for hover, pressed, focused, disabled, loading, success, error, selected, installed, update-available, and modal-open states before visual sign-off.
- [ ] Require data parity by tracing each displayed label and value to settings, scan results, directory metadata, operation state, or versioned Workshop content.
- [ ] Require destructive-path review for install, update, uninstall, loadout application, and folder relocation before visual completeness can be considered release-ready.
