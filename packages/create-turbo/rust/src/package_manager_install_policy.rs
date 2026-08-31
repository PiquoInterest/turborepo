use std::path::Path;

use crate::{PackageManagerSelection, WorkspacePackageManager};

const INSTALL_ARGS: &[&str] = &["install"];
const PNPM_INSTALL_ARGS: &[&str] = &["install", "--fix-lockfile"];
const YARN_BERRY_INSTALL_ARGS: &[&str] = &["install", "--no-immutable"];

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
        if requirement.is_empty() {
            return Err(NodeSemverMatcherError::InvalidRange);
        }

        // RED stub: the concrete Node-compatible parser is added in the
        // following GREEN commit. Keeping this callable makes the behavioral
        // tests compile and fail for the missing matching behavior.
        Ok(false)
    }
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
