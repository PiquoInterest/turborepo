use std::{
    collections::VecDeque,
    error::Error,
    ffi::OsString,
    fs,
    path::Path,
    process::ExitStatus,
    sync::Mutex,
    time::Duration,
};

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use turbo_ignore::{
    BuildDecision, CommandOutput, CommandRunner, CommandSpec, CommitResult, CommitScope,
    ComparisonKind, Environment, ErrorCode, ErrorLevel, FORCE_ALL_COMMITS, Options, ProcessError,
    Reporter, SKIP_ALL_COMMITS, check_commit, classify_error, evaluate, find_turbo_root,
    get_comparison, get_workspace, infer_turbo_version, top_level_keys,
};

#[cfg(unix)]
fn exit_status(success: bool) -> ExitStatus {
    use std::os::unix::process::ExitStatusExt as _;
    ExitStatus::from_raw(if success { 0 } else { 1 << 8 })
}

#[cfg(windows)]
fn exit_status(success: bool) -> ExitStatus {
    use std::os::windows::process::ExitStatusExt as _;
    ExitStatus::from_raw(if success { 0 } else { 1 })
}

#[derive(Debug)]
struct MockRunner {
    responses: Mutex<VecDeque<CommandOutput>>,
    calls: Mutex<Vec<CommandSpec>>,
}

impl MockRunner {
    fn new(responses: impl IntoIterator<Item = CommandOutput>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> Result<usize, Box<dyn Error>> {
        let calls = self
            .calls
            .lock()
            .map_err(|_| std::io::Error::other("calls mutex poisoned"))?;
        Ok(calls.len())
    }

    fn calls(&self) -> Result<Vec<CommandSpec>, Box<dyn Error>> {
        let calls = self
            .calls
            .lock()
            .map_err(|_| std::io::Error::other("calls mutex poisoned"))?;
        Ok(calls.clone())
    }
}

impl CommandRunner for MockRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, ProcessError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(spec.clone());
        }
        let mut responses = self
            .responses
            .lock()
            .map_err(|_| ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })?;
        responses
            .pop_front()
            .ok_or_else(|| ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
    }
}

#[derive(Debug, Default)]
struct RecordingReporter {
    messages: Mutex<Vec<(String, String)>>,
}

impl RecordingReporter {
    fn push(&self, level: &str, message: &str) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push((level.to_owned(), message.to_owned()));
        }
    }

    fn contains(&self, needle: &str) -> Result<bool, Box<dyn Error>> {
        let messages = self
            .messages
            .lock()
            .map_err(|_| std::io::Error::other("messages mutex poisoned"))?;
        Ok(messages.iter().any(|(_, message)| message.contains(needle)))
    }
}

impl Reporter for RecordingReporter {
    fn info(&self, message: &str) {
        self.push("info", message);
    }

    fn warn(&self, message: &str) {
        self.push("warn", message);
    }

    fn error(&self, message: &str) {
        self.push("error", message);
    }

    fn log(&self, message: &str) {
        self.push("log", message);
    }
}

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn make_executable(path: &Path) -> Result<(), Box<dyn Error>> {
    write(path, "#!/bin/sh\nexit 0\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn successful_output(stdout: &str) -> CommandOutput {
    CommandOutput {
        status: exit_status(true),
        stdout: stdout.to_owned(),
        stderr: String::new(),
    }
}

fn failed_output(stderr: &str) -> CommandOutput {
    CommandOutput {
        status: exit_status(false),
        stdout: String::new(),
        stderr: stderr.to_owned(),
    }
}

fn fixture() -> Result<(TempDir, Options), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    write(
        &root.join("package.json"),
        r#"{"name":"root","devDependencies":{"turbo":"2.10.13-canary.1"}}"#,
    )?;
    write(&root.join("turbo.json"), r#"{"tasks":{"build":{}}}"#)?;
    write(&root.join("apps/web/package.json"), r#"{"name":"web"}"#)?;
    let git = root.join("tools/git");
    let turbo = root.join("tools/turbo");
    make_executable(&git)?;
    make_executable(&turbo)?;

    let options = Options {
        workspace: Some("web".to_owned()),
        task: Some("build".to_owned()),
        fallback: Some("main".to_owned()),
        directory: Some(root.join("apps/web")),
        turbo_version: None,
        turbo_path: Some(turbo),
        git_path: Some(git),
        max_output_bytes: 1_024 * 1_024,
        timeout: Duration::from_secs(2),
        current_directory: Some(root.to_path_buf()),
    };
    Ok((directory, options))
}

