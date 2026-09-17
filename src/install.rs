use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ureq::ResponseExt;
use zip::ZipArchive;
#[cfg(test)]
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::directory;

pub const MAX_DOWNLOAD_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_UNCOMPRESSED_BYTES: u64 = 250 * 1024 * 1024;
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(20);

const MARKER_FILE: &str = ".winwam-id";
const MANIFEST_FILE: &str = ".winwam-manifest.json";
const STAGING_DIR: &str = ".winwam-staging";
const BACKUP_DIR: &str = ".winwam-backup";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    MissingArchiveUrl,
    InvalidUrl(String),
    Download(String),
    ChecksumMismatch { expected: String, actual: String },
    ArchiveTooLarge,
    UncompressedTooLarge,
    UnsafePath(String),
    MissingToc { folder: String },
    FilesOutsideFolder,
    EmptyArchive,
    UnmanagedCollision { folder: String },
    Unmanaged,
    Downgrade { local: String, catalog: String },
    MissingAddonsFolder,
    UnknownAddon,
    Zip(String),
    Io(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingArchiveUrl => f.write_str("this addon has no downloadable archive"),
            Self::InvalidUrl(error) => write!(f, "archive URL is not allowed: {error}"),
            Self::Download(error) => write!(f, "could not download archive: {error}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "archive checksum mismatch (expected {expected}, got {actual})"
                )
            }
            Self::ArchiveTooLarge => f.write_str("archive exceeds the 100 MiB download limit"),
            Self::UncompressedTooLarge => {
                f.write_str("archive exceeds the 250 MiB uncompressed limit")
            }
            Self::UnsafePath(path) => write!(f, "archive contains an unsafe path: {path}"),
            Self::MissingToc { folder } => {
                write!(f, "owned folder {folder} does not contain a .toc file")
            }
            Self::FilesOutsideFolder => {
                f.write_str("archive files must live inside an addon folder")
            }
            Self::EmptyArchive => f.write_str("archive does not contain any addon folders"),
            Self::UnmanagedCollision { folder } => {
                write!(f, "refusing to overwrite unmanaged folder {folder}")
            }
            Self::Unmanaged => {
                f.write_str("WinWAM did not install this addon, so it will not remove it")
            }
            Self::Downgrade { local, catalog } => write!(
                f,
                "installed version {local} is newer than catalog version {catalog}"
            ),
            Self::MissingAddonsFolder => f.write_str("the AddOns folder for this SKU is missing"),
            Self::UnknownAddon => f.write_str("addon is not in the current directory"),
            Self::Zip(error) | Self::Io(error) => f.write_str(error),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<zip::result::ZipError> for InstallError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallPlan {
    pub addon_id: String,
    pub owned_folders: Vec<String>,
    pub files: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSource {
    pub host: String,
    pub owner: String,
    pub repo: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedManifest {
    pub addon_id: String,
    pub source: ManifestSource,
    pub release_version: String,
    pub owned_folders: Vec<String>,
    pub file_sha256: BTreeMap<String, String>,
    pub sku: String,
    pub installed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallContext {
    pub addons_folder: PathBuf,
    pub addon_id: String,
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub sku: String,
    pub release_version: String,
    pub archive_url: Option<String>,
    pub expected_sha256: Option<String>,
    pub local_archive: Option<PathBuf>,
    pub local_version: Option<String>,
    pub allow_downgrade: bool,
}

pub fn plan_install(archive: &Path, addon_id: &str) -> Result<InstallPlan, InstallError> {
    let file = File::open(archive)?;
    let mut zip = ZipArchive::new(file)?;
    let mut files = Vec::new();
    let mut folders = BTreeSet::new();
    let mut toc_folders = BTreeSet::new();
    let mut uncompressed = 0u64;

    for index in 0..zip.len() {
        let entry = zip.by_index(index)?;
        let raw_name = entry.name().replace('\\', "/");
        if raw_name.is_empty() {
            continue;
        }
        if entry.is_symlink() {
            return Err(InstallError::UnsafePath(raw_name));
        }
        if has_parent_dir(&raw_name) {
            return Err(InstallError::UnsafePath(raw_name));
        }
        let Some(relative) = entry.enclosed_name() else {
            return Err(InstallError::UnsafePath(raw_name));
        };
        let dest_rel = path_to_rel(&relative)?;
        if dest_rel.is_empty() {
            continue;
        }
        let mut components = dest_rel.split('/');
        let Some(top) = components.next() else {
            continue;
        };
        if components.next().is_none() && !entry.is_dir() {
            return Err(InstallError::FilesOutsideFolder);
        }
        folders.insert(top.to_string());
        if entry.is_dir() {
            continue;
        }
        uncompressed = uncompressed
            .checked_add(entry.size())
            .ok_or(InstallError::UncompressedTooLarge)?;
        if uncompressed > MAX_UNCOMPRESSED_BYTES {
            return Err(InstallError::UncompressedTooLarge);
        }
        if dest_rel.rsplit('/').next().is_some_and(|name| {
            Path::new(name)
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("toc"))
        }) {
            toc_folders.insert(top.to_string());
        }
        if is_internal_marker(&dest_rel) {
            continue;
        }
        files.push((raw_name, dest_rel));
    }

    if folders.is_empty() {
        return Err(InstallError::EmptyArchive);
    }
    for folder in &folders {
        if !toc_folders.contains(folder) {
            return Err(InstallError::MissingToc {
                folder: folder.clone(),
            });
        }
    }

    Ok(InstallPlan {
        addon_id: addon_id.to_string(),
        owned_folders: folders.into_iter().collect(),
        files,
    })
}

pub fn install_addon(ctx: &InstallContext) -> Result<ManagedManifest, InstallError> {
    if !ctx.addons_folder.is_dir() {
        return Err(InstallError::MissingAddonsFolder);
    }
    refuse_downgrade(ctx)?;

    let download_dir = tempfile::Builder::new()
        .prefix("winwam-dl-")
        .tempdir()
        .map_err(|error| InstallError::Io(error.to_string()))?;
    let archive_path = resolve_archive(ctx, download_dir.path())?;
    let plan = plan_install(&archive_path, &ctx.addon_id)?;
    for folder in &plan.owned_folders {
        let dest = ctx.addons_folder.join(folder);
        if dest.exists() && !is_owned_by(&dest, &ctx.addon_id) {
            return Err(InstallError::UnmanagedCollision {
                folder: folder.clone(),
            });
        }
    }

    let run_id = unique_run_id(&ctx.addon_id);
    let staging_root = ctx.addons_folder.join(STAGING_DIR).join(&run_id);
    let backup_root = ctx.addons_folder.join(BACKUP_DIR).join(&run_id);
    fs::create_dir_all(&staging_root)?;
    let extract_result = extract_plan(&archive_path, &plan, &staging_root).and_then(|hashes| {
        commit_swap(&ctx.addons_folder, &plan, &staging_root, &backup_root)?;
        Ok(hashes)
    });
    match extract_result {
        Ok(file_sha256) => {
            let manifest = ManagedManifest {
                addon_id: ctx.addon_id.clone(),
                source: ManifestSource {
                    host: ctx.host.clone(),
                    owner: ctx.owner.clone(),
                    repo: ctx.repo.clone(),
                },
                release_version: ctx.release_version.clone(),
                owned_folders: plan.owned_folders.clone(),
                file_sha256,
                sku: ctx.sku.clone(),
                installed_at: format_utc_now(),
            };
            write_ownership(&ctx.addons_folder, &manifest)?;
            let _ = fs::remove_dir_all(&staging_root);
            let _ = fs::remove_dir_all(&backup_root);
            Ok(manifest)
        }
        Err(error) => {
            restore_backups(&ctx.addons_folder, &plan, &backup_root);
            let _ = fs::remove_dir_all(&staging_root);
            let _ = fs::remove_dir_all(&backup_root);
            Err(error)
        }
    }
}

pub fn uninstall_addon(addons_folder: &Path, addon_id: &str) -> Result<Vec<String>, InstallError> {
    if !addons_folder.is_dir() {
        return Err(InstallError::MissingAddonsFolder);
    }
    let owned = owned_folders_for(addons_folder, addon_id)?;
    if owned.is_empty() {
        return Err(InstallError::Unmanaged);
    }
    for folder in &owned {
        let dest = addons_folder.join(folder);
        if dest.is_dir() {
            fs::remove_dir_all(dest)?;
        }
    }
    Ok(owned)
}

pub fn version_cmp(local: &str, catalog: &str) -> Option<Ordering> {
    Some(parse_dotted_version(local)?.cmp(&parse_dotted_version(catalog)?))
}

pub fn context_from_addon(
    addons_folder: PathBuf,
    addon: &directory::Addon,
    sku: &str,
    local_version: Option<String>,
    local_archive: Option<PathBuf>,
) -> InstallContext {
    let release = addon.latest_release.as_ref();
    InstallContext {
        addons_folder,
        addon_id: addon.id.clone(),
        host: addon.host.clone(),
        owner: addon.owner.clone(),
        repo: addon.repo.clone(),
        sku: sku.to_string(),
        release_version: release
            .map(|item| item.version.clone())
            .or_else(|| addon.version.clone())
            .unwrap_or_default(),
        archive_url: release.map(|item| item.archive_url.clone()),
        expected_sha256: release.map(|item| item.sha256.clone()),
        local_archive,
        local_version,
        allow_downgrade: false,
    }
}

#[cfg(debug_assertions)]
pub fn fixture_package(addon_id: &str) -> Option<PathBuf> {
    let file_name = format!("{addon_id}.zip");
    let mut candidates = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("fixtures")
            .join("packages")
            .join(&file_name),
        PathBuf::from("data")
            .join("fixtures")
            .join("packages")
            .join(&file_name),
    ];
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        candidates.push(
            parent
                .join("data")
                .join("fixtures")
                .join("packages")
                .join(&file_name),
        );
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn refuse_downgrade(ctx: &InstallContext) -> Result<(), InstallError> {
    if ctx.allow_downgrade {
        return Ok(());
    }
    let Some(local) = ctx
        .local_version
        .as_deref()
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    if ctx.release_version.is_empty() {
        return Ok(());
    }
    if version_cmp(local, &ctx.release_version) == Some(Ordering::Greater) {
        return Err(InstallError::Downgrade {
            local: local.to_string(),
            catalog: ctx.release_version.clone(),
        });
    }
    Ok(())
}

fn resolve_archive(ctx: &InstallContext, download_dir: &Path) -> Result<PathBuf, InstallError> {
    if let Some(path) = &ctx.local_archive {
        verify_checksum(path, ctx.expected_sha256.as_deref())?;
        return Ok(path.clone());
    }

    #[cfg(debug_assertions)]
    if let Some(path) = fixture_package(&ctx.addon_id) {
        return Ok(path);
    }

    let Some(url) = ctx.archive_url.as_deref() else {
        return Err(InstallError::MissingArchiveUrl);
    };
    let dest = download_dir.join("archive.zip");
    download_archive(url, &dest)?;
    verify_checksum(&dest, ctx.expected_sha256.as_deref())?;
    Ok(dest)
}

fn download_archive(url: &str, dest: &Path) -> Result<(), InstallError> {
    let url = directory::validated_https_url(url).map_err(InstallError::InvalidUrl)?;
    match download_once(&url, dest) {
        Ok(()) => Ok(()),
        Err(_) => download_once(&url, dest),
    }
}

fn download_once(url: &str, dest: &Path) -> Result<(), InstallError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| InstallError::Download(error.to_string()))?;
    let final_url = response.get_uri().to_string();
    if !final_url.starts_with("https://") {
        return Err(InstallError::InvalidUrl(
            "download redirected away from HTTPS".to_string(),
        ));
    }
    let mut reader = response.body_mut().as_reader();
    let mut file = File::create(dest)?;
    let mut total = 0u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| InstallError::Download(error.to_string()))?;
        if read == 0 {
            break;
        }
        total += read as u64;
        if total > MAX_DOWNLOAD_BYTES {
            return Err(InstallError::ArchiveTooLarge);
        }
        file.write_all(&buffer[..read])?;
    }
    Ok(())
}

