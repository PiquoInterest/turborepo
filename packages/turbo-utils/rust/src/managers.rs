use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

mod process;

pub use process::{SystemManagerCommandRunner, resolve_executable_in_path};

pub const MANAGER_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_MANAGER_OUTPUT_BYTES: usize = 64 * 1_024;
pub const MAX_MANAGER_CONFIG_BYTES: usize = 1_024 * 1_024;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageManagers {
    pub yarn: Option<String>,
    pub npm: Option<String>,
    pub pnpm: Option<String>,
    pub bun: Option<String>,
    pub nub: Option<String>,
    pub aube: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerDetectionOptions {
    pub project_root: Option<PathBuf>,
    pub temp_directory: PathBuf,
}

impl Default for ManagerDetectionOptions {
    fn default() -> Self {
        Self {
            project_root: None,
            temp_directory: env::temp_dir(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerCommand {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

pub trait ManagerCommandRunner: Send + Sync {
    fn run(&self, command: &ManagerCommand, project_root: Option<&Path>) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectYarnState {
    Absent,
    Version(String),
    ConfiguredUnknown,
}

#[derive(Debug)]
enum BoundedText {
    Missing,
    Text(String),
    Unsafe,
}

fn manager_command(
    program: &str,
    args: &[&str],
    options: &ManagerDetectionOptions,
) -> ManagerCommand {
    ManagerCommand {
        program: program.to_owned(),
        args: args.iter().map(|argument| (*argument).to_owned()).collect(),
        cwd: options.temp_directory.clone(),
        environment: BTreeMap::from([(
            "COREPACK_ENABLE_STRICT".to_owned(),
            "0".to_owned(),
        )]),
        timeout: MANAGER_COMMAND_TIMEOUT,
        max_output_bytes: MAX_MANAGER_OUTPUT_BYTES,
    }
}

fn run_output(
    runner: &dyn ManagerCommandRunner,
    program: &str,
    args: &[&str],
    options: &ManagerDetectionOptions,
) -> Option<String> {
    let command = manager_command(program, args, options);
    runner
        .run(&command, options.project_root.as_deref())
        .filter(|output| output.len() <= MAX_MANAGER_OUTPUT_BYTES)
        .map(|output| output.trim().to_owned())
        .filter(|output| !output.is_empty())
}

fn consume_digits(bytes: &[u8], cursor: &mut usize) -> bool {
    let start = *cursor;
    while bytes.get(*cursor).is_some_and(u8::is_ascii_digit) {
        *cursor += 1;
    }
    *cursor > start
}

fn parse_semver_at(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let mut cursor = start;
    for component in 0..3 {
        if !consume_digits(bytes, &mut cursor) {
            return None;
        }
        if component < 2 {
            if bytes.get(cursor) != Some(&b'.') {
                return None;
            }
            cursor += 1;
        }
    }

    if bytes.get(cursor) == Some(&b'-') {
        cursor += 1;
        let prerelease_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'-')
        }) {
            cursor += 1;
        }
        if cursor == prerelease_start {
            return None;
        }
    }

    Some((input.get(start..cursor)?.to_owned(), cursor))
}

#[must_use]
pub fn parse_manager_version(output: &str) -> Option<String> {
    if output.len() > MAX_MANAGER_OUTPUT_BYTES {
        return None;
    }
    output
        .as_bytes()
        .iter()
        .enumerate()
        .find_map(|(index, byte)| {
            if byte.is_ascii_digit() {
                parse_semver_at(output, index)
            } else {
                None
            }
        })
        .map(|(version, _)| version)
}

fn parse_yarn_package_manager(value: &str) -> Option<String> {
    let specification = value.strip_prefix("yarn@")?;
    if specification.len() > 256 {
        return None;
    }
    let (version, end) = parse_semver_at(specification, 0)?;
    if end == specification.len() {
        return Some(version);
    }
    let metadata = specification.get(end..)?.strip_prefix('+')?;
    (!metadata.is_empty()
        && metadata
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
    .then_some(version)
}

fn unquote_yaml_scalar(value: &str) -> &str {
    let trimmed = value.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2
        && matches!(
            (bytes.first(), bytes.last()),
            (Some(b'\''), Some(b'\'')) | (Some(b'"'), Some(b'"'))
        )
    {
        trimmed.get(1..trimmed.len() - 1).unwrap_or_default()
    } else {
        trimmed
    }
}

fn parse_conventional_yarn_path(value: &str) -> Option<String> {
    let unquoted = unquote_yaml_scalar(value);
    let normalized = unquoted.strip_prefix("./").unwrap_or(unquoted);
    let version_text = normalized
        .strip_prefix(".yarn/releases/yarn-")?
        .strip_suffix(".cjs")?;
    let (version, end) = parse_semver_at(version_text, 0)?;
    (end == version_text.len()).then_some(version)
}

fn read_bounded_text(path: &Path) -> BoundedText {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return BoundedText::Missing;
        }
        Err(_) => return BoundedText::Unsafe,
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_MANAGER_CONFIG_BYTES as u64
    {
        return BoundedText::Unsafe;
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return BoundedText::Unsafe,
    };
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let mut limited = file.by_ref().take((MAX_MANAGER_CONFIG_BYTES + 1) as u64);
    if limited.read_to_end(&mut bytes).is_err() || bytes.len() > MAX_MANAGER_CONFIG_BYTES {
        return BoundedText::Unsafe;
    }
    String::from_utf8(bytes)
        .map(BoundedText::Text)
        .unwrap_or(BoundedText::Unsafe)
}

fn package_manager_yarn_state(content: &str) -> Option<ProjectYarnState> {
    let value = serde_json::from_str::<serde_json::Value>(content).ok()?;
    let specification = value.get("packageManager")?.as_str()?;
    if !specification.starts_with("yarn@") {
        return None;
    }
    Some(
        parse_yarn_package_manager(specification)
            .map(ProjectYarnState::Version)
            .unwrap_or(ProjectYarnState::ConfiguredUnknown),
    )
}

fn yarnrc_yarn_state(root: &Path) -> ProjectYarnState {
    match read_bounded_text(&root.join(".yarnrc.yml")) {
        BoundedText::Missing => ProjectYarnState::Absent,
        BoundedText::Unsafe => ProjectYarnState::ConfiguredUnknown,
        BoundedText::Text(content) => content
            .lines()
            .find_map(|line| {
                let line = line.trim_start();
                if line.starts_with('#') {
                    None
                } else {
                    line.strip_prefix("yarnPath:")
                }
            })
            .map(|value| {
                parse_conventional_yarn_path(value)
                    .map(ProjectYarnState::Version)
                    .unwrap_or(ProjectYarnState::ConfiguredUnknown)
            })
            .unwrap_or(ProjectYarnState::Absent),
    }
}

fn project_yarn_state(project_root: Option<&Path>) -> ProjectYarnState {
    let Some(root) = project_root else {
        return ProjectYarnState::Absent;
    };
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return ProjectYarnState::ConfiguredUnknown;
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ProjectYarnState::Absent;
        }
        Err(_) => return ProjectYarnState::ConfiguredUnknown,
    }

    match read_bounded_text(&root.join("package.json")) {
        BoundedText::Unsafe => ProjectYarnState::ConfiguredUnknown,
        BoundedText::Text(content) => {
            if let Some(state) = package_manager_yarn_state(&content) {
                return state;
            }
        }
        BoundedText::Missing => {}
    }
    yarnrc_yarn_state(root)
}

fn command_version(
    runner: &dyn ManagerCommandRunner,
    program: &str,
    options: &ManagerDetectionOptions,
) -> Option<String> {
    run_output(runner, program, &["--version"], options)
        .and_then(|output| parse_manager_version(&output))
}

fn yarn_version(
    runner: &dyn ManagerCommandRunner,
    options: &ManagerDetectionOptions,
) -> Option<String> {
    match project_yarn_state(options.project_root.as_deref()) {
        ProjectYarnState::Version(version) => Some(version),
        ProjectYarnState::ConfiguredUnknown => None,
        ProjectYarnState::Absent => command_version(runner, "yarnpkg", options),
    }
}

#[must_use]
pub fn get_available_package_managers_with(
    runner: &dyn ManagerCommandRunner,
    options: &ManagerDetectionOptions,
) -> PackageManagers {
    PackageManagers {
        yarn: yarn_version(runner, options),
        npm: command_version(runner, "npm", options),
        pnpm: command_version(runner, "pnpm", options),
        bun: command_version(runner, "bun", options),
        nub: command_version(runner, "nub", options),
        aube: command_version(runner, "aube", options),
    }
}

#[must_use]
pub fn get_available_package_managers(options: &ManagerDetectionOptions) -> PackageManagers {
    get_available_package_managers_with(&SystemManagerCommandRunner::default(), options)
}

fn yarn_bin_path(
    runner: &dyn ManagerCommandRunner,
    options: &ManagerDetectionOptions,
) -> Option<String> {
    let version = yarn_version(runner, options)?;
    if version.starts_with("1.") {
        run_output(runner, "yarn", &["global", "bin"], options)
    } else {
        Some(format!(".yarn/releases/yarn-{version}.cjs"))
    }
}

fn command_bin_path(
    runner: &dyn ManagerCommandRunner,
    program: &str,
    args: &[&str],
    options: &ManagerDetectionOptions,
) -> Option<String> {
    run_output(runner, program, args, options)
}

fn executable_parent(
    runner: &dyn ManagerCommandRunner,
    program: &str,
    options: &ManagerDetectionOptions,
) -> Option<String> {
    let executable = run_output(runner, "which", &[program], options)?;
    Path::new(&executable)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_string_lossy().into_owned())
}

#[must_use]
pub fn get_package_managers_bin_paths_with(
    runner: &dyn ManagerCommandRunner,
    options: &ManagerDetectionOptions,
) -> PackageManagers {
    PackageManagers {
        yarn: yarn_bin_path(runner, options),
        npm: command_bin_path(runner, "npm", &["config", "get", "prefix"], options),
        pnpm: command_bin_path(runner, "pnpm", &["bin", "--global"], options),
        bun: command_bin_path(runner, "bun", &["pm", "--g", "bin"], options),
        nub: executable_parent(runner, "nub", options),
        aube: executable_parent(runner, "aube", options),
    }
}

#[must_use]
pub fn get_package_managers_bin_paths(options: &ManagerDetectionOptions) -> PackageManagers {
    get_package_managers_bin_paths_with(&SystemManagerCommandRunner::default(), options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parser_matches_the_typescript_expression() {
        assert_eq!(
            parse_manager_version("1.25.1 macos-arm64"),
            Some("1.25.1".into())
        );
        assert_eq!(
            parse_manager_version("v4.0.0-rc.2+build"),
            Some("4.0.0-rc.2".into())
        );
        assert_eq!(parse_manager_version("version unknown"), None);
    }

    #[test]
    fn yarn_spec_parser_requires_the_complete_value() {
        assert_eq!(
            parse_yarn_package_manager("yarn@4.5.1+sha.123"),
            Some("4.5.1".into())
        );
        assert_eq!(parse_yarn_package_manager("yarn@4.5"), None);
        assert_eq!(parse_yarn_package_manager("npm@10.0.0"), None);
    }

    #[test]
    fn conventional_yarn_path_parser_rejects_custom_paths() {
        assert_eq!(
            parse_conventional_yarn_path("./.yarn/releases/yarn-3.2.1.cjs"),
            Some("3.2.1".into())
        );
        assert_eq!(
            parse_conventional_yarn_path("../../scripts/yarn.cjs"),
            None
        );
    }
}
