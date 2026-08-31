#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use tempfile::TempDir;
use turbo_utils_rs::{
    CommandRequest, PackageManagerCommandRunner, PackageManagerValues,
    get_available_package_managers_with, get_package_managers_bin_paths_with,
};

#[derive(Debug, Default)]
struct FakeRunner {
    outputs: HashMap<String, String>,
    resolutions: HashMap<String, PathBuf>,
    calls: Mutex<Vec<CommandRequest>>,
    resolve_calls: Mutex<Vec<String>>,
}

impl FakeRunner {
    fn with_output(mut self, program: &str, arguments: &[&str], output: &str) -> Self {
        self.outputs
            .insert(command_key(program, arguments), output.to_owned());
        self
    }

    fn with_resolution(mut self, program: &str, path: &str) -> Self {
        self.resolutions
            .insert(program.to_owned(), PathBuf::from(path));
        self
    }

    fn calls(&self) -> Vec<CommandRequest> {
        self.calls.lock().expect("calls lock").clone()
    }
}

impl PackageManagerCommandRunner for FakeRunner {
    fn run(&self, request: &CommandRequest) -> Option<String> {
        self.calls.lock().expect("calls lock").push(request.clone());
        self.outputs
            .get(&command_key_owned(&request.program, &request.arguments))
            .cloned()
    }

    fn resolve(&self, program: &str) -> Option<PathBuf> {
        self.resolve_calls
            .lock()
            .expect("resolve lock")
            .push(program.to_owned());
        self.resolutions.get(program).cloned()
    }
}

fn command_key(program: &str, arguments: &[&str]) -> String {
    let mut key = program.to_owned();
    for argument in arguments {
        key.push('\0');
        key.push_str(argument);
    }
    key
}

fn command_key_owned(program: &str, arguments: &[String]) -> String {
    let mut key = program.to_owned();
    for argument in arguments {
        key.push('\0');
        key.push_str(argument);
    }
    key
}

fn create_project(files: &[(&str, &str)]) -> TempDir {
    let project = tempfile::tempdir().expect("temporary project");
    for &(relative_path, content) in files {
        let path = project.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent");
        }
        fs::write(path, content).expect("fixture file");
    }
    project
}

fn command_cwd() -> &'static Path {
    Path::new("/tmp")
}

#[test]
fn returns_all_available_package_managers() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "1.22.19")
        .with_output("npm", &["--version"], "9.5.0")
        .with_output("pnpm", &["--version"], "8.6.7")
        .with_output("bun", &["--version"], "1.0.0")
        .with_output("nub", &["--version"], "0.1.0")
        .with_output("aube", &["--version"], "0.1.0");
    let missing = Path::new("/tmp/turbo-managers-missing");

    assert_eq!(
        get_available_package_managers_with(missing, command_cwd(), &runner),
        PackageManagerValues {
            yarn: Some("1.22.19".into()),
            npm: Some("9.5.0".into()),
            pnpm: Some("8.6.7".into()),
            bun: Some("1.0.0".into()),
            nub: Some("0.1.0".into()),
            aube: Some("0.1.0".into()),
        }
    );
}

#[test]
fn parses_versions_from_verbose_output() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "1.22.19")
        .with_output("npm", &["--version"], "9.5.0")
        .with_output("pnpm", &["--version"], "8.6.7")
        .with_output("bun", &["--version"], "1.0.0")
        .with_output("nub", &["--version"], "0.1.0")
        .with_output("aube", &["--version"], "1.25.1 macos-arm64 (2026-06-30)");

    let result = get_available_package_managers_with(
        Path::new("/tmp/turbo-managers-missing"),
        command_cwd(),
        &runner,
    );
    assert_eq!(result.aube.as_deref(), Some("1.25.1"));
}

