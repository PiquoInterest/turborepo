use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};

use serde_json::Value;
use thiserror::Error;
use wax::{Glob, Program as _, walk::Entry as _};

use crate::{
    Json5Error, TurboRootOptions, clear_turbo_root_cache, get_turbo_root, parse_json5,
    root::read_regular_utf8_limited,
};

const MAX_PACKAGE_JSON_BYTES: usize = 8 * 1_024 * 1_024;
const MAX_WORKSPACE_YAML_BYTES: usize = 2 * 1_024 * 1_024;
const MAX_DISCOVERED_CONFIGS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigOptions {
    pub cache: bool,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self { cache: true }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceConfig {
    pub workspace_name: Option<String>,
    pub workspace_path: PathBuf,
    pub is_workspace_root: bool,
    pub turbo_config: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TurboConfig {
    pub config: Value,
    pub turbo_config_path: PathBuf,
    pub workspace_path: PathBuf,
    pub is_root_config: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurboConfigPathResolution {
    pub config_path: Option<PathBuf>,
    pub config_exists: bool,
    pub error: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TurboConfigError {
    #[error("Found both turbo.json and turbo.jsonc in the same directory: {directory}\nPlease use either turbo.json or turbo.jsonc, but not both.")]
    DuplicateConfig { directory: String },
}

type TurboConfigsCache = HashMap<PathBuf, Vec<TurboConfig>>;
type WorkspaceConfigsCache = HashMap<PathBuf, Vec<WorkspaceConfig>>;

fn turbo_configs_cache() -> &'static RwLock<TurboConfigsCache> {
    static CACHE: OnceLock<RwLock<TurboConfigsCache>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn workspace_configs_cache() -> &'static RwLock<WorkspaceConfigsCache> {
    static CACHE: OnceLock<RwLock<WorkspaceConfigsCache>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

#[must_use]
pub fn resolve_turbo_config_path(directory: &Path) -> TurboConfigPathResolution {
    let turbo_json = directory.join("turbo.json");
    let turbo_jsonc = directory.join("turbo.jsonc");
    let json_exists = path_exists(&turbo_json);
    let jsonc_exists = path_exists(&turbo_jsonc);

    if json_exists && jsonc_exists {
        return TurboConfigPathResolution {
            config_path: None,
            config_exists: false,
            error: Some(format!(
                "Found both turbo.json and turbo.jsonc in the same directory: {}\nPlease use either turbo.json or turbo.jsonc, but not both.",
                directory.display()
            )),
        };
    }
    if json_exists {
        return TurboConfigPathResolution {
            config_path: Some(turbo_json),
            config_exists: true,
            error: None,
        };
    }
    if jsonc_exists {
        return TurboConfigPathResolution {
            config_path: Some(turbo_jsonc),
            config_exists: true,
            error: None,
        };
    }
    TurboConfigPathResolution {
        config_path: None,
        config_exists: false,
        error: None,
    }
}
