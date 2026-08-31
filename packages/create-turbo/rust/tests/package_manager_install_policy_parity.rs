use std::path::Path;

use create_turbo_rs::{
    AUBE_INSTALL_PROFILES, BUN_INSTALL_PROFILES, NPM_INSTALL_PROFILES, NUB_INSTALL_PROFILES,
    PNPM_INSTALL_PROFILES, PackageManagerInstallPlatform, PackageManagerInstallStdin,
    PackageManagerSelection, PackageManagerVersionMatcher, WorkspacePackageManager,
    YARN_INSTALL_PROFILES, build_package_manager_install_invocation,
    package_manager_install_profiles, resolve_package_manager_install_profile,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatcherError {
    Failed,
}

#[derive(Debug, Default)]
struct RecordingMatcher {
    accepted_requirement: Option<&'static str>,
    fail_at_call: Option<usize>,
    calls: Vec<(String, String)>,
}

impl PackageManagerVersionMatcher for RecordingMatcher {
    type Error = MatcherError;

    fn satisfies(&mut self, version: &str, requirement: &str) -> Result<bool, Self::Error> {
        let call_index = self.calls.len();
        self.calls
            .push((version.to_owned(), requirement.to_owned()));
        if self.fail_at_call == Some(call_index) {
            return Err(MatcherError::Failed);
        }
        Ok(self.accepted_requirement == Some(requirement))
    }
}

struct ExplodingMatcher;

impl PackageManagerVersionMatcher for ExplodingMatcher {
    type Error = MatcherError;

    fn satisfies(&mut self, _version: &str, _requirement: &str) -> Result<bool, Self::Error> {
        panic!("default profile selection must not invoke semver matching");
    }
}

#[test]
fn profile_tables_match_the_typescript_source_order() {
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Npm),
        &NPM_INSTALL_PROFILES
    );
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Pnpm),
        &PNPM_INSTALL_PROFILES
    );
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Yarn),
        &YARN_INSTALL_PROFILES
    );
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Bun),
        &BUN_INSTALL_PROFILES
    );
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Nub),
        &NUB_INSTALL_PROFILES
    );
    assert_eq!(
        package_manager_install_profiles(WorkspacePackageManager::Aube),
        &AUBE_INSTALL_PROFILES
    );

    assert_eq!(PNPM_INSTALL_PROFILES[0].name, "pnpm6");
    assert_eq!(PNPM_INSTALL_PROFILES[0].semver, "6.x");
    assert_eq!(PNPM_INSTALL_PROFILES[1].name, "pnpm");
    assert_eq!(PNPM_INSTALL_PROFILES[1].semver, ">=7");
    assert_eq!(YARN_INSTALL_PROFILES[0].name, "yarn");
    assert_eq!(YARN_INSTALL_PROFILES[0].semver, "<2");
    assert_eq!(YARN_INSTALL_PROFILES[1].name, "berry");
    assert_eq!(YARN_INSTALL_PROFILES[1].semver, ">=2");
}

#[test]
fn missing_versions_select_the_exact_default_profiles_without_matching() {
    let cases = [
        (WorkspacePackageManager::Npm, "npm"),
        (WorkspacePackageManager::Pnpm, "pnpm"),
        (WorkspacePackageManager::Yarn, "yarn"),
        (WorkspacePackageManager::Bun, "bun"),
        (WorkspacePackageManager::Nub, "nub"),
        (WorkspacePackageManager::Aube, "aube"),
    ];

    for (manager, expected_name) in cases {
        let mut matcher = ExplodingMatcher;
        let result = resolve_package_manager_install_profile(
            PackageManagerSelection {
                name: manager,
                version: None,
            },
            &mut matcher,
        );
        let Ok(Some(profile)) = result else {
            panic!("every source manager must have one default install profile");
        };

        assert_eq!(profile.name, expected_name);
        assert!(profile.is_default);
    }
}

#[test]
fn empty_versions_are_javascript_falsy_and_use_the_default() {
    let mut matcher = ExplodingMatcher;
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Pnpm,
            version: Some(""),
        },
        &mut matcher,
    );
    let Ok(Some(profile)) = result else {
        panic!("an empty version must use the default pnpm profile");
    };

    assert_eq!(profile.name, "pnpm");
}

#[test]
fn pnpm_version_selection_preserves_first_match_order() {
    let mut matcher = RecordingMatcher {
        accepted_requirement: Some("6.x"),
        ..RecordingMatcher::default()
    };
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Pnpm,
            version: Some("6.35.1"),
        },
        &mut matcher,
    );
    let Ok(Some(profile)) = result else {
        panic!("pnpm 6 must select the first source profile");
    };

    assert_eq!(profile.name, "pnpm6");
    assert_eq!(matcher.calls, [("6.35.1".to_owned(), "6.x".to_owned())]);
}

#[test]
fn yarn_berry_selection_checks_classic_before_modern() {
    let mut matcher = RecordingMatcher {
        accepted_requirement: Some(">=2"),
        ..RecordingMatcher::default()
    };
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Yarn,
            version: Some("4.1.0"),
        },
        &mut matcher,
    );
    let Ok(Some(profile)) = result else {
        panic!("modern Yarn must select the berry profile");
    };

    assert_eq!(profile.name, "berry");
    assert_eq!(
        matcher.calls,
        [
            ("4.1.0".to_owned(), "<2".to_owned()),
            ("4.1.0".to_owned(), ">=2".to_owned()),
        ]
    );
}

#[test]
fn unsupported_versions_return_none_after_the_bounded_profile_scan() {
    let mut matcher = RecordingMatcher::default();
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Yarn,
            version: Some("unsupported"),
        },
        &mut matcher,
    );
    let Ok(profile) = result else {
        panic!("an ordinary unsupported version must not be a matcher error");
    };

    assert_eq!(profile, None);
    assert_eq!(matcher.calls.len(), 2);
}

#[test]
fn matcher_failures_are_propagated_without_retry_or_fallback() {
    let mut matcher = RecordingMatcher {
        fail_at_call: Some(0),
        ..RecordingMatcher::default()
    };
    let result = resolve_package_manager_install_profile(
        PackageManagerSelection {
            name: WorkspacePackageManager::Yarn,
            version: Some("4.1.0"),
        },
        &mut matcher,
    );

    assert_eq!(result, Err(MatcherError::Failed));
    assert_eq!(matcher.calls.len(), 1);
}

#[test]
fn invocation_preserves_program_arguments_root_and_ignored_stdin() {
    let root = Path::new("/tmp/example");
    let invocation = build_package_manager_install_invocation(
        &PNPM_INSTALL_PROFILES[1],
        root,
        PackageManagerInstallPlatform::Unix,
    );

    assert_eq!(invocation.program, WorkspacePackageManager::Pnpm);
    assert_eq!(invocation.args, &["install", "--fix-lockfile"]);
    assert_eq!(invocation.cwd, root);
    assert_eq!(invocation.stdin, PackageManagerInstallStdin::Ignore);
}
