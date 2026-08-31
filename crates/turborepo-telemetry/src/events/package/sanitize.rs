use sha2::{Digest, Sha256};
use url::Url;

use super::{
    MAX_EVENT_KEY_BYTES, MAX_EVENT_VALUE_BYTES, MAX_METADATA_BYTES, TELEMETRY_ENDPOINT,
    types::PackageTelemetryError,
};

const PACKAGE_MANAGER_ALLOWLIST: [&str; 4] = ["npm", "pnpm", "yarn", "bun"];

pub fn environment_value_is_truthy(value: &str) -> bool {
    value == "1" || value.eq_ignore_ascii_case("true")
}

pub(super) fn environment_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| environment_value_is_truthy(&value))
        .unwrap_or(false)
}

pub(super) fn telemetry_endpoint(api: &str) -> Result<Url, PackageTelemetryError> {
    let base = Url::parse(api).map_err(|_| PackageTelemetryError::InvalidEndpoint)?;
    if base.scheme() != "https"
        || base.host_str().is_none()
        || !base.username().is_empty()
        || base.password().is_some()
        || base.fragment().is_some()
    {
        return Err(PackageTelemetryError::InvalidEndpoint);
    }
    base.join(TELEMETRY_ENDPOINT)
        .map_err(|_| PackageTelemetryError::InvalidEndpoint)
}

pub(super) fn classify_example(value: &str) -> &'static str {
    if value == "default" {
        return "default";
    }
    match Url::parse(value) {
        Ok(url) if url.host_str() == Some("github.com") => "github_url",
        Ok(_) => "other_url",
        Err(_) => "official",
    }
}

pub(super) fn classify_package_manager(value: &str) -> &'static str {
    PACKAGE_MANAGER_ALLOWLIST
        .iter()
        .copied()
        .find(|manager| manager.eq_ignore_ascii_case(value))
        .unwrap_or("other")
}

pub(super) fn checked_metadata(value: String) -> Result<String, PackageTelemetryError> {
    if value.is_empty()
        || value.len() > MAX_METADATA_BYTES
        || value.chars().any(is_unsafe_character)
    {
        return Err(PackageTelemetryError::InvalidMetadata);
    }
    Ok(value)
}

pub(super) fn public_value_or_other(value: &str, limit: usize) -> String {
    bounded_value(value.to_string(), limit, "other")
}

pub(super) fn bounded_event_key(value: String) -> String {
    bounded_value(value, MAX_EVENT_KEY_BYTES, "other")
}

pub(super) fn bounded_event_value(value: String, fallback: &str) -> String {
    bounded_value(value, MAX_EVENT_VALUE_BYTES, fallback)
}

fn bounded_value(value: String, limit: usize, fallback: &str) -> String {
    if value.is_empty() || value.len() > limit || value.chars().any(is_unsafe_character) {
        fallback.to_string()
    } else {
        value
    }
}

fn is_unsafe_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

pub(super) fn one_way_hash_with_salt(salt: &str, input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
