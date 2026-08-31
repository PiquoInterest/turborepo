#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    collections::HashMap,
    error::Error,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use pretty_assertions::assert_eq;
use turbo_utils_rs::{
    MANAGER_COMMAND_TIMEOUT, MAX_MANAGER_CONFIG_BYTES, MAX_MANAGER_OUTPUT_BYTES, ManagerCommand,
    ManagerCommandRunner, ManagerDetectionOptions, PackageManagers,
    get_available_package_managers_with, get_package_managers_bin_paths_with,
    resolve_executable_in_path,
};

fn command_key(program: &str, args: &[&str]) -> (String, Vec<String>) {
    (
        program.to_owned(),
        args.iter().map(|argument| (*argument).to_owned()).collect(),
    )
}

#[derive(Default)]
struct FakeRunner {
    outputs: Mutex<HashMap<(String, Vec<String>), Option<String>>>,
    calls: Mutex<Vec<ManagerCommand>>,
}

impl FakeRunner {
    fn output(&self, program: &str, args: &[&str], output: &str) {
        self.outputs
            .lock()
            .expect("outputs lock")
            .insert(command_key(program, args), Some(output.to_owned()));
    }

    fn calls(&self) -> Vec<ManagerCommand> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn called_programs(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .map(|command| command.program)
            .collect()
    }
}

impl ManagerCommandRunner for FakeRunner {
    fn run(&self, command: &ManagerCommand, _project_root: Option<&Path>) -> Option<String> {
        self.calls
            .lock()
            .expect("calls lock")
            .push(command.clone());
        self.outputs
            .lock()
            .expect("outputs lock")
            .get(&(command.program.clone(), command.args.clone()))
            .cloned()
            .flatten()
    }
}

fn options(project_root: Option<PathBuf>) -> ManagerDetectionOptions {
    ManagerDetectionOptions {
        project_root,
        temp_directory: PathBuf::from("/tmp"),
    }
}

fn create_project(files: &[(&str, &str)]) -> Result<tempfile::TempDir, Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    for (relative, content) in files {
        let path = directory.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }
    Ok(directory)
}

fn all_versions(runner: &FakeRunner) {
    runner.output("yarnpkg", &["--version"], "1.22.19");
    runner.output("npm", &["--version"], "9.5.0");
    runner.output("pnpm", &["--version"], "8.6.7");
    runner.output("bun", &["--version"], "1.0.0");
    runner.output("nub", &["--version"], "0.1.0");
    runner.output("aube", &["--version"], "0.1.0");
}

fn all_bin_paths(runner: &FakeRunner, yarn_version: &str) {
    runner.output("yarnpkg", &["--version"], yarn_version);
    runner.output("npm", &["config", "get", "prefix"], "/usr/local/bin");
    runner.output("pnpm", &["bin", "--global"], "/usr/local/pnpm");
    runner.output("bun", &["pm", "--g", "bin"], "/usr/local/bun");
    runner.output("which", &["nub"], "/usr/local/bin/nub");
    runner.output("which", &["aube"], "/usr/local/bin/aube");
}

