use turbo_workspaces_rs::{
    WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT, WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT,
    WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT, WorkspacePackages, WorkspacePackagesError,
    parse_workspace_packages,
};

#[test]
fn workspace_glob_count_is_bounded() {
    let globs = vec!["apps/*"; WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT + 1];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Err(WorkspacePackagesError::TooManyGlobs {
            actual: WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT + 1,
            limit: WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT,
        })
    );
}

#[test]
fn each_workspace_glob_is_bounded_before_copying_the_result() {
    let oversized = "a".repeat(WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT + 1);
    let globs = [oversized.as_str()];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Err(WorkspacePackagesError::GlobTooLarge {
            index: 0,
            bytes: WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT + 1,
            limit: WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT,
        })
    );
}

#[test]
fn total_workspace_glob_input_is_bounded() {
    let values: Vec<String> = (0..17)
        .map(|index| char::from(b'a' + index).to_string().repeat(4_096))
        .collect();
    let globs: Vec<&str> = values.iter().map(String::as_str).collect();

    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Err(WorkspacePackagesError::TotalInputTooLarge {
            bytes: 69_632,
            limit: WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT,
        })
    );
}

#[test]
fn exact_count_size_and_total_limits_are_accepted() {
    let values: Vec<String> = (0..16)
        .map(|index| char::from(b'a' + index).to_string().repeat(4_096))
        .collect();
    let globs: Vec<&str> = values.iter().map(String::as_str).collect();

    let parsed = parse_workspace_packages(WorkspacePackages::Array(&globs));
    assert_eq!(parsed, Ok(globs));
}

#[test]
fn terminal_active_and_invisible_text_is_rejected() {
    for workspace_glob in [
        "apps/\0*",
        "apps/\u{001b}[31m*",
        "apps/\u{202e}*",
        "apps/\u{2066}*",
        "apps/\u{200b}*",
    ] {
        let globs = [workspace_glob];
        assert_eq!(
            parse_workspace_packages(WorkspacePackages::Array(&globs)),
            Err(WorkspacePackagesError::UnsafeGlobText { index: 0 })
        );
    }
}

#[test]
fn public_errors_do_not_echo_attacker_controlled_glob_text() {
    let workspace_glob = "secret-token\u{001b}]8;;https://attacker.invalid\u{0007}";
    let globs = [workspace_glob];
    let result = parse_workspace_packages(WorkspacePackages::Array(&globs));
    let Err(error) = result else {
        panic!("unsafe workspace text must be rejected");
    };
    let rendered = error.to_string();
    assert_eq!(rendered, "workspace glob contains unsafe text");
    assert!(!rendered.contains("secret-token"));
    assert!(!rendered.contains("attacker.invalid"));
}
