#[cfg(unix)]
use std::os::unix::{fs::PermissionsExt as _, process::CommandExt as _};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

pub const PACKAGE_MANAGER_EXEC_TIMEOUT: Duration = Duration::from_secs(5);
pub const PACKAGE_MANAGER_MAX_OUTPUT_BYTES: usize = 1_024 * 1_024;
const MAX_PROJECT_METADATA_BYTES: u64 = 1_024 * 1_024;
const COREPACK_ENVIRONMENT_KEY: &str = "COREPACK_ENABLE_STRICT";
const COREPACK_ENVIRONMENT_VALUE: &str = "0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageManager {
    Yarn,
    Npm,
    Pnpm,
    Bun,
    Nub,
    Aube,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageManagerValues {
    pub yarn: Option<String>,
    pub npm: Option<String>,
    pub pnpm: Option<String>,
    pub bun: Option<String>,
    pub nub: Option<String>,
    pub aube: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    pub program: String,
    pub arguments: Vec<String>,
    pub current_directory: PathBuf,
    pub environment: Vec<(String, String)>,
    pub timeout: Duration,
    pub maximum_output_bytes: usize,
}

impl CommandRequest {
    #[must_use]
    pub fn new(program: &str, arguments: &[&str], current_directory: &Path) -> Self {
        Self {
            program: program.to_owned(),
            arguments: arguments
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
            current_directory: current_directory.to_path_buf(),
            environment: vec![(
                COREPACK_ENVIRONMENT_KEY.to_owned(),
                COREPACK_ENVIRONMENT_VALUE.to_owned(),
            )],
            timeout: PACKAGE_MANAGER_EXEC_TIMEOUT,
            maximum_output_bytes: PACKAGE_MANAGER_MAX_OUTPUT_BYTES,
        }
    }
}

pub trait PackageManagerCommandRunner: Sync {
    fn run(&self, request: &CommandRequest) -> Option<String>;
    fn resolve(&self, program: &str) -> Option<PathBuf>;
}

#[derive(Debug, Clone)]
pub struct SystemPackageManagerCommandRunner {
    search_path: Option<OsString>,
    temporary_directory: PathBuf,
    protected_project_root: Option<PathBuf>,
    timeout_limit: Duration,
    output_limit: usize,
}

impl SystemPackageManagerCommandRunner {
    #[must_use]
    pub fn new(project_root: Option<&Path>) -> Self {
        Self::with_environment(
            env::var_os("PATH"),
            env::temp_dir(),
            project_root,
            PACKAGE_MANAGER_EXEC_TIMEOUT,
            PACKAGE_MANAGER_MAX_OUTPUT_BYTES,
        )
    }

    #[must_use]
    pub fn with_environment(
        search_path: Option<OsString>,
        temporary_directory: PathBuf,
        project_root: Option<&Path>,
        timeout_limit: Duration,
        output_limit: usize,
    ) -> Self {
        let protected_project_root = project_root.and_then(|root| fs::canonicalize(root).ok());
        Self {
            search_path,
            temporary_directory,
            protected_project_root,
            timeout_limit,
            output_limit,
        }
    }

    fn candidate_names(program: &str) -> Vec<OsString> {
        #[cfg(windows)]
        {
            let path = Path::new(program);
            if path.extension().is_some() {
                return vec![OsString::from(program)];
            }
            return [".exe", ".com"]
                .into_iter()
                .map(|extension| OsString::from(format!("{program}{extension}")))
                .collect();
        }

        #[cfg(not(windows))]
        {
            vec![OsString::from(program)]
        }
    }

    fn is_safe_program_name(program: &str) -> bool {
        if program.is_empty() {
            return false;
        }
        let mut components = Path::new(program).components();
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
    }

    fn is_executable_file(path: &Path) -> bool {
        let Ok(metadata) = fs::metadata(path) else {
            return false;
        };
        if !metadata.is_file() {
            return false;
        }

        #[cfg(unix)]
        {
            metadata.permissions().mode() & 0o111 != 0
        }

        #[cfg(not(unix))]
        {
            true
        }
    }