#[test]
fn returns_all_available_package_manager_versions() {
    let runner = FakeRunner::default();
    all_versions(&runner);

    assert_eq!(
        get_available_package_managers_with(&runner, &options(None)),
        PackageManagers {
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
    let runner = FakeRunner::default();
    all_versions(&runner);
    runner.output("aube", &["--version"], "1.25.1 macos-arm64 (2026-06-30)");

    let result = get_available_package_managers_with(&runner, &options(None));
    assert_eq!(result.aube.as_deref(), Some("1.25.1"));
}

#[test]
fn unavailable_package_managers_are_none() {
    let runner = FakeRunner::default();
    runner.output("yarnpkg", &["--version"], "1.22.19");
    runner.output("pnpm", &["--version"], "8.6.7");

    assert_eq!(
        get_available_package_managers_with(&runner, &options(None)),
        PackageManagers {
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
fn infers_project_yarn_version_from_package_manager() -> Result<(), Box<dyn Error>> {
    let project = create_project(&[(
        "package.json",
        r#"{"packageManager":"yarn@4.5.1+sha.abcdef"}"#,
    )])?;
    let runner = FakeRunner::default();
    runner.output("npm", &["--version"], "9.5.0");
    runner.output("pnpm", &["--version"], "8.6.7");
    runner.output("bun", &["--version"], "1.0.0");
    runner.output("nub", &["--version"], "0.1.0");
    runner.output("aube", &["--version"], "0.1.0");

    let result = get_available_package_managers_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn.as_deref(), Some("4.5.1"));
    assert_eq!(
        runner.called_programs(),
        ["npm", "pnpm", "bun", "nub", "aube"]
    );
    Ok(())
}

#[test]
fn infers_project_yarn_version_from_conventional_yarn_path() -> Result<(), Box<dyn Error>> {
    let project = create_project(&[(
        ".yarnrc.yml",
        "yarnPath: .yarn/releases/yarn-3.2.1.cjs\n",
    )])?;
    let runner = FakeRunner::default();

    let result = get_available_package_managers_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn.as_deref(), Some("3.2.1"));
    assert!(!runner.called_programs().contains(&"yarnpkg".to_owned()));
    Ok(())
}

#[test]
fn custom_yarn_path_disables_execution_and_global_fallback() -> Result<(), Box<dyn Error>> {
    let project = create_project(&[(".yarnrc.yml", "yarnPath: ./scripts/yarn.cjs\n")])?;
    let runner = FakeRunner::default();
    runner.output("yarnpkg", &["--version"], "4.5.1");

    let result = get_available_package_managers_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn, None);
    assert!(!runner.called_programs().contains(&"yarnpkg".to_owned()));
    Ok(())
}

#[test]
fn returns_all_package_manager_bin_paths() {
    let runner = FakeRunner::default();
    all_bin_paths(&runner, "3.2.1");

    assert_eq!(
        get_package_managers_bin_paths_with(&runner, &options(None)),
        PackageManagers {
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
fn yarn_v1_uses_global_bin_path() {
    let runner = FakeRunner::default();
    all_bin_paths(&runner, "1.22.19");
    runner.output("yarn", &["global", "bin"], "/usr/local/yarn");

    let result = get_package_managers_bin_paths_with(&runner, &options(None));
    assert_eq!(result.yarn.as_deref(), Some("/usr/local/yarn"));
}

#[test]
fn failed_bin_checks_are_none() {
    let runner = FakeRunner::default();
    runner.output("pnpm", &["bin", "--global"], "/usr/local/pnpm");

    assert_eq!(
        get_package_managers_bin_paths_with(&runner, &options(None)),
        PackageManagers {
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
fn bin_checks_use_exact_commands_and_bounded_options() {
    let runner = FakeRunner::default();
    all_bin_paths(&runner, "3.2.1");

    let _result = get_package_managers_bin_paths_with(&runner, &options(None));
    let calls = runner.calls();
    assert_eq!(
        calls
            .iter()
            .map(|command| (command.program.as_str(), command.args.as_slice()))
            .collect::<Vec<_>>(),
        vec![
            ("yarnpkg", ["--version".to_owned()].as_slice()),
            (
                "npm",
                ["config".to_owned(), "get".to_owned(), "prefix".to_owned()].as_slice(),
            ),
            (
                "pnpm",
                ["bin".to_owned(), "--global".to_owned()].as_slice(),
            ),
            (
                "bun",
                ["pm".to_owned(), "--g".to_owned(), "bin".to_owned()].as_slice(),
            ),
            ("which", ["nub".to_owned()].as_slice()),
            ("which", ["aube".to_owned()].as_slice()),
        ]
    );

    for command in calls {
        assert_eq!(command.cwd, PathBuf::from("/tmp"));
        assert_eq!(
            command.environment.get("COREPACK_ENABLE_STRICT"),
            Some(&"0".to_owned())
        );
        assert_eq!(command.timeout, MANAGER_COMMAND_TIMEOUT);
        assert_eq!(command.max_output_bytes, MAX_MANAGER_OUTPUT_BYTES);
    }
}

#[test]
fn version_checks_use_exact_commands_and_bounded_options() {
    let runner = FakeRunner::default();
    all_versions(&runner);

    let _result = get_available_package_managers_with(&runner, &options(None));
    let calls = runner.calls();
    assert_eq!(
        calls
            .iter()
            .map(|command| (command.program.clone(), command.args.clone()))
            .collect::<Vec<_>>(),
        vec![
            command_key("yarnpkg", &["--version"]),
            command_key("npm", &["--version"]),
            command_key("pnpm", &["--version"]),
            command_key("bun", &["--version"]),
            command_key("nub", &["--version"]),
            command_key("aube", &["--version"]),
        ]
    );
    assert!(calls.iter().all(|command| {
        command.cwd == PathBuf::from("/tmp")
            && command.timeout == Duration::from_secs(5)
            && command.max_output_bytes == MAX_MANAGER_OUTPUT_BYTES
    }));
}

#[test]
fn project_yarn_berry_path_does_not_execute_yarn() -> Result<(), Box<dyn Error>> {
    let project = create_project(&[(
        "package.json",
        r#"{"packageManager":"yarn@4.5.1"}"#,
    )])?;
    let runner = FakeRunner::default();
    runner.output("npm", &["config", "get", "prefix"], "/usr/local/bin");
    runner.output("pnpm", &["bin", "--global"], "/usr/local/pnpm");
    runner.output("bun", &["pm", "--g", "bin"], "/usr/local/bun");
    runner.output("which", &["nub"], "/usr/local/bin/nub");
    runner.output("which", &["aube"], "/usr/local/bin/aube");

    let result = get_package_managers_bin_paths_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(
        result.yarn.as_deref(),
        Some(".yarn/releases/yarn-4.5.1.cjs")
    );
    assert_eq!(
        runner.called_programs(),
        ["npm", "pnpm", "bun", "which", "which"]
    );
    Ok(())
}

#[test]
fn oversized_package_json_suppresses_yarn_fallback() -> Result<(), Box<dyn Error>> {
    let project = tempfile::tempdir()?;
    fs::write(
        project.path().join("package.json"),
        vec![b'a'; MAX_MANAGER_CONFIG_BYTES + 1],
    )?;
    let runner = FakeRunner::default();
    runner.output("yarnpkg", &["--version"], "4.5.1");

    let result = get_available_package_managers_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn, None);
    assert!(!runner.called_programs().contains(&"yarnpkg".to_owned()));
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_package_json_suppresses_yarn_fallback() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let project = tempfile::tempdir()?;
    let outside = tempfile::NamedTempFile::new()?;
    fs::write(outside.path(), r#"{"packageManager":"yarn@4.5.1"}"#)?;
    symlink(outside.path(), project.path().join("package.json"))?;
    let runner = FakeRunner::default();
    runner.output("yarnpkg", &["--version"], "4.5.1");

    let result = get_available_package_managers_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn, None);
    assert!(!runner.called_programs().contains(&"yarnpkg".to_owned()));
    Ok(())
}

#[cfg(unix)]
#[test]
fn executable_resolution_ignores_relative_and_project_local_entries() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::PermissionsExt;

    let project = tempfile::tempdir()?;
    let local_bin = project.path().join("node_modules/.bin");
    let global = tempfile::tempdir()?;
    fs::create_dir_all(&local_bin)?;
    let local_tool = local_bin.join("npm");
    let global_tool = global.path().join("npm");
    fs::write(&local_tool, "#!/bin/sh\n")?;
    fs::write(&global_tool, "#!/bin/sh\n")?;
    fs::set_permissions(&local_tool, fs::Permissions::from_mode(0o755))?;
    fs::set_permissions(&global_tool, fs::Permissions::from_mode(0o755))?;

    let path_value = std::env::join_paths([
        PathBuf::from("relative-bin"),
        local_bin,
        global.path().to_path_buf(),
    ])?;
    assert_eq!(
        resolve_executable_in_path("npm", &path_value, Some(project.path())),
        Some(fs::canonicalize(global_tool)?)
    );
    assert_eq!(
        resolve_executable_in_path("../npm", OsStr::new("/usr/bin"), None),
        None
    );
    Ok(())
}

#[test]
fn oversized_version_output_is_rejected() {
    let runner = FakeRunner::default();
    let oversized = format!("{}1.2.3", "x".repeat(MAX_MANAGER_OUTPUT_BYTES));
    runner.output("npm", &["--version"], &oversized);

    let result = get_available_package_managers_with(&runner, &options(None));
    assert_eq!(result.npm, None);
}

#[test]
fn custom_yarn_path_with_traversal_is_never_executed() -> Result<(), Box<dyn Error>> {
    let project = create_project(&[(
        ".yarnrc.yml",
        "yarnPath: ../../attacker/yarn.cjs\n",
    )])?;
    let runner = FakeRunner::default();
    runner.output("yarnpkg", &["--version"], "4.5.1");

    let result = get_package_managers_bin_paths_with(
        &runner,
        &options(Some(project.path().to_path_buf())),
    );
    assert_eq!(result.yarn, None);
    assert!(!runner.called_programs().contains(&"yarnpkg".to_owned()));
    assert!(!runner.called_programs().contains(&"yarn".to_owned()));
    Ok(())
}

#[test]
fn constants_preserve_existing_timeout_and_add_explicit_limits() {
    assert_eq!(MANAGER_COMMAND_TIMEOUT, Duration::from_secs(5));
    assert_eq!(MAX_MANAGER_OUTPUT_BYTES, 64 * 1024);
    assert_eq!(MAX_MANAGER_CONFIG_BYTES, 1024 * 1024);
    let _owned_path: OsString = OsString::from("/tmp");
}
