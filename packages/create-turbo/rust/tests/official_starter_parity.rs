use std::path::{Path, PathBuf};

use create_turbo_rs::{
    ExampleRepository, OFFICIAL_REPOSITORIES, OFFICIAL_STARTER_TRANSFORM_NAME,
    OfficialStarterError, OfficialStarterInput, OfficialStarterPackageJson, OfficialStarterStore,
    TransformStatus, is_official_starter, transform_official_starter,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeMeta {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakePackage {
    name: Option<String>,
    turbo: Option<String>,
    turbo_truthy: bool,
    untouched: String,
}

impl OfficialStarterPackageJson for FakePackage {
    fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned());
    }

    fn turbo_dev_dependency_is_truthy(&self) -> bool {
        self.turbo_truthy
    }

    fn set_turbo_dev_dependency(&mut self, version: &str) {
        self.turbo = Some(version.to_owned());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FakeError {
    MetaRead,
    MetaRemove,
    PackageRead,
    PackageWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    PackageExists(PathBuf),
    ReadMeta(PathBuf),
    RemoveMeta(PathBuf),
    ReadPackage(PathBuf),
    WritePackage(PathBuf, FakePackage),
}

struct FakeStore {
    calls: Vec<Call>,
    has_package_json: bool,
    meta_result: Result<FakeMeta, FakeError>,
    remove_meta_result: Result<(), FakeError>,
    package_result: Result<Option<FakePackage>, FakeError>,
    write_result: Result<(), FakeError>,
}

impl Default for FakeStore {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            has_package_json: false,
            meta_result: Err(FakeError::MetaRead),
            remove_meta_result: Ok(()),
            package_result: Ok(None),
            write_result: Ok(()),
        }
    }
}

impl OfficialStarterStore for FakeStore {
    type Error = FakeError;
    type MetaJson = FakeMeta;
    type PackageJson = FakePackage;

    fn package_json_exists(&mut self, root: &Path) -> bool {
        self.calls.push(Call::PackageExists(root.to_path_buf()));
        self.has_package_json
    }

    fn read_meta_json(&mut self, root: &Path) -> Result<Self::MetaJson, Self::Error> {
        self.calls.push(Call::ReadMeta(root.to_path_buf()));
        self.meta_result.clone()
    }

    fn remove_meta_json(&mut self, root: &Path) -> Result<(), Self::Error> {
        self.calls.push(Call::RemoveMeta(root.to_path_buf()));
        self.remove_meta_result
    }

    fn read_package_json(&mut self, root: &Path) -> Result<Option<Self::PackageJson>, Self::Error> {
        self.calls.push(Call::ReadPackage(root.to_path_buf()));
        self.package_result.clone()
    }

    fn write_package_json(
        &mut self,
        root: &Path,
        package_json: &Self::PackageJson,
    ) -> Result<(), Self::Error> {
        self.calls
            .push(Call::WritePackage(root.to_path_buf(), package_json.clone()));
        self.write_result
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
) -> OfficialStarterInput<'a> {
    OfficialStarterInput {
        root,
        example_name,
        repository,
        project_name,
        turbo_version,
        create_turbo_version: "2.10.13-canary.1",
    }
}

fn package(name: &str, turbo: Option<&str>, turbo_truthy: bool) -> FakePackage {
    FakePackage {
        name: Some(name.to_owned()),
        turbo: turbo.map(str::to_owned),
        turbo_truthy,
        untouched: "preserve-me".to_owned(),
    }
}

fn written_package(store: &FakeStore) -> &FakePackage {
    let package_json = store.calls.iter().find_map(|call| match call {
        Call::WritePackage(_, package_json) => Some(package_json),
        _ => None,
    });
    let Some(package_json) = package_json else {
        panic!("the test expects one package.json write");
    };
    package_json
}

#[test]
fn official_repository_constants_match_the_typescript_source() {
    assert_eq!(OFFICIAL_STARTER_TRANSFORM_NAME, "official-starter");
    assert_eq!(OFFICIAL_REPOSITORIES, ["turbo", "turborepo"]);
}