#[test]
fn unavailable_package_managers_are_none() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "1.22.19")
        .with_output("pnpm", &["--version"], "8.6.7");

    assert_eq!(
        get_available_package_managers_with(
            Path::new("/tmp/turbo-managers-missing"),
            command_cwd(),
            &runner,
        ),
        PackageManagerValues {
            yarn: Some("1.22.19".into()),
            npm: None,
            pnpm: Some("8.6.7".into()),
            bun: None,
            nub: None,
            aube: None,
        }
    );
}

#[test]
fn infers_project_yarn_version_from_package_manager() {
    let project = create_project(&[("package.json", r#"{"packageManager":"yarn@4.5.1"}"#)]);
    let runner = FakeRunner::default()
        .with_output("npm", &["--version"], "9.5.0")
        .with_output("pnpm", &["--version"], "8.6.7")
        .with_output("bun", &["--version"], "1.0.0")
        .with_output("nub", &["--version"], "0.1.0")
        .with_output("aube", &["--version"], "0.1.0");

    let result = get_available_package_managers_with(project.path(), command_cwd(), &runner);
    assert_eq!(result.yarn.as_deref(), Some("4.5.1"));
    assert!(runner.calls().iter().all(|call| call.program != "yarnpkg"));
}

#[test]
fn infers_project_yarn_version_from_conventional_yarn_path() {
    let project = create_project(&[(".yarnrc.yml", "yarnPath: .yarn/releases/yarn-3.2.1.cjs\n")]);
    let runner = FakeRunner::default()
        .with_output("npm", &["--version"], "9.5.0")
        .with_output("pnpm", &["--version"], "8.6.7")
        .with_output("bun", &["--version"], "1.0.0")
        .with_output("nub", &["--version"], "0.1.0")
        .with_output("aube", &["--version"], "0.1.0");

    let result = get_available_package_managers_with(project.path(), command_cwd(), &runner);
    assert_eq!(result.yarn.as_deref(), Some("3.2.1"));
    assert!(runner.calls().iter().all(|call| call.program != "yarnpkg"));
}

#[test]
fn custom_yarn_path_disables_execution_and_fallback() {
    let project = create_project(&[(".yarnrc.yml", "yarnPath: ./scripts/yarn.cjs\n")]);
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "4.5.1")
        .with_output("npm", &["--version"], "9.5.0");

    let result = get_available_package_managers_with(project.path(), command_cwd(), &runner);
    assert_eq!(result.yarn, None);
    assert!(runner.calls().iter().all(|call| call.program != "yarnpkg"));
}

#[test]
fn returns_all_package_manager_bin_paths() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "3.2.1")
        .with_output("npm", &["config", "get", "prefix"], "/usr/local/bin")
        .with_output("pnpm", &["bin", "--global"], "/usr/local/pnpm")
        .with_output("bun", &["pm", "--g", "bin"], "/usr/local/bun")
        .with_resolution("nub", "/usr/local/bin/nub")
        .with_resolution("aube", "/usr/local/bin/aube");

    assert_eq!(
        get_package_managers_bin_paths_with(
            Path::new("/tmp/turbo-managers-missing"),
            command_cwd(),
            &runner,
        ),
        PackageManagerValues {
            yarn: Some(".yarn/releases/yarn-3.2.1.cjs".into()),
            npm: Some("/usr/local/bin".into()),
            pnpm: Some("/usr/local/pnpm".into()),
            bun: Some("/usr/local/bun".into()),
            nub: Some("/usr/local/bin".into()),
            aube: Some("/usr/local/bin".into()),
        }
    );
}

#[test]
fn yarn_v1_uses_global_bin_command() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "1.22.19")
        .with_output("yarn", &["global", "bin"], "/usr/local/yarn")
        .with_output("npm", &["config", "get", "prefix"], "/usr/local/bin")
        .with_output("pnpm", &["bin", "--global"], "/usr/local/pnpm")
        .with_output("bun", &["pm", "--g", "bin"], "/usr/local/bun")
        .with_resolution("nub", "/usr/local/bin/nub")
        .with_resolution("aube", "/usr/local/bin/aube");

    let result = get_package_managers_bin_paths_with(
        Path::new("/tmp/turbo-managers-missing"),
        command_cwd(),
        &runner,
    );
    assert_eq!(result.yarn.as_deref(), Some("/usr/local/yarn"));
}

