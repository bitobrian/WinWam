# WinWam launcher concept

This is the standalone “after” design exploration. It borrows the information density, navigation hierarchy, atmospheric presentation, and action styling of modern game launchers while remaining a WinWam addon-management interface.

Open `index.html` directly in a browser. No build step or dependencies are required.

The update action is mocked: it briefly checks the installed collection and reports that everything is current. Manual addon-folder rescanning is intentionally treated as a future Settings action rather than a primary workflow.

The game-version selector mirrors the five SKU palettes authored in `src/theme.rs`: Retail/Midnight purple, Mists jade, Classic forged metal, Burning Crusade fel green, and Forever sky blue. Retail and Forever also use their matching SKU artwork, consistent with the Rust application.

Addon Workshop distinguishes the in-game Lua addon API from the separate Battle.net web API. It points learners to Blizzard's generated in-game `/api` reference and the official Blizzard UI & Macro forum rather than a third-party UI-source mirror.

The original proof of concept remains unchanged in `../before/`.
