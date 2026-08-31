use std::{
    error::Error,
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use pretty_assertions::assert_eq;
use turbo_ignore::{
    BuildDecision, CommandRunner, CommandSpec, Environment, Options, Reporter,
    SystemCommandRunner, check_commit, evaluate, sanitize_for_log, validate_ref,
    validate_task, validate_version_selector, validate_workspace,
};

#[derive(Debug, Default)]
struct SilentReporter;

impl Reporter for SilentReporter {
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn log(&self, _message: &str) {}
}

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn make_executable(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

