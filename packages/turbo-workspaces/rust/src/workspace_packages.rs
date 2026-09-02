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
    input: WorkspacePackages<'a>,
) -> Result<Vec<&'a str>, WorkspacePackagesError> {
    let workspace_globs: &'a [&'a str] = match input {
        WorkspacePackages::Missing => &[],
        WorkspacePackages::Array(workspace_globs) => workspace_globs,
        WorkspacePackages::Object { packages } => packages.unwrap_or(&[]),
    };

    if workspace_globs.len() > WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT {
        return Err(WorkspacePackagesError::TooManyGlobs {
            actual: workspace_globs.len(),
            limit: WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT,
        });
    }

    let mut total_bytes = 0usize;
    for (index, workspace_glob) in workspace_globs.iter().copied().enumerate() {
        let bytes = workspace_glob.len();
        if bytes > WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT {
            return Err(WorkspacePackagesError::GlobTooLarge {
                index,
                bytes,
                limit: WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT,
            });
        }

        let Some(next_total) = total_bytes.checked_add(bytes) else {
            return Err(WorkspacePackagesError::TotalInputTooLarge {
                bytes: usize::MAX,
                limit: WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT,
            });
        };
        total_bytes = next_total;
        if total_bytes > WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT {
            return Err(WorkspacePackagesError::TotalInputTooLarge {
                bytes: total_bytes,
                limit: WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT,
            });
        }

        if contains_unsafe_workspace_glob_text(workspace_glob) {
            return Err(WorkspacePackagesError::UnsafeGlobText { index });
        }
    }

    Ok(workspace_globs.to_vec())
}

fn contains_unsafe_workspace_glob_text(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(
            character,
            '\u{0000}'..='\u{001f}'
                | '\u{007f}'..='\u{009f}'
                | '\u{00ad}'
                | '\u{034f}'
                | '\u{061c}'
                | '\u{180e}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{202e}'
                | '\u{2060}'..='\u{206f}'
                | '\u{feff}'
                | '\u{fff9}'..='\u{fffb}'
        )
    })
}
