#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use turbo_utils_rs::{
    CreateProjectError, CreateProjectOptions, GitHubRepositoryUrl, ProjectSource,
    ProjectSourceError, RepoInfo, create_project,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    GetRepoInfo {
        url: String,
        example_path: Option<String>,
    },
    HasRepo(RepoInfo),
    ExampleExists(String),
    DownloadExample {
        root: PathBuf,
        example: String,
    },
    DownloadRepo {
        root: PathBuf,
        repo_info: RepoInfo,
    },
}

#[derive(Debug)]
struct FakeSource {
    calls: Mutex<Vec<Call>>,
    repo_info: Option<RepoInfo>,
    repo_exists: bool,
    example_exists: bool,
    failures_before_success: Mutex<usize>,
    always_fail: bool,
    package_json: Option<String>,
}

impl Default for FakeSource {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            repo_info: None,
            repo_exists: true,
            example_exists: true,
            failures_before_success: Mutex::new(0),
            always_fail: false,
            package_json: None,
        }
    }
}

impl FakeSource {
    fn with_repo_info(mut self, repo_info: RepoInfo) -> Self {
        self.repo_info = Some(repo_info);
        self
    }

    fn with_failures_before_success(self, failures: usize) -> Self {
        *self
            .failures_before_success
            .lock()
            .expect("failure counter") = failures;
        self
    }

    fn always_failing(mut self) -> Self {
        self.always_fail = true;
        self
    }

    fn with_package_json(mut self, package_json: &str) -> Self {
        self.package_json = Some(package_json.to_owned());
        self
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn perform_download(&self, root: &Path) -> Result<(), ProjectSourceError> {
        let mut failures = self
            .failures_before_success
            .lock()
            .expect("failure counter");
        if self.always_fail || *failures > 0 {
            *failures = failures.saturating_sub(1);
            return Err(ProjectSourceError::new("transient download failure"));
        }
        drop(failures);

        if let Some(package_json) = &self.package_json {
            fs::write(root.join("package.json"), package_json)
                .map_err(|error| ProjectSourceError::new(error.to_string()))?;
        }
        Ok(())
    }
}

impl ProjectSource for FakeSource {
    fn get_repo_info(
        &self,
        url: &GitHubRepositoryUrl,
        example_path: Option<&str>,
    ) -> Result<Option<RepoInfo>, ProjectSourceError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(Call::GetRepoInfo {
                url: url.as_str().to_owned(),
                example_path: example_path.map(ToOwned::to_owned),
            });
        Ok(self.repo_info.clone())
    }

    fn has_repo(&self, repo_info: &RepoInfo) -> Result<bool, ProjectSourceError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(Call::HasRepo(repo_info.clone()));
        Ok(self.repo_exists)
    }

    fn example_exists(&self, example: &str) -> Result<bool, ProjectSourceError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(Call::ExampleExists(example.to_owned()));
        Ok(self.example_exists)
    }

    fn download_example(&self, root: &Path, example: &str) -> Result<(), ProjectSourceError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(Call::DownloadExample {
                root: root.to_path_buf(),
                example: example.to_owned(),
            });
        self.perform_download(root)
    }

    fn download_repo(
        &self,
        root: &Path,
        repo_info: &RepoInfo,
    ) -> Result<(), ProjectSourceError> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(Call::DownloadRepo {
                root: root.to_path_buf(),
                repo_info: repo_info.clone(),
            });
        self.perform_download(root)
    }
}

fn options(base: &Path, app_name: &str, example: &str) -> CreateProjectOptions {
    CreateProjectOptions {
        app_path: base.join(app_name),
        example: example.to_owned(),
        is_default_example: false,
        example_path: None,
        original_directory: base.to_path_buf(),
    }
}

fn repo_info() -> RepoInfo {
    RepoInfo {
        username: "acme".into(),
        name: "starter".into(),
        branch: "main".into(),
        file_path: "examples/web".into(),
    }
}

#[test]
fn default_example_uses_basic_sparse_example_path() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default();
    let mut request = options(base.path(), "my-app", "basic");
    request.is_default_example = true;

    let result = create_project(&request, &source).expect("project result");

    assert_eq!(result.cd_path, PathBuf::from("my-app"));
    assert_eq!(result.repo_info, None);
    assert_eq!(
        source.calls(),
        [Call::DownloadExample {
            root: base.path().join("my-app"),
            example: "basic".into(),
        }]
    );
}

#[test]
fn named_example_uses_the_repository_tarball_path() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default();
    let request = options(base.path(), "my-app", "with-tailwind");
    let expected_repo = RepoInfo {
        username: "vercel".into(),
        name: "turborepo".into(),
        branch: "main".into(),
        file_path: "examples/with-tailwind".into(),
    };

    let result = create_project(&request, &source).expect("project result");

    assert_eq!(result.repo_info, Some(expected_repo.clone()));
    assert_eq!(
        source.calls(),
        [
            Call::ExampleExists("with-tailwind".into()),
            Call::DownloadRepo {
                root: base.path().join("my-app"),
                repo_info: expected_repo,
            },
        ]
    );
}

