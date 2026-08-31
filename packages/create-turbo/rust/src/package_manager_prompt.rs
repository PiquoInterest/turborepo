use crate::{PackageManagerSelection, WorkspacePackageManager};

pub const PACKAGE_MANAGER_PROMPT_ORDER: [WorkspacePackageManager; 6] = [
    WorkspacePackageManager::Npm,
    WorkspacePackageManager::Pnpm,
    WorkspacePackageManager::Yarn,
    WorkspacePackageManager::Bun,
    WorkspacePackageManager::Nub,
    WorkspacePackageManager::Aube,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageManagerPromptChoice<'a> {
    pub name: WorkspacePackageManager,
    pub version: Option<&'a str>,
    pub disabled: bool,
}

pub trait PackageManagerAvailability {
    fn version(&self, manager: WorkspacePackageManager) -> Option<&str>;
}

pub trait PackageManagerSelector {
    type Error;

    fn select(
        &mut self,
        choices: &[PackageManagerPromptChoice<'_>],
    ) -> Result<WorkspacePackageManager, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManagerPromptError<E> {
    Selection(E),
    UnavailableSelection(WorkspacePackageManager),
}

pub fn resolve_package_manager_prompt<'a, A, S>(
    manager: Option<&str>,
    skip_transforms: bool,
    availability: &'a A,
    selector: &mut S,
) -> Result<Option<PackageManagerSelection<'a>>, PackageManagerPromptError<S::Error>>
where
    A: PackageManagerAvailability,
    S: PackageManagerSelector,
{
    if skip_transforms {
        return Ok(None);
    }

    if let Some(manager) = manager.and_then(parse_manager)
        && let Some(version) = available_version(availability, manager)
    {
        return Ok(Some(PackageManagerSelection {
            name: manager,
            version: Some(version),
        }));
    }

    let mut choices: Vec<_> = PACKAGE_MANAGER_PROMPT_ORDER
        .into_iter()
        .map(|name| {
            let version = available_version(availability, name);
            PackageManagerPromptChoice {
                name,
                version,
                disabled: version.is_none(),
            }
        })
        .collect();

    // JavaScript Array.prototype.sort is stable. Sorting only by installed
    // status therefore keeps the source order within both groups.
    choices.sort_by_key(|choice| choice.disabled);

    let selected = selector
        .select(&choices)
        .map_err(PackageManagerPromptError::Selection)?;
    let Some(version) = available_version(availability, selected) else {
        return Err(PackageManagerPromptError::UnavailableSelection(selected));
    };

    Ok(Some(PackageManagerSelection {
        name: selected,
        version: Some(version),
    }))
}

fn available_version<A: PackageManagerAvailability>(
    availability: &A,
    manager: WorkspacePackageManager,
) -> Option<&str> {
    availability
        .version(manager)
        .filter(|version| !version.is_empty())
}

fn parse_manager(value: &str) -> Option<WorkspacePackageManager> {
    match value {
        "npm" => Some(WorkspacePackageManager::Npm),
        "pnpm" => Some(WorkspacePackageManager::Pnpm),
        "yarn" => Some(WorkspacePackageManager::Yarn),
        "bun" => Some(WorkspacePackageManager::Bun),
        "nub" => Some(WorkspacePackageManager::Nub),
        "aube" => Some(WorkspacePackageManager::Aube),
        _ => None,
    }
}
