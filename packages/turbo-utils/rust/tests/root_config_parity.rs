#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{fs, path::Path};

use serde_json::json;
use tempfile::TempDir;
use turbo_utils_rs::{
    ConfigOptions, TurboConfigError, TurboRootOptions, clear_config_caches, for_each_task_def,
    get_turbo_configs, get_turbo_root, get_workspace_configs, parse_json5,
    resolve_turbo_config_path,
};

fn write(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture parent");
    }
    fs::write(path, content).expect("write fixture");
}

fn workspace_fixture() -> TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        temp.path().join("package.json"),
        r#"{
          "private": true,
          "workspaces": ["apps/*", "packages/*"],
          "packageManager": "yarn@1.22.19"
        }"#,
    );
    write(
        temp.path().join("turbo.json"),
        r#"{
          "$schema": "https://turborepo.dev/schema.json",
          "globalEnv": ["CI"],
          "tasks": { "build": { "env": ["ENV_1"] } }
        }"#,
    );
    write(
        temp.path().join("apps/docs/package.json"),
        r#"{"name":"docs"}"#,
    );
    write(
        temp.path().join("apps/docs/turbo.json"),
        r#"{
          "$schema": "https://turborepo.dev/schema.json",
          "extends": ["//"],
          "tasks": { "build": { "env": ["ENV_2"] } }
        }"#,
    );
    write(
        temp.path().join("apps/web/package.json"),
        r#"{"name":"web"}"#,
    );
    write(
        temp.path().join("apps/web/turbo.jsonc"),
        r#"{
          // workspace comment
          "$schema": "https://turborepo.dev/schema.json",
          "extends": ["//"],
          "tasks": { "build": { "env": ["IS_SERVER"], }, },
        }"#,
    );
    write(
        temp.path().join("packages/ui/package.json"),
        r#"{"name":"ui"}"#,
    );
    temp
}

#[test]
fn root_discovery_matches_root_child_and_nonexistent_descendant() {
    clear_config_caches();
    let temp = workspace_fixture();
    let expected = temp.path().to_path_buf();

    for start in [
        expected.clone(),
        expected.join("apps"),
        expected.join("apps/docs"),
        expected.join("not-a-real/path"),
    ] {
        assert_eq!(
            get_turbo_root(Some(&start), TurboRootOptions { cache: false }),
            Some(expected.clone())
        );
    }
}

#[test]
fn nearest_workspace_config_with_extends_does_not_shadow_root() {
    clear_config_caches();
    let temp = workspace_fixture();
    let child = temp.path().join("apps/docs/deep");
    fs::create_dir_all(&child).expect("create child");

    assert_eq!(
        get_turbo_root(Some(&child), TurboRootOptions { cache: false }),
        Some(temp.path().to_path_buf())
    );
}

#[test]
fn jsonc_root_with_comments_is_supported() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(temp.path().join("package.json"), r#"{"name":"app"}"#);
    write(
        temp.path().join("turbo.jsonc"),
        r#"{
          // root config
          tasks: { build: {}, },
        }"#,
    );

    assert_eq!(
        get_turbo_root(
            Some(&temp.path().join("child")),
            TurboRootOptions { cache: false }
        ),
        Some(temp.path().to_path_buf())
    );
}

#[test]
fn root_falls_back_to_nearest_package_json() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(temp.path().join("package.json"), r#"{"name":"app"}"#);

    assert_eq!(
        get_turbo_root(
            Some(&temp.path().join("src/nested")),
            TurboRootOptions { cache: false }
        ),
        Some(temp.path().to_path_buf())
    );
}

#[test]
fn root_cache_can_be_cleared() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(temp.path().join("package.json"), r#"{"name":"app"}"#);
    let child = temp.path().join("child");

    let first = get_turbo_root(Some(&child), TurboRootOptions { cache: true });
    fs::remove_file(temp.path().join("package.json")).expect("remove package json");
    let cached = get_turbo_root(Some(&child), TurboRootOptions { cache: true });
    assert_eq!(cached, first);

    clear_config_caches();
    assert_eq!(
        get_turbo_root(Some(&child), TurboRootOptions { cache: true }),
        None
    );
}