    fn command_status_and_output(&self, request: &CommandRequest) -> Option<(ExitStatus, Vec<u8>)> {
        let executable = self.resolve(&request.program)?;
        let mut command = Command::new(executable);
        command
            .args(&request.arguments)
            .current_dir(&request.current_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &request.environment {
            command.env(key, value);
        }

        #[cfg(unix)]
        command.process_group(0);

        let mut child = command.spawn().ok()?;
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                stop_child(&mut child);
                return None;
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                drop(stdout);
                stop_child(&mut child);
                return None;
            }
        };
        let maximum = request.maximum_output_bytes.min(self.output_limit);
        let stdout_reader = thread::spawn(move || read_bounded(stdout, maximum));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, maximum));
        let timeout = request.timeout.min(self.timeout_limit);
        let deadline = Instant::now().checked_add(timeout)?;

        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) | Err(_) => {
                    stop_child(&mut child);
                    let _stdout = stdout_reader.join();
                    let _stderr = stderr_reader.join();
                    return None;
                }
            }
        };

        let stdout_result = match stdout_reader.join() {
            Ok(result) => result,
            Err(_) => return None,
        };
        let stderr_result = match stderr_reader.join() {
            Ok(result) => result,
            Err(_) => return None,
        };
        if stderr_result.is_err() {
            return None;
        }
        let stdout = stdout_result.ok()?;
        Some((status, stdout))
    }
}

impl PackageManagerCommandRunner for SystemPackageManagerCommandRunner {
    fn run(&self, request: &CommandRequest) -> Option<String> {
        let mut effective_request = request.clone();
        effective_request.current_directory = self.temporary_directory.clone();
        let (status, stdout) = self.command_status_and_output(&effective_request)?;
        if !status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&stdout).trim().to_owned())
    }

    fn resolve(&self, program: &str) -> Option<PathBuf> {
        if !Self::is_safe_program_name(program) {
            return None;
        }
        let search_path = self.search_path.as_ref()?;
        for directory in env::split_paths(search_path) {
            if !directory.is_absolute() {
                continue;
            }
            for candidate_name in Self::candidate_names(program) {
                let candidate = directory.join(candidate_name);
                if !Self::is_executable_file(&candidate) {
                    continue;
                }
                let Ok(canonical) = fs::canonicalize(&candidate) else {
                    continue;
                };
                if self
                    .protected_project_root
                    .as_ref()
                    .is_some_and(|root| canonical.starts_with(root))
                {
                    continue;
                }
                return Some(canonical);
            }
        }
        None
    }
}

fn read_bounded<R: Read>(mut reader: R, maximum: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(output);
        }
        if read > maximum.saturating_sub(output.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "package-manager command output exceeded the safety limit",
            ));
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

fn stop_child(child: &mut Child) {
    #[cfg(unix)]
    {
        if let Ok(process_group) = i32::try_from(child.id()) {
            // SAFETY: the child was placed into a new process group whose ID is
            // its PID. A negative PID asks kill(2) to signal that group. No
            // pointer is passed and the direct child is also killed below.
            let _result = unsafe { libc::kill(-process_group, libc::SIGKILL) };
        }
    }
    let _kill_result = child.kill();
    let _wait_result = child.wait();
}

fn project_root_or_current(project_root: Option<&Path>) -> PathBuf {
    if let Some(root) = project_root {
        return root.to_path_buf();
    }
    match env::current_dir() {
        Ok(directory) => directory,
        Err(_) => PathBuf::from("."),
    }
}

fn read_project_file(project_root: &Path, relative_path: &str) -> Option<String> {
    let path = project_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() > MAX_PROJECT_METADATA_BYTES
    {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn read_package_manager(project_root: &Path) -> Option<String> {
    let raw = read_project_file(project_root, "package.json")?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value.get("packageManager")?.as_str().map(ToOwned::to_owned)
}

fn consume_digits(bytes: &[u8], mut index: usize) -> Option<usize> {
    let start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    (index > start).then_some(index)
}

fn version_end(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut index = consume_digits(bytes, start)?;
    if bytes.get(index) != Some(&b'.') {
        return None;
    }
    index = consume_digits(bytes, index + 1)?;
    if bytes.get(index) != Some(&b'.') {
        return None;
    }
    index = consume_digits(bytes, index + 1)?;
    if bytes.get(index) == Some(&b'-') {
        let prerelease_start = index + 1;
        index = prerelease_start;
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'-'))
        {
            index += 1;
        }
        if index == prerelease_start {
            return None;
        }
    }
    Some(index)
}

fn parse_package_manager_version(output: Option<&str>) -> Option<String> {
    let output = output?;
    for index in 0..output.len() {
        if output.as_bytes().get(index).is_some_and(u8::is_ascii_digit)
            && let Some(end) = version_end(output, index)
        {
            return output.get(index..end).map(ToOwned::to_owned);
        }
    }
    None
}

