use std::path::{Path, PathBuf};

use create_turbo_rs::{
    ExampleRepository, OfficialStarterError, OfficialStarterInput, OfficialStarterPackageJson,
    OfficialStarterStore, TransformStatus, is_official_starter, transform_official_starter,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecurityPackage {
    name: String,
    turbo: Option<String>,
    turbo_truthy: bool,
}

impl OfficialStarterPackageJson for SecurityPackage {
    fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    fn turbo_dev_dependency_is_truthy(&self) -> bool {
        self.turbo_truthy
    }

    fn set_turbo_dev_dependency(&mut self, version: &str) {
        self.turbo = Some(version.to_owned());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecurityError {
    Meta,
    Read,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SecurityCall {
    PackageExists,
    ReadMeta,
    RemoveMeta,
    ReadPackage,
    WritePackage(PathBuf, SecurityPackage),
}

struct SecurityStore {
    calls: Vec<SecurityCall>,
    meta_result: Result<String, SecurityError>,
    remove_result: Result<(), SecurityError>,
    package_result: Result<Option<SecurityPackage>, SecurityError>,
    write_result: Result<(), SecurityError>,
}

impl Default for SecurityStore {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            meta_result: Err(SecurityError::Meta),
            remove_result: Ok(()),
            package_result: Ok(Some(SecurityPackage {
                name: "starter".to_owned(),
                turbo: None,
                turbo_truthy: false,
            })),
            write_result: Ok(()),
        }
    }
}

impl OfficialStarterStore for SecurityStore {
    type Error = SecurityError;
    type MetaJson = String;
    type PackageJson = SecurityPackage;

    fn package_json_exists(&mut self, _root: &Path) -> bool {
        self.calls.push(SecurityCall::PackageExists);
        true
    }

    fn read_meta_json(&mut self, _root: &Path) -> Result<Self::MetaJson, Self::Error> {
        self.calls.push(SecurityCall::ReadMeta);
        self.meta_result.clone()
    }

    fn remove_meta_json(&mut self, _root: &Path) -> Result<(), Self::Error> {
        self.calls.push(SecurityCall::RemoveMeta);
        self.remove_result
    }

    fn read_package_json(
        &mut self,
        _root: &Path,
    ) -> Result<Option<Self::PackageJson>, Self::Error> {
        self.calls.push(SecurityCall::ReadPackage);
        self.package_result.clone()
    }

    fn write_package_json(
        &mut self,
        root: &Path,
        package_json: &Self::PackageJson,
    ) -> Result<(), Self::Error> {
        self.calls.push(SecurityCall::WritePackage(
            root.to_path_buf(),
            package_json.clone(),
        ));
        self.write_result
    }
}

struct ExplodingStore;

impl OfficialStarterStore for ExplodingStore {
    type Error = SecurityError;
    type MetaJson = String;
    type PackageJson = SecurityPackage;

    fn package_json_exists(&mut self, _root: &Path) -> bool {
        panic!("a non-official repository must not access the store")
    }

    fn read_meta_json(&mut self, _root: &Path) -> Result<Self::MetaJson, Self::Error> {
        panic!("a non-official repository must not access the store")
    }

    fn remove_meta_json(&mut self, _root: &Path) -> Result<(), Self::Error> {
        panic!("a non-official repository must not access the store")
    }

    fn read_package_json(
        &mut self,
        _root: &Path,
    ) -> Result<Option<Self::PackageJson>, Self::Error> {
        panic!("a non-official repository must not access the store")
    }

    fn write_package_json(
        &mut self,
        _root: &Path,
        _package_json: &Self::PackageJson,
    ) -> Result<(), Self::Error> {
        panic!("a non-official repository must not access the store")
    }
}

fn repository<'a>(username: &'a str, name: &'a str) -> ExampleRepository<'a> {
    ExampleRepository { username, name }
}

fn input<'a>(
    root: &'a Path,
    example_name: &'a str,
    repository: Option<ExampleRepository<'a>>,
    project_name: &'a str,
    turbo_version: Option<&'a str>,
    create_turbo_version: &'a str,
) -> OfficialStarterInput<'a> {
    OfficialStarterInput {
        root,
        example_name,
        repository,
        project_name,
        turbo_version,
        create_turbo_version,
    }
}

fn written_package(store: &SecurityStore) -> &SecurityPackage {
    let package_json = store.calls.iter().find_map(|call| match call {
        SecurityCall::WritePackage(_, package_json) => Some(package_json),
        _ => None,
    });
    let Some(package_json) = package_json else {
        panic!("the test expects one package.json write");
    };
    package_json
}

#[test]
fn repository_matching_is_case_and_whitespace_sensitive() {
    for candidate in [
        repository("Vercel", "turbo"),
        repository("VERCEL", "turborepo"),
        repository("vercel ", "turbo"),
        repository(" vercel", "turbo"),
        repository("vercel", "Turbo"),
        repository("vercel", "turborepo "),
        repository("vercel", " turborepo"),
    ] {
        assert!(!is_official_starter(Some(candidate)));
    }
}

