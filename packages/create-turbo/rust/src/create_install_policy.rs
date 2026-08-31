use crate::{PackageManagerAvailability, PackageManagerSelection, WorkspacePackageManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateInstallInput<'a> {
    pub has_package_json: bool,
    pub skip_install: bool,
    pub skip_transforms: bool,
    pub example_name: &'a str,
    pub source_package_manager: WorkspacePackageManager,
    pub selected_package_manager: Option<PackageManagerSelection<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateInstallRequest<'a> {
    pub package_manager: PackageManagerSelection<'a>,
    pub interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnavailablePackageManagerWarning<'a> {
    pub example_name: &'a str,
    pub package_manager: WorkspacePackageManager,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateInstallOutcome<'a> {
    Skipped,
    WarnUnavailable(UnavailablePackageManagerWarning<'a>),
    Installed(CreateInstallRequest<'a>),
}

pub trait CreateInstaller {
    type Error;

    fn install(&mut self, request: CreateInstallRequest<'_>) -> Result<(), Self::Error>;
}

pub fn apply_create_install_policy<'a, A, I>(
    input: CreateInstallInput<'a>,
    availability: &'a A,
    installer: &mut I,
) -> Result<CreateInstallOutcome<'a>, I::Error>
where
    A: PackageManagerAvailability + ?Sized,
    I: CreateInstaller + ?Sized,
{
    let _ = (input, availability, installer);
    Ok(CreateInstallOutcome::Skipped)
}