fn parse_yarn_package_manager(package_manager: &str) -> Option<String> {
    let value = package_manager.strip_prefix("yarn@")?;
    let end = version_end(value, 0)?;
    if end == value.len() {
        return value.get(..end).map(ToOwned::to_owned);
    }
    let suffix = value.get(end..)?.strip_prefix('+')?;
    if suffix.is_empty()
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return None;
    }
    value.get(..end).map(ToOwned::to_owned)
}

fn parse_double_quoted_value(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = index.saturating_add(2),
            b'"' => {
                let quoted = value.get(..=index)?;
                return serde_json::from_str(quoted).ok();
            }
            _ => index += 1,
        }
    }
    None
}

fn parse_single_quoted_value(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.first() != Some(&b'\'') {
        return None;
    }
    let mut index = 1;
    let mut output = String::new();
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            if bytes.get(index + 1) == Some(&b'\'') {
                output.push('\'');
                index += 2;
                continue;
            }
            return Some(output);
        }
        let character = value.get(index..)?.chars().next()?;
        output.push(character);
        index += character.len_utf8();
    }
    None
}

fn parse_unquoted_value(value: &str) -> Option<String> {
    let mut comment_start = None;
    let mut previous_was_whitespace = false;
    for (index, character) in value.char_indices() {
        if character == '#' && previous_was_whitespace {
            comment_start = Some(index);
            break;
        }
        previous_was_whitespace = character.is_whitespace();
    }
    let without_comment = comment_start
        .and_then(|index| value.get(..index))
        .unwrap_or(value)
        .trim();
    (!without_comment.is_empty()).then(|| without_comment.to_owned())
}

fn parse_yarn_path(yarn_rc: &str) -> Option<String> {
    for line in yarn_rc.lines() {
        let trimmed = line.trim_start();
        let Some(after_key) = trimmed.strip_prefix("yarnPath") else {
            continue;
        };
        let Some(value) = after_key.trim_start().strip_prefix(':') else {
            continue;
        };
        let value = value.trim_start();
        if value.is_empty() {
            return None;
        }
        return match value.as_bytes().first() {
            Some(b'"') => parse_double_quoted_value(value),
            Some(b'\'') => parse_single_quoted_value(value),
            _ => parse_unquoted_value(value),
        };
    }
    None
}

fn get_yarn_release_path_version(yarn_path: &str) -> Option<String> {
    let without_dot = yarn_path.strip_prefix("./").unwrap_or(yarn_path);
    let version = without_dot
        .strip_prefix(".yarn/releases/yarn-")?
        .strip_suffix(".cjs")?;
    let end = version_end(version, 0)?;
    (end == version.len()).then(|| version.to_owned())
}

