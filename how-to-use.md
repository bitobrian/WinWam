# How to use the local hide-to-tray reactor prototype

WinWam currently depends on crates.io `windows-reactor` **0.100**. That release can
close a window, but it cannot **hide the same window** and keep the component
alive.

The hide-in-place APIs live only in the local checkout at `../windows-rs`
(`C:\gh\windows-rs` next to this repo). They are a machine-local prototype: they
are not on crates.io, and they are not published from this tree.

This is **not** the published `reactor-notifyicon` remount sample. That sample
destroys the window on X and mounts a new component from the tray. Hide-to-tray
keeps **one** `WinWam` instance, so `set_timeout` / `spawn_background` work
(including a later 24-hour update check) survives X.

## 1. Point Cargo at the local crates

In `Cargo.toml`, replace the crates.io pins:

```toml
[dependencies]
windows-reactor = { path = "../windows-rs/crates/libs/reactor" }
windows-notifyicon = { path = "../windows-rs/crates/libs/notifyicon" }

[build-dependencies]
windows-reactor-setup = { path = "../windows-rs/crates/libs/reactor-setup" }
```

From this repo:

```powershell
cargo update -p windows-reactor -p windows-reactor-setup
cargo check
```

`Cargo.lock` currently records `source = "registry+https://github.com/rust-lang/crates.io-index"`
for `windows-reactor` / `windows-reactor-setup`. After the path change those
entries should resolve from `../windows-rs`. crates.io `0.100` does **not** have
`on_window_close_requested` or `request_hide`.

## 2. Optional smoke test (no WinWam changes)

From `C:\gh\windows-rs`:

```powershell
cargo run -p reactor-hide-to-tray
```

Expected:

| Action | Result |
| --- | --- |
| X, or **Hide to tray** | Window hides. The seconds counter keeps ticking. |
| Tray left-click, or **Open** | The **same** window comes back. |
| **Exit** (button or tray menu) | `request_close()`; process ends. |

If that path fails, WinWam will not get hide-in-place either. Fix the prototype
first.

## 3. Wire WinWam

Keep one `WinWam` instance. Do not remount on close. Do not use `App::run_with`
only as a lifetime workaround — hide already keeps the process alive because the
window is not closed. `run_with` is still needed here so `create` can receive
`AppContext` for the tray menu.

Reference implementation:
`../windows-rs/crates/samples/reactor/hide-to-tray/src/main.rs`.

### Messages

Add to `Message` in `src/main.rs`:

```rust
enum Message {
    // ...existing variants...
    CloseRequested,
    Exit,
    OpenFromTray,
    ShowMenu(windows_notifyicon::Point),
}
```

### Component input and state

Change `WinWam` so it owns the app context and the tray icon:

```rust
struct WinWam {
    app: AppContext,
    icon: Option<windows_notifyicon::NotifyIcon>,
    // ...existing fields...
}

impl Component for WinWam {
    type Input = AppContext;
    type Message = Message;
```

In `create`, take `input: &Self::Input`, store `app: input.clone()`, and
register the icon with `assets/icon.ico` using `context.sender()` — same pattern
as the hide-to-tray sample (`Activate` → `OpenFromTray`, `ContextMenu` →
`ShowMenu`, `Unavailable` → `Exit`).

Un-prefix `update`'s `context` argument; it must call `context.window()`.

### Hide / show / exit

```rust
Message::CloseRequested => {
    _ = context.window().request_hide();
}
Message::OpenFromTray => {
    _ = context.window().request_activate();
}
Message::Exit => {
    if !context.window().request_close() {
        _ = self.app.exit();
    }
}
Message::ShowMenu(position) => {
    let sender = context.sender();
    let menu = Menu::new(
        [
            MenuItem::item("open", "Open"),
            MenuItem::separator("separator"),
            MenuItem::item("exit", "Exit"),
        ],
        move |label: String| match label.as_str() {
            "Open" => {
                _ = sender.send(Message::OpenFromTray);
            }
            "Exit" => {
                _ = sender.send(Message::Exit);
            }
            _ => {}
        },
    );
    _ = self
        .app
        .show_menu_at(ScreenPoint::new(position.x, position.y), menu);
}
```

`request_hide` / `request_activate` / `request_close` only succeed during an
active component publication (`create` / `update` / `input_changed`). Same-turn
priority is **close > hide > activate**.

### Subscribe every view

At the top of `view`, before `window_title` / `window_visuals`:

```rust
context.on_window_close_requested(context.callback(|()| Message::CloseRequested));
```

While that observation is published, the native close (X / Alt+F4) is cancelled
and delivered as `CloseRequested`. No subscription → X still destroys the
window, as today.

### Start the app

Replace `App::run_component::<WinWam>(())` in `main`:

```rust
fn main() {
    logging::initialize();
    logging::debug("Starting WinWam");
    if let Err(error) = App::run_with(|app| {
        app.open_window(View::component::<WinWam>(app.clone()))?;
        Ok(())
    }) {
        logging::error(&format!("WinWam failed to start: {error}"));
        panic!("WinWam failed to start: {error}");
    }
}
```

## 4. What should happen

| Action | `WinWam` component | Timers / background work | Native window |
| --- | --- | --- | --- |
| User X | live | live | hidden, same HWND |
| Tray Open | same instance | live | shown + activated |
| Exit / `request_close` | retired | closed | destroyed |

Hiding does not drop `in_flight`, so `App::run` / `run_with` stay alive. Closing
the last **visible** window does not exit while that hidden window still exists.

## 5. What not to do

- Do not remount `WinWam` from the tray. That drops component state and
  scheduled work.
- Do not call `Window.Close()` yourself; use `request_close()` so Exit is not
  cancelled by the close-requested handler.
- Do not leave `windows-reactor = "0.100"` in `Cargo.toml` if you expect hide.
  The published crate does not implement this contract.
- Do not copy `windows-rs` sources into this repo. Path-dep the local crates.

## 6. Revert to crates.io

Put the version pins back and update:

```toml
windows-reactor = "0.100"

[build-dependencies]
windows-reactor-setup = "0.100"
```

```powershell
cargo update -p windows-reactor -p windows-reactor-setup
```

Then remove the hide/tray code; those methods will not compile against crates.io
0.100.
