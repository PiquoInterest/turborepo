use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{TransformResponse, TransformStatus};

pub const GIT_IGNORE_TRANSFORM_NAME: &str = "git-ignore";
pub const DEFAULT_IGNORE: &str = r#"
# See https://help.github.com/articles/ignoring-files/ for more about ignoring files.

# dependencies
node_modules
.pnp
.pnp.js

# testing
coverage

# misc
.DS_Store
*.pem

# debug
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# turbo
.turbo

# vercel
.vercel
"#;

const TEMPORARY_FILE_ATTEMPTS: usize = 32;
static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum GitIgnoreError {
    Write(std::io::Error),
    UnsafeRoot,
    UnsafeIgnore,
    ConcurrentModification,
    TemporaryFileExhausted,
}

impl fmt::Display for GitIgnoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write(_) => formatter.write_str("Unable to write .gitignore"),
            Self::UnsafeRoot => formatter.write_str("project root is not a safe directory"),
            Self::UnsafeIgnore => formatter.write_str(".gitignore is not a safe path"),
            Self::ConcurrentModification => {
                formatter.write_str("project root changed while writing .gitignore")
            }
            Self::TemporaryFileExhausted => {
                formatter.write_str("unable to allocate a temporary .gitignore file")
            }
        }
    }
}

impl std::error::Error for GitIgnoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write(error) => Some(error),
            Self::UnsafeRoot
            | Self::UnsafeIgnore
            | Self::ConcurrentModification
            | Self::TemporaryFileExhausted => None,
        }
    }
}

pub fn create_git_ignore(root: &Path) -> Result<TransformResponse, GitIgnoreError> {
    let root_metadata = fs::symlink_metadata(root).map_err(GitIgnoreError::Write)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(GitIgnoreError::UnsafeRoot);
    }

    let ignore_path = root.join(".gitignore");
    match classify_existing_ignore(&ignore_path)? {
        ExistingIgnore::Absent => {}
        ExistingIgnore::Safe => return Ok(not_applicable()),
    }

    let (temporary_path, mut temporary_file) = create_temporary_file(root)?;
    let result = (|| {
        temporary_file
            .write_all(DEFAULT_IGNORE.as_bytes())
            .map_err(GitIgnoreError::Write)?;
        temporary_file.sync_all().map_err(GitIgnoreError::Write)?;
        drop(temporary_file);

        validate_root_identity(root, &root_metadata)?;
        match fs::hard_link(&temporary_path, &ignore_path) {
            Ok(()) => Ok(TransformResponse {
                result: TransformStatus::Success,
                name: GIT_IGNORE_TRANSFORM_NAME,
            }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                match classify_existing_ignore(&ignore_path)? {
                    ExistingIgnore::Safe => Ok(not_applicable()),
                    ExistingIgnore::Absent => Err(GitIgnoreError::ConcurrentModification),
                }
            }
            Err(error) => Err(GitIgnoreError::Write(error)),
        }
    })();

    let cleanup_result = remove_temporary_file(&temporary_path);
    match (result, cleanup_result) {
        (Ok(response), Ok(())) => Ok(response),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(GitIgnoreError::Write(error)),
    }
}

const fn not_applicable() -> TransformResponse {
    TransformResponse {
        result: TransformStatus::NotApplicable,
        name: GIT_IGNORE_TRANSFORM_NAME,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingIgnore {
    Absent,
    Safe,
}

fn classify_existing_ignore(path: &Path) -> Result<ExistingIgnore, GitIgnoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(GitIgnoreError::UnsafeIgnore),
        Ok(_) => Ok(ExistingIgnore::Safe),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ExistingIgnore::Absent),
        Err(error) => Err(GitIgnoreError::Write(error)),
    }
}

fn create_temporary_file(root: &Path) -> Result<(PathBuf, File), GitIgnoreError> {
    for _ in 0..TEMPORARY_FILE_ATTEMPTS {
        let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!(
            ".gitignore.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o666);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(GitIgnoreError::Write(error)),
        }
    }
    Err(GitIgnoreError::TemporaryFileExhausted)
}

fn validate_root_identity(root: &Path, expected: &fs::Metadata) -> Result<(), GitIgnoreError> {
    let current = fs::symlink_metadata(root).map_err(GitIgnoreError::Write)?;
    if current.file_type().is_symlink() || !current.is_dir() {
        return Err(GitIgnoreError::UnsafeRoot);
    }
    if !same_file(expected, &current) {
        return Err(GitIgnoreError::ConcurrentModification);
    }
    Ok(())
}

fn remove_temporary_file(path: &Path) -> Result<(), std::io::Error> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_left: &fs::Metadata, _right: &fs::Metadata) -> bool {
    true
}
