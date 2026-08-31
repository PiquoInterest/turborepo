use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use super::{ManagerCommand, ManagerCommandRunner};

#[derive(Debug, Clone, Default)]
pub struct SystemManagerCommandRunner {
    path_override: Option<OsString>,
}

impl SystemManagerCommandRunner {
    #[must_use]
    pub fn with_path(path: OsString) -> Self {
        Self {
            path_override: Some(path),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamKind {
    Stdout,
    Stderr,
}

#[derive(Debug)]
enum StreamReadError {
    TooLarge,
    Io,
}

type StreamMessage = (StreamKind, Result<Vec<u8>, StreamReadError>);

fn read_limited<R: Read>(mut reader: R, maximum: usize) -> Result<Vec<u8>, StreamReadError> {
    let mut output = Vec::with_capacity(maximum.min(64 * 1_024));
    let mut buffer = [0_u8; 8 * 1_024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_error| StreamReadError::Io)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(count) > maximum {
            return Err(StreamReadError::TooLarge);
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

fn stop_child(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let process_group = i32::try_from(child.id()).ok().map(|pid| -pid);
        if let Some(process_group) = process_group {
            // SAFETY: the child is created as the leader of a fresh process
            // group. A negative pid targets that group, and `kill` does not
            // retain the pointer or borrow Rust-owned memory.
            let _group_kill_result = unsafe { libc::kill(process_group, libc::SIGKILL) };
        }
    }
    let _kill_result = child.kill();
    let _wait_result = child.wait();
}

fn receive_available(
    receiver: &Receiver<StreamMessage>,
    stdout: &mut Option<Result<Vec<u8>, StreamReadError>>,
    stderr: &mut Option<Result<Vec<u8>, StreamReadError>>,
) {
    while let Ok((kind, result)) = receiver.try_recv() {
        match kind {
            StreamKind::Stdout => *stdout = Some(result),
            StreamKind::Stderr => *stderr = Some(result),
        }
    }
}

fn stream_failed(result: Option<&Result<Vec<u8>, StreamReadError>>) -> bool {
    result.is_some_and(Result::is_err)
}

fn join_reader(handle: thread::JoinHandle<()>) -> bool {
    handle.join().is_ok()
}

fn run_bounded(executable: &Path, request: &ManagerCommand) -> Option<String> {
    let maximum = request.max_output_bytes.max(1);
    let mut command = Command::new(executable);
    command
        .args(&request.args)
        .current_dir(&request.cwd)
        .envs(&request.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }

    let mut child = command.spawn().ok()?;
    let stdout_pipe = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            stop_child(&mut child);
            return None;
        }
    };
    let stderr_pipe = match child.stderr.take() {
        Some(pipe) => pipe,
        None => {
            drop(stdout_pipe);
            stop_child(&mut child);
            return None;
        }
    };

    let (sender, receiver) = mpsc::channel::<StreamMessage>();
    let stdout_sender = sender.clone();
    let stdout_handle = thread::spawn(move || {
        let result = read_limited(stdout_pipe, maximum);
        let _send_result = stdout_sender.send((StreamKind::Stdout, result));
    });
    let stderr_handle = thread::spawn(move || {
        let result = read_limited(stderr_pipe, maximum);
        let _send_result = sender.send((StreamKind::Stderr, result));
    });

    let deadline = match Instant::now().checked_add(request.timeout) {
        Some(deadline) => deadline,
        None => {
            stop_child(&mut child);
            let _stdout_joined = join_reader(stdout_handle);
            let _stderr_joined = join_reader(stderr_handle);
            return None;
        }
    };
    let mut stdout_result = None;
    let mut stderr_result = None;
    let status = loop {
        receive_available(&receiver, &mut stdout_result, &mut stderr_result);
        if stream_failed(stdout_result.as_ref()) || stream_failed(stderr_result.as_ref()) {
            stop_child(&mut child);
            let _stdout_joined = join_reader(stdout_handle);
            let _stderr_joined = join_reader(stderr_handle);
            return None;
        }

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                stop_child(&mut child);
                let _stdout_joined = join_reader(stdout_handle);
                let _stderr_joined = join_reader(stderr_handle);
                return None;
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                stop_child(&mut child);
                let _stdout_joined = join_reader(stdout_handle);
                let _stderr_joined = join_reader(stderr_handle);
                return None;
            }
        }
    };

