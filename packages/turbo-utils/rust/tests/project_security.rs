#![allow(clippy::expect_used, clippy::unwrap_used)]

#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use turbo_utils_rs::{
    CreateProjectError, CreateProjectOptions, GitHubRepositoryUrl, ProjectSource,
    ProjectSourceError, RepoInfo, create_project, is_valid_github_repo_url,
};

#[derive(Debug, Clone)]
enum DownloadAction {
    None,
    WritePackageJson(String),
    #[cfg(unix)]
    SymlinkPackageJson(PathBuf),
}

#[derive(Debug)]
struct SecuritySource {
    calls: AtomicUsize,
    action: Mutex<DownloadAction>,
}

impl Default for SecuritySource {
    fn default() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            action: Mutex::new(DownloadAction::None),
        }
    }
}

impl SecuritySource {
    fn with_action(action: DownloadAction) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            action: Mutex::new(action),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn record(&self) {
        self.calls.fetch_add(1, Ordering::SeqCst);
    }

    fn apply_action(&self, root: &Path) -> Result<(), ProjectSourceError> {
        match &*self.action.lock().expect("action lock") {
            DownloadAction::None => Ok(()),
            DownloadAction::WritePackageJson(contents) => {
                fs::write(root.join("package.json"), contents)
                    .map_err(|error| ProjectSourceError::new(error.to_string()))
            }
            #[cfg(unix)]
            DownloadAction::SymlinkPackageJson(target) => {
                symlink(target, root.join("package.json"))
                    .map_err(|error| ProjectSourceError::new(error.to_string()))
            }
        }
    }
}

impl ProjectSource for SecuritySource {
    fn get_repo_info(
        &self,
        _url: &GitHubRepositoryUrl,
        _example_path: Option<&str>,
    ) -> Result<Option<RepoInfo>, ProjectSourceError> {
        self.record();
        Ok(Some(RepoInfo {
            username: "acme".into(),
            name: "starter".into(),
            branch: "main".into(),
            file_path: String::new(),
        }))
    }

    fn has_repo(&self, _repo_info: &RepoInfo) -> Result<bool, ProjectSourceError> {
        self.record();
        Ok(true)
    }

    fn example_exists(&self, _example: &str) -> Result<bool, ProjectSourceError> {
        self.record();
        Ok(true)
    }

    fn download_example(&self, root: &Path, _example: &str) -> Result<(), ProjectSourceError> {
        self.record();
        self.apply_action(root)
    }

    fn download_repo(&self, root: &Path, _repo_info: &RepoInfo) -> Result<(), ProjectSourceError> {
        self.record();
        self.apply_action(root)
    }
}

fn options(base: &Path, app_path: PathBuf, example: &str) -> CreateProjectOptions {
    CreateProjectOptions {
        app_path,
        example: example.to_owned(),
        is_default_example: false,
        example_path: None,
        original_directory: base.to_path_buf(),
    }
}

#[test]
fn github_url_validation_rejects_scheme_host_credential_and_port_confusion() {
    assert!(is_valid_github_repo_url("https://github.com/acme/starter"));
    assert!(is_valid_github_repo_url(
        "https://GITHUB.COM/acme/starter/tree/main"
    ));

    for invalid in [
        "http://github.com/acme/starter",
        "ftp://github.com/acme/starter",
        "https://github.com.evil.example/acme/starter",
        "https://github.com@evil.example/acme/starter",
        "https://user@github.com/acme/starter",
        "https://github.com:443/acme/starter",
        "https://github.com\u{000a}.evil.example/acme/starter",
    ] {
        assert!(!is_valid_github_repo_url(invalid), "accepted {invalid}");
    }
}

#[test]
fn unsafe_named_example_is_rejected_before_any_source_operation() {
    let base = tempfile::tempdir().expect("base directory");
    let source = SecuritySource::default();
    let request = options(base.path(), base.path().join("app"), "../escape");

    let error = create_project(&request, &source).expect_err("unsafe name must fail");

    assert!(matches!(
        error,
        CreateProjectError::InvalidExampleName { .. }
    ));
    assert_eq!(source.call_count(), 0);
}

#[test]
fn unsafe_repository_subpath_is_rejected_before_network_resolution() {
    let base = tempfile::tempdir().expect("base directory");
    let source = SecuritySource::default();
    let mut request = options(
        base.path(),
        base.path().join("app"),
        "https://github.com/acme/starter",
    );
    request.example_path = Some("../../outside".into());

    let error = create_project(&request, &source).expect_err("unsafe path must fail");

    assert!(matches!(
        error,
        CreateProjectError::UnsafeRepositoryPath { .. }
    ));
    assert_eq!(source.call_count(), 0);
}

#[test]
fn conflicting_target_is_rejected_before_download() {
    let base = tempfile::tempdir().expect("base directory");
    let root = base.path().join("app");
    fs::create_dir(&root).expect("project root");
    fs::write(root.join("owned.txt"), "do not overwrite").expect("conflicting file");
    let source = SecuritySource::default();
    let mut request = options(base.path(), root, "basic");
    request.is_default_example = true;

    let error = create_project(&request, &source).expect_err("conflict must fail");

    assert!(matches!(
        error,
        CreateProjectError::ConflictingFiles { count: 1, .. }
    ));
    assert_eq!(source.call_count(), 0);
}

#[cfg(unix)]
#[test]
fn symlinked_project_root_is_never_followed() {
    let base = tempfile::tempdir().expect("base directory");
    let outside = tempfile::tempdir().expect("outside directory");
    let root = base.path().join("app");
    symlink(outside.path(), &root).expect("root symlink");
    let source = SecuritySource::default();
    let mut request = options(base.path(), root, "basic");
    request.is_default_example = true;

    let error = create_project(&request, &source).expect_err("symlink must fail");

    assert!(matches!(
        error,
        CreateProjectError::UnsafeProjectPath { .. }
    ));
    assert_eq!(source.call_count(), 0);
    assert!(
        fs::read_dir(outside.path())
            .expect("outside directory")
            .next()
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn symlinked_package_json_is_not_read() {
    let base = tempfile::tempdir().expect("base directory");
    let outside = tempfile::NamedTempFile::new().expect("outside package file");
    fs::write(
        outside.path(),
        r#"{"scripts":{"attacker-controlled":"run"}}"#,
    )
    .expect("outside package contents");
    let source = SecuritySource::with_action(DownloadAction::SymlinkPackageJson(
        outside.path().to_path_buf(),
    ));
    let mut request = options(base.path(), base.path().join("app"), "basic");
    request.is_default_example = true;

    let result = create_project(&request, &source).expect("project result");

    assert!(result.has_package_json);
    assert!(result.available_scripts.is_empty());
}

#[test]
fn oversized_package_json_is_not_parsed() {
    let base = tempfile::tempdir().expect("base directory");
    let mut contents = String::from(r#"{"scripts":{"attacker-controlled":"run"},"padding":""#);
    contents.push_str(&"x".repeat(1_024 * 1_024));
    contents.push_str("\"}");
    let source = SecuritySource::with_action(DownloadAction::WritePackageJson(contents));
    let mut request = options(base.path(), base.path().join("app"), "basic");
    request.is_default_example = true;

    let result = create_project(&request, &source).expect("project result");

    assert!(result.has_package_json);
    assert!(result.available_scripts.is_empty());
}
