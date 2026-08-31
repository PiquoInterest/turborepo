use std::{fmt, path::Path};

pub const TRANSFORM_NAME: &str = "update-commands-in-readme";
pub const MAX_README_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

impl PackageManager {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformStatus {
    NotApplicable,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformResponse {
    pub result: TransformStatus,
    pub name: &'static str,
}

#[derive(Debug)]
pub enum TransformError {
    Read(std::io::Error),
    Write(std::io::Error),
    InvalidUtf8,
    ReadmeTooLarge,
    UnsafeRoot,
    UnsafeReadme,
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Read(_) => "unable to read README.md",
            Self::Write(_) => "unable to update README.md",
            Self::InvalidUtf8 => "README.md is not valid UTF-8",
            Self::ReadmeTooLarge => "README.md exceeds the size limit",
            Self::UnsafeRoot => "project root is not a safe directory",
            Self::UnsafeReadme => "README.md is not a safe regular file",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TransformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) | Self::Write(error) => Some(error),
            Self::InvalidUtf8
            | Self::ReadmeTooLarge
            | Self::UnsafeRoot
            | Self::UnsafeReadme => None,
        }
    }
}

pub fn replace_package_manager_references(
    _target: PackageManager,
    text: &str,
) -> Result<String, TransformError> {
    if text.len() > MAX_README_BYTES {
        return Err(TransformError::ReadmeTooLarge);
    }
    Ok(text.to_owned())
}

pub fn transform_readme(
    root: &Path,
    package_manager: Option<PackageManager>,
) -> Result<TransformResponse, TransformError> {
    if package_manager.is_none() {
        return Ok(TransformResponse {
            result: TransformStatus::NotApplicable,
            name: TRANSFORM_NAME,
        });
    }

    let readme = root.join("README.md");
    match std::fs::symlink_metadata(&readme) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            Ok(TransformResponse {
                result: TransformStatus::Success,
                name: TRANSFORM_NAME,
            })
        }
        Ok(_) => Err(TransformError::UnsafeReadme),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(TransformResponse {
            result: TransformStatus::NotApplicable,
            name: TRANSFORM_NAME,
        }),
        Err(error) => Err(TransformError::Read(error)),
    }
}
