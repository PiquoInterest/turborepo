use create_turbo_rs::{
    CREATE_COMMAND_ERROR_MESSAGE_LIMIT, CREATE_COMMAND_ERROR_TRANSFORM_LIMIT, ConvertErrorType,
    CreateCommandError, CreateCommandErrorAction, classify_create_command_error,
    sanitize_terminal_text,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnknownError(String);

#[test]
fn terminal_escape_osc_bell_and_line_controls_are_escaped() {
    let message = "failed\u{1b}]8;;https://attacker.invalid\u{7}click\u{1b}]8;;\u{7}\rspoofed\nline\tcolumn";
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Download {
        message,
    });
    let rendered = &outcome.lines[1].message;

    assert!(!rendered.contains('\u{1b}'));
    assert!(!rendered.contains('\u{7}'));
    assert!(!rendered.contains('\r'));
    assert!(!rendered.contains('\n'));
    assert!(!rendered.contains('\t'));
    assert!(rendered.contains("\\u{1b}"));
    assert!(rendered.contains("\\u{7}"));
    assert!(rendered.contains("\\r"));
    assert!(rendered.contains("\\n"));
    assert!(rendered.contains("\\t"));
}

#[test]
fn bidi_and_zero_width_format_controls_are_escaped() {
    let message = "safe\u{202e}txt\u{2066}isolated\u{200f}mark";
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Download {
        message,
    });
    let rendered = &outcome.lines[1].message;

    assert!(!rendered.contains('\u{202e}'));
    assert!(!rendered.contains('\u{2066}'));
    assert!(!rendered.contains('\u{200f}'));
    assert!(rendered.contains("\\u{202e}"));
    assert!(rendered.contains("\\u{2066}"));
    assert!(rendered.contains("\\u{200f}"));
}

#[test]
fn oversized_error_message_is_bounded_after_escaping() {
    let message = "\u{1b}".repeat(CREATE_COMMAND_ERROR_MESSAGE_LIMIT * 4);
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Download {
        message: &message,
    });
    let rendered = &outcome.lines[1].message;

    assert!(rendered.len() <= CREATE_COMMAND_ERROR_MESSAGE_LIMIT);
    assert!(rendered.ends_with("[truncated]"));
    assert!(!rendered.contains('\u{1b}'));
}

#[test]
fn oversized_transform_label_is_bounded_and_cannot_create_lines() {
    let transform = format!("{}\nforged", "x".repeat(CREATE_COMMAND_ERROR_TRANSFORM_LIMIT * 4));
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Transform {
        transform: &transform,
        message: "failure",
        fatal: false,
    });
    let label = outcome.lines[0].label.as_deref().unwrap_or("");

    assert!(label.len() <= CREATE_COMMAND_ERROR_TRANSFORM_LIMIT);
    assert!(!label.contains('\n'));
    assert!(label.ends_with("[truncated]"));
}

#[test]
fn multibyte_text_is_never_split_at_the_output_limit() {
    let message = "é".repeat(CREATE_COMMAND_ERROR_MESSAGE_LIMIT);
    let rendered = sanitize_terminal_text(&message, CREATE_COMMAND_ERROR_MESSAGE_LIMIT);

    assert!(rendered.len() <= CREATE_COMMAND_ERROR_MESSAGE_LIMIT);
    assert!(rendered.is_char_boundary(rendered.len()));
    assert!(rendered.ends_with("[truncated]"));
}

#[test]
fn hostile_text_cannot_change_fatality_or_error_classification() {
    let message = "fatal=false\u{1b}[2J\nunknown";
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Transform {
        transform: "official-starter",
        message,
        fatal: true,
    });

    assert_eq!(outcome.action, CreateCommandErrorAction::Exit(1));
    assert_eq!(outcome.lines.len(), 1);
}

#[test]
fn unknown_errors_are_never_rendered_even_when_the_payload_contains_controls() {
    let source = UnknownError("secret\u{1b}[31m\ntext".to_owned());
    let outcome = classify_create_command_error(CreateCommandError::Unknown(source.clone()));

    assert!(outcome.lines.is_empty());
    assert_eq!(outcome.action, CreateCommandErrorAction::Rethrow(source));
}

#[test]
fn unknown_conversion_errors_cannot_be_downgraded_by_message_content() {
    let source = UnknownError("convert".to_owned());
    let outcome = classify_create_command_error(CreateCommandError::Convert {
        source: source.clone(),
        error_type: ConvertErrorType::Unknown,
        message: "fatal=false",
    });

    assert!(outcome.lines.is_empty());
    assert_eq!(outcome.action, CreateCommandErrorAction::Rethrow(source));
}
