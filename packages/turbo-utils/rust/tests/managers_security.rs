#![allow(clippy::expect_used, clippy::unwrap_used)]

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt as _, symlink};
#[cfg(unix)]
use std::{
    env,
    time::{Duration, Instant},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

#[cfg(unix)]
use turbo_utils_rs::SystemPackageManagerCommandRunner;
use turbo_utils_rs::{
    CommandRequest, PackageManagerCommandRunner, get_available_package_managers_with,
};

#[derive(Debug, Default)]
struct RecordingRunner {
    calls: Mutex<Vec<CommandRequest>>,
}

impl PackageManagerCommandRunner for RecordingRunner {
    fn run(&self, request: &CommandRequest) -> Option<String> {
        self.calls.lock().expect("calls lock").push(request.clone());
        None
    }

    fn resolve(&self, _program: &str) -> Option<PathBuf> {
        None
    }
}

#[cfg(unix)]
fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("write executable");
    let mut permissions = fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("permissions");
}

#[test]
fn custom_yarn_path_is_never_executed() {
    let project = tempfile::tempdir().expect("project");
    fs::write(
        project.path().join(".yarnrc.yml"),
        "yarnPath: ../../attacker/yarn.cjs\n",
    )
    .expect("yarn config");
    let runner = RecordingRunner::default();

    let result = get_available_package_managers_with(project.path(), Path::new("/tmp"), &runner);
    assert_eq!(result.yarn, None);
    assert!(
        runner
            .calls
            .lock()
            .expect("calls lock")
            .iter()
            .all(|call| call.program != "yarnpkg" && call.program != "yarn")
    );
}

#[cfg(unix)]
#[test]
fn symlinked_package_metadata_is_not_followed() {
    let project = tempfile::tempdir().expect("project");
    let outside = tempfile::NamedTempFile::new().expect("outside package");
    fs::write(outside.path(), r#"{"packageManager":"yarn@9.9.9"}"#)
        .expect("outside package contents");
    symlink(outside.path(), project.path().join("package.json")).expect("package symlink");
    let runner = RecordingRunner::default();

    let result = get_available_package_managers_with(project.path(), Path::new("/tmp"), &runner);
    assert_eq!(result.yarn, None);
    assert!(
        runner
            .calls
            .lock()
            .expect("calls lock")
            .iter()
            .any(|call| call.program == "yarnpkg")
    );
}

#[test]
fn oversized_package_metadata_is_not_parsed() {
    let project = tempfile::tempdir().expect("project");
    let mut contents = String::from(r#"{"packageManager":"yarn@9.9.9","padding":""#);
    contents.push_str(&"x".repeat(1_024 * 1_024));
    contents.push_str("\"}");
    fs::write(project.path().join("package.json"), contents).expect("package metadata");
    let runner = RecordingRunner::default();

    let result = get_available_package_managers_with(project.path(), Path::new("/tmp"), &runner);
    assert_eq!(result.yarn, None);
    assert!(
        runner
            .calls
            .lock()
            .expect("calls lock")
            .iter()
            .any(|call| call.program == "yarnpkg")
    );
}

#[cfg(unix)]
#[test]
fn resolver_skips_relative_and_project_local_path_entries() {
    let project = tempfile::tempdir().expect("project");
    let project_bin = project.path().join("bin");
    let trusted = tempfile::tempdir().expect("trusted bin");
    fs::create_dir_all(&project_bin).expect("project bin");
    write_executable(&project_bin.join("npm"), "#!/bin/sh\nprintf project\n");
    write_executable(&trusted.path().join("npm"), "#!/bin/sh\nprintf trusted\n");
    let search_path = env::join_paths([
        PathBuf::from("relative-bin"),
        project_bin,
        trusted.path().to_path_buf(),
    ])
    .expect("search path");
    let runner = SystemPackageManagerCommandRunner::with_environment(
        Some(search_path),
        env::temp_dir(),
        Some(project.path()),
        Duration::from_secs(1),
        1_024,
    );

    assert_eq!(
        runner.resolve("npm"),
        fs::canonicalize(trusted.path().join("npm")).ok()
    );
    assert_eq!(runner.resolve("../npm"), None);
}

#[cfg(unix)]
#[test]
fn command_output_is_bounded() {
    let binaries = tempfile::tempdir().expect("binaries");
    let noisy = binaries.path().join("noisy");
    write_executable(
        &noisy,
        &format!("#!/bin/sh\nprintf '{}'\n", "x".repeat(2_048)),
    );
    let search_path = env::join_paths([binaries.path()]).expect("search path");
    let runner = SystemPackageManagerCommandRunner::with_environment(
        Some(search_path),
        env::temp_dir(),
        None,
        Duration::from_secs(1),
        128,
    );
    let request = CommandRequest {
        program: "noisy".into(),
        arguments: Vec::new(),
        current_directory: env::temp_dir(),
        environment: Vec::new(),
        timeout: Duration::from_secs(1),
        maximum_output_bytes: 128,
    };

    assert_eq!(runner.run(&request), None);
}

#[cfg(unix)]
#[test]
fn command_execution_has_a_deadline() {
    let binaries = tempfile::tempdir().expect("binaries");
    let sleeper = binaries.path().join("sleeper");
    write_executable(&sleeper, "#!/bin/sh\nsleep 5\nprintf late\n");
    let search_path = env::join_paths([binaries.path()]).expect("search path");
    let runner = SystemPackageManagerCommandRunner::with_environment(
        Some(search_path),
        env::temp_dir(),
        None,
        Duration::from_millis(75),
        1_024,
    );
    let request = CommandRequest {
        program: "sleeper".into(),
        arguments: Vec::new(),
        current_directory: env::temp_dir(),
        environment: Vec::new(),
        timeout: Duration::from_millis(75),
        maximum_output_bytes: 1_024,
    };
    let started = Instant::now();

    assert_eq!(runner.run(&request), None);
    assert!(started.elapsed() < Duration::from_secs(2));
}
