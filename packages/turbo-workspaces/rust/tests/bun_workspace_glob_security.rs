use turbo_workspaces_rs::{
    BUN_WORKSPACE_GLOB_COUNT_LIMIT, BUN_WORKSPACE_GLOB_INPUT_LIMIT,
    BUN_WORKSPACE_GLOB_TOTAL_INPUT_LIMIT, is_compatible_with_bun_workspaces,
};

#[test]
fn rejects_a_workspace_glob_above_the_per_glob_limit() {
    let workspace_glob = format!("{}/*", "a".repeat(BUN_WORKSPACE_GLOB_INPUT_LIMIT - 1));
    assert!(workspace_glob.len() > BUN_WORKSPACE_GLOB_INPUT_LIMIT);
    assert!(!is_compatible_with_bun_workspaces(&[&workspace_glob]));
}

#[test]
fn rejects_more_workspace_globs_than_the_count_limit() {
    let workspace_globs = vec!["workspace"; BUN_WORKSPACE_GLOB_COUNT_LIMIT + 1];
    assert!(!is_compatible_with_bun_workspaces(&workspace_globs));
}

#[test]
fn rejects_workspace_globs_above_the_total_byte_limit() {
    let workspace_glob = format!("{}/*", "a".repeat(4_094));
    let count = (BUN_WORKSPACE_GLOB_TOTAL_INPUT_LIMIT / workspace_glob.len()) + 1;
    let workspace_globs = vec![workspace_glob.as_str(); count];

    assert!(workspace_globs.len() <= BUN_WORKSPACE_GLOB_COUNT_LIMIT);
    assert!(
        workspace_globs.iter().map(|glob| glob.len()).sum::<usize>()
            > BUN_WORKSPACE_GLOB_TOTAL_INPUT_LIMIT
    );
    assert!(!is_compatible_with_bun_workspaces(&workspace_globs));
}

#[test]
fn rejects_terminal_active_and_invisible_workspace_globs() {
    for workspace_glob in [
        "apps/\u{1b}*",
        "apps/line\n*",
        "apps/\u{202e}*",
        "apps/\u{2066}*",
        "apps/zero\u{200b}width/*",
        "apps/\u{feff}*",
    ] {
        assert!(!is_compatible_with_bun_workspaces(&[workspace_glob]));
    }
}

#[test]
fn accepts_a_safe_glob_at_the_exact_per_glob_limit() {
    let workspace_glob = format!("{}/*", "a".repeat(BUN_WORKSPACE_GLOB_INPUT_LIMIT - 2));
    assert_eq!(workspace_glob.len(), BUN_WORKSPACE_GLOB_INPUT_LIMIT);
    assert!(is_compatible_with_bun_workspaces(&[&workspace_glob]));
}

#[test]
fn does_not_reject_safe_unicode_workspace_names() {
    assert!(is_compatible_with_bun_workspaces(&[
        "apps/über",
        "packages/東京/*",
    ]));
}
