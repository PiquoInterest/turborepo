use std::path::Path;

use crate::{PackageManagerSelection, WorkspacePackageManager};

const INSTALL_ARGS: &[&str] = &["install"];
const PNPM_INSTALL_ARGS: &[&str] = &["install", "--fix-lockfile"];
const YARN_BERRY_INSTALL_ARGS: &[&str] = &["install", "--no-immutable"];
const JAVASCRIPT_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub const PACKAGE_MANAGER_VERSION_INPUT_LIMIT: usize = 256;
pub const PACKAGE_MANAGER_RANGE_INPUT_LIMIT: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManagerInstallProfile {
    pub name: &'static str,
    pub template: &'static str,
    pub command: WorkspacePackageManager,
    pub install_args: &'static [&'static str],
    pub version: &'static str,
    pub executable: &'static str,
    pub semver: &'static str,
    pub is_default: bool,
}

pub const NPM_INSTALL_PROFILES: [PackageManagerInstallProfile; 1] =
    [PackageManagerInstallProfile {
        name: "npm",
        template: "npm",
        command: WorkspacePackageManager::Npm,
        install_args: INSTALL_ARGS,
        version: "latest",
        executable: "npx",
        semver: "*",
        is_default: true,
    }];

pub const PNPM_INSTALL_PROFILES: [PackageManagerInstallProfile; 2] = [
    PackageManagerInstallProfile {
        name: "pnpm6",
        template: "pnpm",
        command: WorkspacePackageManager::Pnpm,
        install_args: INSTALL_ARGS,
        version: "latest-6",
        executable: "pnpx",
        semver: "6.x",
        is_default: false,
    },
    PackageManagerInstallProfile {
        name: "pnpm",
        template: "pnpm",
        command: WorkspacePackageManager::Pnpm,
        install_args: PNPM_INSTALL_ARGS,
        version: "latest",
        executable: "pnpm dlx",
        semver: ">=7",
        is_default: true,
    },
];

pub const YARN_INSTALL_PROFILES: [PackageManagerInstallProfile; 2] = [
    PackageManagerInstallProfile {
        name: "yarn",
        template: "yarn",
        command: WorkspacePackageManager::Yarn,
        install_args: INSTALL_ARGS,
        version: "1.x",
        executable: "npx",
        semver: "<2",
        is_default: true,
    },
    PackageManagerInstallProfile {
        name: "berry",
        template: "berry",
        command: WorkspacePackageManager::Yarn,
        install_args: YARN_BERRY_INSTALL_ARGS,
        version: "stable",
        executable: "yarn dlx",
        semver: ">=2",
        is_default: false,
    },
];

pub const BUN_INSTALL_PROFILES: [PackageManagerInstallProfile; 1] =
    [PackageManagerInstallProfile {
        name: "bun",
        template: "bun",
        command: WorkspacePackageManager::Bun,
        install_args: INSTALL_ARGS,
        version: "latest",
        executable: "bunx",
        semver: "^1.0.1",
        is_default: true,
    }];

pub const NUB_INSTALL_PROFILES: [PackageManagerInstallProfile; 1] =
    [PackageManagerInstallProfile {
        name: "nub",
        template: "nub",
        command: WorkspacePackageManager::Nub,
        install_args: INSTALL_ARGS,
        version: "latest",
        executable: "nub exec",
        semver: "*",
        is_default: true,
    }];

pub const AUBE_INSTALL_PROFILES: [PackageManagerInstallProfile; 1] =
    [PackageManagerInstallProfile {
        name: "aube",
        template: "aube",
        command: WorkspacePackageManager::Aube,
        install_args: INSTALL_ARGS,
        version: "latest",
        executable: "aube",
        semver: "*",
        is_default: true,
    }];

pub trait PackageManagerVersionMatcher {
    type Error;

