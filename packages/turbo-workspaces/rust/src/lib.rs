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
            Self::UnableToDetect =>
                "Could not determine package manager. Add `devEngines.packageManager` or legacy `packageManager` to `package.json`, or ensure a lockfile is present."
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