#[test]
fn repository_classification_matches_the_source_contract() {
    assert!(is_official_starter(None));
    assert!(is_official_starter(Some(repository("vercel", "turbo"))));
    assert!(is_official_starter(Some(repository("vercel", "turborepo"))));
    assert!(!is_official_starter(Some(repository("acme", "turbo"))));
    assert!(!is_official_starter(Some(repository("vercel", "other"))));
}

#[test]
fn non_official_repository_is_not_applicable_without_store_access() {
    let root = Path::new("project-root");
    let mut store = FakeStore::default();

    let response = transform_official_starter(
        input(
            root,
            "example",
            Some(repository("acme", "starter")),
            "project",
            None,
        ),
        &mut store,
    )
    .expect("a non-official repository cannot reach a failing provider");

    assert_eq!(response.result, TransformStatus::NotApplicable);
    assert_eq!(response.name, OFFICIAL_STARTER_TRANSFORM_NAME);
    assert_eq!(response.meta_json, None);
    assert!(store.calls.is_empty());
}

#[test]
fn package_existence_is_observed_before_meta_processing() {
    let root = Path::new("project-root");
    let meta = FakeMeta {
        name: "starter-meta".to_owned(),
    };
    let mut store = FakeStore {
        meta_result: Ok(meta.clone()),
        ..FakeStore::default()
    };

    let response =
        transform_official_starter(input(root, "example", None, "project", None), &mut store)
            .expect("the successful fake store must return a response");

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(response.meta_json, Some(meta));
    assert_eq!(
        store.calls,
        [
            Call::PackageExists(root.to_path_buf()),
            Call::ReadMeta(root.to_path_buf()),
            Call::RemoveMeta(root.to_path_buf()),
        ]
    );
}

#[test]
fn meta_read_failure_is_swallowed_without_a_remove_attempt() {
    let root = Path::new("project-root");
    let mut store = FakeStore::default();

    let response =
        transform_official_starter(input(root, "example", None, "project", None), &mut store)
            .expect("the source swallows meta.json read failures");

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(response.meta_json, None);
    assert_eq!(
        store.calls,
        [
            Call::PackageExists(root.to_path_buf()),
            Call::ReadMeta(root.to_path_buf()),
        ]
    );
}

#[test]
fn meta_remove_failure_is_swallowed_and_the_parsed_value_is_returned() {
    let root = Path::new("project-root");
    let meta = FakeMeta {
        name: "starter-meta".to_owned(),
    };
    let mut store = FakeStore {
        meta_result: Ok(meta.clone()),
        remove_meta_result: Err(FakeError::MetaRemove),
        ..FakeStore::default()
    };

    let response =
        transform_official_starter(input(root, "example", None, "project", None), &mut store)
            .expect("the source swallows meta.json removal failures");

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(response.meta_json, Some(meta));
    assert_eq!(store.calls.len(), 3);
}

#[test]
fn missing_package_json_still_returns_success() {
    let root = Path::new("project-root");
    let mut store = FakeStore::default();

    let response = transform_official_starter(
        input(root, "basic", None, "renamed", Some("9.0.0")),
        &mut store,
    )
    .expect("a missing package.json is a successful source branch");

    assert_eq!(response.result, TransformStatus::Success);
    assert!(
        !store
            .calls
            .iter()
            .any(|call| matches!(call, Call::ReadPackage(_)))
    );
    assert!(
        !store
            .calls
            .iter()
            .any(|call| matches!(call, Call::WritePackage(_, _)))
    );
}

#[test]
fn package_read_failure_maps_to_nonfatal_transform_error() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Err(FakeError::PackageRead),
        ..FakeStore::default()
    };

    let error =
        transform_official_starter(input(root, "example", None, "project", None), &mut store)
            .expect_err("a package.json read failure must not become success");

    assert_eq!(
        error,
        OfficialStarterError::ReadPackageJson(FakeError::PackageRead)
    );
    assert_eq!(error.to_string(), "Unable to read package.json");
    assert_eq!(error.transform_name(), OFFICIAL_STARTER_TRANSFORM_NAME);
    assert!(!error.is_fatal());
}