fn verify_checksum(path: &Path, expected: Option<&str>) -> Result<(), InstallError> {
    let Some(expected) = expected.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let actual = sha256_file(path)?;
    if actual != expected.to_ascii_lowercase() {
        return Err(InstallError::ChecksumMismatch {
            expected: expected.to_ascii_lowercase(),
            actual,
        });
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, InstallError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn extract_plan(
    archive: &Path,
    plan: &InstallPlan,
    staging_root: &Path,
) -> Result<BTreeMap<String, String>, InstallError> {
    let file = File::open(archive)?;
    let mut zip = ZipArchive::new(file)?;
    let wanted: BTreeMap<&str, &str> = plan
        .files
        .iter()
        .map(|(zip_path, dest)| (zip_path.as_str(), dest.as_str()))
        .collect();
    let mut hashes = BTreeMap::new();
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let raw_name = entry.name().replace('\\', "/");
        let Some(dest_rel) = wanted.get(raw_name.as_str()) else {
            continue;
        };
        let dest = staging_root.join(Path::new(dest_rel));
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(&dest)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let read = entry.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
            output.write_all(&buffer[..read])?;
        }
        hashes.insert((*dest_rel).to_string(), hex::encode(hasher.finalize()));
    }
    Ok(hashes)
}

fn commit_swap(
    addons_folder: &Path,
    plan: &InstallPlan,
    staging_root: &Path,
    backup_root: &Path,
) -> Result<(), InstallError> {
    fs::create_dir_all(backup_root)?;
    let mut backed_up = Vec::new();
    let mut placed = Vec::new();
    let result = (|| {
        for folder in &plan.owned_folders {
            let dest = addons_folder.join(folder);
            let staged = staging_root.join(folder);
            if !staged.is_dir() {
                return Err(InstallError::Io(format!(
                    "staged folder {folder} is missing"
                )));
            }
            if dest.exists() {
                let backup = backup_root.join(folder);
                if backup.exists() {
                    fs::remove_dir_all(&backup)?;
                }
                fs::rename(&dest, &backup)?;
                backed_up.push(folder.clone());
            }
            fs::rename(&staged, &dest)?;
            placed.push(folder.clone());
        }
        Ok(())
    })();
    if result.is_err() {
        for folder in placed.iter().rev() {
            let dest = addons_folder.join(folder);
            if dest.exists() {
                let _ = fs::remove_dir_all(&dest);
            }
        }
        for folder in backed_up {
            let backup = backup_root.join(&folder);
            let dest = addons_folder.join(&folder);
            if backup.exists() {
                let _ = fs::rename(backup, dest);
            }
        }
    }
    result
}

