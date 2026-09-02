use std::path::{Path, PathBuf};

use create_turbo_rs::{
    PackageManagerConversion, PackageManagerConverter, PackageManagerSelection,
    WorkspacePackageManager, transform_package_manager,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedConversion {
    root: PathBuf,
    to: WorkspacePackageManager,
    skip_install: bool,
}

#[derive(Default)]
struct FakeConverter {
    calls: Vec<OwnedConversion>,
}

impl PackageManagerConverter for FakeConverter {
    type Error = std::convert::Infallible;

    fn convert(&mut self, request: PackageManagerConversion<'_>) -> Result<(), Self::Error> {
        self.calls.push(OwnedConversion {
            root: request.root.to_path_buf(),
            to: request.to,
            skip_install: request.skip_install,
        });
        Ok(())
    }
}

#[test]
fn a_large_untrusted_version_is_borrowed_and_not_forwarded() {
    let version = "secret-".repeat(512 * 1024);
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let _ = transform_package_manager(
        root,
        WorkspacePackageManager::Npm,
        Some(PackageManagerSelection {
            name: WorkspacePackageManager::Pnpm,
            version: Some(&version),
        }),
        &mut converter,
    )
    .expect("the converter is infallible");

    assert_eq!(converter.calls.len(), 1);
    assert_eq!(converter.calls[0].root, root);
    assert_eq!(converter.calls[0].to, WorkspacePackageManager::Pnpm);
    assert!(converter.calls[0].skip_install);
}

#[test]
fn shell_metacharacters_in_the_root_are_not_interpreted_by_the_core() {
    let root = Path::new("/tmp/project-$#;!");
    let mut converter = FakeConverter::default();

    let _ = transform_package_manager(
        root,
        WorkspacePackageManager::Npm,
        Some(PackageManagerSelection {
            name: WorkspacePackageManager::Yarn,
            version: None,
        }),
        &mut converter,
    )
    .expect("the converter is infallible");

    assert_eq!(converter.calls[0].root, root);
}

#[cfg(unix)]
#[test]
fn non_utf8_roots_are_forwarded_without_lossy_string_conversion() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let root = PathBuf::from(OsString::from_vec(b"/tmp/project-\xff".to_vec()));
    let mut converter = FakeConverter::default();

    let _ = transform_package_manager(
        &root,
        WorkspacePackageManager::Npm,
        Some(PackageManagerSelection {
            name: WorkspacePackageManager::Bun,
            version: None,
        }),
        &mut converter,
    )
    .expect("the converter is infallible");

    assert_eq!(converter.calls[0].root, root);
}

#[test]
fn no_mutating_provider_call_occurs_when_the_selection_is_absent_or_unchanged() {
    let root = Path::new("/tmp/project");
    let mut converter = FakeConverter::default();

    let _ = transform_package_manager(root, WorkspacePackageManager::Npm, None, &mut converter)
        .expect("the converter is infallible");
    let _ = transform_package_manager(
        root,
        WorkspacePackageManager::Npm,
        Some(PackageManagerSelection {
            name: WorkspacePackageManager::Npm,
            version: Some("malicious\u{1b}[31m-version"),
        }),
        &mut converter,
    )
    .expect("the converter is infallible");

    assert!(converter.calls.is_empty());
}
