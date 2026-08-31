use create_turbo_rs::{
    UnavailablePackageManagerWarning, WorkspacePackageManager,
    render_unavailable_package_manager_warning,
};

#[test]
fn unavailable_warning_matches_the_typescript_safe_text() {
    let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
        example_name: "community-example",
        package_manager: WorkspacePackageManager::Aube,
    });

    assert_eq!(
        lines,
        [
            "Unable to install dependencies - \"community-example\" uses \"aube\" which could not \
             be found."
                .to_owned(),
            "Try running without \"--skip-transforms\" to convert \"community-example\" to a \
             package manager that is available on your system."
                .to_owned(),
        ]
    );
}

#[test]
fn unavailable_warning_renders_every_closed_manager_name() {
    let managers = [
        WorkspacePackageManager::Yarn,
        WorkspacePackageManager::Npm,
        WorkspacePackageManager::Pnpm,
        WorkspacePackageManager::Bun,
        WorkspacePackageManager::Nub,
        WorkspacePackageManager::Aube,
    ];

    for manager in managers {
        let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
            example_name: "example",
            package_manager: manager,
        });
        let quoted_manager = format!("\"{}\"", manager.as_str());

        assert!(lines[0].contains(&quoted_manager));
        assert_eq!(
            lines[1],
            "Try running without \"--skip-transforms\" to convert \"example\" to a package \
             manager that is available on your system."
        );
    }
}