#[test]
fn resolves_json_jsonc_both_and_missing_config_paths() {
    let temp = tempfile::tempdir().expect("tempdir");

    let missing = resolve_turbo_config_path(temp.path());
    assert!(!missing.config_exists);
    assert!(missing.config_path.is_none());
    assert!(missing.error.is_none());

    write(temp.path().join("turbo.json"), "{}");
    let json = resolve_turbo_config_path(temp.path());
    assert!(json.config_exists);
    assert_eq!(json.config_path, Some(temp.path().join("turbo.json")));

    fs::remove_file(temp.path().join("turbo.json")).expect("remove json");
    write(temp.path().join("turbo.jsonc"), "{}");
    let jsonc = resolve_turbo_config_path(temp.path());
    assert!(jsonc.config_exists);
    assert_eq!(jsonc.config_path, Some(temp.path().join("turbo.jsonc")));

    write(temp.path().join("turbo.json"), "{}");
    let both = resolve_turbo_config_path(temp.path());
    assert!(!both.config_exists);
    assert!(both.config_path.is_none());
    assert!(
        both.error
            .as_deref()
            .is_some_and(|message| message.contains("Found both turbo.json and turbo.jsonc"))
    );
}

#[test]
fn parses_json5_comments_trailing_commas_single_quotes_and_identifier_keys() {
    let parsed = parse_json5(
        r#"{
          // comment
          unquoted: 'value',
          tasks: {
            build: { dependsOn: ['^build'], },
          },
        }"#,
    )
    .expect("parse JSON5");

    assert_eq!(
        parsed,
        json!({
            "unquoted": "value",
            "tasks": { "build": { "dependsOn": ["^build"] } }
        })
    );
}

#[test]
fn discovers_root_and_workspace_turbo_configs_in_stable_order() {
    clear_config_caches();
    let temp = workspace_fixture();
    let configs =
        get_turbo_configs(Some(temp.path()), ConfigOptions { cache: false }).expect("configs");

    assert_eq!(configs.len(), 3);
    assert!(configs[0].is_root_config);
    assert_eq!(configs[0].workspace_path, temp.path());
    assert_eq!(configs[0].config["globalEnv"], json!(["CI"]));

    assert!(!configs[1].is_root_config);
    assert_eq!(configs[1].workspace_path, temp.path().join("apps/docs"));
    assert_eq!(configs[1].config["tasks"]["build"]["env"], json!(["ENV_2"]));

    assert!(!configs[2].is_root_config);
    assert_eq!(configs[2].workspace_path, temp.path().join("apps/web"));
    assert_eq!(
        configs[2].config["tasks"]["build"]["env"],
        json!(["IS_SERVER"])
    );
}

#[test]
fn old_workspace_format_without_workspace_configs_returns_only_root() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        temp.path().join("package.json"),
        r#"{"private":true,"workspaces":{"packages":["packages/*"]}}"#,
    );
    write(
        temp.path().join("turbo.json"),
        r#"{"tasks":{"build":{"outputs":[".next/**"]}}}"#,
    );
    write(
        temp.path().join("packages/ui/package.json"),
        r#"{"name":"ui"}"#,
    );

    let configs =
        get_turbo_configs(Some(temp.path()), ConfigOptions { cache: false }).expect("configs");
    assert_eq!(configs.len(), 1);
    assert!(configs[0].is_root_config);
}

