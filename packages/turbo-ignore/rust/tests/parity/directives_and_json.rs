#[test]
fn global_skip_directives_match_typescript_contract() {
    for directive in SKIP_ALL_COMMITS {
        let decision = check_commit("web", &format!("subject\n\n{directive}"));
        assert_eq!(decision.result, CommitResult::Skip);
        assert_eq!(decision.scope, CommitScope::Global);
        assert_eq!(decision.reason, format!("Found commit message: {directive}"));
    }
}

#[test]
fn global_force_directives_match_typescript_contract() {
    for directive in FORCE_ALL_COMMITS {
        let decision = check_commit("web", &format!("subject\n\n{directive}"));
        assert_eq!(decision.result, CommitResult::Deploy);
        assert_eq!(decision.scope, CommitScope::Global);
        assert_eq!(decision.reason, format!("Found commit message: {directive}"));
    }
}

#[test]
fn workspace_directives_take_precedence_over_global_directives() {
    let decision = check_commit("web", "[skip ci] [vercel deploy web]");
    assert_eq!(decision.result, CommitResult::Deploy);
    assert_eq!(decision.scope, CommitScope::Workspace);

    let decision = check_commit("web", "[vercel deploy] [vercel skip web]");
    assert_eq!(decision.result, CommitResult::Skip);
    assert_eq!(decision.scope, CommitScope::Workspace);
}

#[test]
fn only_directive_matches_typescript_for_single_directive() {
    let deploy = check_commit("web", "feat: change [vercel only web]");
    assert_eq!(deploy.result, CommitResult::Deploy);
    assert_eq!(deploy.scope, CommitScope::Workspace);

    let skip = check_commit("web", "feat: change [vercel only api]");
    assert_eq!(skip.result, CommitResult::Skip);
    assert_eq!(skip.scope, CommitScope::Workspace);
}

#[test]
fn conflicting_directives_match_typescript_contract() {
    let workspace = check_commit(
        "web",
        "[vercel deploy web] and [vercel skip web]",
    );
    assert_eq!(workspace.result, CommitResult::Conflict);
    assert_eq!(workspace.scope, CommitScope::Workspace);

    let global = check_commit("web", "[vercel deploy] and [skip ci]");
    assert_eq!(global.result, CommitResult::Conflict);
    assert_eq!(global.scope, CommitScope::Global);
}

#[test]
fn no_directive_continues_analysis() {
    let decision = check_commit("web", "ordinary commit");
    assert_eq!(decision.result, CommitResult::Continue);
    assert_eq!(decision.scope, CommitScope::Global);
    assert_eq!(
        decision.reason,
        "No deploy or skip string found in commit message."
    );
}

#[test]
fn json5_scanner_recognizes_turbo_top_level_keys() -> Result<(), Box<dyn Error>> {
    let keys = top_level_keys(
        r#"
        // comment
        {
          extends: ["//"],
          'tasks': {
            build: { outputs: ["dist/**",], },
          },
        }
        "#,
    )?;
    assert_eq!(keys, vec!["extends", "tasks"]);
    Ok(())
}

#[test]
fn json5_scanner_validates_the_complete_document() {
    for invalid in [
        "{ tasks: { build: true } trailing: false }",
        "{ tasks: [1, 2 }",
        "{ tasks: unknownIdentifier }",
        "{ tasks: { build: '\\uD800' } }",
        "{ tasks: {} } garbage",
    ] {
        assert!(
            top_level_keys(invalid).is_err(),
            "malformed JSON5 should be rejected: {invalid}"
        );
    }
}

#[test]
fn json5_scanner_has_a_finite_nesting_limit() {
    let nested = format!("{{tasks:{}}}", "[".repeat(140) + &"]".repeat(140));
    assert!(top_level_keys(&nested).is_err());
}

#[test]
fn json5_scanner_accepts_supported_number_forms_and_rejects_ambiguous_ones() {
    for valid in [
        "{value:.5}",
        "{value:1.}",
        "{value:-1.25e+2}",
        "{value:0xCAFE}",
        "{value:+Infinity}",
    ] {
        assert!(top_level_keys(valid).is_ok(), "valid JSON5 rejected: {valid}");
    }

    for invalid in [
        "{value:.}",
        "{value:--1}",
        "{value:1e}",
        "{value:0x}",
        "{value:1.2.3}",
    ] {
        assert!(
            top_level_keys(invalid).is_err(),
            "invalid JSON5 accepted: {invalid}"
        );
    }
}

#[test]
fn root_discovery_prefers_nearest_non_extending_turbo_config() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    write(&directory.path().join("turbo.json"), r#"{"tasks":{}}"#)?;
    write(
        &directory.path().join("apps/web/turbo.json"),
        r#"{"extends":["//"]}"#,
    )?;
    write(
        &directory.path().join("apps/web/src/file.txt"),
        "fixture",
    )?;

    let root = find_turbo_root(&directory.path().join("apps/web/src"));
    assert_eq!(root.as_deref(), Some(directory.path()));
    Ok(())
}

