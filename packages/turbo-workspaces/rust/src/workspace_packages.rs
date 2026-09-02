use std::fmt;

pub const WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT: usize = 4_096;
pub const WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT: usize = 256;
pub const WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePackages<'a> {
    Missing,
    Array(&'a [&'a str]),
    Object { packages: Option<&'a [&'a str]> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePackagesError {
    TooManyGlobs { actual: usize, limit: usize },
    GlobTooLarge {
        index: usize,
        bytes: usize,
        limit: usize,
    },
    TotalInputTooLarge { bytes: usize, limit: usize },
    UnsafeGlobText { index: usize },
}

impl fmt::Display for WorkspacePackagesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyGlobs { limit, .. } => {
                write!(formatter, "workspace glob count exceeds {limit}")
            }
            Self::GlobTooLarge { limit, .. } => {
                write!(formatter, "workspace glob exceeds {limit} UTF-8 bytes")
            }
            Self::TotalInputTooLarge { limit, .. } => {
                write!(formatter, "workspace glob input exceeds {limit} UTF-8 bytes")
            }
            Self::UnsafeGlobText { .. } => {
                formatter.write_str("workspace glob contains unsafe text")
            }
        }
    }
}

impl std::error::Error for WorkspacePackagesError {}

pub fn parse_workspace_packages<'a>(
    _input: WorkspacePackages<'a>,
) -> Result<Vec<&'a str>, WorkspacePackagesError> {
    // Compiling behavioral RED: the final API and error contract exist, but
    // value extraction and input validation are intentionally absent.
    Ok(Vec::new())
}
