use create_turbo_rs::{
    CREATE_INSTALL_WARNING_EXAMPLE_LIMIT, CREATE_INSTALL_WARNING_LINE_LIMIT,
    UnavailablePackageManagerWarning, WorkspacePackageManager,
    render_unavailable_package_manager_warning,
};

const TRUNCATION_MARKER: &str = "[truncated]";

#[test]
fn warning_renderer_escapes_terminal_controls_and_directionality() {
    let hostile_example = concat!(
        "community\u{1b}]8;;https://attacker.invalid\u{7}name\nspoof",
        "\rreset\tcolumn\u{202e}rtl\u{200b}hidden"
    );
    let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
        example_name: hostile_example,
        package_manager: WorkspacePackageManager::Aube,
    });

    for line in &lines {
        assert_terminal_safe(line);
    }

    let rendered = format!("{}{}", lines[0], lines[1]);
    for escaped in [
        "\\u{1b}",
        "\\u{7}",
        "\\n",
        "\\r",
        "\\t",
        "\\u{202e}",
        "\\u{200b}",
    ] {
        assert!(
            rendered.contains(escaped),
            "missing escaped terminal fragment: {escaped}"
        );
    }
}

#[test]
fn warning_renderer_bounds_large_untrusted_example_names() {
    let example_name = "x".repeat(4 * 1024 * 1024);
    let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
        example_name: &example_name,
        package_manager: WorkspacePackageManager::Npm,
    });

    assert_eq!(
        CREATE_INSTALL_WARNING_EXAMPLE_LIMIT,
        CREATE_INSTALL_WARNING_LINE_LIMIT / 2
    );
    for line in &lines {
        assert!(line.len() <= CREATE_INSTALL_WARNING_LINE_LIMIT);
        assert!(line.contains(TRUNCATION_MARKER));
        assert_terminal_safe(line);
    }
}

#[test]
fn warning_renderer_truncates_without_splitting_multibyte_text() {
    let example_name = "🦀".repeat(CREATE_INSTALL_WARNING_EXAMPLE_LIMIT);
    let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
        example_name: &example_name,
        package_manager: WorkspacePackageManager::Pnpm,
    });

    for line in &lines {
        assert!(line.len() <= CREATE_INSTALL_WARNING_LINE_LIMIT);
        assert!(line.contains(TRUNCATION_MARKER));
        assert_terminal_safe(line);
    }
}

#[test]
fn warning_renderer_never_emits_terminal_active_unicode() {
    let example_name = "\u{00ad}\u{034f}\u{061c}\u{180e}\u{200f}\u{2028}\u{2066}\u{feff}\u{fff9}";
    let lines = render_unavailable_package_manager_warning(UnavailablePackageManagerWarning {
        example_name,
        package_manager: WorkspacePackageManager::Bun,
    });

    for line in &lines {
        assert_terminal_safe(line);
    }
}

fn assert_terminal_safe(line: &str) {
    assert!(
        !line
            .chars()
            .any(|character| character.is_control() || is_terminal_format_control(character)),
        "warning output contains terminal-active text: {line:?}"
    );
}

fn is_terminal_format_control(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{2028}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
    )
}
