#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Mutex;

use turbo_utils_rs::{
    ExitCode, NOTIFY_MAX_UNTRUSTED_CHARS, PackageInfo, PreparedUpdateNotification,
    UpdateCheckError, UpdateChecker, UpdateInfo, UpgradeCommand, UpgradeCommandError,
    UpgradeCommandProvider,
};

#[derive(Debug)]
struct Checker(Result<Option<UpdateInfo>, UpdateCheckError>);

impl UpdateChecker for Checker {
    fn check(&self, _package_info: &PackageInfo) -> Result<Option<UpdateInfo>, UpdateCheckError> {
        self.0.clone()
    }
}

#[derive(Debug)]
struct Command(Mutex<Result<Option<String>, UpgradeCommandError>>);

impl UpgradeCommandProvider for Command {
    fn resolve(&self) -> Result<Option<String>, UpgradeCommandError> {
        self.0.lock().expect("command result").clone()
    }
}

fn is_directional_format_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
    )
}

fn assert_no_terminal_controls(values: &[String]) {
    for value in values {
        assert!(
            !value.chars().any(|character| {
                character.is_control() || is_directional_format_control(character)
            }),
            "terminal control leaked in {value:?}"
        );
    }
}

#[test]
fn terminal_controls_are_escaped_in_package_name_and_command() {
    let checker = Checker(Ok(Some(UpdateInfo {
        latest: "2.11.0".into(),
    })));
    let notification = PreparedUpdateNotification::prepare(
        PackageInfo {
            name: "pkg\n\u{001b}[31mspoof".into(),
            version: "1.0.0".into(),
        },
        &checker,
    );

    let outcome = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Static("pnpm add pkg\r\nrm -rf /\u{001b}[0m"),
        false,
    );

    assert_no_terminal_controls(&outcome.stdout);
    assert!(outcome.stdout[1].contains("pkg\\n\\x1b[31mspoof"));
    assert!(outcome.stdout[2].contains("pkg\\r\\nrm -rf /\\x1b[0m"));
}

#[test]
fn unicode_directionality_controls_are_escaped_before_rendering() {
    let checker = Checker(Ok(Some(UpdateInfo {
        latest: "2.11.0".into(),
    })));
    let notification = PreparedUpdateNotification::prepare(
        PackageInfo {
            name: "pkg\u{202e}txt".into(),
            version: "1.0.0".into(),
        },
        &checker,
    );

    let outcome = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Static("pnpm add safe\u{2066}suffix"),
        false,
    );

    assert_no_terminal_controls(&outcome.stdout);
    assert!(outcome.stdout[1].contains("pkg\\u{202e}txt"));
    assert!(outcome.stdout[2].contains("safe\\u{2066}suffix"));
}

#[test]
fn dynamic_error_controls_are_escaped_before_debug_logging() {
    let checker = Checker(Ok(Some(UpdateInfo {
        latest: "2.11.0".into(),
    })));
    let command = Command(Mutex::new(Err(UpgradeCommandError::new(
        "failure\n\u{001b}[2Jspoof",
    ))));
    let notification = PreparedUpdateNotification::prepare(
        PackageInfo {
            name: "pkg".into(),
            version: "1.0.0".into(),
        },
        &checker,
    );

    let outcome = notification.notify(ExitCode::Failure, UpgradeCommand::Dynamic(&command), true);

    assert_no_terminal_controls(&outcome.stderr);
    assert_eq!(
        outcome.stderr,
        ["Update check failed: failure\\n\\x1b[2Jspoof"]
    );
}

#[test]
fn rendered_untrusted_fields_are_bounded() {
    let checker = Checker(Ok(Some(UpdateInfo {
        latest: "2.11.0".into(),
    })));
    let notification = PreparedUpdateNotification::prepare(
        PackageInfo {
            name: "n".repeat(NOTIFY_MAX_UNTRUSTED_CHARS * 4),
            version: "1.0.0".into(),
        },
        &checker,
    );
    let command = "c".repeat(NOTIFY_MAX_UNTRUSTED_CHARS * 4);

    let outcome = notification.notify(ExitCode::Success, UpgradeCommand::Static(&command), false);

    assert!(outcome.stdout[1].chars().count() <= NOTIFY_MAX_UNTRUSTED_CHARS + 50);
    assert!(outcome.stdout[2].chars().count() <= NOTIFY_MAX_UNTRUSTED_CHARS + 40);
    assert!(outcome.stdout[1].ends_with("…` is available!"));
    assert!(outcome.stdout[2].ends_with('…'));
}
