use std::cell::Cell;

use create_turbo_rs::{
    CreateInstallInput, CreateInstallOutcome, CreateInstallRequest, CreateInstaller,
    PackageManagerAvailability, PackageManagerSelection, UnavailablePackageManagerWarning,
    WorkspacePackageManager, apply_create_install_policy,
};

struct ExplodingAvailability;

impl PackageManagerAvailability for ExplodingAvailability {
    fn version(&self, _manager: WorkspacePackageManager) -> Option<&str> {
        panic!("selected package-manager input must not query availability");
    }
}

#[derive(Debug)]
struct StaticAvailability<'a> {
    version: Option<&'a str>,
    calls: Cell<usize>,
    last_manager: Cell<Option<WorkspacePackageManager>>,
}

impl<'a> StaticAvailability<'a> {
    fn new(version: Option<&'a str>) -> Self {
        Self {
            version,
            calls: Cell::new(0),
            last_manager: Cell::new(None),
        }
    }
}

impl PackageManagerAvailability for StaticAvailability<'_> {
    fn version(&self, manager: WorkspacePackageManager) -> Option<&str> {
        self.calls.set(self.calls.get() + 1);
        self.last_manager.set(Some(manager));
        self.version
    }
}

#[derive(Debug)]
struct FlippingAvailability {
    calls: Cell<usize>,
}

impl PackageManagerAvailability for FlippingAvailability {
    fn version(&self, _manager: WorkspacePackageManager) -> Option<&str> {
        let calls = self.calls.get();
        self.calls.set(calls + 1);
        if calls == 0 { Some("9.15.4") } else { None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallerError {
    Failed,
}

#[derive(Debug, Default)]
struct ObservingInstaller {
    calls: usize,
    last_name: Option<WorkspacePackageManager>,
    last_version_pointer: Option<*const u8>,
    last_version_length: Option<usize>,
    fail: bool,
}

impl CreateInstaller for ObservingInstaller {
    type Error = InstallerError;

    fn install(&mut self, request: CreateInstallRequest<'_>) -> Result<(), Self::Error> {
        self.calls += 1;
        self.last_name = Some(request.package_manager.name);
        self.last_version_pointer = request.package_manager.version.map(str::as_ptr);
        self.last_version_length = request.package_manager.version.map(str::len);
        if self.fail {
            Err(InstallerError::Failed)
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
fn trusted_selected_manager_cannot_reach_the_availability_provider() {
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Pnpm,
        version: Some("9.15.4"),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("the selected manager must install without availability discovery");
    };

    assert!(matches!(outcome, CreateInstallOutcome::Installed(_)));
    assert_eq!(installer.calls, 1);
    assert_eq!(installer.last_name, Some(WorkspacePackageManager::Pnpm));
}

#[test]
fn skip_transforms_snapshots_availability_once() {
    let availability = FlippingAvailability {
        calls: Cell::new(0),
    };
    let mut installer = ObservingInstaller::default();
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
        panic!("the first availability snapshot must determine the outcome");
    };

    assert!(matches!(outcome, CreateInstallOutcome::Installed(_)));
    assert_eq!(availability.calls.get(), 1);
    assert_eq!(installer.calls, 1);
}

#[test]
fn unavailable_source_manager_never_reaches_the_installer() {
    let availability = StaticAvailability::new(None);
    let mut installer = ObservingInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Aube, None);
    install_input.skip_transforms = true;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("unavailability must be represented without invoking install");
    };

    assert!(matches!(outcome, CreateInstallOutcome::WarnUnavailable(_)));
    assert_eq!(installer.calls, 0);
    assert_eq!(
        availability.last_manager.get(),
        Some(WorkspacePackageManager::Aube)
    );
}

#[test]
fn a_large_warning_name_is_borrowed_without_a_policy_layer_copy() {
    let example_name = "x".repeat(4 * 1024 * 1024);
    let availability = StaticAvailability::new(None);
    let mut installer = ObservingInstaller::default();
    let mut install_input = input(WorkspacePackageManager::Npm, None);
    install_input.skip_transforms = true;
    install_input.example_name = &example_name;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(CreateInstallOutcome::WarnUnavailable(warning)) = result else {
        panic!("the unavailable source manager must produce warning data");
    };

    assert_eq!(warning.example_name.len(), example_name.len());
    assert_eq!(warning.example_name.as_ptr(), example_name.as_ptr());
    assert_eq!(
        warning,
        UnavailablePackageManagerWarning {
            example_name: &example_name,
            package_manager: WorkspacePackageManager::Npm,
        }
    );
}

#[test]
fn a_large_selected_version_is_borrowed_through_the_install_request() {
    let version = "9".repeat(4 * 1024 * 1024);
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Bun,
        version: Some(&version),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(CreateInstallOutcome::Installed(request)) = result else {
        panic!("the selected manager must install");
    };
    let Some(installed_version) = request.package_manager.version else {
        panic!("the installed request must retain its version");
    };

    assert_eq!(installed_version.as_ptr(), version.as_ptr());
    assert_eq!(installed_version.len(), version.len());
    assert_eq!(installer.last_version_pointer, Some(version.as_ptr()));
    assert_eq!(installer.last_version_length, Some(version.len()));
}

#[test]
fn installer_failure_is_not_retried_or_downgraded() {
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller {
        fail: true,
        ..ObservingInstaller::default()
    };
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Yarn,
        version: Some("1.22.22"),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );

    assert_eq!(result, Err(InstallerError::Failed));
    assert_eq!(installer.calls, 1);
}

#[test]
fn empty_selected_version_cannot_trigger_installation() {
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Nub,
        version: Some(""),
    };

    let result = apply_create_install_policy(
        input(WorkspacePackageManager::Npm, Some(selected)),
        &availability,
        &mut installer,
    );
    let Ok(outcome) = result else {
        panic!("an empty version must be a successful no-install branch");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(installer.calls, 0);
}

#[test]
fn skip_install_cannot_invoke_the_installer() {
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Npm,
        version: Some("10.9.0"),
    };
    let mut install_input = input(WorkspacePackageManager::Pnpm, Some(selected));
    install_input.skip_install = true;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("skip-install must not fail");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(installer.calls, 0);
}

#[test]
fn missing_package_json_cannot_invoke_the_installer() {
    let availability = ExplodingAvailability;
    let mut installer = ObservingInstaller::default();
    let selected = PackageManagerSelection {
        name: WorkspacePackageManager::Npm,
        version: Some("10.9.0"),
    };
    let mut install_input = input(WorkspacePackageManager::Pnpm, Some(selected));
    install_input.has_package_json = false;

    let result = apply_create_install_policy(install_input, &availability, &mut installer);
    let Ok(outcome) = result else {
        panic!("a missing package.json must not fail");
    };

    assert_eq!(outcome, CreateInstallOutcome::Skipped);
    assert_eq!(installer.calls, 0);
}
