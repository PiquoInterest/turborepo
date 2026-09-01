use std::{
    fmt,
    path::{Path, PathBuf},
};

pub const MANAGER_DETECTION_ORDER: [WorkspaceManager; 6] = [
    WorkspaceManager::Aube,
    WorkspaceManager::Nub,
    WorkspaceManager::Pnpm,
    WorkspaceManager::Yarn,
    WorkspaceManager::Npm,
    WorkspaceManager::Bun,
];

pub const BUN_WORKSPACE_GLOB_INPUT_LIMIT: usize = 4_096;
pub const BUN_WORKSPACE_GLOB_COUNT_LIMIT: usize = 256;
pub const BUN_WORKSPACE_GLOB_TOTAL_INPUT_LIMIT: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceManager {
    Aube,
    Nub,
    Pnpm,
    Yarn,
    Npm,
    Bun,
}

impl WorkspaceManager {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aube => "aube",
            Self::Nub => "nub",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Npm => "npm",
            Self::Bun => "bun",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryInfo {
    pub absolute: PathBuf,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDetailsKnownError {
    InvalidDirectory { absolute: PathBuf },
    UnableToDetect,
}

impl WorkspaceDetailsKnownError {
    #[must_use]
    pub const fn error_type(&self) -> &'static str {
        match self {
            Self::InvalidDirectory { .. } => "invalid_directory",
            Self::UnableToDetect => "package_manager-unable_to_detect",
        }
    }

    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::InvalidDirectory { absolute } => format!(
                "Could not find directory at {}. Ensure the directory exists.",
                absolute.display()
            ),
            Self::UnableToDetect => "Could not determine package manager. Add \
                                     `devEngines.packageManager` or legacy `packageManager` to \
                                     `package.json`, or ensure a lockfile is present."
                .to_owned(),
        }
    }
}

impl fmt::Display for WorkspaceDetailsKnownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDetailsError<E> {
    Known(WorkspaceDetailsKnownError),
    Provider(E),
}

impl<E: fmt::Display> fmt::Display for WorkspaceDetailsError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(error) => error.fmt(formatter),
            Self::Provider(error) => error.fmt(formatter),
        }
    }
}

pub trait WorkspaceDetailsProvider {
    type Project;
    type Error;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error>;
    fn detect(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<bool, Self::Error>;
    fn read(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<Self::Project, Self::Error>;
}

pub fn get_workspace_details<P>(
    root: &Path,
    provider: &mut P,
) -> Result<P::Project, WorkspaceDetailsError<P::Error>>
where
    P: WorkspaceDetailsProvider,
{
    let WorkspaceDirectoryInfo {
        absolute: workspace_root,
        exists,
    } = provider
        .directory_info(root)
        .map_err(WorkspaceDetailsError::Provider)?;

    if !exists {
        return Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::InvalidDirectory {
                absolute: workspace_root,
            },
        ));
    }

    for manager in MANAGER_DETECTION_ORDER {
        let detected = provider
            .detect(manager, &workspace_root)
            .map_err(WorkspaceDetailsError::Provider)?;
        if detected {
            return provider
                .read(manager, &workspace_root)
                .map_err(WorkspaceDetailsError::Provider);
        }
    }

    Err(WorkspaceDetailsError::Known(
        WorkspaceDetailsKnownError::UnableToDetect,
    ))
}

#[must_use]
pub fn is_compatible_with_bun_workspaces(workspace_globs: &[&str]) -> bool {
    if workspace_globs.len() > BUN_WORKSPACE_GLOB_COUNT_LIMIT {
        return false;
    }

    let mut total_bytes = 0usize;
    workspace_globs.iter().copied().all(|workspace_glob| {
        let Some(next_total) = total_bytes.checked_add(workspace_glob.len()) else {
            return false;
        };
        total_bytes = next_total;

        if workspace_glob.len() > BUN_WORKSPACE_GLOB_INPUT_LIMIT
            || total_bytes > BUN_WORKSPACE_GLOB_TOTAL_INPUT_LIMIT
            || contains_unsafe_workspace_glob_text(workspace_glob)
        {
            return false;
        }

        if workspace_glob.contains('*') {
            if workspace_glob.contains("**") {
                return false;
            }
            if workspace_glob
                .rsplit_once('/')
                .is_some_and(|(prefix, _)| prefix.contains('*'))
            {
                return false;
            }
        }

        !workspace_glob
            .bytes()
            .any(|byte| matches!(byte, b'!' | b'[' | b']' | b'{' | b'}'))
    })
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
