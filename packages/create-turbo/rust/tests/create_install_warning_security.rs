use create_turbo_rs::{
    MAX_TERMINAL_DIAGNOSTIC_SCALARS, UnavailablePackageManagerWarning,
    WorkspacePackageManager,
};

#[test]
fn warning_terminal_fields_escape_control_text_without_mutating_raw_data() {
    let example_name =
        "community\u{1b}]8;;https://attacker.invalid\u{7}name\nspoof\u{202e}";
    let warning = UnavailablePackageManagerWarning {
        example_name,
        package_manager: WorkspacePackageManager::Aube,
    };

    assert_eq!(warning.example_name, example_name);
    assert_eq!(
        warning.terminal_example_name(),
        "community\\u{1b}]8;;https://attacker.invalid\\u{7}name\\nspoof\\u{202e}"
    );
    assert_eq!(warning.terminal_package_manager(), "aube");
}

#[test]
fn warning_terminal_example_name_is_bounded_without_copying_the_raw_value() {
    let example_name = "x".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4_096);
    let warning = UnavailablePackageManagerWarning {
        example_name: &example_name,
        package_manager: WorkspacePackageManager::Npm,
    };

    let terminal_example_name = warning.terminal_example_name();

    assert_eq!(warning.example_name.as_ptr(), example_name.as_ptr());
    assert_eq!(
        terminal_example_name.chars().count(),
        MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    assert!(terminal_example_name.ends_with('…'));
}