#[test]
fn unsafe_workspace_globs_cannot_escape_root() {
    clear_config_caches();
    let parent = tempfile::tempdir().expect("tempdir");
    let root = parent.path().join("repo");
    let outside = parent.path().join("outside-workspace");
    fs::create_dir_all(&root).expect("root");
    write(
        root.join("package.json"),
        &format!(
            r#"{{"private":true,"workspaces":["apps/*","../outside-workspace","{}"]}}"#,
            outside.display()
        ),
    );
    write(root.join("turbo.json"), r#"{"tasks":{"build":{}}}"#);
    write(
        root.join("apps/web/turbo.json"),
        r#"{"extends":["//"],"tasks":{"build":{}}}"#,
    );
    write(
        outside.join("turbo.json"),
        r#"{"extends":["//"],"tasks":{"build":{"env":["OUTSIDE"]}}}"#,
    );

    let configs = get_turbo_configs(Some(&root), ConfigOptions { cache: false }).expect("configs");
    assert_eq!(configs.len(), 2);
    assert!(
        configs
            .iter()
            .all(|config| config.turbo_config_path.starts_with(&root))
    );
}

#[cfg(unix)]
#[test]
fn symlinked_workspace_config_outside_root_is_rejected() {
    use std::os::unix::fs::symlink;

    clear_config_caches();
    let parent = tempfile::tempdir().expect("tempdir");
    let root = parent.path().join("repo");
    let outside = parent.path().join("outside");
    write(
        root.join("package.json"),
        r#"{"private":true,"workspaces":["apps/*"]}"#,
    );
    write(root.join("turbo.json"), r#"{"tasks":{"build":{}}}"#);
    write(
        outside.join("turbo.json"),
        r#"{"extends":["//"],"tasks":{"build":{"env":["OUTSIDE"]}}}"#,
    );
    fs::create_dir_all(root.join("apps/linked")).expect("linked dir");
    symlink(
        outside.join("turbo.json"),
        root.join("apps/linked/turbo.json"),
    )
    .expect("symlink");

    let configs = get_turbo_configs(Some(&root), ConfigOptions { cache: false }).expect("configs");
    assert_eq!(configs.len(), 1);
    assert!(configs[0].is_root_config);
}

#[test]
fn duplicate_json_and_jsonc_in_one_directory_is_an_error() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(temp.path().join("package.json"), r#"{"name":"app"}"#);
    write(temp.path().join("turbo.json"), "{}");
    write(temp.path().join("turbo.jsonc"), "{}");

    let result = get_turbo_configs(Some(temp.path()), ConfigOptions { cache: false });
    assert!(matches!(
        result,
        Err(TurboConfigError::DuplicateConfig { .. })
    ));
}

#[test]
fn invalid_root_and_workspace_config_shapes_are_skipped() {
    clear_config_caches();
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        temp.path().join("package.json"),
        r#"{"private":true,"workspaces":["apps/*"]}"#,
    );
    write(
        temp.path().join("turbo.json"),
        r#"{"extends":["//"],"tasks":{}}"#,
    );
    write(
        temp.path().join("apps/web/turbo.json"),
        r#"{"tasks":{"build":{}}}"#,
    );

    let configs =
        get_turbo_configs(Some(temp.path()), ConfigOptions { cache: false }).expect("configs");
    assert!(configs.is_empty());
}

#[test]
fn workspace_configs_include_packages_without_turbo_config() {
    clear_config_caches();
    let temp = workspace_fixture();
    let configs = get_workspace_configs(Some(temp.path()), ConfigOptions { cache: false });

    assert_eq!(configs.len(), 4);
    assert!(configs[0].is_workspace_root);
    assert_eq!(configs[0].workspace_name, None);
    assert_eq!(configs[1].workspace_name.as_deref(), Some("docs"));
    assert!(configs[1].turbo_config.is_some());
    assert_eq!(configs[2].workspace_name.as_deref(), Some("web"));
    assert!(configs[2].turbo_config.is_some());
    assert_eq!(configs[3].workspace_name.as_deref(), Some("ui"));
    assert!(configs[3].turbo_config.is_none());
}

#[test]
fn task_iteration_prefers_pipeline_when_present() {
    let legacy = json!({
        "pipeline": { "build": {"cache": true}, "test": {} },
        "tasks": { "ignored": {} }
    });
    let mut legacy_names = Vec::new();
    for_each_task_def(&legacy, |name, _definition| {
        legacy_names.push(name.to_owned())
    });
    assert_eq!(legacy_names, ["build", "test"]);

    let modern = json!({ "tasks": { "lint": {}, "build": {} } });
    let mut modern_names = Vec::new();
    for_each_task_def(&modern, |name, _definition| {
        modern_names.push(name.to_owned())
    });
    assert_eq!(modern_names, ["lint", "build"]);
}