#[test]
fn failed_bin_path_checks_are_none() {
    let runner = FakeRunner::default().with_output("pnpm", &["bin", "--global"], "/usr/local/pnpm");

    assert_eq!(
        get_package_managers_bin_paths_with(
            Path::new("/tmp/turbo-managers-missing"),
            command_cwd(),
            &runner,
        ),
        PackageManagerValues {
            yarn: None,
            npm: None,
            pnpm: Some("/usr/local/pnpm".into()),
            bun: None,
            nub: None,
            aube: None,
        }
    );
}

#[test]
fn command_requests_preserve_typescript_execution_contract() {
    let runner = FakeRunner::default()
        .with_output("yarnpkg", &["--version"], "3.2.1")
        .with_output("npm", &["config", "get", "prefix"], "/usr/local/bin")
        .with_output("pnpm", &["bin", "--global"], "/usr/local/pnpm")
        .with_output("bun", &["pm", "--g", "bin"], "/usr/local/bun");

    let _result = get_package_managers_bin_paths_with(
        Path::new("/tmp/turbo-managers-missing"),
        command_cwd(),
        &runner,
    );
    let calls = runner.calls();
    let npm = calls
        .iter()
        .find(|call| call.program == "npm")
        .expect("npm request");
    assert_eq!(npm.arguments, ["config", "get", "prefix"]);
    assert_eq!(npm.current_directory, Path::new("/tmp"));
    assert_eq!(
        npm.environment,
        [("COREPACK_ENABLE_STRICT".into(), "0".into())]
    );
    assert_eq!(npm.timeout, Duration::from_secs(5));
    assert_eq!(npm.maximum_output_bytes, 1_024 * 1_024);
}

#[test]
fn project_yarn_berry_bin_path_does_not_execute_yarn() {
    let project = create_project(&[("package.json", r#"{"packageManager":"yarn@4.5.1"}"#)]);
    let runner = FakeRunner::default()
        .with_output("npm", &["config", "get", "prefix"], "/usr/local/bin")
        .with_output("pnpm", &["bin", "--global"], "/usr/local/pnpm")
        .with_output("bun", &["pm", "--g", "bin"], "/usr/local/bun")
        .with_resolution("nub", "/usr/local/bin/nub")
        .with_resolution("aube", "/usr/local/bin/aube");

    let result = get_package_managers_bin_paths_with(project.path(), command_cwd(), &runner);
    assert_eq!(
        result.yarn.as_deref(),
        Some(".yarn/releases/yarn-4.5.1.cjs")
    );
    assert!(
        runner
            .calls()
            .iter()
            .all(|call| call.program != "yarnpkg" && call.program != "yarn")
    );
}

#[test]
fn parses_quoted_yarn_paths_like_the_typescript_implementation() {
    let double_quoted = create_project(&[(
        ".yarnrc.yml",
        "yarnPath: \".yarn/releases/yarn-4.1.0.cjs\" # comment\n",
    )]);
    let single_quoted =
        create_project(&[(".yarnrc.yml", "yarnPath: '.yarn/releases/yarn-4.2.0.cjs'\n")]);
    let runner = FakeRunner::default();

    let double_result =
        get_available_package_managers_with(double_quoted.path(), command_cwd(), &runner);
    let single_result =
        get_available_package_managers_with(single_quoted.path(), command_cwd(), &runner);
    assert_eq!(double_result.yarn.as_deref(), Some("4.1.0"));
    assert_eq!(single_result.yarn.as_deref(), Some("4.2.0"));
}