#[test]
fn repository_matching_rejects_unicode_confusables_and_normalization() {
    for candidate in [
        repository("vеrcel", "turbo"),
        repository("verceⅼ", "turbo"),
        repository("vercel", "turbо"),
        repository("vercel", "turbo\u{200d}"),
        repository("vercel", "ｔｕｒｂｏ"),
    ] {
        assert!(!is_official_starter(Some(candidate)));
    }
}

#[test]
fn repository_matching_rejects_prefix_suffix_and_path_values() {
    for candidate in [
        repository("vercel", "turbo-example"),
        repository("vercel", "my-turborepo"),
        repository("vercel", "turbo/../starter"),
        repository("vercel", "examples/turbo"),
        repository("vercel/turbo", "turborepo"),
    ] {
        assert!(!is_official_starter(Some(candidate)));
    }
}

#[test]
fn non_official_repository_cannot_reach_any_side_effect_provider() {
    let root = Path::new("project-root");
    let mut store = ExplodingStore;

    let response = transform_official_starter(
        input(
            root,
            "default",
            Some(repository("attacker", "turbo")),
            "project",
            None,
            "2.10.13",
        ),
        &mut store,
    )
    .expect("the non-applicable branch must not call the exploding store");

    assert_eq!(response.result, TransformStatus::NotApplicable);
}

#[test]
fn very_large_untrusted_repository_name_is_borrowed_and_rejected() {
    let root = Path::new("project-root");
    let large_name = "turbo".repeat(800_000);
    let mut store = ExplodingStore;

    let response = transform_official_starter(
        input(
            root,
            "default",
            Some(repository("vercel", &large_name)),
            "project",
            None,
            "2.10.13",
        ),
        &mut store,
    )
    .expect("a large non-matching name must not reach the store");

    assert_eq!(response.result, TransformStatus::NotApplicable);
}

#[test]
fn project_name_and_explicit_version_remain_data_not_command_text() {
    let root = Path::new("project-root");
    let project_name = "name\n\u{1b}[31m${PATH}; rm -rf /";
    let version = "9.0.0\n\u{1b}[2J;$(command)";
    let mut store = SecurityStore {
        package_result: Ok(Some(SecurityPackage {
            name: "old".to_owned(),
            turbo: Some("old".to_owned()),
            turbo_truthy: true,
        })),
        ..SecurityStore::default()
    };

    transform_official_starter(
        input(
            root,
            "default",
            None,
            project_name,
            Some(version),
            "2.10.13",
        ),
        &mut store,
    )
    .expect("the typed fake store must accept data values");

    assert_eq!(written_package(&store).name, project_name);
    assert_eq!(written_package(&store).turbo.as_deref(), Some(version));
}

#[test]
fn falsey_turbo_dependency_does_not_copy_a_large_fallback_version() {
    let root = Path::new("project-root");
    let large_version = "9".repeat(4 * 1024 * 1024);
    let mut store = SecurityStore::default();

    transform_official_starter(
        input(root, "example", None, "project", None, &large_version),
        &mut store,
    )
    .expect("a falsey dependency must not construct or assign a fallback version");

    assert_eq!(written_package(&store).turbo, None);
}

#[test]
fn only_meta_failures_are_swallowed_and_package_failures_remain_fatal() {
    let root = Path::new("project-root");
    let mut store = SecurityStore {
        meta_result: Err(SecurityError::Meta),
        package_result: Err(SecurityError::Read),
        ..SecurityStore::default()
    };

    let error = transform_official_starter(
        input(root, "example", None, "project", None, "2.10.13"),
        &mut store,
    )
    .expect_err("a package read failure must not be swallowed with meta failures");

    assert_eq!(
        error,
        OfficialStarterError::ReadPackageJson(SecurityError::Read)
    );
    assert_eq!(
        store.calls,
        [
            SecurityCall::PackageExists,
            SecurityCall::ReadMeta,
            SecurityCall::ReadPackage,
        ]
    );
}

#[test]
fn write_failure_after_meta_removal_cannot_report_success() {
    let root = Path::new("project-root");
    let mut store = SecurityStore {
        meta_result: Ok("metadata".to_owned()),
        write_result: Err(SecurityError::Write),
        ..SecurityStore::default()
    };

    let error = transform_official_starter(
        input(root, "example", None, "project", None, "2.10.13"),
        &mut store,
    )
    .expect_err("a provider write failure must not become a success response");

    assert_eq!(
        error,
        OfficialStarterError::WritePackageJson(SecurityError::Write)
    );
    assert!(store.calls.contains(&SecurityCall::RemoveMeta));
    assert!(
        store
            .calls
            .iter()
            .any(|call| matches!(call, SecurityCall::WritePackage(_, _)))
    );
}
