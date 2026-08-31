use std::{path::PathBuf, process::ExitCode, time::Duration};

use clap::Parser;
use turbo_ignore::{ConsoleReporter, Environment, Options, SystemCommandRunner, evaluate};

const MAX_BUFFER_KIB: usize = 64 * 1_024;
const MAX_TIMEOUT_SECONDS: u64 = 30 * 60;

fn parse_buffer_kib(value: &str) -> Result<usize, String> {
    let kib = value
        .parse::<usize>()
        .map_err(|_| "max-buffer must be a positive integer in KiB".to_owned())?;
    if kib == 0 || kib > MAX_BUFFER_KIB {
        return Err(format!(
            "max-buffer must be between 1 and {MAX_BUFFER_KIB} KiB"
        ));
    }
    kib.checked_mul(1_024)
        .ok_or_else(|| "max-buffer is too large".to_owned())
}

fn parse_timeout(value: &str) -> Result<u64, String> {
    let seconds = value
        .parse::<u64>()
        .map_err(|_| "timeout must be a positive integer in seconds".to_owned())?;
    if seconds == 0 || seconds > MAX_TIMEOUT_SECONDS {
        return Err(format!(
            "timeout must be between 1 and {MAX_TIMEOUT_SECONDS} seconds"
        ));
    }
    Ok(seconds)
}

#[derive(Debug, Parser)]
#[command(
    name = "turbo-ignore",
    version,
    about = "Only proceed with deployment if the workspace or any dependency changed",
    disable_help_subcommand = true
)]
struct Cli {
    /// Workspace being deployed. Inferred from package.json when omitted.
    workspace: Option<String>,

    /// Task to execute.
    #[arg(short = 't', long, default_value = "build")]
    task: Option<String>,

    /// Comparison ref used when Vercel has no previous deployment SHA.
    #[arg(short = 'f', long)]
    fallback: Option<String>,

    /// Directory to inspect. Defaults to the current directory.
    #[arg(short = 'd', long)]
    directory: Option<PathBuf>,

    /// Expected Turbo semver requirement. Remote package specs are rejected.
    #[arg(long)]
    turbo_version: Option<String>,

    /// Absolute path to a trusted, already installed Turbo binary.
    #[arg(long)]
    turbo_path: Option<PathBuf>,

    /// Absolute path to a trusted Git binary.
    #[arg(long)]
    git_path: Option<PathBuf>,

    /// Maximum captured stdout or stderr in KiB.
    #[arg(short = 'b', long = "max-buffer", value_parser = parse_buffer_kib, default_value = "1024")]
    max_output_bytes: usize,

    /// Per-process timeout in seconds.
    #[arg(long, value_parser = parse_timeout, default_value = "120")]
    timeout: u64,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = Options {
        workspace: cli.workspace,
        task: cli.task,
        fallback: cli.fallback,
        directory: cli.directory,
        turbo_version: cli.turbo_version,
        turbo_path: cli.turbo_path,
        git_path: cli.git_path,
        max_output_bytes: cli.max_output_bytes,
        timeout: Duration::from_secs(cli.timeout),
        current_directory: None,
    };
    let decision = evaluate(
        &options,
        &Environment::from_process(),
        &SystemCommandRunner,
        &ConsoleReporter,
    );
    let code = match decision {
        turbo_ignore::BuildDecision::Skip => 0_u8,
        turbo_ignore::BuildDecision::Deploy => 1_u8,
    };
    ExitCode::from(code)
}
