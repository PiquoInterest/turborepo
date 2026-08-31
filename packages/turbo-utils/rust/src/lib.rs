use std::{
    fmt,
    fs,
    io,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

#[cfg(unix)]
use std::ffi::CString;

const MAX_SEARCH_CONTENT_BYTES: u64 = 4 * 1_024 * 1_024;

const VALID_EMPTY_FOLDER_ENTRIES: [&str; 20] = [
    ".DS_Store",
    ".git",
    ".gitattributes",
    ".gitignore",
    ".gitlab-ci.yml",
    ".hg",
    ".hgcheck",
    ".hgignore",
    ".idea",
    ".npmignore",
    ".travis.yml",
    "LICENSE",
    "Thumbs.db",
    "docs",
    "mkdocs.yml",
    "npm-debug.log",
    "yarn-debug.log",
    "yarn-error.log",
    "yarnrc.yml",
    ".yarn",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    Camel,
    Pascal,
    Kebab,
    Snake,
}

impl fmt::Display for CaseStyle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Camel => "camel",
            Self::Pascal => "pascal",
            Self::Kebab => "kebab",
            Self::Snake => "snake",
        })
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConvertCaseError {
    #[error("case conversion to {0} is not implemented")]
    NotImplemented(CaseStyle),
}

/// Matches the current TypeScript implementation's ASCII camel-case rule.
///
/// Only `-` or `_` immediately followed by an ASCII lowercase letter is
/// replaced. Other case styles remain explicit errors, matching the source.
pub fn convert_case(input: &str, style: CaseStyle) -> Result<String, ConvertCaseError> {
    if style != CaseStyle::Camel {
        return Err(ConvertCaseError::NotImplemented(style));
    }

    let mut output = String::with_capacity(input.len());
    let mut characters = input.chars().peekable();
    while let Some(character) = characters.next() {
        if matches!(character, '-' | '_')
            && characters
                .peek()
                .is_some_and(|next| next.is_ascii_lowercase())
        {
            if let Some(next) = characters.next() {
                output.push(next.to_ascii_uppercase());
            }
        } else {
            output.push(character);
        }
    }
    Ok(output)
}

#[derive(Debug, Error)]
pub enum SearchUpError {
    #[error("search start must be an absolute directory: {0}")]
    RelativeStart(PathBuf),
    #[error("search target must be a non-empty relative path without parent components: {0}")]
    UnsafeTarget(PathBuf),
}

fn validate_search_target(target: &Path) -> Result<(), SearchUpError> {
    if target.as_os_str().is_empty() || target.is_absolute() {
        return Err(SearchUpError::UnsafeTarget(target.to_path_buf()));
    }
    if target.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(SearchUpError::UnsafeTarget(target.to_path_buf()));
    }
    Ok(())
}

/// Searches the current directory and each parent, excluding the filesystem
/// root, exactly like the TypeScript loop.
pub fn search_up(
    target: &Path,
    cwd: &Path,
    content_check: Option<&dyn Fn(&str) -> bool>,
) -> Result<Option<PathBuf>, SearchUpError> {
    validate_search_target(target)?;
    if !cwd.is_absolute() {
        return Err(SearchUpError::RelativeStart(cwd.to_path_buf()));
    }

    let mut current = cwd.to_path_buf();
    while current.parent().is_some() {
        let candidate = current.join(target);
        let found = if let Some(check) = content_check {
            fs::metadata(&candidate)
                .ok()
                .filter(|metadata| metadata.is_file() && metadata.len() <= MAX_SEARCH_CONTENT_BYTES)
                .and_then(|_| fs::read_to_string(&candidate).ok())
                .is_some_and(|content| check(&content))
        } else {
            candidate.exists()
        };

        if found {
            return Ok(Some(current));
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEmptyResult {
    pub is_empty: bool,
    pub conflicts: Vec<String>,
}

pub fn is_folder_empty(root: &Path) -> io::Result<FolderEmptyResult> {
    let mut conflicts = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !VALID_EMPTY_FOLDER_ENTRIES.contains(&name.as_str()) && !name.ends_with(".iml") {
            conflicts.push(name);
        }
    }
    Ok(FolderEmptyResult {
        is_empty: conflicts.is_empty(),
        conflicts,
    })
}

#[must_use]
pub fn is_writeable(directory: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;

        let Ok(path) = CString::new(directory.as_os_str().as_bytes()) else {
            return false;
        };
        // SAFETY: `path` is a NUL-terminated CString that remains alive for the
        // duration of the call. `access` does not retain the pointer.
        (unsafe { libc::access(path.as_ptr(), libc::W_OK) }) == 0
    }

    #[cfg(not(unix))]
    {
        fs::metadata(directory)
            .map(|metadata| metadata.is_dir() && !metadata.permissions().readonly())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryValidation {
    pub valid: bool,
    pub root: PathBuf,
    pub project_name: String,
    pub error: Option<String>,
}

fn lexical_resolve(directory: &str, current_directory: &Path) -> PathBuf {
    let path = Path::new(directory);
    let combined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_directory.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in combined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if resolved.file_name().is_some() {
                    let _removed = resolved.pop();
                }
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    resolved
}

fn invalid_directory(
    root: PathBuf,
    project_name: String,
    description: String,
) -> DirectoryValidation {
    DirectoryValidation {
        valid: false,
        root,
        project_name,
        error: Some(description),
    }
}

/// Validates a project directory using the current TypeScript contract for
/// normal inputs, while treating metadata/read failures as invalid rather than
/// allowing an uncertain filesystem state to continue.
#[must_use]
pub fn validate_directory(directory: &str, current_directory: &Path) -> DirectoryValidation {
    if directory.trim().is_empty() || directory.contains('\0') {
        return invalid_directory(
            PathBuf::new(),
            String::new(),
            format!(
                "{} is not a valid directory name - please try a different location",
                if directory.is_empty() { "<empty>" } else { directory }
            ),
        );
    }

    let root = lexical_resolve(directory, current_directory);
    let project_name = root
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let project_name_is_valid = !project_name.is_empty()
        && project_name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        });
    let root_text = root.to_string_lossy();
    if root_text.is_empty()
        || root_text.starts_with('-')
        || root_text.contains('\0')
        || !project_name_is_valid
    {
        return invalid_directory(
            root,
            project_name.clone(),
            format!(
                "{} is not a valid directory - please try a different location",
                if project_name.is_empty() {
                    "<unknown>"
                } else {
                    &project_name
                }
            ),
        );
    }

    match fs::symlink_metadata(&root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return invalid_directory(
                root,
                project_name.clone(),
                format!(
                    "{project_name} is not a directory - please try a different location"
                ),
            );
        }
        Ok(_) => match is_folder_empty(&root) {
            Ok(result) if !result.is_empty => {
                let count = result.conflicts.len();
                let noun = if count == 1 { "file" } else { "files" };
                return invalid_directory(
                    root.clone(),
                    project_name.clone(),
                    format!(
                        "{project_name} ({}) has {count} conflicting {noun} - please try a different location",
                        root.display()
                    ),
                );
            }
            Ok(_) => {}
            Err(error) => {
                return invalid_directory(
                    root,
                    project_name,
                    format!("directory could not be inspected safely: {error}"),
                );
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return invalid_directory(
                root,
                project_name,
                format!("directory metadata could not be read safely: {error}"),
            );
        }
    }

    DirectoryValidation {
        valid: true,
        root,
        project_name,
        error: None,
    }
}