    let drain_deadline = Instant::now()
        .checked_add(Duration::from_secs(2))
        .unwrap_or_else(Instant::now);
    while (stdout_result.is_none() || stderr_result.is_none()) && Instant::now() < drain_deadline {
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok((StreamKind::Stdout, result)) => stdout_result = Some(result),
            Ok((StreamKind::Stderr, result)) => stderr_result = Some(result),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if stdout_result.is_none() || stderr_result.is_none() {
        stop_child(&mut child);
        let _stdout_joined = join_reader(stdout_handle);
        let _stderr_joined = join_reader(stderr_handle);
        return None;
    }
    if !join_reader(stdout_handle) || !join_reader(stderr_handle) || !status.success() {
        return None;
    }

    let stdout = match stdout_result? {
        Ok(bytes) => bytes,
        Err(StreamReadError::TooLarge | StreamReadError::Io) => return None,
    };
    match stderr_result? {
        Ok(_) => {}
        Err(StreamReadError::TooLarge | StreamReadError::Io) => return None,
    }
    Some(String::from_utf8_lossy(&stdout).into_owned())
}

fn is_safe_program_name(program: &str) -> bool {
    let mut components = Path::new(program).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

#[cfg(not(windows))]
fn executable_names(program: &str) -> Vec<OsString> {
    vec![OsString::from(program)]
}

#[cfg(windows)]
fn executable_names(program: &str) -> Vec<OsString> {
    if Path::new(program).extension().is_some() {
        return vec![OsString::from(program)];
    }
    let extensions = env::var_os("PATHEXT").unwrap_or_else(|| OsString::from(".COM;.EXE"));
    extensions
        .to_string_lossy()
        .split(';')
        .filter(|extension| matches!(extension.to_ascii_uppercase().as_str(), ".COM" | ".EXE"))
        .map(|extension| OsString::from(format!("{program}{extension}")))
        .collect()
}

fn is_executable(metadata: &fs::Metadata) -> bool {
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[must_use]
pub fn resolve_executable_in_path(
    program: &str,
    path_value: &OsStr,
    project_root: Option<&Path>,
) -> Option<PathBuf> {
    if program.is_empty() || !is_safe_program_name(program) {
        return None;
    }

    let project_root_absolute = project_root.filter(|root| root.is_absolute());
    let project_root_canonical = project_root.and_then(|root| fs::canonicalize(root).ok());
    let names = executable_names(program);

    for directory in env::split_paths(path_value) {
        if !directory.is_absolute() {
            continue;
        }
        if project_root_absolute.is_some_and(|root| directory.starts_with(root)) {
            continue;
        }
        let Ok(canonical_directory) = fs::canonicalize(&directory) else {
            continue;
        };
        if project_root_canonical
            .as_deref()
            .is_some_and(|root| canonical_directory.starts_with(root))
        {
            continue;
        }

        for name in &names {
            let candidate = canonical_directory.join(name);
            if project_root_absolute.is_some_and(|root| candidate.starts_with(root)) {
                continue;
            }
            let Ok(canonical_candidate) = fs::canonicalize(&candidate) else {
                continue;
            };
            if project_root_canonical
                .as_deref()
                .is_some_and(|root| canonical_candidate.starts_with(root))
            {
                continue;
            }
            let Ok(metadata) = fs::metadata(&canonical_candidate) else {
                continue;
            };
            if is_executable(&metadata) {
                return Some(canonical_candidate);
            }
        }
    }
    None
}

impl ManagerCommandRunner for SystemManagerCommandRunner {
    fn run(&self, request: &ManagerCommand, project_root: Option<&Path>) -> Option<String> {
        let path_value = self
            .path_override
            .clone()
            .or_else(|| env::var_os("PATH"))?;

        if request.program == "which" {
            let [program] = request.args.as_slice() else {
                return None;
            };
            return resolve_executable_in_path(program, &path_value, project_root)
                .map(|path| path.to_string_lossy().into_owned());
        }

        let executable = resolve_executable_in_path(&request.program, &path_value, project_root)?;
        run_bounded(&executable, request)
    }
}
