use std::path::{Path, PathBuf};

use turbo_workspaces_rs::{
    MANAGER_DETECTION_ORDER, WorkspaceDetailsError, WorkspaceDetailsProvider,
    WorkspaceDirectoryInfo, WorkspaceManager, get_workspace_details,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderError {
    Detect(WorkspaceManager),
    Read(WorkspaceManager),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Directory(PathBuf),
    Detect(WorkspaceManager, PathBuf),
    Read(WorkspaceManager, PathBuf),
}

struct SecurityProvider {
    absolute: PathBuf,
    detected: Option<WorkspaceManager>,
    detect_error: Option<WorkspaceManager>,
    read_error: Option<WorkspaceManager>,
    calls: Vec<Call>,
}

impl SecurityProvider {
    fn new(absolute: &Path) -> Self {
        Self {
            absolute: absolute.to_path_buf(),
            detected: None,
            detect_error: None,
            read_error: None,
            calls: Vec::new(),
        }
    }
}

impl WorkspaceDetailsProvider for SecurityProvider {
    type Project = WorkspaceManager;
    type Error = ProviderError;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error> {
        self.calls.push(Call::Directory(root.to_path_buf()));
        Ok(WorkspaceDirectoryInfo {
            absolute: self.absolute.clone(),
            exists: true,
        })
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
        Ok(manager)
    }
}

#[test]
fn manager_identity_is_closed_ascii_data() {
    let names = MANAGER_DETECTION_ORDER.map(WorkspaceManager::as_str);
    assert_eq!(names, ["aube", "nub", "pnpm", "yarn", "npm", "bun"]);
    for name in names {
        assert!(!name.is_empty());
        assert!(name.is_ascii());
        assert!(name.bytes().all(|byte| byte.is_ascii_lowercase()));
    }
}

#[test]
fn detection_error_stops_without_trying_a_less_trusted_parser() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);
    provider.detect_error = Some(WorkspaceManager::Pnpm);
    provider.detected = Some(WorkspaceManager::Yarn);

    assert_eq!(
        get_workspace_details(root, &mut provider),
        Err(WorkspaceDetailsError::Provider(ProviderError::Detect(
            WorkspaceManager::Pnpm
        )))
    );
    assert!(!provider.calls.iter().any(|call| matches!(
        call,
        Call::Detect(WorkspaceManager::Yarn, _) | Call::Read(WorkspaceManager::Yarn, _)
    )));
}

#[test]
fn false_detectors_never_receive_read_authority() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);
    provider.detected = Some(WorkspaceManager::Npm);

    let _ = get_workspace_details(root, &mut provider);
    for manager in [
        WorkspaceManager::Aube,
        WorkspaceManager::Nub,
        WorkspaceManager::Pnpm,
        WorkspaceManager::Yarn,
    ] {
        assert!(
            !provider
                .calls
                .contains(&Call::Read(manager, root.to_path_buf()))
        );
    }
    assert!(
        provider
            .calls
            .contains(&Call::Read(WorkspaceManager::Npm, root.to_path_buf()))
    );
}

#[test]
fn the_provider_absolute_path_is_the_only_path_given_to_managers() {
    let raw = Path::new("relative-or-user-facing-input");
    let absolute = Path::new("/trusted/absolute/workspace");
    let mut provider = SecurityProvider::new(absolute);
    provider.detected = Some(WorkspaceManager::Aube);

    assert_eq!(
        get_workspace_details(raw, &mut provider),
        Ok(WorkspaceManager::Aube)
    );
    assert_eq!(provider.calls[0], Call::Directory(raw.to_path_buf()));
    assert_eq!(
        provider.calls[1],
        Call::Detect(WorkspaceManager::Aube, absolute.to_path_buf())
    );
    assert_eq!(
        provider.calls[2],
        Call::Read(WorkspaceManager::Aube, absolute.to_path_buf())
    );
}

#[test]
fn unable_to_detect_work_is_bounded_to_the_fixed_registry() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);

    let _ = get_workspace_details(root, &mut provider);
    assert_eq!(
        provider
            .calls
            .iter()
            .filter(|call| matches!(call, Call::Detect(_, _)))
            .count(),
        MANAGER_DETECTION_ORDER.len()
    );
    assert!(
        !provider
            .calls
            .iter()
            .any(|call| matches!(call, Call::Read(_, _)))
    );
}
