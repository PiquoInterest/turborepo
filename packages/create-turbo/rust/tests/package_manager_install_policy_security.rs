use std::path::Path;

use create_turbo_rs::{
    NodeSemverMatcher, NodeSemverMatcherError, PACKAGE_MANAGER_RANGE_INPUT_LIMIT,
    PACKAGE_MANAGER_VERSION_INPUT_LIMIT, PackageManagerInstallPlatform, PackageManagerSelection,
    PackageManagerVersionMatcher, WorkspacePackageManager,
    build_package_manager_install_invocation, package_manager_install_profiles,
    resolve_package_manager_install_profile,
};

#[derive(Debug, Default)]
struct BorrowObservingMatcher {
    version_pointer: Option<*const u8>,
    version_length: Option<usize>,
    calls: usize,
}

impl PackageManagerVersionMatcher for BorrowObservingMatcher {
    type Error = ();

    fn satisfies(&mut self, version: &str, _requirement: &str) -> Result<bool, Self::Error> {
        self.calls += 1;
        self.version_pointer = Some(version.as_ptr());
        self.version_length = Some(version.len());
        Ok(true)
    }
}

#[test]
fn install_invocations_never_request_a_shell_on_any_platform() {
    for manager in all_managers() {
        for profile in package_manager_install_profiles(manager) {
            for platform in [
                PackageManagerInstallPlatform::Unix,
                PackageManagerInstallPlatform::Windows,
            ] {
                let invocation = build_package_manager_install_invocation(
                    profile,
                    Path::new("/tmp/project"),
                    platform,
                );

                assert!(
                    !invocation.shell,
                    "install profile {} requested shell execution on {platform:?}",
                    profile.name
                );
            }
        }
    }
}

#[test]
fn install_invocations_never_prefer_project_local_executables() {
    for manager in all_managers() {
        for profile in package_manager_install_profiles(manager) {
            for platform in [
                PackageManagerInstallPlatform::Unix,
                PackageManagerInstallPlatform::Windows,
            ] {
                let invocation = build_package_manager_install_invocation(
                    profile,
                    Path::new("/tmp/project"),
                    platform,
                );

                assert!(
                    !invocation.prefer_local,
                    "install profile {} allowed project-local executable substitution",
                    profile.name
                );
            }
        }
    }
}

#[test]
fn free_form_versions_remain_borrowed_data_and_cannot_become_commands() {
    let version = "9".repeat(4 * 1024 * 1024);
    let mut matcher = BorrowObservingMatcher::default();
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Npm,
            version: Some(&version),
        },
        &mut matcher,
    );
    let Ok(Some(profile)) = result else {
        panic!("the matcher-selected npm profile must resolve");
    };
    let invocation = build_package_manager_install_invocation(
        profile,
        Path::new("/tmp/project"),
        PackageManagerInstallPlatform::Unix,
    );

    assert_eq!(matcher.version_pointer, Some(version.as_ptr()));
    assert_eq!(matcher.version_length, Some(version.len()));
    assert_eq!(matcher.calls, 1);
    assert_eq!(invocation.program, WorkspacePackageManager::Npm);
    assert_eq!(invocation.args, &["install"]);
}

#[test]
fn programs_and_arguments_come_only_from_closed_static_profiles() {
    for manager in all_managers() {
        for profile in package_manager_install_profiles(manager) {
            let invocation = build_package_manager_install_invocation(
                profile,
                Path::new("/tmp/project"),
                PackageManagerInstallPlatform::Unix,
            );
            let program = invocation.program.as_str();

            assert!(!program.is_empty());
            assert!(!program.chars().any(char::is_whitespace));
            assert!(!contains_shell_control(program));
            for argument in invocation.args {
                assert!(!argument.is_empty());
                assert!(!argument.chars().any(char::is_control));
                assert!(!contains_shell_control(argument));
            }
        }
    }
}

#[test]
fn concrete_matcher_rejects_oversized_versions_before_parsing() {
    let version = "1".repeat(PACKAGE_MANAGER_VERSION_INPUT_LIMIT + 1);
    let mut matcher = NodeSemverMatcher;

    assert_eq!(
        matcher.satisfies(&version, "*"),
        Err(NodeSemverMatcherError::VersionTooLong)
    );
}

#[test]
fn concrete_matcher_rejects_oversized_ranges_before_parsing() {
    let requirement = "1".repeat(PACKAGE_MANAGER_RANGE_INPUT_LIMIT + 1);
    let mut matcher = NodeSemverMatcher;

    assert_eq!(
        matcher.satisfies("1.2.3", &requirement),
        Err(NodeSemverMatcherError::RangeTooLong)
    );
}

#[test]
fn concrete_matcher_does_not_normalize_hostile_or_ambiguous_version_text() {
    let inputs = [
        " 1.2.3",
        "1.2.3 ",
        "1.2.3\n",
        "１.２.３",
        "1.2.3\u{202e}",
        "1.2.3\u{0}",
    ];

    for input in inputs {
        let mut matcher = NodeSemverMatcher;
        assert_eq!(matcher.satisfies(input, "*"), Ok(false));
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_project_roots_are_forwarded_without_lossy_conversion() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    let root = PathBuf::from(OsString::from_vec(b"/tmp/project-\xff".to_vec()));
    let profile = &package_manager_install_profiles(WorkspacePackageManager::Bun)[0];
    let invocation = build_package_manager_install_invocation(
        profile,
        &root,
        PackageManagerInstallPlatform::Unix,
    );

    assert_eq!(invocation.cwd, root.as_path());
}

fn all_managers() -> [WorkspacePackageManager; 6] {
    [
        WorkspacePackageManager::Npm,
        WorkspacePackageManager::Pnpm,
        WorkspacePackageManager::Yarn,
        WorkspacePackageManager::Bun,
        WorkspacePackageManager::Nub,
        WorkspacePackageManager::Aube,
    ]
}

fn contains_shell_control(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, ';' | '&' | '|' | '<' | '>' | '`' | '$'))
}