fn restore_backups(addons_folder: &Path, plan: &InstallPlan, backup_root: &Path) {
    for folder in &plan.owned_folders {
        let backup = backup_root.join(folder);
        if !backup.exists() {
            continue;
        }
        let dest = addons_folder.join(folder);
        if dest.exists() {
            let _ = fs::remove_dir_all(&dest);
        }
        let _ = fs::rename(backup, dest);
    }
}

fn write_ownership(addons_folder: &Path, manifest: &ManagedManifest) -> Result<(), InstallError> {
    let body =
        serde_json::to_vec_pretty(manifest).map_err(|error| InstallError::Io(error.to_string()))?;
    for folder in &manifest.owned_folders {
        let dest = addons_folder.join(folder);
        fs::write(dest.join(MARKER_FILE), &manifest.addon_id)?;
        fs::write(dest.join(MANIFEST_FILE), &body)?;
    }
    Ok(())
}

fn owned_folders_for(addons_folder: &Path, addon_id: &str) -> Result<Vec<String>, InstallError> {
    let mut from_manifest = None;
    let mut from_marker = Vec::new();
    let entries = match fs::read_dir(addons_folder) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(folder) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if folder.starts_with('.') {
            continue;
        }
        if let Some(manifest) = read_manifest(&path)
            && manifest.addon_id == addon_id
        {
            from_manifest = Some(manifest.owned_folders);
            break;
        }
        if let Ok(marker) = fs::read_to_string(path.join(MARKER_FILE))
            && marker.trim() == addon_id
        {
            from_marker.push(folder.to_string());
        }
    }
    if from_manifest.is_none()
        && from_marker.is_empty()
        && let Some(path) = crate::scan::find_addon_directory(addons_folder, addon_id)
        && is_owned_by(&path, addon_id)
        && let Some(folder) = path.file_name().and_then(|name| name.to_str())
    {
        from_marker.push(folder.to_string());
    }
    Ok(from_manifest.unwrap_or(from_marker))
}

