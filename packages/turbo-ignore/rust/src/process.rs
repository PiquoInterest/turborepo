use std::{
    ffi::OsString,
    io::{self, Read},
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("failed to start {program}: {source}")]
    Spawn {
        program: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to wait for {program}: {source}")]
    Wait {
        program: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("subprocess {program} exceeded its {timeout:?} timeout")]
    Timeout { program: PathBuf, timeout: Duration },
    #[error("subprocess {program} exceeded the {maximum}-byte {stream} limit")]
    OutputTooLarge {
        program: PathBuf,
        stream: &'static str,
        maximum: usize,
    },
    #[error("failed reading {stream} from {program}: {source}")]
    Read {
        program: PathBuf,
        stream: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("subprocess {program} did not expose its {stream} pipe")]
    MissingPipe {
        program: PathBuf,
        stream: &'static str,
    },
    #[error("subprocess {program} output reader panicked")]
    ReaderPanic { program: PathBuf },
    #[error("subprocess output channel closed unexpectedly for {program}")]
    ReaderChannelClosed { program: PathBuf },
    #[error("subprocess timeout is too large to represent for {program}: {timeout:?}")]
    InvalidTimeout { program: PathBuf, timeout: Duration },
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, ProcessError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemCommandRunner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamKind {
    Stdout,
    Stderr,
}

impl StreamKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

#[derive(Debug)]
enum StreamReadError {
    TooLarge,
    Io(io::Error),
}

type StreamMessage = (StreamKind, Result<Vec<u8>, StreamReadError>);

fn read_limited<R: Read>(mut reader: R, maximum: usize) -> Result<Vec<u8>, StreamReadError> {
    let mut output = Vec::with_capacity(maximum.min(64 * 1_024));
    let mut buffer = [0_u8; 8 * 1_024];

    loop {
        let count = reader.read(&mut buffer).map_err(StreamReadError::Io)?;
        if count == 0 {
            return Ok(output);
        }
        if output.len().saturating_add(count) > maximum {
            return Err(StreamReadError::TooLarge);
        }
        output.extend_from_slice(&buffer[..count]);
    }
}

fn receive_stream(
    receiver: &Receiver<StreamMessage>,
    stdout: &mut Option<Result<Vec<u8>, StreamReadError>>,
    stderr: &mut Option<Result<Vec<u8>, StreamReadError>>,
) -> Result<(), TryRecvError> {
    match receiver.try_recv() {
        Ok((StreamKind::Stdout, result)) => {
            *stdout = Some(result);
            Ok(())
        }
        Ok((StreamKind::Stderr, result)) => {
            *stderr = Some(result);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn stream_error(
    program: &std::path::Path,
    kind: StreamKind,
    result: &Result<Vec<u8>, StreamReadError>,
    maximum: usize,
) -> Option<ProcessError> {
    match result {
        Ok(_) => None,
        Err(StreamReadError::TooLarge) => Some(ProcessError::OutputTooLarge {
            program: program.to_path_buf(),
            stream: kind.label(),
            maximum,
        }),
        Err(StreamReadError::Io(source)) => Some(ProcessError::Read {
            program: program.to_path_buf(),
            stream: kind.label(),
            source: io::Error::new(source.kind(), source.to_string()),
        }),
    }
}

fn stop_child(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let process_group = i32::try_from(child.id()).ok().map(|pid| -pid);
        if let Some(process_group) = process_group {
            // SAFETY: the child was spawned into a new process group whose id is
            // its pid. A negative pid targets that group and SIGKILL has no
            // borrowed-memory or lifetime requirements.
            let _group_kill_result = unsafe { libc::kill(process_group, libc::SIGKILL) };
        }
    }
    let _kill_result = child.kill();
    let _wait_result = child.wait();
}

fn join_reader(
    handle: thread::JoinHandle<()>,
    program: &std::path::Path,
) -> Result<(), ProcessError> {
    handle.join().map_err(|_| ProcessError::ReaderPanic {
        program: program.to_path_buf(),
    })
}

impl CommandRunner for SystemCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, ProcessError> {
        let maximum = spec.max_output_bytes.max(1);
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("TURBO_TELEMETRY_DISABLED", "1");

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt as _;
            command.process_group(0);
        }

        let mut child = command.spawn().map_err(|source| ProcessError::Spawn {
            program: spec.program.clone(),
            source,
        })?;

        let stdout_pipe = match child.stdout.take() {
            Some(pipe) => pipe,
            None => {
                stop_child(&mut child);
                return Err(ProcessError::MissingPipe {
                    program: spec.program.clone(),
                    stream: "stdout",
                });
            }
        };
        let stderr_pipe = match child.stderr.take() {
            Some(pipe) => pipe,
            None => {
                drop(stdout_pipe);
                stop_child(&mut child);
                return Err(ProcessError::MissingPipe {
                    program: spec.program.clone(),
                    stream: "stderr",
                });
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

        let Some(deadline) = Instant::now().checked_add(spec.timeout) else {
            stop_child(&mut child);
            let _stdout_join_result = join_reader(stdout_handle, &spec.program);
            let _stderr_join_result = join_reader(stderr_handle, &spec.program);
            return Err(ProcessError::InvalidTimeout {
                program: spec.program.clone(),
                timeout: spec.timeout,
            });
        };
        let mut stdout_result = None;
        let mut stderr_result = None;
        let status = loop {
            while let Ok(()) = receive_stream(&receiver, &mut stdout_result, &mut stderr_result) {}

            if let Some(result) = stdout_result.as_ref()
                && let Some(error) =
                    stream_error(&spec.program, StreamKind::Stdout, result, maximum)
            {
                stop_child(&mut child);
                let _stdout_join_result = join_reader(stdout_handle, &spec.program);
                let _stderr_join_result = join_reader(stderr_handle, &spec.program);
                return Err(error);
            }
            if let Some(result) = stderr_result.as_ref()
                && let Some(error) =
                    stream_error(&spec.program, StreamKind::Stderr, result, maximum)
            {
                stop_child(&mut child);
                let _stdout_join_result = join_reader(stdout_handle, &spec.program);
                let _stderr_join_result = join_reader(stderr_handle, &spec.program);
                return Err(error);
            }

            let wait_result = match child.try_wait() {
                Ok(result) => result,
                Err(source) => {
                    stop_child(&mut child);
                    let _stdout_join_result = join_reader(stdout_handle, &spec.program);
                    let _stderr_join_result = join_reader(stderr_handle, &spec.program);
                    return Err(ProcessError::Wait {
                        program: spec.program.clone(),
                        source,
                    });
                }
            };
            match wait_result {
                Some(status) => break status,
                None if Instant::now() >= deadline => {
                    stop_child(&mut child);
                    let _stdout_join_result = join_reader(stdout_handle, &spec.program);
                    let _stderr_join_result = join_reader(stderr_handle, &spec.program);
                    return Err(ProcessError::Timeout {
                        program: spec.program.clone(),
                        timeout: spec.timeout,
                    });
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        };

        while stdout_result.is_none() || stderr_result.is_none() {
            match receiver.recv_timeout(Duration::from_secs(2)) {
                Ok((StreamKind::Stdout, result)) => stdout_result = Some(result),
                Ok((StreamKind::Stderr, result)) => stderr_result = Some(result),
                Err(_) => {
                    stop_child(&mut child);
                    let _stdout_join_result = join_reader(stdout_handle, &spec.program);
                    let _stderr_join_result = join_reader(stderr_handle, &spec.program);
                    return Err(ProcessError::ReaderChannelClosed {
                        program: spec.program.clone(),
                    });
                }
            }
        }

        join_reader(stdout_handle, &spec.program)?;
        join_reader(stderr_handle, &spec.program)?;

        let stdout_result = stdout_result.ok_or_else(|| ProcessError::ReaderChannelClosed {
            program: spec.program.clone(),
        })?;
        let stderr_result = stderr_result.ok_or_else(|| ProcessError::ReaderChannelClosed {
            program: spec.program.clone(),
        })?;

        if let Some(error) =
            stream_error(&spec.program, StreamKind::Stdout, &stdout_result, maximum)
        {
            return Err(error);
        }
        if let Some(error) =
            stream_error(&spec.program, StreamKind::Stderr, &stderr_result, maximum)
        {
            return Err(error);
        }

        let stdout = match stdout_result {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => String::new(),
        };
        let stderr = match stderr_result {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(_) => String::new(),
        };

        Ok(CommandOutput {
            status,
            stdout,
            stderr,
        })
    }
}
