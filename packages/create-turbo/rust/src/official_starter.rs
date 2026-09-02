use std::{fmt, path::Path};

use crate::{TransformStatus, is_default_example};

pub const OFFICIAL_STARTER_TRANSFORM_NAME: &str = "official-starter";
pub const OFFICIAL_REPOSITORIES: [&str; 2] = ["turbo", "turborepo"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExampleRepository<'a> {
    pub username: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficialStarterInput<'a> {
    pub root: &'a Path,
    pub example_name: &'a str,
    pub repository: Option<ExampleRepository<'a>>,
    pub project_name: &'a str,
    pub turbo_version: Option<&'a str>,
    pub create_turbo_version: &'a str,
}

pub trait OfficialStarterPackageJson {
    fn set_name(&mut self, name: &str);
    fn turbo_dev_dependency_is_truthy(&self) -> bool;
    fn set_turbo_dev_dependency(&mut self, version: &str);
}

pub trait OfficialStarterStore {
    type Error;
    type MetaJson;
    type PackageJson: OfficialStarterPackageJson;

    fn package_json_exists(&mut self, root: &Path) -> bool;
    fn read_meta_json(&mut self, root: &Path) -> Result<Self::MetaJson, Self::Error>;
    fn remove_meta_json(&mut self, root: &Path) -> Result<(), Self::Error>;
    fn read_package_json(&mut self, root: &Path) -> Result<Option<Self::PackageJson>, Self::Error>;
    fn write_package_json(
        &mut self,
        root: &Path,
        package_json: &Self::PackageJson,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialStarterResponse<M> {
    pub result: TransformStatus,
    pub name: &'static str,
    pub meta_json: Option<M>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfficialStarterError<E> {
    ReadPackageJson(E),
    WritePackageJson(E),
}

impl<E> OfficialStarterError<E> {
    #[must_use]
    pub const fn transform_name(&self) -> &'static str {
        OFFICIAL_STARTER_TRANSFORM_NAME
    }

    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        false
    }
}

impl<E> fmt::Display for OfficialStarterError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReadPackageJson(_) => "Unable to read package.json",
            Self::WritePackageJson(_) => "Unable to write package.json",
        })
    }
}

impl<E: fmt::Debug> std::error::Error for OfficialStarterError<E> {}

#[must_use]
pub fn is_official_starter(repository: Option<ExampleRepository<'_>>) -> bool {
    match repository {
        None => true,
        Some(repository) => {
            repository.username == "vercel" && OFFICIAL_REPOSITORIES.contains(&repository.name)
        }
    }
}

pub fn transform_official_starter<S>(
    input: OfficialStarterInput<'_>,
    store: &mut S,
) -> Result<OfficialStarterResponse<S::MetaJson>, OfficialStarterError<S::Error>>
where
    S: OfficialStarterStore,
{
    if !is_official_starter(input.repository) {
        return Ok(response(TransformStatus::NotApplicable, None));
    }

    // Preserve the TypeScript source order: existsSync(package.json) runs
    // before the best-effort meta.json read/remove block.
    let has_package_json = store.package_json_exists(input.root);

    let meta_json = match store.read_meta_json(input.root) {
        Ok(meta_json) => {
            drop(store.remove_meta_json(input.root));
            Some(meta_json)
        }
        Err(_) => None,
    };

    if has_package_json {
        let package_json = store
            .read_package_json(input.root)
            .map_err(OfficialStarterError::ReadPackageJson)?;

        if let Some(mut package_json) = package_json {
            if is_default_example(input.example_name) {
                package_json.set_name(input.project_name);
            }

            if package_json.turbo_dev_dependency_is_truthy() {
                if let Some(version) = input.turbo_version.filter(|version| !version.is_empty()) {
                    package_json.set_turbo_dev_dependency(version);
                } else {
                    let version = format!("^{}", input.create_turbo_version);
                    package_json.set_turbo_dev_dependency(&version);
                }
            }

            store
                .write_package_json(input.root, &package_json)
                .map_err(OfficialStarterError::WritePackageJson)?;
        }
    }

    Ok(response(TransformStatus::Success, meta_json))
}

fn response<M>(result: TransformStatus, meta_json: Option<M>) -> OfficialStarterResponse<M> {
    OfficialStarterResponse {
        result,
        name: OFFICIAL_STARTER_TRANSFORM_NAME,
        meta_json,
    }
}
