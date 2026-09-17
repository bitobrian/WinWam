use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::loadout::Loadout;

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SOURCE_BASE_URL: &str =
    "https://raw.githubusercontent.com/bitobrian/wow-addons-directory/refs/heads/main";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_source_base_url")]
    pub source_base_url: String,
    #[serde(default)]
    pub wow_folder: String,
    #[serde(default = "default_flavor")]
    pub selected_flavor: String,
    #[serde(default = "default_true")]
    pub check_for_updates: bool,
    #[serde(default = "default_telemetry_level")]
    pub telemetry_level: usize,
    #[serde(default)]
    pub loadouts: Vec<Loadout>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            source_base_url: DEFAULT_SOURCE_BASE_URL.to_string(),
            wow_folder: String::new(),
            selected_flavor: default_flavor(),
            check_for_updates: true,
            telemetry_level: default_telemetry_level(),
            loadouts: Vec::new(),
        }
    }
}

impl AppSettings {
    pub fn sanitized(&self) -> Self {
        let url = self.source_base_url.trim().trim_end_matches('/');
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            source_base_url: if url.starts_with("https://") {
                url.to_string()
            } else {
                DEFAULT_SOURCE_BASE_URL.to_string()
            },
            wow_folder: self.wow_folder.trim().to_string(),
            selected_flavor: if self.selected_flavor.trim().is_empty() {
                default_flavor()
            } else {
                self.selected_flavor.trim().to_string()
            },
            check_for_updates: self.check_for_updates,
            telemetry_level: self.telemetry_level.min(3),
            loadouts: self.loadouts.clone(),
        }
    }
}

pub fn settings_file() -> PathBuf {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local).join("WinWam").join("settings.json");
    }
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("settings.json")))
        .unwrap_or_else(|| PathBuf::from("settings.json"))
}

pub fn load() -> AppSettings {
    load_from(&settings_file())
}

pub fn load_from(path: &Path) -> AppSettings {
    let Ok(contents) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str::<AppSettings>(contents.trim_start_matches('\u{feff}'))
        .unwrap_or_default()
        .sanitized()
}

pub fn save(settings: &AppSettings) -> std::io::Result<()> {
    save_to(&settings_file(), settings)
}

pub fn save_to(path: &Path, settings: &AppSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json =
        serde_json::to_string_pretty(&settings.sanitized()).map_err(std::io::Error::other)?;
    fs::write(path, json)
}

fn default_schema_version() -> u32 {
    SETTINGS_SCHEMA_VERSION
}

fn default_source_base_url() -> String {
    DEFAULT_SOURCE_BASE_URL.to_string()
}

fn default_flavor() -> String {
    "retail".to_string()
}

fn default_true() -> bool {
    true
}

fn default_telemetry_level() -> usize {
    2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "winwam-settings-{label}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    #[test]
    fn round_trips_sku_specific_loadouts() {
        let path = temp_path("roundtrip");
        let settings = AppSettings {
            wow_folder: r"C:\Games\World of Warcraft".to_string(),
            selected_flavor: "classic".to_string(),
            telemetry_level: 3,
            loadouts: vec![Loadout {
                name: "Raid Night".to_string(),
                flavor: "classic".to_string(),
                addon_ids: BTreeSet::from(["gratwurst".to_string()]),
            }],
            ..AppSettings::default()
        };
        save_to(&path, &settings).unwrap();
        let loaded = load_from(&path);
        fs::remove_file(&path).ok();
        assert_eq!(loaded.wow_folder, settings.wow_folder);
        assert_eq!(loaded.selected_flavor, "classic");
        assert_eq!(loaded.telemetry_level, 3);
        assert_eq!(loaded.loadouts, settings.loadouts);
    }

    #[test]
    fn sanitizes_invalid_source_and_telemetry() {
        let settings = AppSettings {
            source_base_url: "http://example.invalid".to_string(),
            telemetry_level: 99,
            selected_flavor: "  mop-classic  ".to_string(),
            ..AppSettings::default()
        }
        .sanitized();
        assert_eq!(settings.source_base_url, DEFAULT_SOURCE_BASE_URL);
        assert_eq!(settings.telemetry_level, 3);
        assert_eq!(settings.selected_flavor, "mop-classic");
    }

    #[test]
    fn missing_file_returns_defaults() {
        let loaded = load_from(&temp_path("missing"));
        assert_eq!(loaded, AppSettings::default());
    }
}
