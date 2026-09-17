use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[cfg(debug_assertions)]
const DEVELOPMENT_DIRECTORY_JSON: &str = include_str!("../data/addon-sources-fake.json");
#[cfg(debug_assertions)]
const LOCAL_TEST_ROOT: &str = "local-test";

const URL_HOST_ALLOWLIST: &[&str] = &[
    "github.com",
    "gitlab.com",
    "gitea.com",
    "ko-fi.com",
    "patreon.com",
    "paypal.com",
    "paypal.me",
    "buymeacoffee.com",
];

const FLAVORS: [(&str, &str); 5] = [
    ("retail", "Retail"),
    ("mop-classic", "MoPClassic"),
    ("classic", "Classic"),
    ("bc-anniversary", "BCAnniversary"),
    ("forever", "Forever"),
];

pub const STALE_DIRECTORY_MESSAGE: &str = "Showing the last saved directory.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryError {
    UnsupportedSchema { version: String },
    FlavorMismatch { expected: String, received: String },
    Parse(String),
    Network(String),
    Io(String),
}

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchema { version } => {
                write!(f, "unsupported directory schema {version}")
            }
            Self::FlavorMismatch { expected, received } => {
                write!(f, "expected {expected} directory, received {received}")
            }
            Self::Parse(error) | Self::Network(error) | Self::Io(error) => f.write_str(error),
        }
    }
}

impl std::error::Error for DirectoryError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonSourceList {
    pub schema_version: String,
    pub directory: DirectoryInfo,
    #[serde(default = "default_flavor")]
    pub flavor: String,
    pub addons: Vec<Addon>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryInfo {
    pub name: String,
    pub repository: String,
    pub description: Option<String>,
    #[serde(default)]
    pub generated_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Addon {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub author: String,
    pub source_kind: String,
    #[serde(default = "default_host")]
    pub host: String,
    pub owner: String,
    pub repo: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub support_urls: Vec<SupportUrl>,
    #[serde(default)]
    pub icon: Option<MediaAsset>,
    #[serde(default)]
    pub banner: Option<MediaAsset>,
    #[serde(default)]
    pub latest_release: Option<LatestRelease>,
    #[serde(default)]
    pub download_count: Option<u64>,
    #[serde(default)]
    pub download_count_captured_at: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for Addon {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            summary: String::new(),
            author: String::new(),
            source_kind: String::new(),
            host: default_host(),
            owner: String::new(),
            repo: String::new(),
            version: None,
            homepage: None,
            category: String::new(),
            tags: Vec::new(),
            source_url: None,
            support_urls: Vec::new(),
            icon: None,
            banner: None,
            latest_release: None,
            download_count: None,
            download_count_captured_at: None,
            description: None,
            features: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportUrl {
    pub label: String,
    pub url: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub sha256: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRelease {
    pub version: String,
    pub published_at: String,
    pub archive_url: String,
    pub sha256: String,
    #[serde(default)]
    pub flavors: Vec<String>,
    #[serde(default)]
    pub interface_versions: Vec<String>,
    #[serde(default)]
    pub changelog_url: Option<String>,
}

impl AddonSourceList {
    pub fn from_json(json: &str) -> Result<Self, DirectoryError> {
        let parsed: Self = serde_json::from_str(json.trim_start_matches('\u{feff}'))
            .map_err(|error| DirectoryError::Parse(error.to_string()))?;
        parsed.normalized()
    }

    pub fn normalized(mut self) -> Result<Self, DirectoryError> {
        if schema_major(&self.schema_version) != Some(1) {
            return Err(DirectoryError::UnsupportedSchema {
                version: self.schema_version,
            });
        }
        for addon in &mut self.addons {
            addon.sanitize_urls();
        }
        Ok(self)
    }
}

impl Addon {
    fn sanitize_urls(&mut self) {
        let host = self.host.clone();
        self.source_url = take_allowed_url(self.source_url.take(), &host);
        self.support_urls
            .retain(|item| url_allowed(&item.url, &host));
        if self
            .icon
            .as_ref()
            .is_some_and(|asset| !url_allowed(&asset.url, &host))
        {
            self.icon = None;
        }
        if self
            .banner
            .as_ref()
            .is_some_and(|asset| !url_allowed(&asset.url, &host))
        {
            self.banner = None;
        }
        if let Some(release) = self.latest_release.as_mut() {
            release.changelog_url = take_allowed_url(release.changelog_url.take(), &host);
            if !url_allowed(&release.archive_url, &host) {
                self.latest_release = None;
            }
        }
    }
}

pub fn flavor_slug(index: usize) -> &'static str {
    FLAVORS.get(index).unwrap_or(&FLAVORS[0]).0
}

fn flavor_schema_value(index: usize) -> &'static str {
    FLAVORS.get(index).unwrap_or(&FLAVORS[0]).1
}

fn default_flavor() -> String {
    "Retail".to_string()
}

fn default_host() -> String {
    "github.com".to_string()
}

fn schema_major(version: &str) -> Option<u32> {
    version.trim().split('.').next()?.parse().ok()
}

fn take_allowed_url(url: Option<String>, addon_host: &str) -> Option<String> {
    url.filter(|value| url_allowed(value, addon_host))
}

fn url_allowed(url: &str, addon_host: &str) -> bool {
    let Some(host) = https_host(url) else {
        return false;
    };
    URL_HOST_ALLOWLIST.iter().any(|allowed| host == *allowed)
        || host.eq_ignore_ascii_case(addon_host.trim())
}

fn https_host(url: &str) -> Option<String> {
    let rest = url.trim().strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() {
        return None;
    }
    let host = authority.rsplit('@').next()?.split(':').next()?.trim();
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() || host.contains(' ') {
        None
    } else {
        Some(host)
    }
}

pub fn cache_dir() -> PathBuf {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local).join("WinWam").join("cache");
    }
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("cache")))
        .unwrap_or_else(|| PathBuf::from("cache"))
}

