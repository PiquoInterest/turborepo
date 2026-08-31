use create_turbo_rs::{
    CreateDisplayScript, CreateWorkspaceDisplay, PNPM_INSTALL_PROFILES, render_create_get_started,
    render_create_success, render_create_workspace_summary,
};

#[test]
fn empty_workspace_summary_uses_the_project_name_under_apps() {
    assert_eq!(
        render_create_workspace_summary(&[], "web-app"),
        ["apps".to_owned(), " - web-app".to_owned()]
    );
}

#[test]
fn workspace_summary_preserves_group_transitions_and_falsey_descriptions() {
    let workspaces = [
        CreateWorkspaceDisplay {
            group: "Application packages",
            title: "apps/admin",
            description: Some("Admin application"),
        },
        CreateWorkspaceDisplay {
            group: "Application packages",
            title: "apps/web",
            description: Some(""),
        },
        CreateWorkspaceDisplay {
            group: "Library packages",
            title: "packages/ui",
            description: None,
        },
    ];

    assert_eq!(
        render_create_workspace_summary(&workspaces, "unused"),
        [
            "Application packages".to_owned(),
            " - apps/admin: Admin application".to_owned(),
            " - apps/web".to_owned(),
            "Library packages".to_owned(),
            " - packages/ui".to_owned(),
        ]
    );
}

#[test]
fn success_output_matches_the_current_directory_branch() {
    assert_eq!(
        render_create_success(true, "ignored"),
        ">>> Success! Your new Turborepo is ready."
    );
}

#[test]
fn success_output_matches_the_created_directory_branch() {
    assert_eq!(
        render_create_success(false, "examples/basic"),
        ">>> Success! Created your Turborepo at examples/basic"
    );
}

#[test]
fn get_started_output_matches_the_typescript_safe_text() {
    let lines = render_create_get_started(
        true,
        false,
        "examples/basic",
        Some(&PNPM_INSTALL_PROFILES[1]),
        &[CreateDisplayScript::Build, CreateDisplayScript::Test],
    );

    assert_eq!(
        lines,
        [
            String::new(),
            "To get started:".to_owned(),
            "- Change to the directory: cd examples/basic".to_owned(),
            "- Enable Remote Caching (recommended): pnpm dlx turbo login".to_owned(),
            "   - Learn more: https://turborepo.dev/remote-cache".to_owned(),
            String::new(),
            "- Run commands with Turborepo:".to_owned(),
            "   - pnpm run build: Build all apps and packages".to_owned(),
            "   - pnpm run test: Test all apps and packages".to_owned(),
            "- Run a command twice to hit cache".to_owned(),
        ]
    );
}

#[test]
fn get_started_output_is_absent_without_package_json_or_manager_metadata() {
    assert!(
        render_create_get_started(
            false,
            false,
            "example",
            Some(&PNPM_INSTALL_PROFILES[1]),
            &[CreateDisplayScript::Build],
        )
        .is_empty()
    );
    assert!(
        render_create_get_started(true, false, "example", None, &[CreateDisplayScript::Build],)
            .is_empty()
    );
}
