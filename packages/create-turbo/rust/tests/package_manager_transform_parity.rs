use std::path::{Path, PathBuf};

use create_turbo_rs::{
    PACKAGE_MANAGER_TRANSFORM_NAME, PackageManagerConversion, PackageManagerConverter,
    PackageManagerSelection, TransformStatus, WorkspacePackageManager, transform_package_manager,
};

const ALL_MANAGERS: [WorkspacePackageManager; 6] = [
    WorkspacePackageManager::Yarn,
    WorkspacePackageManager::Npm,
    WorkspacePackageManager::Pnpm,
    WorkspacePackageManager::Bun,
    WorkspacePackageManager::Nub,
    WorkspacePackageManager::Aube,
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedConversion {
    root: PathBuf,
    to: WorkspacePackageManager,
    skip_install: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConversionError;

#[derive(Default)]
struct FakeConverter {
    calls: Vec<OwnedConversion>,
    fail: bool,
}

impl PackageManagerConverter for FakeConverter {
    type Error = ConversionError;

    fn convert(&mut self, request: PackageManagerConversion<'_>) -> Result<(), Self::Error> {
        self.calls.push(OwnedConversion {
            root: request.root.to_path_buf(),
            to: request.to,
            skip_install: request.skip_install,
        });
        if self.fail {
            Err(ConversionError)
        } else {
            Ok(())
        }
    }
}

fn selection(name: WorkspacePackageManager, version: Option<&str>) -> PackageManagerSelection<'_> {
    PackageManagerSelection { name, version }
}

#[test]
fn exported_manager_names_match_the_typescript_union() {
    assert_eq!(
        ALL_MANAGERS.map(WorkspacePackageManager::as_str),
        ["yarn", "npm", "pnpm", "bun", "nub", "aube"]
    );
}

#[test]
fn no_prompt_selection_is_not_applicable_without_conversion() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let response =
        transform_package_manager(root, WorkspacePackageManager::Pnpm, None, &mut converter)
            .expect("the fake converter cannot fail without a call");

    assert_eq!(response.result, TransformStatus::NotApplicable);
    assert_eq!(response.name, PACKAGE_MANAGER_TRANSFORM_NAME);
    assert!(converter.calls.is_empty());
}

#[test]
fn unchanged_manager_is_not_applicable_without_conversion() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let response = transform_package_manager(
        root,
        WorkspacePackageManager::Yarn,
        Some(selection(WorkspacePackageManager::Yarn, Some("4.9.2"))),
        &mut converter,
    )
    .expect("the fake converter cannot fail without a call");

    assert_eq!(response.result, TransformStatus::NotApplicable);
    assert_eq!(response.name, PACKAGE_MANAGER_TRANSFORM_NAME);
    assert!(converter.calls.is_empty());
}

#[test]
fn changed_manager_requests_conversion_with_skip_install() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let response = transform_package_manager(
        root,
        WorkspacePackageManager::Pnpm,
        Some(selection(WorkspacePackageManager::Bun, Some("1.2.3"))),
        &mut converter,
    )
    .expect("successful fake conversion must produce a response");

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(response.name, PACKAGE_MANAGER_TRANSFORM_NAME);
    assert_eq!(
        converter.calls,
        [OwnedConversion {
            root: root.to_path_buf(),
            to: WorkspacePackageManager::Bun,
            skip_install: true,
        }]
    );
}

#[test]
fn every_typescript_package_manager_can_be_a_conversion_target() {
    for target in ALL_MANAGERS {
        let current = if target == WorkspacePackageManager::Yarn {
            WorkspacePackageManager::Npm
        } else {
            WorkspacePackageManager::Yarn
        };
        let root = Path::new("/tmp/project");
        let mut converter = FakeConverter::default();

        let response =
            transform_package_manager(root, current, Some(selection(target, None)), &mut converter)
                .expect("successful fake conversion must produce a response");

        assert_eq!(response.result, TransformStatus::Success);
        assert_eq!(converter.calls.len(), 1);
        assert_eq!(converter.calls[0].to, target);
        assert!(converter.calls[0].skip_install);
    }
}

#[test]
fn prompt_version_is_not_forwarded_to_the_converter() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let _ = transform_package_manager(
        root,
        WorkspacePackageManager::Npm,
        Some(selection(WorkspacePackageManager::Pnpm, Some("9.15.4"))),
        &mut converter,
    )
    .expect("successful fake conversion must produce a response");

    assert_eq!(
        converter.calls,
        [OwnedConversion {
            root: root.to_path_buf(),
            to: WorkspacePackageManager::Pnpm,
            skip_install: true,
        }]
    );
}

#[test]
fn converter_failure_is_propagated_without_a_success_response() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter {
        calls: Vec::new(),
        fail: true,
    };

    let result = transform_package_manager(
        root,
        WorkspacePackageManager::Npm,
        Some(selection(WorkspacePackageManager::Yarn, None)),
        &mut converter,
    );

    assert_eq!(result, Err(ConversionError));
    assert_eq!(converter.calls.len(), 1);
}
