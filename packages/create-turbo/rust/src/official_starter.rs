use std::fmt;

use serde_json::{Map, Value};

use crate::{TransformResponse, TransformStatus, is_default_example};

pub const OFFICIAL_STARTER_TRANSFORM_NAME: &str = "official-starter";
pub const OFFICIAL_REPOSITORY_OWNER: &str = "vercel";
pub const OFFICIAL_REPOSITORY_NAMES: [&str; 2] = ["turbo", "turborepo"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExampleRepository<'a> {
    pub username: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct OfficialStarterInput<'a> {
    pub example_name: &'a str,
    pub repository: Option<ExampleRepository<'a>>,
    pub project_name: &'a str,
    pub requested_turbo_version: Option<&'a str>,
    pub invocation_version: &'a str,
    pub package_json: Option<&'a Value>,
    pub meta_json: Option<&'a Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfficialStarterPlan {
    pub response: TransformResponse,
    pub meta_json: Option<Value>,
    pub remove_meta_json: bool,
    pub package_json_contents: Option<String>,
}

#[derive(Debug)]
pub enum OfficialStarterError {
    PackageJsonMustBeObject,
    Serialize(serde_json::Error),
}

impl fmt::Display for OfficialStarterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageJsonMustBeObject => {
                formatter.write_str("package.json must contain a JSON object")
            }
            Self::Serialize(_) => formatter.write_str("unable to serialize package.json"),
        }
    }
}

impl std::error::Error for OfficialStarterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Serialize(error) => Some(error),
            Self::PackageJsonMustBeObject => None,
        }
    }
}

#[must_use]
pub fn is_official_starter(repository: Option<ExampleRepository<'_>>) -> bool {
    match repository {
        None => true,
        Some(repository) => {
            repository.username == OFFICIAL_REPOSITORY_OWNER
                && OFFICIAL_REPOSITORY_NAMES.contains(&repository.name)
        }
    }
}

pub fn plan_official_starter(
    input: OfficialStarterInput<'_>,
) -> Result<OfficialStarterPlan, OfficialStarterError> {
    if !is_official_starter(input.repository) {
        return Ok(OfficialStarterPlan {
            response: response(TransformStatus::NotApplicable),
            meta_json: None,
            remove_meta_json: false,
            package_json_contents: None,
        });
    }

    let meta_json = input.meta_json.cloned();
    let remove_meta_json = meta_json.is_some();
    let package_json_contents = match input.package_json {
        None => None,
        Some(package_json) if !is_javascript_truthy(package_json) => None,
        Some(package_json) => {
            let Value::Object(mut package) = package_json.clone() else {
                return Err(OfficialStarterError::PackageJsonMustBeObject);
            };

            if is_default_example(input.example_name) {
                package.insert(
                    "name".into(),
                    Value::String(input.project_name.to_owned()),
                );
            }

            if let Some(Value::Object(dev_dependencies)) = package.get_mut("devDependencies")
                && dev_dependencies
                    .get("turbo")
                    .is_some_and(is_javascript_truthy)
            {
                let version = input
                    .requested_turbo_version
                    .filter(|version| !version.is_empty())
                    .map_or_else(
                        || format!("^{}", input.invocation_version),
                        str::to_owned,
                    );
                dev_dependencies.insert("turbo".into(), Value::String(version));
            }

            let normalized = normalize_javascript_property_order(Value::Object(package));
            let mut rendered =
                serde_json::to_string_pretty(&normalized).map_err(OfficialStarterError::Serialize)?;
            rendered.push('\n');
            Some(rendered)
        }
    };

    Ok(OfficialStarterPlan {
        response: response(TransformStatus::Success),
        meta_json,
        remove_meta_json,
        package_json_contents,
    })
}

const fn response(result: TransformStatus) -> TransformResponse {
    TransformResponse {
        result,
        name: OFFICIAL_STARTER_TRANSFORM_NAME,
    }
}

fn is_javascript_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                value != 0
            } else if let Some(value) = value.as_u64() {
                value != 0
            } else {
                value.as_f64().is_some_and(|value| value != 0.0)
            }
        }
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn normalize_javascript_property_order(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(normalize_javascript_property_order)
                .collect(),
        ),
        Value::Object(object) => {
            let mut indexed_properties = Vec::new();
            let mut string_properties = Vec::new();

            for (key, value) in object {
                let value = normalize_javascript_property_order(value);
                if let Some(index) = javascript_array_index(&key) {
                    indexed_properties.push((index, key, value));
                } else {
                    string_properties.push((key, value));
                }
            }

            indexed_properties.sort_unstable_by_key(|(index, _, _)| *index);
            let mut normalized = Map::new();
            for (_, key, value) in indexed_properties {
                normalized.insert(key, value);
            }
            for (key, value) in string_properties {
                normalized.insert(key, value);
            }
            Value::Object(normalized)
        }
        primitive => primitive,
    }
}

fn javascript_array_index(key: &str) -> Option<u32> {
    if key.is_empty()
        || !key.bytes().all(|byte| byte.is_ascii_digit())
        || (key.len() > 1 && key.starts_with('0'))
    {
        return None;
    }

    let index = key.parse::<u64>().ok()?;
    if index >= u64::from(u32::MAX) {
        return None;
    }
    u32::try_from(index).ok()
}
