use std::{
    env, fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("configured {tool} path must be absolute: {path}")]
    NotAbsolute {
        tool: &'static str,
        path: PathBuf,
    },
    #[error("configured {tool} path does not exist: {path}")]
    Missing {
        tool: &'static str,
        path: PathBuf,
    },
    #[error("configured {tool} path is not a regular file: {path}")]
    NotAFile {
        tool: &'static str,
        path: PathBuf,
    },
    #[error("configured {tool} path is not executable: {path}")]
    NotExecutable {
        tool: &'static str,
        path: PathBuf,
    },
    #[error("could not canonicalize {tool} path {path}: {source}")]
    Canonicalize {
        tool: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("trusted {tool} executable was not found; provide an explicit absolute path")]
    NotFound { tool: &'static str },
}

fn validate_tool(tool: &'static str, path: &Path) -> Result<PathBuf, ToolError> {
    if !path.exists() {
        return Err(ToolError::Missing {
            tool,
            path: path.to_path_buf(),
        });
    }

    let canonical = fs::canonicalize(path).map_err(|source| ToolError::Canonicalize {
        tool,
        path: path.to_path_buf(),
        source,
    })?;
    let metadata = fs::metadata(&canonical).map_err(|source| ToolError::Canonicalize {
        tool,
        path: canonical.clone(),
        source,
    })?;
    if !metadata.is_file() {
        return Err(ToolError::NotAFile {
            tool,
            path: canonical,
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(ToolError::NotExecutable {
                tool,
                path: canonical,
            });
        }
    }

    Ok(canonical)
}

fn first_valid(tool: &'static str, candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    for candidate in candidates {
        if let Ok(path) = validate_tool(tool, &candidate) {
            return Some(path);
        }
    }
    None
}

pub fn resolve_git(explicit: Option<&Path>) -> Result<PathBuf, ToolError> {
    if let Some(path) = explicit {
        if !path.is_absolute() {
            return Err(ToolError::NotAbsolute {
                tool: "git",
                path: path.to_path_buf(),
            });
        }
        return validate_tool("git", path);
    }

    #[cfg(unix)]
    let candidates = [PathBuf::from("/usr/bin/git"), PathBuf::from("/usr/local/bin/git")];

    #[cfg(windows)]
    let candidates = [
        PathBuf::from(r"C:\Program Files\Git\cmd\git.exe"),
        PathBuf::from(r"C:\Program Files\Git\bin\git.exe"),
    ];

    #[cfg(not(any(unix, windows)))]
    let candidates: [PathBuf; 0] = [];

    first_valid("git", candidates).ok_or(ToolError::NotFound { tool: "git" })
}

fn platform_turbo_package() -> Option<(&'static str, &'static str)> {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Some(("turbo-linux-64", "turbo")),
        ("linux", "aarch64") => Some(("turbo-linux-arm64", "turbo")),
        ("macos", "x86_64") => Some(("turbo-darwin-64", "turbo")),
        ("macos", "aarch64") => Some(("turbo-darwin-arm64", "turbo")),
        ("windows", "x86_64") => Some(("turbo-windows-64", "turbo.exe")),
        ("windows", "aarch64") => Some(("turbo-windows-arm64", "turbo.exe")),
        _ => None,
    }
}

fn pnpm_turbo_candidates(root: &Path, package: &str, binary: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let pnpm_root = root.join("node_modules").join(".pnpm");
    let Ok(entries) = fs::read_dir(pnpm_root) else {
        return candidates;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == package || name.starts_with(&format!("{package}@")) {
            candidates.push(
                entry
                    .path()
                    .join("node_modules")
                    .join(package)
                    .join("bin")
                    .join(binary),
            );
        }
    }
    candidates
}

pub fn resolve_turbo(root: &Path, explicit: Option<&Path>) -> Result<PathBuf, ToolError> {
    if let Some(path) = explicit {
        if !path.is_absolute() {
            return Err(ToolError::NotAbsolute {
                tool: "turbo",
                path: path.to_path_buf(),
            });
        }
        return validate_tool("turbo", path);
    }

    let mut candidates = vec![
        root.join("target").join("debug").join(if cfg!(windows) {
            "turbo.exe"
        } else {
            "turbo"
        }),
        root.join("target").join("release").join(if cfg!(windows) {
            "turbo.exe"
        } else {
            "turbo"
        }),
    ];

    if let Some((package, binary)) = platform_turbo_package() {
        candidates.push(
            root.join("node_modules")
                .join(package)
                .join("bin")
                .join(binary),
        );
        candidates.extend(pnpm_turbo_candidates(root, package, binary));
    }

    if !cfg!(windows) {
        candidates.push(root.join("node_modules").join(".bin").join("turbo"));
    }

    if let Ok(current_executable) = env::current_exe() {
        if let Some(parent) = current_executable.parent() {
            candidates.push(parent.join(if cfg!(windows) {
                "turbo.exe"
            } else {
                "turbo"
            }));
        }
    }

    first_valid("turbo", candidates).ok_or(ToolError::NotFound { tool: "turbo" })
}
