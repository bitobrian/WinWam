use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogEntry<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub repo: &'a str,
    pub catalog_version: Option<&'a str>,
    pub interface_versions: &'a [String],
}

impl<'a> CatalogEntry<'a> {
    pub fn new(id: &'a str, name: &'a str, repo: &'a str) -> Self {
        Self {
            id,
            name,
            repo,
            catalog_version: None,
            interface_versions: &[],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TocMetadata {
    pub title: Option<String>,
    pub version: Option<String>,
    pub interface: Option<String>,
    pub author: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallState {
    ManagedCurrent,
    ManagedUpdateAvailable,
    ManagedModified,
    UnmanagedMatched,
    UnmanagedUnknown,
    Incompatible,
    Broken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledAddon {
    pub id: String,
    pub folder: String,
    pub folders: Vec<String>,
    pub title: String,
    pub local_version: Option<String>,
    pub interface: Option<String>,
    pub author: Option<String>,
    pub managed: bool,
    pub state: InstallState,
}

pub fn kebab_case(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !result.is_empty() && !result.ends_with('-') {
                result.push('-');
            }
            continue;
        }
        if !ch.is_ascii_alphanumeric() {
            continue;
        }
        if ch.is_uppercase() {
            if index > 0 {
                let previous = chars[index - 1];
                let next_is_lower = chars.get(index + 1).is_some_and(|next| next.is_lowercase());
                if (previous.is_lowercase()
                    || previous.is_ascii_digit()
                    || (previous.is_uppercase() && next_is_lower))
                    && !result.ends_with('-')
                {
                    result.push('-');
                }
            }
            result.extend(ch.to_lowercase());
            continue;
        }
        result.push(ch.to_ascii_lowercase());
    }
    result.trim_matches('-').to_string()
}

pub fn strip_wow_ui_escapes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '|' {
            output.push(ch);
            continue;
        }
        match chars.peek().copied() {
            Some('c' | 'C') => {
                chars.next();
                for _ in 0..8 {
                    chars.next();
                }
            }
            Some('r' | 'R' | 'n' | 'N') => {
                chars.next();
            }
            Some('|') => {
                chars.next();
                output.push('|');
            }
            _ => output.push(ch),
        }
    }
    output.trim().to_string()
}

pub fn parse_toc(contents: &str) -> TocMetadata {
    let mut metadata = TocMetadata::default();
    let mut localized_title = None;
    for line in contents.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("##") else {
            continue;
        };
        let rest = rest.trim();
        let Some((key, value)) = rest.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = strip_wow_ui_escapes(value.trim());
        if value.is_empty() {
            continue;
        }
        match key {
            "Title" => metadata.title = Some(value),
            "Version" => metadata.version = Some(value),
            "Interface" => metadata.interface = Some(value),
            "Author" => metadata.author = Some(value),
            "Title-enUS" if localized_title.is_none() => localized_title = Some(value),
            _ => {}
        }
    }
    if metadata.title.is_none() {
        metadata.title = localized_title;
    }
    metadata
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn resolve_addon_id(
    folder: &str,
    marker_id: Option<&str>,
    toc: &TocMetadata,
    catalog: &[CatalogEntry<'_>],
) -> String {
    resolve_identity(folder, marker_id, toc, catalog).addon_id()
}

pub fn find_addon_directory(addons_folder: &Path, id: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(addons_folder).ok()?;
    let mut case_insensitive = None;
    let mut kebab_match = None;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if let Ok(marker) = fs::read_to_string(path.join(".winwam-id"))
            && marker.trim() == id
        {
            return Some(path);
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if name == id {
                return Some(path);
            }
            if name.eq_ignore_ascii_case(id) {
                case_insensitive = Some(path.clone());
            } else if kebab_case(name) == id {
                kebab_match = Some(path);
            }
        }
    }
    case_insensitive.or(kebab_match)
}

pub fn pending_update_ids(installed: &[InstalledAddon]) -> Vec<String> {
    installed
        .iter()
        .filter(|addon| addon.state == InstallState::ManagedUpdateAvailable)
        .map(|addon| addon.id.clone())
        .collect()
}

pub fn scan_installed_addons(
    addons_folder: Option<&Path>,
    catalog: &[CatalogEntry<'_>],
) -> Vec<InstalledAddon> {
    let Some(addons_folder) = addons_folder.filter(|path| path.is_dir()) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(addons_folder) else {
        return Vec::new();
    };

    let mut groups: BTreeMap<String, Vec<FolderRecord>> = BTreeMap::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if folder.starts_with('.') || folder.starts_with("Blizzard_") {
            continue;
        }
        let Some(record) = inspect_addon_folder(&path, folder, catalog) else {
            continue;
        };
        groups
            .entry(record.identity.group_key())
            .or_default()
            .push(record);
    }
    let mut installed = groups
        .into_values()
        .map(|group| merge_group(addons_folder, group, catalog))
        .collect::<Vec<_>>();
    installed.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    installed
}

fn merge_group(
    addons_folder: &Path,
    group: Vec<FolderRecord>,
    catalog: &[CatalogEntry<'_>],
) -> InstalledAddon {
    let identity = group[0].identity.clone();
    let id = identity.addon_id();
    let managed = identity.managed() || group.iter().any(|item| item.identity.managed());
    let folders: Vec<String> = group.iter().map(|item| item.folder.clone()).collect();
    let folder = folders[0].clone();
    let title = group
        .iter()
        .find_map(|item| item.toc.title.clone().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| folder.clone());
    let interface = group.iter().find_map(|item| item.toc.interface.clone());
    let author = group.iter().find_map(|item| item.toc.author.clone());
    let local_version = group
        .iter()
        .find_map(|item| item.toc.version.clone())
        .or_else(|| {
            group
                .iter()
                .find_map(|item| read_manifest_version(&item.path))
        });
    let has_toc = group.iter().any(|item| item.has_toc);
    let catalog_entry = catalog.iter().find(|entry| entry.id == id);
    let state = classify_state(
        addons_folder,
        &folders,
        managed,
        has_toc,
        interface.as_deref(),
        local_version.as_deref(),
        catalog_entry,
        matches!(identity, Identity::Unknown { .. }),
    );
    InstalledAddon {
        id,
        folder,
        folders,
        title,
        local_version,
        interface,
        author,
        managed,
        state,
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_state(
    addons_folder: &Path,
    folders: &[String],
    managed: bool,
    has_toc: bool,
    interface: Option<&str>,
    local_version: Option<&str>,
    catalog: Option<&CatalogEntry<'_>>,
    unknown: bool,
) -> InstallState {
    if !has_toc {
        return InstallState::Broken;
    }
    if managed && hashes_differ(addons_folder, folders) {
        return InstallState::ManagedModified;
    }
    if let (Some(interface), Some(entry)) = (interface, catalog)
        && !entry.interface_versions.is_empty()
        && !entry
            .interface_versions
            .iter()
            .any(|value| value == interface)
    {
        return InstallState::Incompatible;
    }
    if managed {
        return match crate::install::version_status(
            local_version,
            catalog.and_then(|entry| entry.catalog_version),
        ) {
            crate::install::VersionStatus::Older => InstallState::ManagedUpdateAvailable,
            _ => InstallState::ManagedCurrent,
        };
    }
    if unknown || catalog.is_none() {
        InstallState::UnmanagedUnknown
    } else {
        InstallState::UnmanagedMatched
    }
}

fn hashes_differ(addons_folder: &Path, folders: &[String]) -> bool {
    folders.iter().any(|folder| {
        let path = addons_folder.join(folder);
        let Some(manifest) = read_manifest_hashes(&path) else {
            return false;
        };
        manifest.iter().any(|(relative, expected)| {
            let file = addons_folder.join(relative);
            file_sha256(&file).as_deref() != Some(expected.as_str())
        })
    })
}

fn read_manifest_version(path: &Path) -> Option<String> {
    let body = fs::read_to_string(path.join(".winwam-manifest.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    value
        .get("releaseVersion")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn read_manifest_hashes(path: &Path) -> Option<BTreeMap<String, String>> {
    let body = fs::read_to_string(path.join(".winwam-manifest.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    let map = value.get("fileSha256")?.as_object()?;
    Some(
        map.iter()
            .filter_map(|(key, value)| {
                value
                    .as_str()
                    .map(|hash| (key.clone(), hash.to_ascii_lowercase()))
            })
            .collect(),
    )
}

fn file_sha256(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Some(hex::encode(hasher.finalize()))
}

fn inspect_addon_folder(
    path: &Path,
    folder: &str,
    catalog: &[CatalogEntry<'_>],
) -> Option<FolderRecord> {
    let marker = fs::read_to_string(path.join(".winwam-id"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            read_manifest_version(path).and_then(|_| {
                fs::read_to_string(path.join(".winwam-manifest.json"))
                    .ok()
                    .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok())
                    .and_then(|value| {
                        value
                            .get("addonId")
                            .and_then(|value| value.as_str())
                            .filter(|id| !id.is_empty())
                            .map(ToString::to_string)
                    })
            })
        });
    let toc = read_toc(path, folder);
    if marker.is_none() && toc.is_none() {
        return None;
    }
    let has_toc = toc.is_some();
    let toc = toc.unwrap_or_default();
    let identity = resolve_identity(folder, marker.as_deref(), &toc, catalog);
    Some(FolderRecord {
        folder: folder.to_string(),
        path: path.to_path_buf(),
        toc,
        has_toc,
        identity,
    })
}

struct FolderRecord {
    folder: String,
    path: PathBuf,
    toc: TocMetadata,
    has_toc: bool,
    identity: Identity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Identity {
    Managed(String),
    Matched(String),
    Unknown { key: String, id: String },
}

impl Identity {
    fn group_key(&self) -> String {
        match self {
            Self::Managed(id) | Self::Matched(id) => id.clone(),
            Self::Unknown { key, .. } => key.clone(),
        }
    }

    fn addon_id(&self) -> String {
        match self {
            Self::Managed(id) | Self::Matched(id) => id.clone(),
            Self::Unknown { id, .. } => id.clone(),
        }
    }

    fn managed(&self) -> bool {
        matches!(self, Self::Managed(_))
    }
}

fn resolve_identity(
    folder: &str,
    marker: Option<&str>,
    toc: &TocMetadata,
    catalog: &[CatalogEntry<'_>],
) -> Identity {
    if let Some(marker) = marker.map(str::trim).filter(|id| !id.is_empty()) {
        return Identity::Managed(marker.to_string());
    }

    let mut matched = Vec::new();
    let mut push = |id: &str| {
        if !matched.iter().any(|existing| *existing == id) {
            matched.push(id.to_string());
        }
    };
    let kebab = kebab_case(folder);
    for entry in catalog {
        if entry.id.eq_ignore_ascii_case(folder)
            || entry.repo.eq_ignore_ascii_case(folder)
            || toc
                .title
                .as_deref()
                .is_some_and(|title| entry.name.eq_ignore_ascii_case(title))
            || entry.id == kebab
        {
            push(entry.id);
        }
    }
    match matched.as_slice() {
        [id] => Identity::Matched(id.clone()),
        [] => Identity::Unknown {
            key: format!("folder:{folder}"),
            id: if kebab.is_empty() {
                folder.to_ascii_lowercase()
            } else {
                kebab
            },
        },
        _ => Identity::Unknown {
            key: format!("folder:{folder}"),
            id: if kebab.is_empty() {
                folder.to_ascii_lowercase()
            } else {
                kebab
            },
        },
    }
}

fn read_toc(path: &Path, folder: &str) -> Option<TocMetadata> {
    let preferred = path.join(format!("{folder}.toc"));
    if preferred.is_file() {
        return fs::read_to_string(preferred)
            .ok()
            .map(|contents| parse_toc(&contents));
    }
    let entries = fs::read_dir(path).ok()?;
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|candidate| {
            candidate
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("toc"))
        })
        .and_then(|candidate| fs::read_to_string(candidate).ok())
        .map(|contents| parse_toc(&contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "winwam-scan-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp dir");
        path
    }

    #[test]
    fn kebab_case_converts_pascal_folder_names() {
        assert_eq!(kebab_case("TestBagManager"), "test-bag-manager");
        assert_eq!(kebab_case("Gratwurst"), "gratwurst");
        assert_eq!(kebab_case("UIParent"), "ui-parent");
        assert_eq!(kebab_case("already-kebab"), "already-kebab");
    }

    #[test]
    fn parse_toc_reads_fields_and_strips_color_codes() {
        let toc = parse_toc(
            "## Interface: 110200\n## Title: |cffff8000Gratwurst|r\n## Version: 1.10.0\nGratwurst.lua\n",
        );
        assert_eq!(toc.title.as_deref(), Some("Gratwurst"));
        assert_eq!(toc.version.as_deref(), Some("1.10.0"));
        assert_eq!(toc.interface.as_deref(), Some("110200"));
    }

    #[test]
    fn resolve_prefers_marker_then_catalog_matches() {
        let catalog = [CatalogEntry::new(
            "test-bag-manager",
            "Test Bag Manager",
            "TestBagManager",
        )];
        let toc = TocMetadata {
            title: Some("Test Bag Manager".into()),
            ..TocMetadata::default()
        };
        assert_eq!(
            resolve_addon_id("TestBagManager", Some("test-bag-manager"), &toc, &catalog),
            "test-bag-manager"
        );
        assert_eq!(
            resolve_addon_id("TestBagManager", None, &toc, &catalog),
            "test-bag-manager"
        );
        assert_eq!(
            resolve_addon_id("MysteryAddon", None, &TocMetadata::default(), &catalog),
            "mystery-addon"
        );
    }

    #[test]
    fn scan_discovers_toc_addons_and_skips_blizzard_folders() {
        let root = temp_dir("discover");
        let managed = root.join("Gratwurst");
        fs::create_dir_all(&managed).unwrap();
        fs::write(managed.join(".winwam-id"), "gratwurst").unwrap();
        fs::write(
            managed.join("Gratwurst.toc"),
            "## Title: Gratwurst\n## Version: 1.10.0\n",
        )
        .unwrap();

        let unmanaged = root.join("TestBagManager");
        fs::create_dir_all(&unmanaged).unwrap();
        fs::write(
            unmanaged.join("TestBagManager.toc"),
            "## Title: Test Bag Manager\n## Version: 1.0.1\n",
        )
        .unwrap();

        let blizzard = root.join("Blizzard_ObjectiveTracker");
        fs::create_dir_all(&blizzard).unwrap();
        fs::write(
            blizzard.join("Blizzard_ObjectiveTracker.toc"),
            "## Title: Blizzard\n",
        )
        .unwrap();

        let catalog = [
            CatalogEntry::new("gratwurst", "Gratwurst", "Gratwurst"),
            CatalogEntry::new("test-bag-manager", "Test Bag Manager", "TestBagManager"),
        ];
        let installed = scan_installed_addons(Some(&root), &catalog);
        fs::remove_dir_all(&root).ok();

        assert_eq!(installed.len(), 2);
        assert_eq!(installed[0].id, "gratwurst");
        assert!(installed[0].managed);
        assert_eq!(installed[1].id, "test-bag-manager");
        assert!(!installed[1].managed);
        assert_eq!(installed[1].local_version.as_deref(), Some("1.0.1"));
        assert_eq!(installed[0].folders, vec!["Gratwurst".to_string()]);
    }

    #[test]
    fn find_addon_directory_matches_marker_and_folder_name() {
        let root = temp_dir("find");
        let addon = root.join("Gratwurst");
        fs::create_dir_all(&addon).unwrap();
        fs::write(addon.join(".winwam-id"), "gratwurst").unwrap();

        let found = find_addon_directory(&root, "gratwurst");
        fs::remove_dir_all(&root).ok();
        assert_eq!(found, Some(addon));
    }

    #[test]
    fn groups_multi_folder_addons_by_resolved_id() {
        let root = temp_dir("multi");
        for name in ["ArcaneAlerts", "ArcaneAlerts_Options"] {
            let folder = root.join(name);
            fs::create_dir_all(&folder).unwrap();
            fs::write(folder.join(".winwam-id"), "arcane-alerts").unwrap();
            fs::write(
                folder.join(format!("{name}.toc")),
                format!("## Title: {name}\n## Version: 1.2.0\n## Interface: 120000\n"),
            )
            .unwrap();
        }
        let catalog = [CatalogEntry::new(
            "arcane-alerts",
            "Arcane Alerts",
            "arcane-alerts",
        )];
        let installed = scan_installed_addons(Some(&root), &catalog);
        fs::remove_dir_all(&root).ok();
        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].id, "arcane-alerts");
        assert_eq!(installed[0].folders.len(), 2);
        assert_eq!(installed[0].interface.as_deref(), Some("120000"));
        assert_eq!(installed[0].local_version.as_deref(), Some("1.2.0"));
        assert_ne!(
            installed[0].local_version.as_deref(),
            catalog[0].catalog_version
        );
        assert!(installed[0].managed);
    }

    #[test]
    fn ambiguous_catalog_matches_stay_unknown_and_unmerged() {
        let catalog = [
            CatalogEntry::new("alpha", "Shared", "Alpha"),
            CatalogEntry::new("beta", "Shared", "Beta"),
        ];
        let toc = TocMetadata {
            title: Some("Shared".into()),
            ..TocMetadata::default()
        };
        assert!(matches!(
            resolve_identity("Mystery", None, &toc, &catalog),
            Identity::Unknown { .. }
        ));
    }
}