pub fn cache_file(slug: &str) -> PathBuf {
    cache_file_in(&cache_dir(), slug)
}

fn cache_file_in(dir: &Path, slug: &str) -> PathBuf {
    dir.join(format!("addons.{slug}.json"))
}

pub fn save_cached_directory_to(
    dir: &Path,
    slug: &str,
    list: &AddonSourceList,
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let body = serde_json::to_vec_pretty(list)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    fs::write(cache_file_in(dir, slug), body)
}

pub fn save_cached_directory(slug: &str, list: &AddonSourceList) -> std::io::Result<()> {
    save_cached_directory_to(
        cache_file(slug).parent().unwrap_or(Path::new(".")),
        slug,
        list,
    )
}

pub fn load_cached_directory_from(dir: &Path, slug: &str) -> Option<AddonSourceList> {
    let body = fs::read_to_string(cache_file_in(dir, slug)).ok()?;
    AddonSourceList::from_json(&body).ok()
}

pub fn load_cached_directory(slug: &str) -> Option<AddonSourceList> {
    load_cached_directory_from(&cache_dir(), slug)
}

pub fn keep_last_cache(
    result: Result<AddonSourceList, DirectoryError>,
    cache: Option<AddonSourceList>,
) -> (Option<AddonSourceList>, bool) {
    match result {
        Ok(list) => (Some(list), false),
        Err(_) => {
            let stale = cache.is_some();
            (cache, stale)
        }
    }
}

#[cfg(debug_assertions)]
pub fn development_source_list() -> Option<AddonSourceList> {
    AddonSourceList::from_json(DEVELOPMENT_DIRECTORY_JSON).ok()
}

#[cfg(not(debug_assertions))]
pub fn development_source_list() -> Option<AddonSourceList> {
    None
}

pub fn load_source_list(
    index: usize,
    source_base_url: &str,
) -> Result<AddonSourceList, DirectoryError> {
    let slug = flavor_slug(index);
    let schema_value = flavor_schema_value(index);
    #[cfg(debug_assertions)]
    let local_path = PathBuf::from(LOCAL_TEST_ROOT)
        .join("directory")
        .join(format!("addons.{slug}.json"));
    #[cfg(debug_assertions)]
    let body = if local_path.is_file() {
        fs::read_to_string(&local_path).map_err(|error| DirectoryError::Io(error.to_string()))?
    } else {
        fetch_directory(source_base_url, slug)?
    };
    #[cfg(not(debug_assertions))]
    let body = fetch_directory(source_base_url, slug)?;
    let source_list = AddonSourceList::from_json(&body)?;
    if source_list.flavor != schema_value {
        return Err(DirectoryError::FlavorMismatch {
            expected: schema_value.to_string(),
            received: source_list.flavor,
        });
    }
    if let Err(error) = save_cached_directory(slug, &source_list) {
        crate::logging::error(&format!("Could not write directory cache: {error}"));
    }
    Ok(source_list)
}

fn fetch_directory(source_base_url: &str, slug: &str) -> Result<String, DirectoryError> {
    let url = format!("{source_base_url}/addons.{slug}.json");
    let mut response = ureq::get(&url)
        .call()
        .map_err(|error| DirectoryError::Network(error.to_string()))?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|error| DirectoryError::Network(error.to_string()))
}

