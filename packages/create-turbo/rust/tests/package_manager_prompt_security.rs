use std::cell::Cell;

use create_turbo_rs::{
    PACKAGE_MANAGER_PROMPT_ORDER, PackageManagerAvailability, PackageManagerPromptChoice,
    PackageManagerPromptError, PackageManagerSelector, WorkspacePackageManager,
    resolve_package_manager_prompt,
};

struct Availability {
    enabled: Option<WorkspacePackageManager>,
    version_calls: Cell<usize>,
}

impl PackageManagerAvailability for Availability {
    fn version(&self, manager: WorkspacePackageManager) -> Option<&str> {
        self.version_calls.set(self.version_calls.get() + 1);
        (Some(manager) == self.enabled).then_some("1.0.0")
    }
}

struct RecordingSelector {
    selected: WorkspacePackageManager,
    calls: usize,
    choice_count: usize,
}

impl PackageManagerSelector for RecordingSelector {
    type Error = &'static str;

    fn select(
        &mut self,
        choices: &[PackageManagerPromptChoice<'_>],
    ) -> Result<WorkspacePackageManager, Self::Error> {
        self.calls += 1;
        self.choice_count = choices.len();
        Ok(self.selected)
    }
}

#[test]
fn case_whitespace_paths_and_confusables_do_not_become_direct_manager_values() {
    for candidate in [
        "NPM",
        "npm ",
        " npm",
        "npm/../pnpm",
        "pnрm",
        "ｎｐｍ",
        "bun\u{200d}",
    ] {
        let availability = Availability {
            enabled: Some(WorkspacePackageManager::Pnpm),
            version_calls: Cell::new(0),
        };
        let mut selector = RecordingSelector {
            selected: WorkspacePackageManager::Pnpm,
            calls: 0,
            choice_count: 0,
        };

        let result = resolve_package_manager_prompt(
            Some(candidate),
            false,
            &availability,
            &mut selector,
        )
        .expect("the typed selector returns an installed manager")
        .expect("transforms are enabled");

        assert_eq!(result.name, WorkspacePackageManager::Pnpm);
        assert_eq!(selector.calls, 1);
    }
}

#[test]
fn a_large_unknown_manager_is_borrowed_and_cannot_expand_the_choice_set() {
    let candidate = "attacker-manager".repeat(500_000);
    let availability = Availability {
        enabled: Some(WorkspacePackageManager::Npm),
        version_calls: Cell::new(0),
    };
    let mut selector = RecordingSelector {
        selected: WorkspacePackageManager::Npm,
        calls: 0,
        choice_count: 0,
    };

    let result = resolve_package_manager_prompt(
        Some(&candidate),
        false,
        &availability,
        &mut selector,
    )
    .expect("the typed selector returns an installed manager")
    .expect("transforms are enabled");

    assert_eq!(result.name, WorkspacePackageManager::Npm);
    assert_eq!(selector.choice_count, PACKAGE_MANAGER_PROMPT_ORDER.len());
}

#[test]
fn selector_cannot_smuggle_an_unavailable_manager_into_the_result() {
    let availability = Availability {
        enabled: Some(WorkspacePackageManager::Npm),
        version_calls: Cell::new(0),
    };
    let mut selector = RecordingSelector {
        selected: WorkspacePackageManager::Aube,
        calls: 0,
        choice_count: 0,
    };

    let error = resolve_package_manager_prompt(None, false, &availability, &mut selector)
        .expect_err("disabled selections must fail closed");

    assert_eq!(
        error,
        PackageManagerPromptError::UnavailableSelection(WorkspacePackageManager::Aube)
    );
    assert_eq!(selector.calls, 1);
}

#[test]
fn choice_count_is_always_bounded_to_the_closed_manager_enum() {
    for enabled in PACKAGE_MANAGER_PROMPT_ORDER {
        let availability = Availability {
            enabled: Some(enabled),
            version_calls: Cell::new(0),
        };
        let mut selector = RecordingSelector {
            selected: enabled,
            calls: 0,
            choice_count: 0,
        };

        resolve_package_manager_prompt(None, false, &availability, &mut selector)
            .expect("the selected installed manager succeeds");

        assert_eq!(selector.choice_count, 6);
        assert_eq!(selector.calls, 1);
    }
}

#[test]
fn the_core_never_retries_a_rejected_disabled_selection() {
    let availability = Availability {
        enabled: None,
        version_calls: Cell::new(0),
    };
    let mut selector = RecordingSelector {
        selected: WorkspacePackageManager::Npm,
        calls: 0,
        choice_count: 0,
    };

    let error = resolve_package_manager_prompt(None, false, &availability, &mut selector)
        .expect_err("an all-disabled prompt must not synthesize a version");

    assert_eq!(
        error,
        PackageManagerPromptError::UnavailableSelection(WorkspacePackageManager::Npm)
    );
    assert_eq!(selector.calls, 1);
}
