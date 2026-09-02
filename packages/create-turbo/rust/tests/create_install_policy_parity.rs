use std::cell::RefCell;

use create_turbo_rs::{
    CreateInstallInput, CreateInstallOutcome, CreateInstallRequest, CreateInstaller,
    PackageManagerAvailability, PackageManagerSelection, UnavailablePackageManagerWarning,
    WorkspacePackageManager, apply_create_install_policy,
};

#[derive(Debug, Default)]
struct FakeAvailability<'a> {
    npm: Option<&'a str>,
    pnpm: Option<&'a str>,
    yarn: Option<&'a str>,
    bun: Option<&'a str>,
    nub: Option<&'a str>,
    aube: Option<&'a str>,
    calls: RefCell<Vec<WorkspacePackageManager>>,
}

impl<'a> FakeAvailability<'a> {
    fn with_version(manager: WorkspacePackageManager, version: Option<&'a str>) -> Self {
        let mut availability = Self::default();
        match manager {
            WorkspacePackageManager::Npm => availability.npm = version,
            WorkspacePackageManager::Pnpm => availability.pnpm = version,
            WorkspacePackageManager::Yarn => availability.yarn = version,
            WorkspacePackageManager::Bun => availability.bun = version,
            WorkspacePackageManager::Nub => availability.nub = version,
            WorkspacePackageManager::Aube => availability.aube = version,
        }
        availability
    }
}

impl PackageManagerAvailability for FakeAvailability<'_> {
    fn version(&self, manager: WorkspacePackageManager) -> Option<&str> {
        self.calls.borrow_mut().push(manager);
        match manager {
            WorkspacePackageManager::Npm => self.npm,
            WorkspacePackageManager::Pnpm => self.pnpm,
            WorkspacePackageManager::Yarn => self.yarn,
            WorkspacePackageManager::Bun => self.bun,
            WorkspacePackageManager::Nub => self.nub,
            WorkspacePackageManager::Aube => self.aube,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedInstall {
    name: WorkspacePackageManager,
    version: Option<String>,
    interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FakeInstallError {
    Failed,
}

#[derive(Debug, Default)]
struct FakeInstaller {
    calls: Vec<RecordedInstall>,
    fail: bool,
}

impl CreateInstaller for FakeInstaller {
    type Error = FakeInstallError;

    fn install(&mut self, request: CreateInstallRequest<'_>) -> Result<(), Self::Error> {
        self.calls.push(RecordedInstall {
            name: request.package_manager.name,
            version: request.package_manager.version.map(str::to_owned),
            interactive: request.interactive,
        });
        if self.fail {
            Err(FakeInstallError::Failed)
        } else {
            Ok(())
        }
    }
}

fn input<'a>(
    source_package_manager: WorkspacePackageManager,
    selected_package_manager: Option<PackageManagerSelection<'a>>,
) -> CreateInstallInput<'a> {
    CreateInstallInput {
        has_package_json: true,
        skip_install: false,
        skip_transforms: false,
        example_name: "basic",
        source_package_manager,
        selected_package_manager,
    }
}

#[test]
fn selected_manager_is_installed_non_interactively() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Npm,
        version: Some("10.9.0"),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Pnpm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("the selected package-manager install must succeed");
    };

    assert_eq!(
        outcome,
        CreateInstallOutcome::Installed(CreateInstallRequest {
            package_manager: selected,
            interactive: false,
        })
    );
    assert!(availability.calls.borrow().is_empty());
    assert_eq!(
        installer.calls,
        [RecordedInstall {
            name: WorkspacePackageManager::Npm,
            version: Some("10.9.0".to_owned()),
            interactive: false,
        }]
    );
}

#[test]
fn skip_transforms_uses_the_source_manager_and_ignores_the_selection() {
    let availability =
        FakeAvailability::with_version(WorkspacePackageManager::Pnpm, Some("9.15.4"));
    let mut installer = FakeInstaller::default();
    let mut install_input = input(
        WorkspacePackageManager::Pnpm,
        Some(PackageManagerSelection {
            name: WorkspacePackageManager::Npm,
            version: Some("10.9.0"),
        }),
    );
    install_input.skip_transforms = true;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("the available source package manager must install");
    };

    assert_eq!(
        outcome,
        CreateInstallOutcome::Installed(CreateInstallRequest {
            package_manager: PackageManagerSelection {
                name: WorkspacePackageManager::Pnpm,
                version: Some("9.15.4"),
            },
            interactive: false,
        })
    );
    assert_eq!(
        &*availability.calls.borrow(),
        &[WorkspacePackageManager::Pnpm]
    );
}