fn get_yarn_release_path(version: &str) -> String {
    format!(".yarn/releases/yarn-{version}.cjs")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectYarnMetadata {
    version: Option<String>,
    has_project_yarn_config: bool,
}

fn get_project_yarn_metadata(project_root: &Path) -> ProjectYarnMetadata {
    if let Some(package_manager) = read_package_manager(project_root)
        && package_manager.starts_with("yarn@")
    {
        return ProjectYarnMetadata {
            version: parse_yarn_package_manager(&package_manager),
            has_project_yarn_config: true,
        };
    }

    let yarn_path = read_project_file(project_root, ".yarnrc.yml")
        .as_deref()
        .and_then(parse_yarn_path);
    let Some(yarn_path) = yarn_path else {
        return ProjectYarnMetadata {
            version: None,
            has_project_yarn_config: false,
        };
    };
    ProjectYarnMetadata {
        version: get_yarn_release_path_version(&yarn_path),
        has_project_yarn_config: true,
    }
}

fn request(program: &str, arguments: &[&str], command_cwd: &Path) -> CommandRequest {
    CommandRequest::new(program, arguments, command_cwd)
}

fn join_probe(handle: thread::ScopedJoinHandle<'_, Option<String>>) -> Option<String> {
    handle.join().ok().flatten()
}

pub fn get_available_package_managers(project_root: Option<&Path>) -> PackageManagerValues {
    let project_root = project_root_or_current(project_root);
    let command_cwd = env::temp_dir();
    let runner = SystemPackageManagerCommandRunner::new(Some(&project_root));
    get_available_package_managers_with(&project_root, &command_cwd, &runner)
}

pub fn get_available_package_managers_with<R: PackageManagerCommandRunner + ?Sized>(
    project_root: &Path,
    command_cwd: &Path,
    runner: &R,
) -> PackageManagerValues {
    let yarn_metadata = get_project_yarn_metadata(project_root);
    let (yarn, npm, pnpm, bun, nub, aube) = thread::scope(|scope| {
        let yarn_request = request("yarnpkg", &["--version"], command_cwd);
        let yarn_handle = if yarn_metadata.has_project_yarn_config {
            None
        } else {
            let runner = runner;
            Some(scope.spawn(move || runner.run(&yarn_request)))
        };
        let npm_request = request("npm", &["--version"], command_cwd);
        let pnpm_request = request("pnpm", &["--version"], command_cwd);
        let bun_request = request("bun", &["--version"], command_cwd);
        let nub_request = request("nub", &["--version"], command_cwd);
        let aube_request = request("aube", &["--version"], command_cwd);
        let npm_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&npm_request))
        };
        let pnpm_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&pnpm_request))
        };
        let bun_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&bun_request))
        };
        let nub_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&nub_request))
        };
        let aube_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&aube_request))
        };

        let yarn = yarn_metadata
            .version
            .clone()
            .or_else(|| yarn_handle.and_then(join_probe));
        (
            yarn,
            join_probe(npm_handle),
            join_probe(pnpm_handle),
            join_probe(bun_handle),
            join_probe(nub_handle),
            join_probe(aube_handle),
        )
    });

    PackageManagerValues {
        yarn: parse_package_manager_version(yarn.as_deref()),
        npm: parse_package_manager_version(npm.as_deref()),
        pnpm: parse_package_manager_version(pnpm.as_deref()),
        bun: parse_package_manager_version(bun.as_deref()),
        nub: parse_package_manager_version(nub.as_deref()),
        aube: parse_package_manager_version(aube.as_deref()),
    }
}

pub fn get_package_managers_bin_paths(project_root: Option<&Path>) -> PackageManagerValues {
    let project_root = project_root_or_current(project_root);
    let command_cwd = env::temp_dir();
    let runner = SystemPackageManagerCommandRunner::new(Some(&project_root));
    get_package_managers_bin_paths_with(&project_root, &command_cwd, &runner)
}

pub fn get_package_managers_bin_paths_with<R: PackageManagerCommandRunner + ?Sized>(
    project_root: &Path,
    command_cwd: &Path,
    runner: &R,
) -> PackageManagerValues {
    let yarn_metadata = get_project_yarn_metadata(project_root);
    let (initial_yarn, npm, pnpm, bun) = thread::scope(|scope| {
        let yarn_request = request("yarnpkg", &["--version"], command_cwd);
        let yarn_handle = if yarn_metadata.has_project_yarn_config {
            None
        } else {
            let runner = runner;
            Some(scope.spawn(move || runner.run(&yarn_request)))
        };
        let npm_request = request("npm", &["config", "get", "prefix"], command_cwd);
        let pnpm_request = request("pnpm", &["bin", "--global"], command_cwd);
        let bun_request = request("bun", &["pm", "--g", "bin"], command_cwd);
        let npm_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&npm_request))
        };
        let pnpm_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&pnpm_request))
        };
        let bun_handle = {
            let runner = runner;
            scope.spawn(move || runner.run(&bun_request))
        };
        let yarn = yarn_metadata
            .version
            .clone()
            .or_else(|| yarn_handle.and_then(join_probe));
        (
            yarn,
            join_probe(npm_handle),
            join_probe(pnpm_handle),
            join_probe(bun_handle),
        )
    });

    let yarn = if yarn_metadata.has_project_yarn_config && initial_yarn.is_none() {
        None
    } else if initial_yarn
        .as_deref()
        .is_some_and(|version| !version.starts_with("1."))
    {
        initial_yarn.as_deref().map(get_yarn_release_path)
    } else if initial_yarn.is_some() {
        runner.run(&request("yarn", &["global", "bin"], command_cwd))
    } else {
        None
    };
    let nub = runner
        .resolve("nub")
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.to_string_lossy().into_owned());
    let aube = runner
        .resolve("aube")
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|path| path.to_string_lossy().into_owned());

    PackageManagerValues {
        yarn,
        npm,
        pnpm,
        bun,
        nub,
        aube,
    }
}
