#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use turbo_utils_rs::{
    ExitCode, NotifyUpdateOutcome, PackageInfo, PreparedUpdateNotification, UpdateCheckError,
    UpdateChecker, UpdateInfo, UpgradeCommand, UpgradeCommandError, UpgradeCommandProvider,
};

#[derive(Debug)]
struct FakeChecker {
    calls: AtomicUsize,
    result: Mutex<Result<Option<UpdateInfo>, UpdateCheckError>>,
}

impl FakeChecker {
    fn new(result: Result<Option<UpdateInfo>, UpdateCheckError>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Mutex::new(result),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl UpdateChecker for FakeChecker {
    fn check(
        &self,
        _package_info: &PackageInfo,
    ) -> Result<Option<UpdateInfo>, UpdateCheckError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().expect("checker result").clone()
    }
}

#[derive(Debug)]
struct FakeUpgradeCommand {
    calls: AtomicUsize,
    result: Mutex<Result<Option<String>, UpgradeCommandError>>,
}

impl FakeUpgradeCommand {
    fn new(result: Result<Option<String>, UpgradeCommandError>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            result: Mutex::new(result),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl UpgradeCommandProvider for FakeUpgradeCommand {
    fn resolve(&self) -> Result<Option<String>, UpgradeCommandError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.result.lock().expect("command result").clone()
    }
}

fn package_info() -> PackageInfo {
    PackageInfo {
        name: "create-turbo".into(),
        version: "2.10.13".into(),
    }
}

fn available_update() -> UpdateInfo {
    UpdateInfo {
        latest: "2.11.0".into(),
    }
}

fn assert_exit(outcome: &NotifyUpdateOutcome, exit_code: ExitCode) {
    assert_eq!(outcome.exit_code, exit_code);
}

#[test]
fn checker_starts_when_notification_is_prepared_and_only_runs_once() {
    let checker = FakeChecker::new(Ok(Some(available_update())));

    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);
    assert_eq!(checker.calls(), 1);

    notification.notify(ExitCode::Success, UpgradeCommand::None, false);
    notification.notify(ExitCode::Failure, UpgradeCommand::None, false);
    assert_eq!(checker.calls(), 1);
}

#[test]
fn available_update_without_command_logs_announcement() {
    let checker = FakeChecker::new(Ok(Some(available_update())));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(ExitCode::Failure, UpgradeCommand::None, false);

    assert_exit(&outcome, ExitCode::Failure);
    assert_eq!(
        outcome.stdout,
        ["", "A new version of `create-turbo` is available!", "",]
    );
    assert!(outcome.stderr.is_empty());
}

#[test]
fn static_upgrade_command_is_logged() {
    let checker = FakeChecker::new(Ok(Some(available_update())));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Static("pnpm add -g create-turbo@latest"),
        false,
    );

    assert_eq!(
        outcome.stdout,
        [
            "",
            "A new version of `create-turbo` is available!",
            "You can update by running: pnpm add -g create-turbo@latest",
            "",
        ]
    );
}

#[test]
fn dynamic_upgrade_command_is_resolved_after_update_detection() {
    let checker = FakeChecker::new(Ok(Some(available_update())));
    let command = FakeUpgradeCommand::new(Ok(Some("npm install -g create-turbo".into())));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Dynamic(&command),
        false,
    );

    assert_eq!(command.calls(), 1);
    assert_eq!(
        outcome.stdout,
        [
            "",
            "A new version of `create-turbo` is available!",
            "You can update by running: npm install -g create-turbo",
            "",
        ]
    );
}

#[test]
fn dynamic_none_suppresses_the_command_line() {
    let checker = FakeChecker::new(Ok(Some(available_update())));
    let command = FakeUpgradeCommand::new(Ok(None));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Dynamic(&command),
        false,
    );

    assert_eq!(command.calls(), 1);
    assert_eq!(
        outcome.stdout,
        ["", "A new version of `create-turbo` is available!", "",]
    );
}

#[test]
fn empty_latest_is_treated_as_no_update() {
    let checker = FakeChecker::new(Ok(Some(UpdateInfo {
        latest: String::new(),
    })));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(ExitCode::Success, UpgradeCommand::None, false);

    assert_exit(&outcome, ExitCode::Success);
    assert!(outcome.stdout.is_empty());
    assert!(outcome.stderr.is_empty());
}

#[test]
fn no_update_skips_the_dynamic_command() {
    let checker = FakeChecker::new(Ok(None));
    let command = FakeUpgradeCommand::new(Ok(Some("must not run".into())));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(
        ExitCode::Failure,
        UpgradeCommand::Dynamic(&command),
        true,
    );

    assert_exit(&outcome, ExitCode::Failure);
    assert_eq!(command.calls(), 0);
    assert!(outcome.stdout.is_empty());
    assert!(outcome.stderr.is_empty());
}

#[test]
fn checker_failure_is_swallowed_even_in_debug_mode() {
    let checker = FakeChecker::new(Err(UpdateCheckError::new("registry unavailable")));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let outcome = notification.notify(ExitCode::Failure, UpgradeCommand::None, true);

    assert_exit(&outcome, ExitCode::Failure);
    assert!(outcome.stdout.is_empty());
    assert!(outcome.stderr.is_empty());
}

#[test]
fn dynamic_command_failure_preserves_exit_code_and_debug_behavior() {
    let checker = FakeChecker::new(Ok(Some(available_update())));
    let command = FakeUpgradeCommand::new(Err(UpgradeCommandError::new("command failed")));
    let notification = PreparedUpdateNotification::prepare(package_info(), &checker);

    let quiet = notification.notify(
        ExitCode::Failure,
        UpgradeCommand::Dynamic(&command),
        false,
    );
    assert_exit(&quiet, ExitCode::Failure);
    assert_eq!(quiet.stdout, ["", "A new version of `create-turbo` is available!"]);
    assert!(quiet.stderr.is_empty());

    let debug = notification.notify(
        ExitCode::Success,
        UpgradeCommand::Dynamic(&command),
        true,
    );
    assert_exit(&debug, ExitCode::Success);
    assert_eq!(debug.stderr, ["Update check failed: command failed"]);
    assert_eq!(command.calls(), 2);
}