#[test]
fn falsey_package_json_content_is_not_written() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(None),
        ..FakeStore::default()
    };

    let response =
        transform_official_starter(input(root, "basic", None, "project", None), &mut store)
            .expect("falsey parsed package content follows the source no-write branch");

    assert_eq!(response.result, TransformStatus::Success);
    assert!(
        !store
            .calls
            .iter()
            .any(|call| matches!(call, Call::WritePackage(_, _)))
    );
}

#[test]
fn basic_example_renames_package_and_uses_explicit_turbo_version() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("old-name", Some("^1.0.0"), true))),
        ..FakeStore::default()
    };

    let response = transform_official_starter(
        input(root, "basic", None, "new-name", Some("9.15.4")),
        &mut store,
    )
    .expect("the fake package write must succeed");

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(written_package(&store).name.as_deref(), Some("new-name"));
    assert_eq!(written_package(&store).turbo.as_deref(), Some("9.15.4"));
    assert_eq!(written_package(&store).untouched, "preserve-me");
}

#[test]
fn default_example_renames_package_and_uses_invocation_version() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("old-name", Some("^1.0.0"), true))),
        ..FakeStore::default()
    };

    transform_official_starter(input(root, "default", None, "new-name", None), &mut store)
        .expect("the fake package write must succeed");

    assert_eq!(written_package(&store).name.as_deref(), Some("new-name"));
    assert_eq!(
        written_package(&store).turbo.as_deref(),
        Some("^2.10.13-canary.1")
    );
}

#[test]
fn non_default_example_keeps_name_but_updates_turbo() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("starter-name", Some("^1.0.0"), true))),
        ..FakeStore::default()
    };

    transform_official_starter(
        input(root, "with-next", None, "ignored-name", Some("3.0.0")),
        &mut store,
    )
    .expect("the fake package write must succeed");

    assert_eq!(
        written_package(&store).name.as_deref(),
        Some("starter-name")
    );
    assert_eq!(written_package(&store).turbo.as_deref(), Some("3.0.0"));
}

#[test]
fn empty_explicit_turbo_version_uses_the_invocation_fallback() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("starter", Some("^1.0.0"), true))),
        ..FakeStore::default()
    };

    transform_official_starter(
        input(root, "example", None, "project", Some("")),
        &mut store,
    )
    .expect("the fake package write must succeed");

    assert_eq!(
        written_package(&store).turbo.as_deref(),
        Some("^2.10.13-canary.1")
    );
}

#[test]
fn falsey_turbo_dependency_is_not_updated() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("starter", Some(""), false))),
        ..FakeStore::default()
    };

    transform_official_starter(
        input(root, "example", None, "project", Some("9.0.0")),
        &mut store,
    )
    .expect("the fake package write must succeed");

    assert_eq!(written_package(&store).turbo.as_deref(), Some(""));
}

#[test]
fn truthy_package_is_written_even_when_no_field_changes() {
    let root = Path::new("project-root");
    let original = package("starter", None, false);
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(original.clone())),
        ..FakeStore::default()
    };

    transform_official_starter(input(root, "example", None, "ignored", None), &mut store)
        .expect("the fake package write must succeed");

    assert_eq!(written_package(&store), &original);
}

#[test]
fn package_write_failure_maps_to_nonfatal_transform_error() {
    let root = Path::new("project-root");
    let mut store = FakeStore {
        has_package_json: true,
        package_result: Ok(Some(package("starter", None, false))),
        write_result: Err(FakeError::PackageWrite),
        ..FakeStore::default()
    };

    let error =
        transform_official_starter(input(root, "example", None, "project", None), &mut store)
            .expect_err("a package.json write failure must not become success");

    assert_eq!(
        error,
        OfficialStarterError::WritePackageJson(FakeError::PackageWrite)
    );
    assert_eq!(error.to_string(), "Unable to write package.json");
    assert_eq!(error.transform_name(), OFFICIAL_STARTER_TRANSFORM_NAME);
    assert!(!error.is_fatal());
}
