use std::{
    collections::HashMap,
    env, fs,
    io::Read as _,
    path::{Component, Path, PathBuf},
    sync::{OnceLock, RwLock},
};

use serde_json::Value;

use crate::{json5::MAX_JSON5_BYTES, parse_json5};

const TURBO_CONFIG_FILES: [&str; 2] = ["turbo.json", "turbo.jsonc"];
const MAX_PACKAGE_JSON_BYTES: usize = 8 * 1_024 * 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurboRootOptions {
    pub cache: bool,
}

impl Default for TurboRootOptions {
    fn default() -> Self {
        Self { cache: true }
    }
}

type RootCache = HashMap<PathBuf, PathBuf>;

fn root_cache() -> &'static RwLock<RootCache> {
    static CACHE: OnceLock<RwLock<RootCache>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub(crate) fn read_regular_utf8_limited(path: &Path, maximum: usize) -> Option<String> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > u64::try_from(maximum).ok()?
    {
        return None;
    }

    let file = fs::File::open(path).ok()?;
    let limit = u64::try_from(maximum).ok()?.saturating_add(1);
    let mut bytes = Vec::with_capacity(maximum.min(64 * 1_024));
    file.take(limit).read_to_end(&mut bytes).ok()?;
    if bytes.len() > maximum {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn lexical_absolute(path: &Path) -> Option<PathBuf> {
    let combined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().ok()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in combined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _removed = normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Some(normalized)
}

fn is_root_turbo_config(path: &Path) -> bool {
    read_regular_utf8_limited(path, MAX_JSON5_BYTES)
        .and_then(|content| parse_json5(&content).ok())
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|object| !object.contains_key("extends"))
}

fn package_json_value(directory: &Path) -> Option<Value> {
    read_regular_utf8_limited(&directory.join("package.json"), MAX_PACKAGE_JSON_BYTES)
        .and_then(|content| serde_json::from_str(&content).ok())
}

fn has_regular_file(directory: &Path, name: &str) -> bool {
    fs::symlink_metadata(directory.join(name))
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn is_monorepo_root(directory: &Path) -> bool {
    if has_regular_file(directory, "pnpm-workspace.yaml")
        || has_regular_file(directory, "lerna.json")
        || has_regular_file(directory, "rush.json")
    {
        return true;
    }

    package_json_value(directory)
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|object| {
            object.contains_key("workspaces")
                || object
                    .get("bolt")
                    .and_then(Value::as_object)
                    .is_some_and(|bolt| bolt.contains_key("workspaces"))
        })
}

fn find_uncached(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(directory) = current {
        if directory.parent().is_none() {
            break;
        }
        for filename in TURBO_CONFIG_FILES {
            if is_root_turbo_config(&directory.join(filename)) {
                return Some(directory.to_path_buf());
            }
        }
        current = directory.parent();
    }

    let mut current = Some(start);
    while let Some(directory) = current {
        if directory.parent().is_none() {
            break;
        }
        if is_monorepo_root(directory) {
            return Some(directory.to_path_buf());
        }
        current = directory.parent();
    }

    let mut current = Some(start);
    while let Some(directory) = current {
        if directory.parent().is_none() {
            break;
        }
        if has_regular_file(directory, "package.json") {
            return Some(directory.to_path_buf());
        }
        current = directory.parent();
    }

    None
}

/// Finds the Turborepo root from a directory or a not-yet-created descendant.
///
/// This preserves the TypeScript precedence: nearest root `turbo.json` or
/// `turbo.jsonc`, then a supported monorepo root, then the nearest package.
#[must_use]
pub fn get_turbo_root(cwd: Option<&Path>, options: TurboRootOptions) -> Option<PathBuf> {
    let requested = match cwd {
        Some(path) => path.to_path_buf(),
        None => env::current_dir().ok()?,
    };
    let start = lexical_absolute(&requested)?;

    if options.cache
        && let Ok(cache) = root_cache().read()
        && let Some(root) = cache.get(&start)
    {
        return Some(root.clone());
    }

    let root = find_uncached(&start);
    if options.cache
        && let Some(root) = root.as_ref()
        && let Ok(mut cache) = root_cache().write()
    {
        cache.insert(start, root.clone());
    }
    root
}

pub fn clear_turbo_root_cache() {
    if let Ok(mut cache) = root_cache().write() {
        cache.clear();
    }
}