pub fn empty_source_list(index: usize) -> AddonSourceList {
    AddonSourceList {
        schema_version: "1.0".to_string(),
        directory: DirectoryInfo {
            name: "WoW Addons Directory".to_string(),
            repository: "https://github.com/bitobrian/wow-addons-directory".to_string(),
            description: Some("No directory data is available for this flavor.".to_string()),
            generated_at: None,
        },
        flavor: flavor_schema_value(index).to_string(),
        addons: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("fixtures")
            .join(name);
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {name}: {error}"))
    }

    fn unique_temp_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "winwam-directory-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp cache dir");
        path
    }

    #[test]
    fn min_entry_deserializes() {
        let list = AddonSourceList::from_json(&fixture("min-1.0.json")).unwrap();
        assert_eq!(list.schema_version, "1.0");
        assert_eq!(list.flavor, "Retail");
        assert_eq!(list.addons.len(), 1);
        assert_eq!(list.addons[0].id, "arcane-alerts");
        assert!(list.addons[0].source_url.is_none());
        assert!(list.addons[0].latest_release.is_none());
        assert_eq!(list.addons[0].host, "github.com");
    }

    #[test]
    fn full_entry_round_trips() {
        let parsed = AddonSourceList::from_json(&fixture("full-1.1.json")).unwrap();
        assert_eq!(parsed.schema_version, "1.1");
        assert_eq!(
            parsed.directory.generated_at.as_deref(),
            Some("2026-09-17T18:00:00Z")
        );
        let addon = &parsed.addons[0];
        assert_eq!(
            addon.source_url.as_deref(),
            Some("https://github.com/winwam-samples/arcane-alerts")
        );
        assert_eq!(addon.support_urls.len(), 2);
        assert_eq!(addon.download_count, Some(12_300));
        assert_eq!(addon.features.len(), 2);
        assert_eq!(
            addon
                .latest_release
                .as_ref()
                .map(|release| release.version.as_str()),
            Some("1.2.0")
        );
        let encoded = serde_json::to_string(&parsed).unwrap();
        let again = AddonSourceList::from_json(&encoded).unwrap();
        assert_eq!(parsed, again);
    }

    #[test]
    fn bad_urls_are_stripped() {
        let list = AddonSourceList::from_json(&fixture("malformed-url.json")).unwrap();
        let addon = &list.addons[0];
        assert!(addon.source_url.is_none());
        assert_eq!(addon.support_urls.len(), 1);
        assert_eq!(
            addon.support_urls[0].url,
            "https://github.com/winwam-samples/arcane-alerts"
        );
        assert!(addon.icon.is_none());
        assert!(addon.banner.is_none());
        assert!(addon.latest_release.is_none());
    }

    #[test]
    fn schema_2_0_is_rejected() {
        let error = AddonSourceList::from_json(&fixture("schema-2.0.json")).unwrap_err();
        assert_eq!(
            error,
            DirectoryError::UnsupportedSchema {
                version: "2.0".into()
            }
        );
    }

    #[test]
    fn each_flavor_is_accepted() {
        for (name, flavor) in [
            ("flavor-Retail.json", "Retail"),
            ("flavor-MoPClassic.json", "MoPClassic"),
            ("flavor-Classic.json", "Classic"),
            ("flavor-BCAnniversary.json", "BCAnniversary"),
            ("flavor-Forever.json", "Forever"),
        ] {
            let list = AddonSourceList::from_json(&fixture(name)).unwrap();
            assert_eq!(list.flavor, flavor, "{name}");
        }
    }

    #[test]
    fn empty_source_list_uses_schema_value() {
        assert_eq!(empty_source_list(0).flavor, "Retail");
        assert_eq!(empty_source_list(1).flavor, "MoPClassic");
        assert_eq!(empty_source_list(3).flavor, "BCAnniversary");
    }

    #[test]
    fn keep_last_cache_helper_prefers_saved_directory() {
        let cached = AddonSourceList::from_json(&fixture("min-1.0.json")).unwrap();
        let (recovered, stale) = keep_last_cache(
            Err(DirectoryError::UnsupportedSchema {
                version: "2.0".into(),
            }),
            Some(cached.clone()),
        );
        assert!(stale);
        assert_eq!(recovered.unwrap().addons[0].id, "arcane-alerts");

        let (fresh, stale) = keep_last_cache(Ok(cached.clone()), Some(empty_source_list(0)));
        assert!(!stale);
        assert_eq!(fresh.unwrap().addons[0].id, "arcane-alerts");

        let (missing, stale) =
            keep_last_cache(Err(DirectoryError::Network("offline".into())), None);
        assert!(!stale);
        assert!(missing.is_none());
    }

    #[test]
    fn schema_2_0_does_not_wipe_cache() {
        let dir = unique_temp_dir();
        let cached = AddonSourceList::from_json(&fixture("min-1.0.json")).unwrap();
        save_cached_directory_to(&dir, "retail", &cached).unwrap();
        assert!(AddonSourceList::from_json(&fixture("schema-2.0.json")).is_err());
        let loaded = load_cached_directory_from(&dir, "retail").unwrap();
        assert_eq!(loaded.schema_version, "1.0");
        assert_eq!(loaded.addons[0].id, "arcane-alerts");
        fs::remove_dir_all(dir).ok();
    }
}