fn read_manifest(folder: &Path) -> Option<ManagedManifest> {
    let body = fs::read_to_string(folder.join(MANIFEST_FILE)).ok()?;
    serde_json::from_str(&body).ok()
}

fn is_owned_by(folder: &Path, addon_id: &str) -> bool {
    if let Some(manifest) = read_manifest(folder)
        && manifest.addon_id == addon_id
    {
        return true;
    }
    fs::read_to_string(folder.join(MARKER_FILE))
        .ok()
        .is_some_and(|marker| marker.trim() == addon_id)
}

fn path_to_rel(path: &Path) -> Result<String, InstallError> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let name = part
                    .to_str()
                    .ok_or_else(|| InstallError::UnsafePath(path.display().to_string()))?;
                if is_reserved_device(name) || !is_safe_component(name) {
                    return Err(InstallError::UnsafePath(name.to_string()));
                }
                parts.push(name);
            }
            Component::CurDir => {}
            _ => return Err(InstallError::UnsafePath(path.display().to_string())),
        }
    }
    Ok(parts.join("/"))
}

fn has_parent_dir(name: &str) -> bool {
    name.split(['/', '\\']).any(|part| part == "..")
}

fn is_internal_marker(dest_rel: &str) -> bool {
    dest_rel
        .rsplit('/')
        .next()
        .is_some_and(|name| name == MARKER_FILE || name == MANIFEST_FILE)
}

