# WinWAM vNext backlog

Each checkbox is intentionally scoped as a one-line work item; mock references are the visual and interaction acceptance target.

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

## Addon detail experience

- [ ] Replace inline card expansion with the themed modal layout shown in the [detail mock](poc/after/index.html#L130).
- [ ] Populate detail identity, author, category, version, compatible SKU, download count, last-updated value, description, and installed state from real catalog data.
- [ ] Add source-project and author-support actions to the detail sidebar, hiding either action when its URL is unavailable, following the [detail action layout](poc/after/index.html#L136).
- [ ] Keep install, remove, and update actions synchronized between the modal, catalog cards, and My Addons view using the [detail interaction model](poc/after/app.js#L48).
- [ ] Add keyboard focus containment, Escape dismissal, focus restoration, and screen-reader labels to the addon detail modal.

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

## Quality and accessibility

- [ ] Reproduce the mock’s desktop-only sizing at supported Windows scale factors without introducing responsive web reflow.
- [ ] Define keyboard navigation order, visible focus styling, accessible names, and high-contrast behavior for every new interactive control.
- [ ] Add unit tests for SKU selection, category filtering, search, install-state transitions, update-state transitions, and Workshop navigation.
- [ ] Add UI automation covering the My Addons landing route, SKU theme changes, addon details, source/support actions, and Workshop examples.
