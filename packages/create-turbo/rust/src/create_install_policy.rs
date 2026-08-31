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
    // TypeScript chooses the source manager whenever transforms are skipped
    // or the prompt returned no selection. Snapshot availability once so a
    // mutable provider cannot change the install decision between checks.
    let project_package_manager = match (
        input.skip_transforms,
        input.selected_package_manager,
    ) {
        (false, Some(selection)) => selection,
        _ => PackageManagerSelection {
            name: input.source_package_manager,
            version: available_version(availability, input.source_package_manager),
        },
    };

    // Preserve source ordering: manager resolution happens before these two
    // gates, even though neither branch performs an installation.
    if !input.has_package_json || input.skip_install {
        return Ok(CreateInstallOutcome::Skipped);
    }

    if input.skip_transforms && project_package_manager.version.is_none() {
        return Ok(CreateInstallOutcome::WarnUnavailable(
            UnavailablePackageManagerWarning {
                example_name: input.example_name,
                package_manager: input.source_package_manager,
            },
        ));
    }

    let Some(version) = project_package_manager
        .version
        .filter(|version| !version.is_empty())
    else {
        return Ok(CreateInstallOutcome::Skipped);
    };

    let request = CreateInstallRequest {
        package_manager: PackageManagerSelection {
            name: project_package_manager.name,
            version: Some(version),
        },
        interactive: false,
    };
    installer.install(request)?;

    Ok(CreateInstallOutcome::Installed(request))
}

fn available_version<A>(
    availability: &A,
    manager: WorkspacePackageManager,
) -> Option<&str>
where
    A: PackageManagerAvailability + ?Sized,
{
    availability
        .version(manager)
        .filter(|version| !version.is_empty())
}
