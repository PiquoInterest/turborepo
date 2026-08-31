use std::path::Path;

use crate::{TransformResponse, TransformStatus};

pub const PACKAGE_MANAGER_TRANSFORM_NAME: &str = "package-manager";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspacePackageManager {
    Yarn,
    Npm,
    Pnpm,
    Bun,
    Nub,
    Aube,
}

impl WorkspacePackageManager {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Yarn => "yarn",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Nub => "nub",
            Self::Aube => "aube",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManagerSelection<'a> {
    pub name: WorkspacePackageManager,
    pub version: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManagerConversion<'a> {
    pub root: &'a Path,
    pub to: WorkspacePackageManager,
    pub skip_install: bool,
}

pub trait PackageManagerConverter {
    type Error;

    fn convert(&mut self, request: PackageManagerConversion<'_>) -> Result<(), Self::Error>;
}

pub fn transform_package_manager<C: PackageManagerConverter>(
    root: &Path,
    current: WorkspacePackageManager,
    selection: Option<PackageManagerSelection<'_>>,
    converter: &mut C,
) -> Result<TransformResponse, C::Error> {
    let Some(selection) = selection else {
        return Ok(not_applicable());
    };
    if selection.name == current {
        return Ok(not_applicable());
    }

    converter.convert(PackageManagerConversion {
        root,
        to: selection.name,
        skip_install: true,
    })?;

    Ok(TransformResponse {
        result: TransformStatus::Success,
        name: PACKAGE_MANAGER_TRANSFORM_NAME,
    })
}

fn not_applicable() -> TransformResponse {
    TransformResponse {
        result: TransformStatus::NotApplicable,
        name: PACKAGE_MANAGER_TRANSFORM_NAME,
    }
}
