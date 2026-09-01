use turbo_workspaces_rs::is_compatible_with_bun_workspaces;

#[test]
fn accepts_one_simple_workspace_glob() {
    assert!(is_compatible_with_bun_workspaces(&["apps/*"]));
}

#[test]
fn accepts_multiple_simple_workspace_globs() {
    assert!(is_compatible_with_bun_workspaces(&[
        "apps/*",
        "packages/*",
    ]));
}

#[test]
fn accepts_a_root_wildcard() {
    assert!(is_compatible_with_bun_workspaces(&["*"]));
}

#[test]
fn rejects_a_double_star_glob() {
    assert!(!is_compatible_with_bun_workspaces(&[
        "workspaces/**/*",
    ]));
}

#[test]
fn rejects_when_any_workspace_glob_uses_double_star() {
    assert!(!is_compatible_with_bun_workspaces(&[
        "apps/*",
        "packages/**/*",
    ]));
}

#[test]
fn rejects_a_wildcard_before_the_final_path_segment() {
    assert!(!is_compatible_with_bun_workspaces(&[
        "apps/*",
        "packages/*/utils/*",
    ]));
}

#[test]
fn rejects_a_wildcard_in_an_intermediate_hyphenated_segment() {
    assert!(!is_compatible_with_bun_workspaces(&[
        "internal-*/*",
    ]));
}

#[test]
fn accepts_an_empty_workspace_list_like_array_every() {
    assert!(is_compatible_with_bun_workspaces(&[]));
}

#[test]
fn accepts_literal_workspace_paths() {
    assert!(is_compatible_with_bun_workspaces(&[
        "apps/web",
        "packages/config",
    ]));
}

#[test]
fn preserves_the_source_single_segment_wildcard_behavior() {
    assert!(is_compatible_with_bun_workspaces(&["internal-*"]));
}

#[test]
fn preserves_question_mark_as_an_ordinary_character() {
    assert!(is_compatible_with_bun_workspaces(&["apps/?"]));
}

#[test]
fn rejects_the_source_fancy_glob_characters() {
    for workspace_glob in [
        "!apps/*",
        "apps/[ab]",
        "apps/{web,docs}",
        "apps/]",
        "apps/!docs",
    ] {
        assert!(!is_compatible_with_bun_workspaces(&[workspace_glob]));
    }
}
