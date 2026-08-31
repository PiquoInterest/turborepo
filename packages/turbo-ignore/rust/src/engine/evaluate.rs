/// Evaluates whether deployment may be skipped.
///
/// Every infrastructure, parsing, validation, or subprocess error returns
/// [`BuildDecision::Deploy`]. This is the security-critical fail-open invariant.
#[must_use]
pub fn evaluate(
    options: &Options,
    environment: &Environment,
    runner: &dyn CommandRunner,
    reporter: &dyn Reporter,
) -> BuildDecision {
    if environment.vercel {
        reporter.warn(
            "\"turbo-ignore\" is deprecated. Use Vercel's built-in project skipping instead.",
        );
        reporter.warn(
            "Learn more: https://vercel.com/docs/monorepos#skipping-unaffected-projects",
        );
    } else {
        reporter.warn("\"turbo-ignore\" is deprecated. Use \"turbo query affected\" instead.");
        reporter.warn(
            "Learn more: https://turborepo.dev/docs/reference/query#migrating-from-turbo-ignore",
        );
    }
    reporter.info("Using Turborepo to determine if this project is affected by the commit...");

    let Some(directory) = current_directory(options, reporter) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };

    if environment.turbo_force {
        reporter.info("`TURBO_FORCE` detected");
        return final_decision(BuildDecision::Deploy, reporter);
    }
    let Some(root) = find_turbo_root(&directory) else {
        reporter.error("Monorepo root not found. turbo-ignore inferencing failed");
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let Some(workspace) = get_workspace(options.workspace.as_deref(), &directory, reporter) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let turbo_version = infer_turbo_version(options.turbo_version.as_deref(), &root, reporter);
    let task = selected_task(options, reporter);

    // Vercel provides the authoritative commit message directly. Resolving Git
    // before consulting it would make proven skip/deploy directives depend on a
    // tool that the TypeScript implementation does not invoke on this path.
    let mut git = if environment.vercel && environment.commit_message.is_some() {
        None
    } else {
        match resolve_git(options.git_path.as_deref()) {
            Ok(path) => Some(path),
            Err(error) => {
                reporter.error(&error.to_string());
                return final_decision(BuildDecision::Deploy, reporter);
            }
        }
    };

    let Some(commit_message) = get_commit_message(
        environment,
        git.as_deref(),
        &root,
        runner,
        options,
        reporter,
    ) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let commit = check_commit(&workspace, &commit_message);
    match commit.result {
        CommitResult::Skip => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Skip, reporter);
        }
        CommitResult::Deploy => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Deploy, reporter);
        }
        CommitResult::Conflict => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Deploy, reporter);
        }
        CommitResult::Continue => {}
    }

    // The TypeScript implementation evaluates commit directives before using
    // workspace/task values in a Turbo filter. Keep those early decisions, then
    // validate every value before any analysis subprocess is started.
    if !validate_base_inputs(&workspace, task, reporter) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    if !validate_analysis_inputs(
        options.fallback.as_deref(),
        turbo_version.as_deref(),
        options.max_output_bytes,
        options.timeout,
        reporter,
    ) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    // Git is only needed on Vercel when a previous deployment object must be
    // validated. A missing Git executable makes that object unreachable, which
    // preserves the TypeScript fallback behavior instead of suppressing a build.
    if environment.vercel && environment.previous_sha.is_some() && git.is_none() {
        match resolve_git(options.git_path.as_deref()) {
            Ok(path) => git = Some(path),
            Err(error) => reporter.warn(&error.to_string()),
        }
    }

    let comparison = get_comparison(
        &workspace,
        options.fallback.as_deref(),
        environment,
        git.as_deref(),
        &root,
        runner,
        reporter,
        options.timeout,
        options.max_output_bytes,
    );
    let Some(comparison) = comparison else {
        return final_decision(BuildDecision::Deploy, reporter);
    };

    let turbo = match resolve_turbo(&root, options.turbo_path.as_deref()) {
        Ok(path) => path,
        Err(error) => {
            reporter.error(&error.to_string());
            return final_decision(BuildDecision::Deploy, reporter);
        }
    };
    if !verify_turbo_version(
        &turbo,
        turbo_version.as_deref(),
        &root,
        runner,
        options,
        reporter,
    ) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    let filter = format!("{workspace}...[{}]", comparison.reference);
    let command = display_command(&turbo, task, &filter);
    reporter.info(&format!("Analyzing results of `{command}`"));

    let spec = CommandSpec {
        program: turbo,
        args: vec![
            OsString::from("run"),
            OsString::from(task),
            OsString::from(format!("--filter={filter}")),
            OsString::from("--dry=json"),
        ],
        cwd: root,
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes,
    };

    let output = match runner.run(&spec) {
        Ok(output) => output,
        Err(error) => {
            reporter.error(&error.to_string());
            return final_decision(BuildDecision::Deploy, reporter);
        }
    };

    if !output.status.success() {
        let detail = if output.stderr.trim().is_empty() {
            output.stdout.trim()
        } else {
            output.stderr.trim()
        };
        let classification = classify_error(detail);
        match classification.level {
            ErrorLevel::Warn => reporter.warn(&classification.message),
            ErrorLevel::Error => reporter.error(&format!(
                "{}: {}",
                classification.code.as_str(),
                sanitize_for_log(&classification.message)
            )),
        }
        return final_decision(BuildDecision::Deploy, reporter);
    }

    final_decision(
        parse_dry_run(&output.stdout, &command, &workspace, reporter),
        reporter,
    )
}