#[test]
fn missing_selection_falls_back_to_the_source_manager() {
    let availability =
        FakeAvailability::with_version(WorkspacePackageManager::Yarn, Some("1.22.22"));
    let mut installer = FakeInstaller::default();

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Yarn, None),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("the available source package manager must install");
    };

    assert_eq!(
        outcome,
        CreateInstallOutcome::Installed(CreateInstallRequest {
            package_manager: PackageManagerSelection {
                name: WorkspacePackageManager::Yarn,
                version: Some("1.22.22"),
            },
            interactive: false,
        })
    );
    assert_eq!(
        &*availability.calls.borrow(),
        &[WorkspacePackageManager::Yarn]
    );
}

#[test]
fn missing_package_json_skips_after_resolving_the_source_manager() {
    let availability = FakeAvailability::with_version(WorkspacePackageManager::Bun, Some("1.2.3"));
    let mut installer = FakeInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Bun, None);
    install_input.has_package_json = false;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("a missing package.json must not fail");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(
        &*availability.calls.borrow(),
        &[WorkspacePackageManager::Bun]
    );
    assert!(installer.calls.is_empty());
}

#[test]
fn skip_install_skips_after_resolving_the_source_manager() {
    let availability = FakeAvailability::with_version(WorkspacePackageManager::Nub, Some("0.1.0"));
    let mut installer = FakeInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Nub, None);
    install_input.skip_install = true;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("skip-install must not fail");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(
        &*availability.calls.borrow(),
        &[WorkspacePackageManager::Nub]
    );
    assert!(installer.calls.is_empty());
}

#[test]
fn unavailable_source_manager_warns_when_transforms_are_skipped() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Aube, None);
    install_input.skip_transforms = true;
    install_input.example_name = "with-aube";

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("an unavailable source manager produces a warning outcome");
    };

    assert_eq!(
        outcome,
        CreateInstallOutcome::WarnUnavailable(UnavailablePackageManagerWarning {
            example_name: "with-aube",
            package_manager: WorkspacePackageManager::Aube,
        })
    );
    assert!(installer.calls.is_empty());
}

#[test]
fn empty_source_version_is_javascript_falsy_and_warns() {
    let availability = FakeAvailability::with_version(WorkspacePackageManager::Npm, Some(""));
    let mut installer = FakeInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Npm, None);
    install_input.skip_transforms = true;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("an empty source-manager version produces a warning outcome");
    };

    assert!(matches!(
        outcome,
        CreateInstallOutcome::WarnUnavailable(UnavailablePackageManagerWarning {
            package_manager: WorkspacePackageManager::Npm,
            ..
        })
    ));
    assert!(installer.calls.is_empty());
}

#[test]
fn selected_manager_without_a_version_silently_skips_installation() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Pnpm,
        version: None,
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("an absent selected version is a successful no-install branch");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert!(availability.calls.borrow().is_empty());
    assert!(installer.calls.is_empty());
}

#[test]
fn selected_manager_with_an_empty_version_silently_skips_installation() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Pnpm,
        version: Some(""),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("an empty selected version is a successful no-install branch");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert!(availability.calls.borrow().is_empty());
    assert!(installer.calls.is_empty());
}

#[test]
fn unavailable_fallback_without_skip_transforms_silently_skips() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller::default();

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, None),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("an unavailable implicit fallback is a successful no-install branch");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(
        &*availability.calls.borrow(),
        &[WorkspacePackageManager::Npm]
    );
    assert!(installer.calls.is_empty());
}

#[test]
fn installer_failure_is_propagated_after_one_attempt() {
    let availability = FakeAvailability::default();
    let mut installer = FakeInstaller {
        calls: Vec::new(),
        fail: true,
    };
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Bun,
        version: Some("1.2.3"),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );

    assert_eq!(result, Err(FakeInstallError::Failed));
    assert_eq!(installer.calls.len(), 1);
}

#[test]
fn every_workspace_manager_variant_can_be_installed() {
    let managers = [
        WorkspacePackageManager::Yarn,
        WorkspacePackageManager::Npm,
        WorkspacePackageManager::Pnpm,
        WorkspacePackageManager::Bun,
        WorkspacePackageManager::Nub,
        WorkspacePackageManager::Aube,
    ];

    for manager in managers {
        let availability = FakeAvailability::default();
        let mut installer = FakeInstaller::default();
        let selected = PackageManagerSelection {
            name: manager,
            version: Some("1.0.0"),
        };

        let result = apply_create_install_policy(
            input(WorkspacePackageManager::Npm, Some(selected)),
            &availability,
            &mut installer,
        );
        let Ok(outcome) = result else {
            panic!("every closed package-manager variant must be installable");
        };

        assert_eq!(
            outcome,
            CreateInstallOutcome::Installed(CreateInstallRequest {
                package_manager: selected,
                interactive: false,
            })
        );
        assert_eq!(installer.calls.len(), 1);
    }
}
