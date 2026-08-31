use semver::VersionReq;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InputError {
    #[error("{field} must not be empty")]
    Empty { field: &'static str },
    #[error("{field} exceeds the maximum length of {maximum} characters")]
    TooLong {
        field: &'static str,
        maximum: usize,
    },
    #[error("{field} contains a control character")]
    ControlCharacter { field: &'static str },
    #[error("{field} must not begin with '-' because it is passed to a subprocess")]
    LeadingDash { field: &'static str },
    #[error("workspace is not a safe Turbo filter atom: {0}")]
    UnsafeWorkspace(String),
    #[error("task is not a safe Turbo task name: {0}")]
    UnsafeTask(String),
    #[error("comparison ref contains unsupported revision or filter syntax: {0}")]
    UnsafeRef(String),
    #[error("unsafe turbo version selector: {0}")]
    UnsafeVersionSelector(String),
    #[error("invalid turbo version requirement: {0}")]
    InvalidVersionSelector(String),
}

fn valid_workspace_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
}

pub fn validate_workspace(value: &str) -> Result<(), InputError> {
    validate_text_field("workspace", value, 512)?;

    let valid = if let Some(scoped) = value.strip_prefix('@') {
        let Some((scope, name)) = scoped.split_once('/') else {
            return Err(InputError::UnsafeWorkspace(value.to_owned()));
        };
        !name.contains('/') && valid_workspace_segment(scope) && valid_workspace_segment(name)
    } else {
        !value.contains('/') && valid_workspace_segment(value)
    };

    if !valid || value.contains("...") {
        return Err(InputError::UnsafeWorkspace(value.to_owned()));
    }
    Ok(())
}

pub fn validate_task(value: &str) -> Result<(), InputError> {
    validate_text_field("task", value, 512)?;
    if value.starts_with('-')
        || value.chars().any(char::is_whitespace)
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '.' | '_' | '-' | ':' | '#' | '/' | '@')
        })
    {
        return Err(InputError::UnsafeTask(value.to_owned()));
    }
    Ok(())
}

pub fn validate_text_field(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), InputError> {
    if value.is_empty() {
        return Err(InputError::Empty { field });
    }
    if value.chars().count() > maximum {
        return Err(InputError::TooLong { field, maximum });
    }
    if value.chars().any(char::is_control) {
        return Err(InputError::ControlCharacter { field });
    }
    Ok(())
}

pub fn validate_ref(value: &str) -> Result<(), InputError> {
    validate_text_field("comparison ref", value, 1_024)?;
    if value.starts_with('-') {
        return Err(InputError::LeadingDash {
            field: "comparison ref",
        });
    }
    let allowed = value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(character, '/' | '.' | '_' | '-' | '^' | '~' | '@' | '+')
    });
    let invalid_structure = value.contains("..")
        || value.contains("//")
        || value.contains("@{")
        || value.starts_with('/')
        || value.ends_with('/')
        || value.ends_with('.')
        || value.ends_with(".lock");
    if !allowed || invalid_structure {
        return Err(InputError::UnsafeRef(value.to_owned()));
    }
    Ok(())
}

pub fn validate_version_selector(value: &str) -> Result<VersionReq, InputError> {
    let value = value.trim();
    validate_text_field("turbo version", value, 128)?;

    let lower = value.to_ascii_lowercase();
    let unsafe_prefixes = [
        "file:", "link:", "git:", "git+", "http:", "https:", "ssh:", "github:",
        "npm:", "workspace:", "catalog:",
    ];
    let contains_path_or_package_syntax = value.contains('/')
        || value.contains('\\')
        || value.contains('@')
        || value.contains('#')
        || value.contains(':');

    if unsafe_prefixes
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        || contains_path_or_package_syntax
    {
        return Err(InputError::UnsafeVersionSelector(value.to_owned()));
    }

    let allowed = value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(
                character,
                '.' | '-' | '+' | '*' | '^' | '~' | '<' | '>' | '=' | ',' | ' ' | '|'
            )
    });
    if !allowed || !value.chars().any(|character| character.is_ascii_digit()) {
        return Err(InputError::UnsafeVersionSelector(value.to_owned()));
    }

    VersionReq::parse(value)
        .map_err(|_| InputError::InvalidVersionSelector(value.to_owned()))
}
