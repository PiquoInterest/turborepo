use create_turbo_rs::{
    CREATE_COMMAND_ERROR_EXIT_CODE, DOWNLOAD_ERROR_HEADING, ConvertErrorType,
    CreateCommandError, CreateCommandErrorAction, CreateCommandErrorLine,
    classify_create_command_error,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnknownError(&'static str);

#[test]
fn nonfatal_transform_error_logs_once_and_continues() {
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Transform {
        transform: "official-starter",
        message: "Unable to read package.json",
        fatal: false,
    });

    assert!(outcome.track_error_status);
    assert_eq!(
        outcome.lines,
        vec![CreateCommandErrorLine {
            label: Some("official-starter".to_owned()),
            message: "Unable to read package.json".to_owned(),
        }]
    );
    assert_eq!(outcome.action, CreateCommandErrorAction::Continue);
}

#[test]
fn fatal_transform_error_logs_once_and_requests_exit_one() {
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Transform {
        transform: "git-ignore",
        message: "Unable to write .gitignore",
        fatal: true,
    });

    assert_eq!(outcome.lines.len(), 1);
    assert_eq!(
        outcome.action,
        CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE)
    );
}

#[test]
fn known_conversion_error_logs_message_and_requests_exit_one() {
    let outcome = classify_create_command_error(CreateCommandError::Convert {
        source: UnknownError("known conversion"),
        error_type: ConvertErrorType::Known,
        message: "Unable to convert package manager",
    });

    assert_eq!(
        outcome.lines,
        vec![CreateCommandErrorLine {
            label: None,
            message: "Unable to convert package manager".to_owned(),
        }]
    );
    assert_eq!(
        outcome.action,
        CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE)
    );
}

#[test]
fn unknown_conversion_error_is_rethrown_without_logging() {
    let source = UnknownError("unknown conversion");
    let outcome = classify_create_command_error(CreateCommandError::Convert {
        source: source.clone(),
        error_type: ConvertErrorType::Unknown,
        message: "opaque conversion failure",
    });

    assert!(outcome.track_error_status);
    assert!(outcome.lines.is_empty());
    assert_eq!(outcome.action, CreateCommandErrorAction::Rethrow(source));
}

#[test]
fn download_error_logs_heading_then_message_and_requests_exit_one() {
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Download {
        message: "Could not connect",
    });

    assert_eq!(
        outcome.lines,
        vec![
            CreateCommandErrorLine {
                label: None,
                message: DOWNLOAD_ERROR_HEADING.to_owned(),
            },
            CreateCommandErrorLine {
                label: None,
                message: "Could not connect".to_owned(),
            },
        ]
    );
    assert_eq!(
        outcome.action,
        CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE)
    );
}

#[test]
fn unknown_error_is_rethrown_after_error_status_is_recorded() {
    let source = UnknownError("programming error");
    let outcome = classify_create_command_error(CreateCommandError::Unknown(source.clone()));

    assert!(outcome.track_error_status);
    assert!(outcome.lines.is_empty());
    assert_eq!(outcome.action, CreateCommandErrorAction::Rethrow(source));
}

#[test]
fn safe_text_is_preserved_exactly() {
    let outcome = classify_create_command_error(CreateCommandError::<UnknownError>::Transform {
        transform: "package-manager",
        message: "conversion failed: npm -> pnpm",
        fatal: false,
    });

    assert_eq!(outcome.lines[0].label.as_deref(), Some("package-manager"));
    assert_eq!(outcome.lines[0].message, "conversion failed: npm -> pnpm");
}
