# WinWam UI proof of concept

This folder is a standalone, dependency-free mock of the desktop addon browser. It does not import or modify the Rust application source.

Open `index.html` directly in a browser, or serve the repository root with any static file server and visit `/poc/`.

Implemented mock interactions:

- browse and installed views
- category and text filtering
- sorting and pagination
- install/remove state
- addon details dialog
- release picker, loadouts, and settings placeholders

The mock reuses image assets from the repository's `assets/` folder.
