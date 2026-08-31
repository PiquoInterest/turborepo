use create_turbo_rs::{
    ExampleRepository, OFFICIAL_STARTER_TRANSFORM_NAME, OfficialStarterInput,
    TransformStatus, is_official_starter, plan_official_starter,
};
use serde_json::{Map, Value, json};

fn input<'a>(
    example_name: &'a str,
    repository: Option<ExampleRepository<'a>>,
    project_name: &'a str,
    requested_turbo_version: Option<&'a str>,
    invocation_version: &'a str,
    package_json: Option<&'a Value>,
    meta_json: Option<&'a Value>,
) -> OfficialStarterInput<'a> {
    OfficialStarterInput {
        example_name,
        repository,
        project_name,
        requested_turbo_version,
        invocation_version,
        package_json,
        meta_json,
    }
}

#[test]
fn classifies_missing_and_exact_vercel_repositories_as_official() {
    assert!(is_official_starter(None));
    assert!(is_official_starter(Some(ExampleRepository {
        username: "vercel",
        name: "turbo",
    })));
    assert!(is_official_starter(Some(ExampleRepository {
        username: "vercel",
        name: "turborepo",
    })));
}

#[test]
fn repository_classification_is_exact_and_case_sensitive() {
    for repository in [
        ExampleRepository {
            username: "Vercel",
            name: "turbo",
        },
        ExampleRepository {
            username: "vercel",
            name: "Turbo",
        },
        ExampleRepository {
            username: "vercel",
            name: "turbo-rs",
        },
    ] {
        assert!(!is_official_starter(Some(repository)));
    }
}

#[test]
fn nonofficial_starter_is_not_applicable_and_has_no_actions() {
    let package = json!({ "name": "before" });
    let meta = json!({ "title": "Example" });
    let plan = plan_official_starter(input(
        "basic",
        Some(ExampleRepository {
            username: "someone",
            name: "starter",
        }),
        "after",
        None,
        "2.0.0",
        Some(&package),
        Some(&meta),
    ))
    .expect("nonofficial classification should not fail");

    assert_eq!(plan.response.result, TransformStatus::NotApplicable);
    assert_eq!(plan.response.name, OFFICIAL_STARTER_TRANSFORM_NAME);
    assert_eq!(plan.meta_json, None);
    assert!(!plan.remove_meta_json);
    assert_eq!(plan.package_json_contents, None);
}

#[test]
fn basic_and_default_examples_receive_the_project_name() {
    for example_name in ["basic", "default"] {
        let package = json!({ "name": "old-name", "private": true });
        let plan = plan_official_starter(input(
            example_name,
            None,
            "new-project",
            None,
            "2.0.0",
            Some(&package),
            None,
        ))
        .expect("default starter should be planned");
        let rendered = plan
            .package_json_contents
            .as_deref()
            .expect("truthy package JSON should be written");
        let parsed: Value = serde_json::from_str(rendered).expect("rendered JSON must parse");
        assert_eq!(parsed["name"], "new-project");
    }
}

#[test]
fn nondefault_examples_preserve_the_package_name() {
    let package = json!({ "name": "kept-name", "private": true });
    let plan = plan_official_starter(input(
        "with-tailwind",
        None,
        "ignored-project-name",
        None,
        "2.0.0",
        Some(&package),
        None,
    ))
    .expect("official starter should be planned");
    let parsed: Value = serde_json::from_str(
        plan.package_json_contents
            .as_deref()
            .expect("truthy package JSON should be written"),
    )
    .expect("rendered JSON must parse");
    assert_eq!(parsed["name"], "kept-name");
}

#[test]
fn explicit_turbo_version_replaces_a_truthy_dependency() {
    let package = json!({
        "devDependencies": {
            "turbo": "^1.0.0",
            "typescript": "5.0.0"
        }
    });
    let plan = plan_official_starter(input(
        "starter",
        None,
        "project",
        Some("2.4.6-canary.1"),
        "9.9.9",
        Some(&package),
        None,
    ))
    .expect("official starter should be planned");
    let parsed: Value = serde_json::from_str(
        plan.package_json_contents
            .as_deref()
            .expect("truthy package JSON should be written"),
    )
    .expect("rendered JSON must parse");
    assert_eq!(parsed["devDependencies"]["turbo"], "2.4.6-canary.1");
    assert_eq!(parsed["devDependencies"]["typescript"], "5.0.0");
}

