fn display_command(turbo: &Path, task: &str, filter: &str) -> String {
    format!(
        "{} run {} --filter=\"{}\" --dry=json",
        sanitize_for_log(&turbo.display().to_string()),
        sanitize_for_log(task),
        sanitize_for_log(filter)
    )
}

fn reported_turbo_version(output: &str) -> Option<Version> {
    output.split_whitespace().find_map(|candidate| {
        let candidate =
            candidate.trim_matches(|character: char| matches!(character, '"' | '\'' | ',' | ';'));
        let candidate = candidate.strip_prefix('v').unwrap_or(candidate);
        Version::parse(candidate).ok()
    })
}

fn verify_turbo_version(
    turbo: &Path,
    selector: Option<&str>,
    root: &Path,
    runner: &dyn CommandRunner,
    options: &Options,
    reporter: &dyn Reporter,
) -> bool {
    let Some(selector) = selector else {
        return true;
    };
    let requirement = match validate_version_selector(selector) {
        Ok(requirement) => requirement,
        Err(error) => {
            reporter.error(&format!(
                "Refusing unsafe or unsupported turbo version selector \"{}\": {error}",
                sanitize_for_log(selector)
            ));
            return false;
        }
    };
    let spec = CommandSpec {
        program: turbo.to_path_buf(),
        args: vec![OsString::from("--version")],
        cwd: root.to_path_buf(),
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes.min(64 * 1_024),
    };

    let output = match runner.run(&spec) {
        Ok(output) => output,
        Err(error) => {
            reporter.error(&format!("Could not verify the trusted Turbo binary: {error}"));
            return false;
        }
    };
    if !output.status.success() {
        let detail = if output.stderr.trim().is_empty() {
            output.stdout.trim()
        } else {
            output.stderr.trim()
        };
        reporter.error(&format!(
            "Could not verify the trusted Turbo binary: {}",
            sanitize_for_log(detail)
        ));
        return false;
    }

    let version = reported_turbo_version(&output.stdout)
        .or_else(|| reported_turbo_version(&output.stderr));
    let Some(version) = version else {
        reporter.error("Trusted Turbo binary returned an unrecognized version string");
        return false;
    };
    if !requirement.matches(&version) {
        reporter.error(&format!(
            "Trusted Turbo binary version {version} does not satisfy requested selector {}",
            sanitize_for_log(selector)
        ));
        return false;
    }

    reporter.info(&format!(
        "Using trusted Turbo version {version} from \"{}\"",
        sanitize_for_log(&turbo.display().to_string())
    ));
    true
}

fn parse_dry_run(
    stdout: &str,
    command: &str,
    workspace: &str,
    reporter: &dyn Reporter,
) -> BuildDecision {
    let parsed = match serde_json::from_str::<Option<DryRun>>(stdout) {
        Ok(Some(value)) => value,
        Ok(None) | Err(_) => {
            reporter.error(&format!("Failed to parse JSON output from `{command}`."));
            return BuildDecision::Deploy;
        }
    };

    let Some(packages) = parsed.packages else {
        reporter.info("Detected single package repo");
        return BuildDecision::Deploy;
    };

    if packages.is_empty() {
        reporter.info("This project and its dependencies are not affected");
        return BuildDecision::Skip;
    }

    if packages.len() == 1 {
        reporter.info(&format!(
            "This commit affects \"{}\"",
            sanitize_for_log(workspace)
        ));
    } else {
        let dependencies: Vec<String> = packages
            .iter()
            .skip(1)
            .take(MAX_LOGGED_DEPENDENCIES)
            .map(|package| sanitize_for_log(package))
            .collect();
        let count = packages.len() - 1;
        let noun = if count == 1 {
            "dependency"
        } else {
            "dependencies"
        };
        let omitted = count.saturating_sub(dependencies.len());
        let displayed = if omitted == 0 {
            dependencies.join(", ")
        } else {
            format!("{}, … +{omitted} more", dependencies.join(", "))
        };
        reporter.info(&format!(
            "This commit affects \"{}\" and {count} {noun} ({})",
            sanitize_for_log(workspace),
            displayed
        ));
    }

    BuildDecision::Deploy
}
