use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use turbopath::AbsoluteSystemPathBuf;
use uuid::Uuid;

use super::{
    MAX_CONFIG_BYTES,
    sanitize::{checked_metadata, environment_flag, one_way_hash_with_salt},
    types::PackageTelemetryError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageTelemetryConfigContents {
    telemetry_enabled: bool,
    telemetry_id: String,
    telemetry_salt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    telemetry_alerted: Option<String>,
}

impl Default for PackageTelemetryConfigContents {
    fn default() -> Self {
        let telemetry_salt = Uuid::new_v4().to_string();
        let raw_telemetry_id = Uuid::new_v4().to_string();
        let telemetry_id = one_way_hash_with_salt(&telemetry_salt, &raw_telemetry_id);
        Self {
            telemetry_enabled: true,
            telemetry_id,
            telemetry_salt,
            telemetry_alerted: None,
        }
    }
}

#[derive(Debug)]
pub struct PackageTelemetryConfig {
    config_path: AbsoluteSystemPathBuf,
    config: PackageTelemetryConfigContents,
}

impl PackageTelemetryConfig {
    pub fn with_default_config_path() -> Result<Self, PackageTelemetryError> {
        let root = turborepo_dirs::config_dir()
            .map_err(|_| PackageTelemetryError::ConfigPath)?
            .ok_or(PackageTelemetryError::ConfigPath)?;
        Self::new(root.join_components(&["turborepo", "telemetry.json"]))
    }

    pub fn new(config_path: AbsoluteSystemPathBuf) -> Result<Self, PackageTelemetryError> {
        match read_config(&config_path) {
            Ok(Some(config)) => Ok(Self {
                config_path,
                config,
            }),
            Ok(None) => Self::create(config_path),
            Err(PackageTelemetryError::UnsafeConfigPath) => {
                Err(PackageTelemetryError::UnsafeConfigPath)
            }
            Err(_) => {
                remove_regular_file(&config_path)?;
                Self::create(config_path)
            }
        }
    }

    fn create(config_path: AbsoluteSystemPathBuf) -> Result<Self, PackageTelemetryError> {
        let config = PackageTelemetryConfigContents::default();
        write_config_atomically(&config_path, &config)?;
        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn try_write(&self) -> bool {
        write_config_atomically(&self.config_path, &self.config).is_ok()
    }

    pub fn has_seen_alert(&self) -> bool {
        self.config.telemetry_alerted.is_some()
    }

    pub fn is_enabled(&self) -> bool {
        if environment_flag("DO_NOT_TRACK") || environment_flag("TURBO_TELEMETRY_DISABLED") {
            return false;
        }
        self.config.telemetry_enabled
    }

    pub fn is_telemetry_warning_enabled() -> bool {
        !environment_flag("TURBO_TELEMETRY_MESSAGE_DISABLED")
    }

    pub fn is_debug() -> bool {
        environment_flag("TURBO_TELEMETRY_DEBUG")
    }

    pub fn id(&self) -> &str {
        &self.config.telemetry_id
    }

    pub fn enable(&mut self) -> bool {
        self.config.telemetry_enabled = true;
        self.try_write()
    }

    pub fn disable(&mut self) -> bool {
        self.config.telemetry_enabled = false;
        self.try_write()
    }

    pub fn alert_shown(&mut self) -> bool {
        if self.has_seen_alert() {
            return true;
        }
        self.config.telemetry_alerted = Some(chrono::Utc::now().to_rfc3339());
        self.try_write()
    }

    pub fn one_way_hash_value(&self, input: &str) -> String {
        one_way_hash_with_salt(&self.config.telemetry_salt, input)
    }
}

fn read_config(
    path: &AbsoluteSystemPathBuf,
) -> Result<Option<PackageTelemetryConfigContents>, PackageTelemetryError> {
    let metadata = match fs::symlink_metadata(path.as_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(PackageTelemetryError::ConfigIo(error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(PackageTelemetryError::UnsafeConfigPath);
    }
    if metadata.len() > MAX_CONFIG_BYTES {
        return Err(PackageTelemetryError::InvalidConfig);
    }

    let file = fs::File::open(path.as_path()).map_err(PackageTelemetryError::ConfigIo)?;
    let opened_metadata = file.metadata().map_err(PackageTelemetryError::ConfigIo)?;
    if !opened_metadata.is_file()
        || opened_metadata.len() > MAX_CONFIG_BYTES
        || !same_file(&metadata, &opened_metadata)
    {
        return Err(PackageTelemetryError::InvalidConfig);
    }
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(PackageTelemetryError::ConfigIo)?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(PackageTelemetryError::InvalidConfig);
    }
    let contents = std::str::from_utf8(&bytes).map_err(|_| PackageTelemetryError::InvalidConfig)?;
    let config: PackageTelemetryConfigContents =
        serde_json::from_str(contents).map_err(|_| PackageTelemetryError::InvalidConfig)?;
    validate_config(&config)?;
    Ok(Some(config))
}

fn validate_config(config: &PackageTelemetryConfigContents) -> Result<(), PackageTelemetryError> {
    checked_metadata(config.telemetry_id.clone())?;
    checked_metadata(config.telemetry_salt.clone())?;
    if let Some(alerted) = &config.telemetry_alerted {
        checked_metadata(alerted.clone())?;
    }
    Ok(())
}

fn remove_regular_file(path: &AbsoluteSystemPathBuf) -> Result<(), PackageTelemetryError> {
    match fs::symlink_metadata(path.as_path()) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(PackageTelemetryError::UnsafeConfigPath);
            }
            fs::remove_file(path.as_path()).map_err(PackageTelemetryError::ConfigIo)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PackageTelemetryError::ConfigIo(error)),
    }
}

fn write_config_atomically(
    path: &AbsoluteSystemPathBuf,
    config: &PackageTelemetryConfigContents,
) -> Result<(), PackageTelemetryError> {
    if let Ok(metadata) = fs::symlink_metadata(path.as_path()) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(PackageTelemetryError::UnsafeConfigPath);
        }
    }

    let parent = path
        .as_path()
        .parent()
        .ok_or(PackageTelemetryError::ConfigPath)?;
    ensure_safe_parent(parent)?;
    fs::create_dir_all(parent).map_err(PackageTelemetryError::ConfigIo)?;
    ensure_safe_parent(parent)?;

    let serialized = serde_json::to_vec_pretty(config)
        .map_err(|_| PackageTelemetryError::InvalidConfig)?;
    if serialized.len() as u64 > MAX_CONFIG_BYTES {
        return Err(PackageTelemetryError::InvalidConfig);
    }

    let temporary = temporary_path(path.as_path());
    let result = write_temporary_file(&temporary, &serialized)
        .and_then(|()| replace_file(&temporary, path.as_path()));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(PackageTelemetryError::ConfigIo)
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("telemetry.json");
    path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()))
}

fn write_temporary_file(path: &Path, contents: &[u8]) -> Result<(), std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(contents)?;
    file.sync_all()
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, target)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    if target.exists() {
        fs::remove_file(target)?;
    }
    fs::rename(temporary, target)
}

fn ensure_safe_parent(parent: &Path) -> Result<(), PackageTelemetryError> {
    for ancestor in parent.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(PackageTelemetryError::UnsafeConfigPath);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(PackageTelemetryError::ConfigIo(error)),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn same_file(before: &fs::Metadata, opened: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    before.dev() == opened.dev() && before.ino() == opened.ino()
}

#[cfg(not(unix))]
fn same_file(_before: &fs::Metadata, _opened: &fs::Metadata) -> bool {
    true
}