fn is_safe_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.ends_with(' ')
        && !name.ends_with('.')
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' '))
}

fn is_reserved_device(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end_matches([' ', '.']);
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM0"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT0"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn parse_dotted_version(value: &str) -> Option<Vec<u64>> {
    let value = value.trim().trim_start_matches(['v', 'V']);
    let main = value.split(['-', '+']).next()?.trim();
    if main.is_empty() {
        return None;
    }
    main.split('.')
        .map(|part| part.parse().ok())
        .collect::<Option<Vec<u64>>>()
        .filter(|parts| !parts.is_empty())
}

fn unique_run_id(addon_id: &str) -> String {
    let safe: String = addon_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{safe}-{nanos}")
}

fn format_utc_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format_utc(secs)
}

fn format_utc(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (year, month, day) = civil_from_days(days);
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let second = rem % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
    let year = year + i32::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
fn write_stored_zip(path: &Path, files: &[(&str, &[u8])]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, body) in files {
        zip.start_file(*name, options)?;
        zip.write_all(body)?;
    }
    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "winwam-install-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("temp root");
        path
    }

    fn sample_toc(name: &str, version: &str) -> String {
        format!("## Title: {name}\n## Version: {version}\n## Interface: 120000\n{name}.lua\n")
    }

    fn ctx(addons: PathBuf, archive: PathBuf, version: &str) -> InstallContext {
        InstallContext {
            addons_folder: addons,
            addon_id: "arcane-alerts".into(),
            host: "github.com".into(),
            owner: "winwam-samples".into(),
            repo: "arcane-alerts".into(),
            sku: "retail".into(),
            release_version: version.into(),
            archive_url: None,
            expected_sha256: None,
            local_archive: Some(archive),
            local_version: None,
            allow_downgrade: false,
        }
    }

    #[test]
    fn plan_install_accepts_single_folder() {
        let root = temp_root("single-plan");
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[
                (
                    "ArcaneAlerts/ArcaneAlerts.toc",
                    sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
                ),
                ("ArcaneAlerts/ArcaneAlerts.lua", b"print('hi')\n"),
            ],
        )
        .unwrap();
        let plan = plan_install(&zip_path, "arcane-alerts").unwrap();
        fs::remove_dir_all(&root).ok();
        assert_eq!(plan.owned_folders, vec!["ArcaneAlerts".to_string()]);
        assert_eq!(plan.files.len(), 2);
    }

    #[test]
    fn plan_install_accepts_multi_folder() {
        let root = temp_root("multi-plan");
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[
                (
                    "ArcaneAlerts/ArcaneAlerts.toc",
                    sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
                ),
                (
                    "ArcaneAlerts_Options/ArcaneAlerts_Options.toc",
                    sample_toc("ArcaneAlerts_Options", "1.0.0").as_bytes(),
                ),
            ],
        )
        .unwrap();
        let plan = plan_install(&zip_path, "arcane-alerts").unwrap();
        fs::remove_dir_all(&root).ok();
        assert_eq!(
            plan.owned_folders,
            vec![
                "ArcaneAlerts".to_string(),
                "ArcaneAlerts_Options".to_string()
            ]
        );
    }

    #[test]
    fn path_traversal_is_rejected() {
        let root = temp_root("traversal");
        let zip_path = root.join("evil.zip");
        write_stored_zip(
            &zip_path,
            &[("../Evil/Evil.toc", sample_toc("Evil", "1.0.0").as_bytes())],
        )
        .unwrap();
        let error = plan_install(&zip_path, "evil").unwrap_err();
        fs::remove_dir_all(&root).ok();
        assert!(matches!(
            error,
            InstallError::UnsafePath(_) | InstallError::FilesOutsideFolder
        ));
    }

    #[test]
    fn install_single_folder_writes_markers() {
        let root = temp_root("install-single");
        let addons = root.join("AddOns");
        fs::create_dir_all(&addons).unwrap();
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[
                (
                    "ArcaneAlerts/ArcaneAlerts.toc",
                    sample_toc("ArcaneAlerts", "1.2.0").as_bytes(),
                ),
                ("ArcaneAlerts/ArcaneAlerts.lua", b"-- alerts\n"),
            ],
        )
        .unwrap();
        let manifest = install_addon(&ctx(addons.clone(), zip_path, "1.2.0")).unwrap();
        let dest = addons.join("ArcaneAlerts");
        assert_eq!(
            fs::read_to_string(dest.join(MARKER_FILE)).unwrap().trim(),
            "arcane-alerts"
        );
        let loaded: ManagedManifest =
            serde_json::from_str(&fs::read_to_string(dest.join(MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(loaded.addon_id, "arcane-alerts");
        assert_eq!(loaded.release_version, "1.2.0");
        assert_eq!(manifest.owned_folders, vec!["ArcaneAlerts".to_string()]);
        assert!(dest.join("ArcaneAlerts.lua").is_file());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn install_multi_folder_and_uninstall_owned_only() {
        let root = temp_root("install-multi");
        let addons = root.join("AddOns");
        fs::create_dir_all(&addons).unwrap();
        let foreign = addons.join("KeepMe");
        fs::create_dir_all(&foreign).unwrap();
        fs::write(foreign.join("KeepMe.toc"), sample_toc("KeepMe", "1.0.0")).unwrap();
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[
                (
                    "ArcaneAlerts/ArcaneAlerts.toc",
                    sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
                ),
                (
                    "ArcaneAlerts_Options/ArcaneAlerts_Options.toc",
                    sample_toc("ArcaneAlerts_Options", "1.0.0").as_bytes(),
                ),
            ],
        )
        .unwrap();
        install_addon(&ctx(addons.clone(), zip_path, "1.0.0")).unwrap();
        let removed = uninstall_addon(&addons, "arcane-alerts").unwrap();
        assert!(removed.contains(&"ArcaneAlerts".to_string()));
        assert!(removed.contains(&"ArcaneAlerts_Options".to_string()));
        assert!(!addons.join("ArcaneAlerts").exists());
        assert!(foreign.is_dir());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn checksum_mismatch_is_rejected() {
        let root = temp_root("checksum");
        let addons = root.join("AddOns");
        fs::create_dir_all(&addons).unwrap();
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[(
                "ArcaneAlerts/ArcaneAlerts.toc",
                sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
            )],
        )
        .unwrap();
        let mut context = ctx(addons, zip_path, "1.0.0");
        context.expected_sha256 = Some("ab".repeat(32));
        let error = install_addon(&context).unwrap_err();
        fs::remove_dir_all(&root).ok();
        assert!(matches!(error, InstallError::ChecksumMismatch { .. }));
    }

    #[test]
    fn unmanaged_collision_refuses_overwrite() {
        let root = temp_root("collision");
        let addons = root.join("AddOns");
        let existing = addons.join("ArcaneAlerts");
        fs::create_dir_all(&existing).unwrap();
        fs::write(
            existing.join("ArcaneAlerts.toc"),
            sample_toc("ArcaneAlerts", "0.1.0"),
        )
        .unwrap();
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[(
                "ArcaneAlerts/ArcaneAlerts.toc",
                sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
            )],
        )
        .unwrap();
        let error = install_addon(&ctx(addons, zip_path, "1.0.0")).unwrap_err();
        fs::remove_dir_all(&root).ok();
        assert!(matches!(
            error,
            InstallError::UnmanagedCollision { folder } if folder == "ArcaneAlerts"
        ));
    }

    #[test]
    fn rollback_restores_managed_folder_on_swap_failure() {
        let root = temp_root("rollback");
        let addons = root.join("AddOns");
        let existing = addons.join("ArcaneAlerts");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join(MARKER_FILE), "arcane-alerts").unwrap();
        fs::write(existing.join("sentinel.txt"), "keep-me").unwrap();
        let staging = addons.join(STAGING_DIR).join("run");
        fs::create_dir_all(staging.join("ArcaneAlerts")).unwrap();
        fs::write(staging.join("ArcaneAlerts").join("new.txt"), "new").unwrap();
        let backup = addons.join(BACKUP_DIR).join("run");
        let plan = InstallPlan {
            addon_id: "arcane-alerts".into(),
            owned_folders: vec!["ArcaneAlerts".into(), "MissingFolder".into()],
            files: Vec::new(),
        };
        let error = commit_swap(&addons, &plan, &staging, &backup).unwrap_err();
        assert!(matches!(error, InstallError::Io(_)));
        assert_eq!(
            fs::read_to_string(existing.join("sentinel.txt")).unwrap(),
            "keep-me"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn uninstall_without_marker_is_unmanaged() {
        let root = temp_root("unmanaged");
        let addons = root.join("AddOns");
        let existing = addons.join("LocalUI");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("LocalUI.toc"), sample_toc("LocalUI", "1.0.0")).unwrap();
        let error = uninstall_addon(&addons, "local-ui").unwrap_err();
        fs::remove_dir_all(&root).ok();
        assert_eq!(error, InstallError::Unmanaged);
    }

    #[test]
    fn downgrade_is_refused_when_local_semver_is_newer() {
        let root = temp_root("downgrade");
        let addons = root.join("AddOns");
        fs::create_dir_all(&addons).unwrap();
        let zip_path = root.join("addon.zip");
        write_stored_zip(
            &zip_path,
            &[(
                "ArcaneAlerts/ArcaneAlerts.toc",
                sample_toc("ArcaneAlerts", "1.0.0").as_bytes(),
            )],
        )
        .unwrap();
        let mut context = ctx(addons, zip_path, "1.0.0");
        context.local_version = Some("2.0.0".into());
        let error = install_addon(&context).unwrap_err();
        fs::remove_dir_all(&root).ok();
        assert!(matches!(error, InstallError::Downgrade { .. }));
        assert_eq!(version_cmp("2.0.0", "1.2.0"), Some(Ordering::Greater));
        assert_eq!(version_cmp("1.2.0", "1.2.0"), Some(Ordering::Equal));
    }
}
