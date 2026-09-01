use std::path::{Path, PathBuf};

use turbo_workspaces_rs::{
    MANAGER_DETECTION_ORDER, WorkspaceDetailsError, WorkspaceDetailsKnownError,
    WorkspaceDetailsProvider, WorkspaceDirectoryInfo, WorkspaceManager, get_workspace_details,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeProject {
    manager: WorkspaceManager,
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderError {
    Directory,
    Detect(WorkspaceManager),
    Read(WorkspaceManager),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Directory(PathBuf),
    Detect(WorkspaceManager, PathBuf),
    Read(WorkspaceManager, PathBuf),
}

struct FakeProvider {
    directory: Result<WorkspaceDirectoryInfo, ProviderError>,
    detected: Option<WorkspaceManager>,
    detect_error: Option<WorkspaceManager>,
    read_error: Option<WorkspaceManager>,
    calls: Vec<Call>,
}

impl FakeProvider {
    fn existing(absolute: &Path) -> Self {
        Self {
            directory: Ok(WorkspaceDirectoryInfo {
                absolute: absolute.to_path_buf(),
                exists: true,
            }),
            detected: None,
            detect_error: None,
            read_error: None,
            calls: Vec::new(),
        }
    }
}

impl WorkspaceDetailsProvider for FakeProvider {
    type Project = FakeProject;
    type Error = ProviderError;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error> {
        self.calls.push(Call::Directory(root.to_path_buf()));
        self.directory.clone()
    }

    fn detect(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<bool, Self::Error> {
        self.calls
            .push(Call::Detect(manager, workspace_root.to_path_buf()));
        if self.detect_error == Some(manager) {
            return Err(ProviderError::Detect(manager));
        }
        Ok(self.detected == Some(manager))
    }

    fn read(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<Self::Project, Self::Error> {
        self.calls
            .push(Call::Read(manager, workspace_root.to_path_buf()));
        if self.read_error == Some(manager) {
            return Err(ProviderError::Read(manager));
        }
        Ok(FakeProject {
            manager,
            root: workspace_root.to_path_buf(),
        })
    }
}

#[test]
fn manager_order_matches_the_typescript_registry() {
    assert_eq!(
        MANAGER_DETECTION_ORDER.map(WorkspaceManager::as_str),
        ["aube", "nub", "pnpm", "yarn", "npm", "bun"]
    );
}

#[test]
fn missing_directory_returns_the_exact_known_error_before_detection() {
    let raw = Path::new("relative-input");
    let absolute = PathBuf::from("/safe/absolute/missing");
    let mut provider = FakeProvider {
        directory: Ok(WorkspaceDirectoryInfo {
            absolute: absolute.clone(),
            exists: false,
        }),
        detected: Some(WorkspaceManager::Aube),
        detect_error: None,
        read_error: None,
        calls: Vec::new(),
    };

    let result = get_workspace_details(raw, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::InvalidDirectory {
                absolute: absolute.clone(),
            }
        ))
    );
    let WorkspaceDetailsError::Known(error) = result.err().unwrap_or_else(|| {
        panic!("the missing directory must return a known error");
    }) else {
        panic!("the missing directory must not return a provider error");
    };
    assert_eq!(error.error_type(), "invalid_directory");
    assert_eq!(
        error.message(),
        "Could not find directory at /safe/absolute/missing. Ensure the directory exists."
    );
    assert_eq!(provider.calls, [Call::Directory(raw.to_path_buf())]);
}

#[test]
fn first_detected_manager_is_read_and_later_managers_are_not_consulted() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);
    provider.detected = Some(WorkspaceManager::Pnpm);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Ok(FakeProject {
            manager: WorkspaceManager::Pnpm,
            root: root.to_path_buf(),
        })
    );
    assert_eq!(
        provider.calls,
        [
            Call::Directory(root.to_path_buf()),
            Call::Detect(WorkspaceManager::Aube, root.to_path_buf()),
            Call::Detect(WorkspaceManager::Nub, root.to_path_buf()),
            Call::Detect(WorkspaceManager::Pnpm, root.to_path_buf()),
            Call::Read(WorkspaceManager::Pnpm, root.to_path_buf()),
        ]
    );
}

#[test]
fn selected_manager_read_failure_propagates_without_parser_fallback() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);
    provider.detected = Some(WorkspaceManager::Pnpm);
    provider.read_error = Some(WorkspaceManager::Pnpm);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Provider(ProviderError::Read(
            WorkspaceManager::Pnpm
        )))
    );
    assert_eq!(
        provider.calls.last(),
        Some(&Call::Read(WorkspaceManager::Pnpm, root.to_path_buf()))
    );
    assert!(!provider.calls.iter().any(|call| matches!(
        call,
        Call::Detect(WorkspaceManager::Yarn, _)
            | Call::Detect(WorkspaceManager::Npm, _)
            | Call::Detect(WorkspaceManager::Bun, _)
    )));
}

#[test]
fn all_six_rejections_return_the_exact_unable_to_detect_error() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::UnableToDetect
        ))
    );
    let WorkspaceDetailsError::Known(error) = result.err().unwrap_or_else(|| {
        panic!("all manager rejections must return a known error");
    }) else {
        panic!("all manager rejections must not return a provider error");
    };
    assert_eq!(error.error_type(), "package_manager-unable_to_detect");
    assert_eq!(
        error.message(),
        "Could not determine package manager. Add `devEngines.packageManager` or legacy \
         `packageManager` to `package.json`, or ensure a lockfile is present."
    );
    assert_eq!(
        provider
            .calls
            .iter()
            .filter(|call| matches!(call, Call::Detect(_, _)))
            .count(),
        6
    );
}

#[test]
fn directory_provider_failure_propagates_before_manager_authority() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider {
        directory: Err(ProviderError::Directory),
        detected: None,
        detect_error: None,
        read_error: None,
        calls: Vec::new(),
    };

    assert_eq!(
        get_workspace_details(root, &mut provider),
        Err(WorkspaceDetailsError::Provider(ProviderError::Directory))
    );
    assert_eq!(provider.calls, [Call::Directory(root.to_path_buf())]);
}