    fn satisfies(&mut self, version: &str, requirement: &str) -> Result<bool, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSemverMatcherError {
    VersionTooLong,
    RangeTooLong,
    InvalidRange,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeSemverMatcher;

impl PackageManagerVersionMatcher for NodeSemverMatcher {
    type Error = NodeSemverMatcherError;

    fn satisfies(&mut self, version: &str, requirement: &str) -> Result<bool, Self::Error> {
        if version.len() > PACKAGE_MANAGER_VERSION_INPUT_LIMIT {
            return Err(NodeSemverMatcherError::VersionTooLong);
        }
        if requirement.len() > PACKAGE_MANAGER_RANGE_INPUT_LIMIT {
            return Err(NodeSemverMatcherError::RangeTooLong);
        }

        let Some(version) = ParsedVersion::parse(version) else {
            return Ok(false);
        };

        // npm's default range policy excludes prereleases unless a comparator
        // explicitly opts into the same prerelease tuple. None of the six
        // repository-owned profile ranges contains such a comparator.
        if version.has_prerelease {
            return Ok(false);
        }

        match requirement {
            "*" => Ok(true),
            "6.x" => Ok(version.major == 6),
            ">=7" => Ok(version.major >= 7),
            "<2" => Ok(version.major < 2),
            ">=2" => Ok(version.major >= 2),
            "^1.0.1" => Ok(
                version.major == 1 && (version.major, version.minor, version.patch) >= (1, 0, 1),
            ),
            _ => Err(NodeSemverMatcherError::InvalidRange),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    has_prerelease: bool,
}

impl ParsedVersion {
    fn parse(input: &str) -> Option<Self> {
        if input.is_empty()
            || !input.is_ascii()
            || input
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        {
            return None;
        }

        // npm retains compatibility with one leading `v` or `=` marker. The
        // marker is removed without trimming or otherwise normalizing input.
        let input = input
            .strip_prefix('v')
            .or_else(|| input.strip_prefix('='))
            .unwrap_or(input);

        let (without_build, build) = match input.split_once('+') {
            Some((version, build)) => {
                if !valid_identifiers(build, false) {
                    return None;
                }
                (version, Some(build))
            }
            None => (input, None),
        };

        if build.is_some_and(|build| build.contains('+')) {
            return None;
        }

        let (core, prerelease) = match without_build.split_once('-') {
            Some((core, prerelease)) => {
                if !valid_identifiers(prerelease, true) {
                    return None;
                }
                (core, Some(prerelease))
            }
            None => (without_build, None),
        };

        let mut components = core.split('.');
        let major = parse_core_component(components.next()?)?;
        let minor = parse_core_component(components.next()?)?;
        let patch = parse_core_component(components.next()?)?;
        if components.next().is_some() {
            return None;
        }

        Some(Self {
            major,
            minor,
            patch,
            has_prerelease: prerelease.is_some(),
        })
    }
}

fn parse_core_component(component: &str) -> Option<u64> {
    if component.is_empty()
        || !component.bytes().all(|byte| byte.is_ascii_digit())
        || (component.len() > 1 && component.starts_with('0'))
    {
        return None;
    }

    let value = component.parse::<u64>().ok()?;
    (value <= JAVASCRIPT_MAX_SAFE_INTEGER).then_some(value)
}

fn valid_identifiers(identifiers: &str, reject_numeric_leading_zero: bool) -> bool {
    !identifiers.is_empty()
        && identifiers.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!reject_numeric_leading_zero
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || identifier.len() == 1
                    || !identifier.starts_with('0'))
        })
}

pub fn resolve_package_manager_install_profile<M>(
    package_manager: PackageManagerSelection<'_>,
    matcher: &mut M,
) -> Result<Option<&'static PackageManagerInstallProfile>, M::Error>
where
    M: PackageManagerVersionMatcher + ?Sized,
{
    let profiles = package_manager_install_profiles(package_manager.name);
    let Some(version) = package_manager
        .version
        .filter(|version| !version.is_empty())
    else {
        return Ok(profiles.iter().find(|profile| profile.is_default));
    };

    for profile in profiles {
        if matcher.satisfies(version, profile.semver)? {
            return Ok(Some(profile));
        }
    }

    Ok(None)
}

#[must_use]
pub const fn package_manager_install_profiles(
    package_manager: WorkspacePackageManager,
) -> &'static [PackageManagerInstallProfile] {
    match package_manager {
        WorkspacePackageManager::Npm => &NPM_INSTALL_PROFILES,
        WorkspacePackageManager::Pnpm => &PNPM_INSTALL_PROFILES,
        WorkspacePackageManager::Yarn => &YARN_INSTALL_PROFILES,
        WorkspacePackageManager::Bun => &BUN_INSTALL_PROFILES,
        WorkspacePackageManager::Nub => &NUB_INSTALL_PROFILES,
        WorkspacePackageManager::Aube => &AUBE_INSTALL_PROFILES,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerInstallPlatform {
    Unix,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerInstallStdin {
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManagerInstallInvocation<'a> {
    pub program: WorkspacePackageManager,
    pub args: &'static [&'static str],
    pub cwd: &'a Path,
    pub prefer_local: bool,
    pub shell: bool,
    pub stdin: PackageManagerInstallStdin,
}

#[must_use]
pub fn build_package_manager_install_invocation<'a>(
    profile: &'static PackageManagerInstallProfile,
    project_root: &'a Path,
    _platform: PackageManagerInstallPlatform,
) -> PackageManagerInstallInvocation<'a> {
    PackageManagerInstallInvocation {
        program: profile.command,
        args: profile.install_args,
        cwd: project_root,
        prefer_local: false,
        shell: false,
        stdin: PackageManagerInstallStdin::Ignore,
    }
}