#[test]
fn absent_or_empty_requested_version_uses_the_invocation_version() {
    for requested in [None, Some("")] {
        let package = json!({ "devDependencies": { "turbo": "latest" } });
        let plan = plan_official_starter(input(
            "starter",
            None,
            "project",
            requested,
            "3.2.1",
            Some(&package),
            None,
        ))
        .expect("official starter should be planned");
        let parsed: Value = serde_json::from_str(
            plan.package_json_contents
                .as_deref()
                .expect("truthy package JSON should be written"),
        )
        .expect("rendered JSON must parse");
        assert_eq!(parsed["devDependencies"]["turbo"], "^3.2.1");
    }
}

#[test]
fn missing_or_falsy_turbo_dependency_is_not_rewritten() {
    for package in [
        json!({ "devDependencies": { "typescript": "5.0.0" } }),
        json!({ "devDependencies": { "turbo": false } }),
        json!({ "devDependencies": { "turbo": "" } }),
        json!({ "devDependencies": { "turbo": 0 } }),
        json!({ "devDependencies": null }),
    ] {
        let plan = plan_official_starter(input(
            "starter",
            None,
            "project",
            Some("9.9.9"),
            "3.2.1",
            Some(&package),
            None,
        ))
        .expect("official starter should be planned");
        let parsed: Value = serde_json::from_str(
            plan.package_json_contents
                .as_deref()
                .expect("truthy package JSON should still be serialized"),
        )
        .expect("rendered JSON must parse");
        assert_eq!(parsed, package);
    }
}

#[test]
fn absent_or_javascript_falsy_package_json_skips_the_write() {
    let absent = plan_official_starter(input(
        "starter", None, "project", None, "1.0.0", None, None,
    ))
    .expect("missing package JSON should not fail");
    assert_eq!(absent.package_json_contents, None);

    for package in [
        Value::Null,
        Value::Bool(false),
        json!(0),
        Value::String(String::new()),
    ] {
        let plan = plan_official_starter(input(
            "starter",
            None,
            "project",
            None,
            "1.0.0",
            Some(&package),
            None,
        ))
        .expect("falsy package JSON should not fail");
        assert_eq!(plan.package_json_contents, None);
    }
}

#[test]
fn parsed_meta_json_is_returned_and_scheduled_for_removal() {
    let meta = json!({ "title": "Starter", "featured": true });
    let plan = plan_official_starter(input(
        "starter",
        None,
        "project",
        None,
        "1.0.0",
        None,
        Some(&meta),
    ))
    .expect("official starter should be planned");

    assert_eq!(plan.response.result, TransformStatus::Success);
    assert_eq!(plan.meta_json, Some(meta));
    assert!(plan.remove_meta_json);
}

#[test]
fn package_json_uses_two_space_indentation_and_a_final_newline() {
    let package = json!({
        "private": true,
        "name": "old-name",
        "devDependencies": {
            "turbo": "^1.0.0",
            "typescript": "5.0.0"
        }
    });
    let plan = plan_official_starter(input(
        "basic",
        None,
        "new-name",
        Some("2.0.0"),
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("official starter should be planned");

    assert_eq!(
        plan.package_json_contents.as_deref(),
        Some(
            "{\n  \"private\": true,\n  \"name\": \"new-name\",\n  \"devDependencies\": {\n    \"turbo\": \"2.0.0\",\n    \"typescript\": \"5.0.0\"\n  }\n}\n"
        )
    );
}

#[test]
fn serialization_uses_javascript_property_enumeration_order_recursively() {
    let mut nested = Map::new();
    nested.insert("3".into(), json!("three"));
    nested.insert("02".into(), json!("leading"));
    nested.insert("1".into(), json!("one"));

    let mut package = Map::new();
    package.insert("10".into(), json!("ten"));
    package.insert("01".into(), json!("leading"));
    package.insert("4294967295".into(), json!("not-index"));
    package.insert("2".into(), json!("two"));
    package.insert("4294967294".into(), json!("last-index"));
    package.insert("nested".into(), Value::Object(nested));
    let package = Value::Object(package);

    let plan = plan_official_starter(input(
        "starter",
        None,
        "project",
        None,
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("official starter should be planned");

    assert_eq!(
        plan.package_json_contents.as_deref(),
        Some(
            "{\n  \"2\": \"two\",\n  \"10\": \"ten\",\n  \"4294967294\": \"last-index\",\n  \"01\": \"leading\",\n  \"4294967295\": \"not-index\",\n  \"nested\": {\n    \"1\": \"one\",\n    \"3\": \"three\",\n    \"02\": \"leading\"\n  }\n}\n"
        )
    );
}
