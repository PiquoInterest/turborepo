use create_turbo_rs::{
    ExampleRepository, OfficialStarterError, OfficialStarterInput, is_official_starter,
    plan_official_starter,
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
fn confusable_and_path_like_repository_identifiers_are_not_official() {
    for repository in [
        ExampleRepository {
            username: "vеrcel",
            name: "turbo",
        },
        ExampleRepository {
            username: "vercel/",
            name: "turbo",
        },
        ExampleRepository {
            username: "vercel",
            name: "../turbo",
        },
        ExampleRepository {
            username: "vercel",
            name: "turbo\0suffix",
        },
    ] {
        assert!(!is_official_starter(Some(repository)));
    }
}

#[test]
fn truthy_nonobject_package_json_is_rejected() {
    for package in [json!(true), json!(1), json!("starter"), json!([])] {
        let error = plan_official_starter(input(
            "starter",
            None,
            "project",
            None,
            "1.0.0",
            Some(&package),
            None,
        ))
        .expect_err("truthy non-object package JSON must be rejected");
        assert!(matches!(
            error,
            OfficialStarterError::PackageJsonMustBeObject
        ));
    }
}

#[test]
fn project_name_control_characters_and_quotes_round_trip_as_json_data() {
    let project_name = "safe\"\n\u{001b}[31m\u{202e}txt";
    let package = json!({ "name": "old" });
    let plan = plan_official_starter(input(
        "basic",
        None,
        project_name,
        None,
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("project name should serialize safely");
    let rendered = plan
        .package_json_contents
        .as_deref()
        .expect("truthy package JSON should be written");
    let parsed: Value = serde_json::from_str(rendered).expect("rendered JSON must parse");

    assert_eq!(parsed["name"], project_name);
    assert!(rendered.contains("\\n"));
    assert!(!rendered.contains('\u{001b}'));
}

#[test]
fn prototype_named_properties_remain_plain_json_data() {
    let package = json!({
        "__proto__": { "polluted": true },
        "constructor": { "prototype": { "owned": true } },
        "name": "starter"
    });
    let plan = plan_official_starter(input(
        "custom",
        None,
        "project",
        None,
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("prototype-shaped data should serialize");
    let parsed: Value = serde_json::from_str(
        plan.package_json_contents
            .as_deref()
            .expect("truthy package JSON should be written"),
    )
    .expect("rendered JSON must parse");

    assert_eq!(parsed, package);
}

#[test]
fn planning_does_not_mutate_the_callers_package_json() {
    let package = json!({
        "name": "old",
        "devDependencies": { "turbo": "^1.0.0" }
    });
    let before = package.clone();
    let plan = plan_official_starter(input(
        "basic",
        None,
        "new",
        Some("2.0.0"),
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("official starter should be planned");

    assert!(plan.package_json_contents.is_some());
    assert_eq!(package, before);
}

#[test]
fn javascript_array_index_boundaries_are_ordered_without_coercion() {
    let mut package = Map::new();
    package.insert("4294967295".into(), json!("not-index"));
    package.insert("00".into(), json!("double-zero"));
    package.insert("4294967294".into(), json!("last-index"));
    package.insert("-0".into(), json!("negative-zero"));
    package.insert("0".into(), json!("zero"));
    let package = Value::Object(package);

    let plan = plan_official_starter(input(
        "custom",
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
            "{\n  \"0\": \"zero\",\n  \"4294967294\": \"last-index\",\n  \"4294967295\": \"not-index\",\n  \"00\": \"double-zero\",\n  \"-0\": \"negative-zero\"\n}\n"
        )
    );
}

#[test]
fn hostile_version_text_is_serialized_as_a_dependency_value_only() {
    let requested_version = "1.0.0\"\n,\"scripts\":{\"postinstall\":\"owned\"}";
    let package = json!({ "devDependencies": { "turbo": "old" } });
    let plan = plan_official_starter(input(
        "custom",
        None,
        "project",
        Some(requested_version),
        "1.0.0",
        Some(&package),
        None,
    ))
    .expect("version text should serialize safely");
    let parsed: Value = serde_json::from_str(
        plan.package_json_contents
            .as_deref()
            .expect("truthy package JSON should be written"),
    )
    .expect("rendered JSON must parse");

    assert_eq!(parsed["devDependencies"]["turbo"], requested_version);
    assert!(parsed.get("scripts").is_none());
}
