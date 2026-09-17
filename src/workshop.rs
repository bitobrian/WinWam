use std::{collections::BTreeMap, net::ToSocketAddrs, sync::OnceLock};

use serde::Deserialize;

const MANIFEST_JSON: &str = include_str!("../data/workshop/v1/manifest.json");
const OVERVIEW_JSON: &str = include_str!("../data/workshop/v1/overview.json");
const ANATOMY_JSON: &str = include_str!("../data/workshop/v1/anatomy.json");
const APIS_JSON: &str = include_str!("../data/workshop/v1/apis.json");
const EXAMPLES_JSON: &str = include_str!("../data/workshop/v1/examples.json");

pub const REQUIRED_SLUGS: [&str; 5] = [
    "retail",
    "mop-classic",
    "classic",
    "bc-anniversary",
    "forever",
];

const LESSON_HOSTS: &[&str] = &[
    "warcraft.wiki.gg",
    "us.forums.blizzard.com",
    "forums.blizzard.com",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Anatomy,
    Apis,
    Examples,
}

impl Screen {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "overview" => Some(Self::Overview),
            "anatomy" => Some(Self::Anatomy),
            "apis" => Some(Self::Apis),
            "examples" => Some(Self::Examples),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Anatomy => "anatomy",
            Self::Apis => "apis",
            Self::Examples => "examples",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyStatus {
    Copied,
    Denied,
    Failed,
    #[allow(dead_code)]
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: String,
    pub sku_interface: BTreeMap<String, String>,
    pub ai_assistance: AiAssistance,
    pub screens: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AiAssistance {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub id: String,
    pub nav_label: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub start_label: String,
    pub start_screen: String,
    pub path_eyebrow: String,
    pub path_title: String,
    pub path_body: String,
    pub steps: Vec<PathStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct PathStep {
    pub index: String,
    pub title: String,
    pub body: String,
    pub action: String,
    pub screen: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anatomy {
    pub id: String,
    pub nav_label: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub tree_title: String,
    pub files: Vec<FileLesson>,
    pub concepts: Vec<ConceptCard>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLesson {
    pub id: String,
    pub label: String,
    pub badge: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub code: String,
    pub note_title: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ConceptCard {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apis {
    pub id: String,
    pub nav_label: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub callout_mark: String,
    pub callout_title: String,
    pub callout_body: String,
    pub sku_note: String,
    pub api_groups: Vec<ConceptCard>,
    pub resources: Vec<ResourceLink>,
    pub workflow_title: String,
    pub workflow: Vec<ConceptCard>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ResourceLink {
    pub mark: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub action: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Examples {
    pub id: String,
    pub nav_label: String,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub reload_title: String,
    pub reload_body: String,
    pub samples: Vec<Sample>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub id: String,
    pub tab: String,
    pub concept: String,
    pub title: String,
    pub description: String,
    pub challenge: String,
    pub file_name: String,
    pub code: String,
    pub compatible_skus: Vec<String>,
    pub sku_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub manifest: Manifest,
    pub overview: Overview,
    pub anatomy: Anatomy,
    pub apis: Apis,
    pub examples: Examples,
}

pub fn content() -> &'static Content {
    static CONTENT: OnceLock<Content> = OnceLock::new();
    CONTENT.get_or_init(|| load().expect("workshop v1 content must parse"))
}

pub fn load() -> Result<Content, String> {
    load_from(
        MANIFEST_JSON,
        OVERVIEW_JSON,
        ANATOMY_JSON,
        APIS_JSON,
        EXAMPLES_JSON,
    )
}

pub fn load_from(
    manifest: &str,
    overview: &str,
    anatomy: &str,
    apis: &str,
    examples: &str,
) -> Result<Content, String> {
    let manifest: Manifest =
        serde_json::from_str(manifest).map_err(|error| format!("manifest: {error}"))?;
    if manifest.schema_version != "1" {
        return Err(format!(
            "unsupported workshop schema {}",
            manifest.schema_version
        ));
    }
    for slug in REQUIRED_SLUGS {
        if manifest
            .sku_interface
            .get(slug)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return Err(format!("missing skuInterface for {slug}"));
        }
    }
    if manifest.screens
        != ["overview", "anatomy", "apis", "examples"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    {
        return Err("screens must be overview, anatomy, apis, examples".to_string());
    }

    let overview: Overview =
        serde_json::from_str(overview).map_err(|error| format!("overview: {error}"))?;
    let anatomy: Anatomy =
        serde_json::from_str(anatomy).map_err(|error| format!("anatomy: {error}"))?;
    let apis: Apis = serde_json::from_str(apis).map_err(|error| format!("apis: {error}"))?;
    let examples: Examples =
        serde_json::from_str(examples).map_err(|error| format!("examples: {error}"))?;

    for url in apis.resources.iter().map(|resource| resource.url.as_str()) {
        validated_lesson_url(url)?;
    }

    Ok(Content {
        manifest,
        overview,
        anatomy,
        apis,
        examples,
    })
}

pub fn interface_for(slug: &str) -> &str {
    content()
        .manifest
        .sku_interface
        .get(slug)
        .map(String::as_str)
        .unwrap_or("120000")
}

pub fn apply_placeholders(text: &str, slug: &str, sku_label: &str) -> String {
    text.replace("{{interface}}", interface_for(slug))
        .replace("{interface}", interface_for(slug))
        .replace("{sku}", sku_label)
}

pub fn sample_sku_status(sample: &Sample, slug: &str, sku_label: &str) -> String {
    if sample.compatible_skus.iter().any(|item| item == slug) {
        format!("Fits {sku_label}")
    } else {
        format!("Needs changes for {sku_label}: {}", sample.sku_note)
    }
}

pub fn file_lesson<'a>(anatomy: &'a Anatomy, id: &str) -> &'a FileLesson {
    anatomy
        .files
        .iter()
        .find(|file| file.id == id)
        .or_else(|| anatomy.files.first())
        .expect("anatomy files")
}

pub fn sample<'a>(examples: &'a Examples, id: &str) -> &'a Sample {
    examples
        .samples
        .iter()
        .find(|sample| sample.id == id)
        .or_else(|| examples.samples.first())
        .expect("hello world samples")
}

pub fn lesson_hosts() -> Vec<String> {
    content()
        .apis
        .resources
        .iter()
        .filter_map(|resource| https_host(&resource.url))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn validated_lesson_url(url: &str) -> Result<String, String> {
    let host = https_host(url).ok_or_else(|| "lesson URL must use https".to_string())?;
    if LESSON_HOSTS.iter().any(|allowed| host == *allowed) {
        Ok(url.to_string())
    } else {
        Err(format!("blocked lesson host {host}"))
    }
}

pub fn https_host(url: &str) -> Option<String> {
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

pub fn host_reachable(host: &str) -> bool {
    (host, 443u16)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .is_some()
}

pub fn offline_link_message(host: &str) -> String {
    format!("This link needs a network connection ({host})")
}

pub fn format_copy_toast(status: CopyStatus) -> &'static str {
    match status {
        CopyStatus::Copied => "Copied",
        CopyStatus::Denied => "Denied",
        CopyStatus::Failed => "Failed",
        CopyStatus::Unavailable => "Copy is unavailable in this build.",
    }
}

pub fn copy_code(text: &str) -> CopyStatus {
    match clipboard_win::set_clipboard(clipboard_win::formats::Unicode, text) {
        Ok(()) => CopyStatus::Copied,
        Err(error) => {
            let lowered = error.to_string().to_ascii_lowercase();
            if lowered.contains("denied") || lowered.contains("access") {
                CopyStatus::Denied
            } else {
                CopyStatus::Failed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parses() {
        let content = load().expect("workshop content");
        assert_eq!(content.manifest.schema_version, "1");
        assert_eq!(content.overview.id, "overview");
        assert_eq!(content.anatomy.files.len(), 4);
        assert_eq!(content.apis.resources.len(), 2);
        assert_eq!(content.examples.samples.len(), 3);
        assert!(content.anatomy.files[0].code.contains("{{interface}}"));
    }

    #[test]
    fn sku_interface_covers_all_five_slugs() {
        let content = load().unwrap();
        for slug in REQUIRED_SLUGS {
            let interface = content.manifest.sku_interface.get(slug).unwrap();
            assert!(!interface.is_empty(), "{slug}");
            assert_eq!(interface_for(slug), interface);
        }
    }

    #[test]
    fn copy_helper_formats() {
        assert_eq!(format_copy_toast(CopyStatus::Copied), "Copied");
        assert_eq!(format_copy_toast(CopyStatus::Denied), "Denied");
        assert_eq!(format_copy_toast(CopyStatus::Failed), "Failed");
        assert_eq!(
            format_copy_toast(CopyStatus::Unavailable),
            "Copy is unavailable in this build."
        );
    }

    #[test]
    fn http_lesson_links_are_rejected() {
        assert!(
            validated_lesson_url("http://warcraft.wiki.gg/wiki/World_of_Warcraft_API").is_err()
        );
        assert!(validated_lesson_url("javascript:alert(1)").is_err());
        assert!(validated_lesson_url("https://github.com/bitobrian/WinWam").is_err());
        assert!(
            validated_lesson_url("https://warcraft.wiki.gg/wiki/World_of_Warcraft_API").is_ok()
        );
        assert!(
            validated_lesson_url("https://us.forums.blizzard.com/en/wow/c/guides/ui-macro/35")
                .is_ok()
        );
    }

    #[test]
    fn placeholders_and_sku_fit_labels() {
        let content = load().unwrap();
        let toc = apply_placeholders(&content.anatomy.files[0].code, "classic", "Classic");
        assert!(toc.contains("## Interface: 11507"));
        assert!(!toc.contains("{{interface}}"));
        let login = &content.examples.samples[0];
        assert_eq!(sample_sku_status(login, "retail", "Retail"), "Fits Retail");
        let button = content
            .examples
            .samples
            .iter()
            .find(|sample| sample.id == "button")
            .unwrap();
        let status = sample_sku_status(button, "classic", "Classic");
        assert!(status.starts_with("Needs changes for Classic:"));
    }

    #[test]
    fn load_rejects_missing_sku_interface() {
        let mut manifest: serde_json::Value = serde_json::from_str(MANIFEST_JSON).unwrap();
        manifest["skuInterface"]
            .as_object_mut()
            .unwrap()
            .remove("forever");
        let error = load_from(
            &manifest.to_string(),
            OVERVIEW_JSON,
            ANATOMY_JSON,
            APIS_JSON,
            EXAMPLES_JSON,
        )
        .unwrap_err();
        assert!(error.contains("forever"));
    }
}
