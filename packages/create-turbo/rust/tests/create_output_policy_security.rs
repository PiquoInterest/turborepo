use create_turbo_rs::{
    CREATE_OUTPUT_LINE_LIMIT, CREATE_OUTPUT_SCRIPT_LIMIT, CREATE_OUTPUT_TRUNCATION_LINE,
    CREATE_OUTPUT_WORKSPACE_LIMIT, CreateDisplayScript, CreateWorkspaceDisplay,
    PNPM_INSTALL_PROFILES, render_create_get_started, render_create_success,
    render_create_workspace_summary,
};

#[test]
fn workspace_output_escapes_terminal_controls_and_directionality() {
    let workspaces = [CreateWorkspaceDisplay {
        group: "packages\u{1b}]8;;https://attacker.invalid\u{7}",
        title: "packages/ui\nspoof",
        description: Some("description\rreset\tcolumn\u{202e}rtl\u{200b}hidden"),
    }];
    let lines = render_create_workspace_summary(&workspaces, "unused");

    for line in &lines {
        assert_terminal_safe(line);
    }

    let rendered = lines.join("");
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
fn workspace_output_bounds_lines_and_entry_count() {
    let description = "A".repeat(4 * 1024 * 1024);
    let entry = CreateWorkspaceDisplay {
        group: "Library packages",
        title: "packages/ui",
        description: Some(&description),
    };
    let workspaces = vec![entry; CREATE_OUTPUT_WORKSPACE_LIMIT + 100];
    let lines = render_create_workspace_summary(&workspaces, "unused");

    assert!(
        lines.len() <= CREATE_OUTPUT_WORKSPACE_LIMIT.saturating_mul(2) + 2,
        "workspace output was not bounded: {} lines",
        lines.len()
    );
    assert!(
        lines.iter().any(|line| line == CREATE_OUTPUT_TRUNCATION_LINE),
        "bounded output must make truncation visible"
    );
    for line in &lines {
        assert!(line.len() <= CREATE_OUTPUT_LINE_LIMIT);
        assert_terminal_safe(line);
    }
}

#[test]
fn success_output_sanitizes_and_bounds_the_relative_path() {
    let relative_path = format!(
        "{}\u{1b}]8;;https://attacker.invalid\u{7}\nspoof\u{202e}",
        "x".repeat(4 * 1024 * 1024)
    );
    let line = render_create_success(false, &relative_path);

    assert!(line.len() <= CREATE_OUTPUT_LINE_LIMIT);
    assert!(line.contains("[truncated]"));
    assert_terminal_safe(&line);
}

#[test]
fn get_started_output_sanitizes_and_bounds_the_relative_path() {
    let relative_path = format!(
        "{}\u{1b}]8;;https://attacker.invalid\u{7}\nspoof\u{200b}",
        "x".repeat(4 * 1024 * 1024)
    );
    let lines = render_create_get_started(
        true,
        false,
        &relative_path,
        Some(&PNPM_INSTALL_PROFILES[1]),
        &[CreateDisplayScript::Build],
    );

    for line in &lines {
        assert!(line.len() <= CREATE_OUTPUT_LINE_LIMIT);
        assert_terminal_safe(line);
    }
    assert!(lines.iter().any(|line| line.contains("[truncated]")));
}

#[test]
fn repeated_script_output_is_bounded() {
    let scripts = vec![CreateDisplayScript::Build; CREATE_OUTPUT_SCRIPT_LIMIT + 100];
    let lines = render_create_get_started(
        true,
        true,
        "",
        Some(&PNPM_INSTALL_PROFILES[1]),
        &scripts,
    );

    assert!(lines.iter().any(|line| line == CREATE_OUTPUT_TRUNCATION_LINE));
    assert!(
        lines
            .iter()
            .filter(|line| line.contains("pnpm run build"))
            .count()
            <= CREATE_OUTPUT_SCRIPT_LIMIT
    );
}

#[test]
fn fallback_project_name_is_terminal_safe_and_bounded() {
    let project_name = format!(
        "{}\u{1b}]8;;https://attacker.invalid\u{7}\nspoof",
        "x".repeat(4 * 1024 * 1024)
    );
    let lines = render_create_workspace_summary(&[], &project_name);

    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert!(line.len() <= CREATE_OUTPUT_LINE_LIMIT);
        assert_terminal_safe(line);
    }
    assert!(lines[1].contains("[truncated]"));
}

fn assert_terminal_safe(line: &str) {
    assert!(
        !line
            .chars()
            .any(|character| character.is_control() || is_terminal_format_control(character)),
        "output contains terminal-active text: {line:?}"
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
