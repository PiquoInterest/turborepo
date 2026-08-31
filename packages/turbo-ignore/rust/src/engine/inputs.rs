fn get_commit_message(
    environment: &Environment,
    git: Option<&Path>,
    root: &Path,
    runner: &dyn CommandRunner,
    options: &Options,
    reporter: &dyn Reporter,
) -> Option<String> {
    if environment.vercel {
        if let Some(message) = environment.commit_message.as_ref() {
            if message.len() > MAX_COMMIT_MESSAGE_BYTES {
                reporter.error("Vercel commit message exceeds the 1 MiB safety limit");
                return None;
            }
            return Some(message.clone());
        }
    }

    let Some(git) = git else {
        reporter.error("Trusted Git executable is required to read the commit message");
        return None;
    };

    let spec = CommandSpec {
        program: git.to_path_buf(),
        args: vec![
            OsString::from("show"),
            OsString::from("-s"),
            OsString::from("--format=%B"),
        ],
        cwd: root.to_path_buf(),
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes,
    };

    match runner.run(&spec) {
        Ok(output) if output.status.success() => {
            if output.stdout.len() > MAX_COMMIT_MESSAGE_BYTES {
                reporter.error("Git commit message exceeds the 1 MiB safety limit");
                None
            } else {
                Some(output.stdout)
            }
        }
        Ok(output) => {
            let detail = if output.stderr.trim().is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            reporter.error(&format!(
                "Could not read the Git commit message: {}",
                sanitize_for_log(detail.trim())
            ));
            None
        }
        Err(error) => {
            reporter.error(&format!("Could not read the Git commit message: {error}"));
            None
        }
    }
}

fn selected_task<'a>(options: &'a Options, reporter: &dyn Reporter) -> &'a str {
    match options.task.as_deref() {
        Some(task) => {
            reporter.info(&format!(
                "Using \"{}\" as the task from the arguments",
                sanitize_for_log(task)
            ));
            task
        }
        None => {
            reporter.info("Using \"build\" as the task as it was unspecified");
            "build"
        }
    }
}

fn validate_base_inputs(
    workspace: &str,
    task: &str,
    reporter: &dyn Reporter,
) -> bool {
    if let Err(error) = validate_workspace(workspace) {
        reporter.error(&error.to_string());
        return false;
    }
    if let Err(error) = validate_task(task) {
        reporter.error(&error.to_string());
        return false;
    }

    true
}

fn validate_analysis_inputs(
    fallback: Option<&str>,
    turbo_version: Option<&str>,
    max_output_bytes: usize,
    timeout: Duration,
    reporter: &dyn Reporter,
) -> bool {
    if !(1_024..=MAX_OUTPUT_BYTES).contains(&max_output_bytes) {
        reporter.error(&format!(
            "max output must be between 1024 and {MAX_OUTPUT_BYTES} bytes"
        ));
        return false;
    }
    if timeout.is_zero() || timeout > MAX_TIMEOUT {
        reporter.error("timeout must be between 1 second and 30 minutes");
        return false;
    }
    if let Some(reference) = fallback {
        if let Err(error) = validate_ref(reference) {
            reporter.error(&format!("Invalid fallback ref: {error}"));
            return false;
        }
    }

    if let Some(version) = turbo_version {
        if let Err(error) = validate_version_selector(version) {
            reporter.error(&format!(
                "Refusing unsafe or unsupported turbo version selector \"{}\": {error}",
                sanitize_for_log(version)
            ));
            return false;
        }
    }

    true
}
