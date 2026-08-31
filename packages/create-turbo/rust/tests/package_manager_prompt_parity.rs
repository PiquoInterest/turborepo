use std::cell::RefCell;

use create_turbo_rs::{
    PACKAGE_MANAGER_PROMPT_ORDER, PackageManagerAvailability, PackageManagerPromptChoice,
    PackageManagerPromptError, PackageManagerSelector, WorkspacePackageManager,
    resolve_package_manager_prompt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedChoice {
    name: WorkspacePackageManager,
    version: Option<String>,
    disabled: bool,
}

struct Availability {
    versions: [Option<String>; 6],
    calls: RefCell<Vec<WorkspacePackageManager>>,
}

impl Default for Availability {
    fn default() -> Self {
        Self {
            versions: std::array::from_fn(|_| None),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Availability {
    fn installed(manager: WorkspacePackageManager, version: &str) -> Self {
        let mut value = Self::default();
        value.versions[index(manager)] = Some(version.to_owned());
        value
    }

    fn set(&mut self, manager: WorkspacePackageManager, version: Option<&str>) {
        self.versions[index(manager)] = version.map(str::to_owned);
    }
}

impl PackageManagerAvailability for Availability {
    fn version(&self, manager: WorkspacePackageManager) -> Option<&str> {
        self.calls.borrow_mut().push(manager);
        self.versions[index(manager)].as_deref()
    }
}

struct Selector {
    result: Result<WorkspacePackageManager, &'static str>,
    calls: Vec<Vec<OwnedChoice>>,
}

impl Selector {
    fn returning(manager: WorkspacePackageManager) -> Self {
        Self {
            result: Ok(manager),
            calls: Vec::new(),
        }
    }

    fn failing(message: &'static str) -> Self {
        Self {
            result: Err(message),
            calls: Vec::new(),
        }
    }
}

impl PackageManagerSelector for Selector {
    type Error = &'static str;

    fn select(
        &mut self,
        choices: &[PackageManagerPromptChoice<'_>],
    ) -> Result<WorkspacePackageManager, Self::Error> {
        self.calls.push(
            choices
                .iter()
                .map(|choice| OwnedChoice {
                    name: choice.name,
                    version: choice.version.map(str::to_owned),
                    disabled: choice.disabled,
                })
                .collect(),
        );
        self.result
    }
}

struct ExplodingSelector;

impl PackageManagerSelector for ExplodingSelector {
    type Error = &'static str;

    fn select(
        &mut self,
        _choices: &[PackageManagerPromptChoice<'_>],
    ) -> Result<WorkspacePackageManager, Self::Error> {
        panic!("the source branch must not prompt")
    }
}

fn index(manager: WorkspacePackageManager) -> usize {
    match manager {
        WorkspacePackageManager::Npm => 0,
        WorkspacePackageManager::Pnpm => 1,
        WorkspacePackageManager::Yarn => 2,
        WorkspacePackageManager::Bun => 3,
        WorkspacePackageManager::Nub => 4,
        WorkspacePackageManager::Aube => 5,
    }
}

#[test]
fn prompt_order_matches_the_typescript_choice_source() {
    assert_eq!(
        PACKAGE_MANAGER_PROMPT_ORDER,
        [
            WorkspacePackageManager::Npm,
            WorkspacePackageManager::Pnpm,
            WorkspacePackageManager::Yarn,
            WorkspacePackageManager::Bun,
            WorkspacePackageManager::Nub,
            WorkspacePackageManager::Aube,
        ]
    );
}

#[test]
fn skip_transforms_returns_none_without_availability_or_prompt_access() {
    let availability = Availability::default();
    let mut selector = ExplodingSelector;

    let result = resolve_package_manager_prompt(
        Some("npm"),
        true,
        &availability,
        &mut selector,
    )
    .expect("skipTransforms is a successful no-selection branch");

    assert_eq!(result, None);
    assert!(availability.calls.borrow().is_empty());
}

#[test]
fn every_exact_installed_manager_is_returned_without_prompting() {
    for manager in PACKAGE_MANAGER_PROMPT_ORDER {
        let availability = Availability::installed(manager, "9.1.0");
        let mut selector = ExplodingSelector;

        let result = resolve_package_manager_prompt(
            Some(manager.as_str()),
            false,
            &availability,
            &mut selector,
        )
        .expect("an exact installed manager must resolve directly")
        .expect("transforms are enabled, so a selection must be returned");

        assert_eq!(result.name, manager);
        assert_eq!(result.version, Some("9.1.0"));
        assert_eq!(&*availability.calls.borrow(), &[manager]);
    }
}

#[test]
fn unavailable_manager_argument_falls_back_to_the_prompt() {
    let availability = Availability::installed(WorkspacePackageManager::Pnpm, "10.0.0");
    let mut selector = Selector::returning(WorkspacePackageManager::Pnpm);

    let result = resolve_package_manager_prompt(
        Some("npm"),
        false,
        &availability,
        &mut selector,
    )
    .expect("the prompt selection succeeds")
    .expect("transforms are enabled");

    assert_eq!(result.name, WorkspacePackageManager::Pnpm);
    assert_eq!(result.version, Some("10.0.0"));
    assert_eq!(selector.calls.len(), 1);
}

#[test]
fn empty_version_is_javascript_falsey_and_does_not_enable_direct_selection() {
    let mut availability = Availability::default();
    availability.set(WorkspacePackageManager::Npm, Some(""));
    availability.set(WorkspacePackageManager::Pnpm, Some("10.0.0"));
    let mut selector = Selector::returning(WorkspacePackageManager::Pnpm);

    let result = resolve_package_manager_prompt(
        Some("npm"),
        false,
        &availability,
        &mut selector,
    )
    .expect("the prompt selection succeeds")
    .expect("transforms are enabled");

    assert_eq!(result.name, WorkspacePackageManager::Pnpm);
    assert_eq!(result.version, Some("10.0.0"));
    assert_eq!(selector.calls.len(), 1);
}

#[test]
fn installed_choices_are_stably_sorted_before_unavailable_choices() {
    let mut availability = Availability::default();
    availability.set(WorkspacePackageManager::Yarn, Some("4.9.0"));
    availability.set(WorkspacePackageManager::Nub, Some("1.2.3"));
    let mut selector = Selector::returning(WorkspacePackageManager::Yarn);

    resolve_package_manager_prompt(None, false, &availability, &mut selector)
        .expect("the prompt selection succeeds");

    let choices = &selector.calls[0];
    assert_eq!(
        choices.iter().map(|choice| choice.name).collect::<Vec<_>>(),
        [
            WorkspacePackageManager::Yarn,
            WorkspacePackageManager::Nub,
            WorkspacePackageManager::Npm,
            WorkspacePackageManager::Pnpm,
            WorkspacePackageManager::Bun,
            WorkspacePackageManager::Aube,
        ]
    );
    assert!(choices[..2].iter().all(|choice| !choice.disabled));
    assert!(choices[2..].iter().all(|choice| choice.disabled));
    assert_eq!(choices[0].version.as_deref(), Some("4.9.0"));
    assert_eq!(choices[1].version.as_deref(), Some("1.2.3"));
}

#[test]
fn selected_manager_returns_the_exact_discovered_version() {
    let availability = Availability::installed(WorkspacePackageManager::Aube, "0.8.7-canary.2");
    let mut selector = Selector::returning(WorkspacePackageManager::Aube);

    let result = resolve_package_manager_prompt(None, false, &availability, &mut selector)
        .expect("the prompt selection succeeds")
        .expect("transforms are enabled");

    assert_eq!(result.name, WorkspacePackageManager::Aube);
    assert_eq!(result.version, Some("0.8.7-canary.2"));
}

#[test]
fn selector_failure_is_propagated_without_retry() {
    let availability = Availability::installed(WorkspacePackageManager::Npm, "11.0.0");
    let mut selector = Selector::failing("cancelled");

    let error = resolve_package_manager_prompt(None, false, &availability, &mut selector)
        .expect_err("selector failure must not become a selection");

    assert_eq!(error, PackageManagerPromptError::Selection("cancelled"));
    assert_eq!(selector.calls.len(), 1);
}
