use std::{
    env,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn initialize() {
    let directory = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(ToOwned::to_owned))
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = LOG_DIRECTORY.set(directory);

    let enabled = cfg!(debug_assertions)
        || env::var("WINWAM_DEBUG_LOG")
            .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
    let _ = DEBUG_ENABLED.set(enabled);

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error(&format!("Unhandled panic: {panic_info}"));
        previous_hook(panic_info);
    }));

    debug("WinWam logging initialized");
}

pub fn error(message: &str) {
    write_line("error.txt", "ERROR", message);
}

pub fn debug(message: &str) {
    if DEBUG_ENABLED
        .get()
        .copied()
        .unwrap_or(cfg!(debug_assertions))
    {
        write_line("log.txt", "DEBUG", message);
    }
}

fn write_line(file_name: &str, level: &str, message: &str) {
    let directory = LOG_DIRECTORY
        .get()
        .cloned()
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let path = directory.join(file_name);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let sanitized = message.replace(['\r', '\n'], " ");
        let _ = writeln!(file, "[{timestamp}] [{level}] {sanitized}");
    }
}
