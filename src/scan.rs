use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogEntry<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub repo: &'a str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TocMetadata {
    pub title: Option<String>,
    pub version: Option<String>,
    pub interface: Option<String>,
    pub author: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledAddon {
    pub id: String,
    pub folder: String,
    pub title: String,
    pub version: Option<String>,
    pub managed: bool,
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

pub fn resolve_addon_id(
    folder: &str,
    marker_id: Option<&str>,
    toc: &TocMetadata,
    catalog: &[CatalogEntry<'_>],
) -> String {
    if let Some(marker) = marker_id.map(str::trim).filter(|id| !id.is_empty()) {
        return marker.to_string();
    }

    let kebab = kebab_case(folder);
    if let Some(entry) = catalog.iter().find(|entry| {
        entry.id == kebab
            || entry.id.eq_ignore_ascii_case(folder)
            || entry.repo.eq_ignore_ascii_case(folder)
            || entry.name.eq_ignore_ascii_case(folder)
    }) {
        return entry.id.to_string();
    }

    if let Some(title) = toc.title.as_deref()
        && let Some(entry) = catalog
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(title))
    {
        return entry.id.to_string();
    }

    if kebab.is_empty() {
        folder.to_ascii_lowercase()
    } else {
        kebab
    }
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

    let mut seen = BTreeSet::new();
    let mut installed = Vec::new();
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
        let Some(addon) = inspect_addon_folder(&path, folder, catalog) else {
            continue;
        };
        if seen.insert(addon.id.clone()) {
            installed.push(addon);
        }
    }
    installed.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    installed
}

fn inspect_addon_folder(
    path: &Path,
    folder: &str,
    catalog: &[CatalogEntry<'_>],
) -> Option<InstalledAddon> {
    let marker = fs::read_to_string(path.join(".winwam-id"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let toc = read_toc(path, folder);
    if marker.is_none() && toc.is_none() {
        return None;
    }
    let toc = toc.unwrap_or_default();
    let id = resolve_addon_id(folder, marker.as_deref(), &toc, catalog);
    let title = toc
        .title
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| folder.to_string());
    Some(InstalledAddon {
        id,
        folder: folder.to_string(),
        title,
        version: toc.version,
        managed: marker.is_some(),
    })
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
        let catalog = [CatalogEntry {
            id: "test-bag-manager",
            name: "Test Bag Manager",
            repo: "TestBagManager",
        }];
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
            CatalogEntry {
                id: "gratwurst",
                name: "Gratwurst",
                repo: "Gratwurst",
            },
            CatalogEntry {
                id: "test-bag-manager",
                name: "Test Bag Manager",
                repo: "TestBagManager",
            },
        ];
        let installed = scan_installed_addons(Some(&root), &catalog);
        fs::remove_dir_all(&root).ok();

        assert_eq!(installed.len(), 2);
        assert_eq!(installed[0].id, "gratwurst");
        assert!(installed[0].managed);
        assert_eq!(installed[1].id, "test-bag-manager");
        assert!(!installed[1].managed);
        assert_eq!(installed[1].version.as_deref(), Some("1.0.1"));
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
}
