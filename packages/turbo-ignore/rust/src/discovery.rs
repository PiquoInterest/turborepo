use std::{
    fs,
    io::Read as _,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{Reporter, sanitize_for_log, top_level_keys};

const TURBO_CONFIG_FILES: [&str; 2] = ["turbo.json", "turbo.jsonc"];
const MAX_PACKAGE_JSON_BYTES: usize = 8 * 1_024 * 1_024;
const MAX_TURBO_CONFIG_BYTES: usize = 4 * 1_024 * 1_024;
const LOCKFILES: [&str; 5] = [
    "pnpm-lock.yaml",
    "package-lock.json",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
];

fn read_regular_utf8_limited(path: &Path, maximum: usize) -> Option<String> {
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

fn is_root_turbo_config(path: &Path) -> bool {
    read_regular_utf8_limited(path, MAX_TURBO_CONFIG_BYTES)
        .and_then(|content| top_level_keys(&content).ok())
        .is_some_and(|keys| !keys.iter().any(|key| key == "extends"))
}

fn package_json_is_workspace_root(path: &Path) -> bool {
    let Some(content) = read_regular_utf8_limited(path, MAX_PACKAGE_JSON_BYTES) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<Value>(&content) else {
        return false;
    };
    value
        .as_object()
        .is_some_and(|object| object.contains_key("workspaces"))
}

/// Mirrors `@turbo/utils#getTurboRoot`: prefer the nearest root turbo config,
/// then fall back to a package-manager/workspaces root.
#[must_use]
pub fn find_turbo_root(start: &Path) -> Option<PathBuf> {
    let start = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent()?.to_path_buf()
    };

    let mut current = Some(start.as_path());
    while let Some(directory) = current {
        for filename in TURBO_CONFIG_FILES {
            if is_root_turbo_config(&directory.join(filename)) {
                return Some(directory.to_path_buf());
            }
        }
        current = directory.parent();
    }

    let mut current = Some(start.as_path());
    while let Some(directory) = current {
        let package_root = package_json_is_workspace_root(&directory.join("package.json"));
        let has_lockfile = LOCKFILES
            .iter()
            .any(|filename| directory.join(filename).is_file());
        if package_root || has_lockfile {
            return Some(directory.to_path_buf());
        }
        current = directory.parent();
    }

    None
}

pub fn get_workspace(
    explicit: Option<&str>,
    directory: &Path,
    reporter: &dyn Reporter,
) -> Option<String> {
    if let Some(workspace) = explicit {
        reporter.info(&format!(
            "Using workspace \"{}\" from arguments",
            sanitize_for_log(workspace)
        ));
        return Some(workspace.to_owned());
    }

    let package_json_path = directory.join("package.json");
    let result = read_regular_utf8_limited(&package_json_path, MAX_PACKAGE_JSON_BYTES)
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .and_then(|value| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
        });

    match result {
        Some(workspace) => {
            reporter.info(&format!(
                "Inferred workspace \"{}\" from \"package.json\"",
                sanitize_for_log(&workspace)
            ));
            Some(workspace)
        }
        None => {
            reporter.error(&format!(
                "\"{}\" could not be read or has no string name. turbo-ignore workspace inference failed",
                sanitize_for_log(&package_json_path.display().to_string())
            ));
            None
        }
    }
}

fn package_turbo_version(value: &Value) -> Option<&str> {
    value
        .get("dependencies")
        .and_then(|dependencies| dependencies.get("turbo"))
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("devDependencies")
                .and_then(|dependencies| dependencies.get("turbo"))
                .and_then(Value::as_str)
        })
}

pub fn infer_turbo_version(
    explicit: Option<&str>,
    root: &Path,
    reporter: &dyn Reporter,
) -> Option<String> {
    if let Some(version) = explicit {
        reporter.info(&format!(
            "Using turbo version \"{}\" from arguments",
            sanitize_for_log(version)
        ));
        return Some(version.to_owned());
    }

    let package_json_path = root.join("package.json");
    let package_json = match read_regular_utf8_limited(&package_json_path, MAX_PACKAGE_JSON_BYTES)
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
    {
        Some(value) => value,
        None => {
            reporter.error(&format!(
                "\"{}\" could not be read. turbo-ignore turbo version inference failed",
                sanitize_for_log(&package_json_path.display().to_string())
            ));
            return None;
        }
    };

    if let Some(version) = package_turbo_version(&package_json) {
        if !version.starts_with("catalog:") {
            reporter.info(&format!(
                "Inferred turbo version \"{}\" from \"package.json\"",
                sanitize_for_log(version)
            ));
            return Some(version.to_owned());
        }
        reporter.warn(
            "Cannot infer turbo version due to use of `catalog` protocol. Remove `turbo` from your PNPM catalog to ensure correct turbo version is used",
        );
    }

    let turbo_json_path = root.join("turbo.json");
    let keys = match read_regular_utf8_limited(&turbo_json_path, MAX_TURBO_CONFIG_BYTES)
        .and_then(|content| top_level_keys(&content).ok())
    {
        Some(keys) => keys,
        None => {
            reporter.error(&format!(
                "\"{}\" could not be read. turbo-ignore turbo version inference failed",
                sanitize_for_log(&turbo_json_path.display().to_string())
            ));
            return None;
        }
    };

    if keys.iter().any(|key| key == "tasks") {
        reporter.info("Inferred turbo version ^2 based on \"tasks\" in \"turbo.json\"");
        return Some("^2".to_owned());
    }
    if keys.iter().any(|key| key == "pipeline") {
        reporter.info("Inferred turbo version ^1 based on \"pipeline\" in \"turbo.json\"");
        return Some("^1".to_owned());
    }

    None
}