#[test]
fn github_repository_uses_repo_information_and_repo_download() {
    let base = tempfile::tempdir().expect("base directory");
    let expected_repo = repo_info();
    let source = FakeSource::default().with_repo_info(expected_repo.clone());
    let mut request = options(
        base.path(),
        "my-app",
        "https://github.com/acme/starter/tree/main/examples/web",
    );
    request.example_path = Some("/examples/web".into());

    let result = create_project(&request, &source).expect("project result");

    assert_eq!(result.repo_info, Some(expected_repo.clone()));
    assert_eq!(
        source.calls(),
        [
            Call::GetRepoInfo {
                url: request.example.clone(),
                example_path: Some("examples/web".into()),
            },
            Call::HasRepo(expected_repo.clone()),
            Call::DownloadRepo {
                root: base.path().join("my-app"),
                repo_info: expected_repo,
            },
        ]
    );
}

#[test]
fn retries_three_times_and_succeeds_on_the_fourth_attempt() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default().with_failures_before_success(3);
    let mut request = options(base.path(), "my-app", "basic");
    request.is_default_example = true;

    create_project(&request, &source).expect("fourth attempt succeeds");

    assert_eq!(
        source
            .calls()
            .iter()
            .filter(|call| matches!(call, Call::DownloadExample { .. }))
            .count(),
        4
    );
}

#[test]
fn stops_after_four_failed_download_attempts() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default().always_failing();
    let mut request = options(base.path(), "my-app", "basic");
    request.is_default_example = true;

    let error = create_project(&request, &source).expect_err("download must fail");

    assert!(matches!(error, CreateProjectError::Download(_)));
    assert_eq!(
        source
            .calls()
            .iter()
            .filter(|call| matches!(call, Call::DownloadExample { .. }))
            .count(),
        4
    );
}

#[test]
fn package_scripts_follow_javascript_object_key_order() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default().with_package_json(
        r#"{"scripts":{"2":"two","10":"ten","1":"one","build":"build","01":"leading"}}"#,
    );
    let mut request = options(base.path(), "my-app", "basic");
    request.is_default_example = true;

    let result = create_project(&request, &source).expect("project result");

    assert!(result.has_package_json);
    assert_eq!(result.available_scripts, ["1", "2", "10", "build", "01"]);
}

#[test]
fn malformed_package_json_is_present_but_has_no_scripts() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default().with_package_json("{");
    let mut request = options(base.path(), "my-app", "basic");
    request.is_default_example = true;

    let result = create_project(&request, &source).expect("project result");

    assert!(result.has_package_json);
    assert!(result.available_scripts.is_empty());
}

#[test]
fn missing_named_example_returns_the_existing_failure_semantics() {
    let base = tempfile::tempdir().expect("base directory");
    let mut source = FakeSource::default();
    source.example_exists = false;
    let request = options(base.path(), "my-app", "does-not-exist");

    let error = create_project(&request, &source).expect_err("example must be missing");

    assert!(matches!(
        error,
        CreateProjectError::ExampleNotFound { ref example }
            if example == "does-not-exist"
    ));
    assert_eq!(
        source.calls(),
        [Call::ExampleExists("does-not-exist".into())]
    );
}

#[test]
fn unavailable_repository_information_is_reported_before_download() {
    let base = tempfile::tempdir().expect("base directory");
    let source = FakeSource::default();
    let request = options(
        base.path(),
        "my-app",
        "https://github.com/acme/missing",
    );

    let error = create_project(&request, &source).expect_err("repo info must be unavailable");

    assert!(matches!(
        error,
        CreateProjectError::RepositoryInfoUnavailable { .. }
    ));
    assert_eq!(source.calls().len(), 1);
}

#[test]
fn repository_existence_is_checked_before_download() {
    let base = tempfile::tempdir().expect("base directory");
    let expected_repo = repo_info();
    let mut source = FakeSource::default().with_repo_info(expected_repo.clone());
    source.repo_exists = false;
    let request = options(
        base.path(),
        "my-app",
        "https://github.com/acme/starter",
    );

    let error = create_project(&request, &source).expect_err("repository must be missing");

    assert!(matches!(
        error,
        CreateProjectError::RepositoryNotFound { .. }
    ));
    assert_eq!(
        source.calls(),
        [
            Call::GetRepoInfo {
                url: request.example.clone(),
                example_path: None,
            },
            Call::HasRepo(expected_repo),
        ]
    );
}
