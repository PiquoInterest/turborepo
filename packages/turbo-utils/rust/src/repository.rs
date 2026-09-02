use thiserror::Error;

use crate::project::RepoInfo;

pub const GITHUB_REPOSITORY_URL_MAX_CHARS: usize = 4_096;
pub const GIT_REFERENCE_MAX_CHARS: usize = 1_024;
const GITHUB_OWNER_MAX_CHARS: usize = 39;
const GITHUB_REPOSITORY_NAME_MAX_CHARS: usize = 100;
const REPOSITORY_PATH_MAX_CHARS: usize = 2_048;
const REPOSITORY_PATH_MAX_COMPONENTS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubRepositoryLocation {
    NeedsDefaultBranch {
        username: String,
        name: String,
        file_path: String,
    },
    Resolved(RepoInfo),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GitHubRepositoryLocationError {
    #[error("invalid GitHub repository URL")]
    InvalidUrl,
    #[error("invalid GitHub owner")]
    InvalidOwner,
    #[error("invalid GitHub repository name")]
    InvalidRepository,
    #[error("unsupported GitHub repository URL path")]
    UnsupportedPath,
    #[error("unsafe repository subpath")]
    UnsafeRepositoryPath,
    #[error("invalid Git reference")]
    InvalidReference,
}

fn parse_github_path(input: &str) -> Result<&str, GitHubRepositoryLocationError> {
    if input.chars().count() > GITHUB_REPOSITORY_URL_MAX_CHARS
        || input
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(GitHubRepositoryLocationError::InvalidUrl);
    }

    let scheme_end = input
        .find("://")
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    let scheme = input
        .get(..scheme_end)
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    if !scheme.eq_ignore_ascii_case("https") {
        return Err(GitHubRepositoryLocationError::InvalidUrl);
    }

    let after_scheme = input
        .get(scheme_end + 3..)
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    let authority_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let authority = after_scheme
        .get(..authority_end)
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    if !authority.eq_ignore_ascii_case("github.com") {
        return Err(GitHubRepositoryLocationError::InvalidUrl);
    }

    let suffix = after_scheme
        .get(authority_end..)
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    let path_end = suffix.find(['?', '#']).unwrap_or(suffix.len());
    let path = suffix
        .get(..path_end)
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    let path = if path.is_empty() { "/" } else { path };
    if !path.starts_with('/') || path.contains(['\\', '%']) {
        return Err(GitHubRepositoryLocationError::InvalidUrl);
    }
    Ok(path)
}

fn split_path(path: &str) -> Result<Vec<&str>, GitHubRepositoryLocationError> {
    if path == "/" {
        return Ok(Vec::new());
    }
    let body = path
        .strip_prefix('/')
        .ok_or(GitHubRepositoryLocationError::InvalidUrl)?;
    let body = body.strip_suffix('/').unwrap_or(body);
    if body.is_empty() || body.contains("//") {
        return Err(GitHubRepositoryLocationError::UnsupportedPath);
    }

    let segments = body.split('/').collect::<Vec<_>>();
    if segments.len() > REPOSITORY_PATH_MAX_COMPONENTS
        || segments.iter().any(|segment| {
            segment.is_empty()
                || matches!(*segment, "." | "..")
                || segment
                    .chars()
                    .any(|character| character.is_control() || character.is_whitespace())
        })
    {
        return Err(GitHubRepositoryLocationError::UnsupportedPath);
    }
    Ok(segments)
}

fn valid_owner(owner: &str) -> bool {
    let length = owner.chars().count();
    (1..=GITHUB_OWNER_MAX_CHARS).contains(&length)
        && !owner.starts_with('-')
        && !owner.ends_with('-')
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn valid_repository_name(name: &str) -> bool {
    let length = name.chars().count();
    (1..=GITHUB_REPOSITORY_NAME_MAX_CHARS).contains(&length)
        && !matches!(name, "." | "..")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn normalize_repository_path(path: &str) -> Result<String, GitHubRepositoryLocationError> {
    let normalized = path.strip_prefix('/').unwrap_or(path);
    if normalized.is_empty() {
        return Ok(String::new());
    }
    if normalized.chars().count() > REPOSITORY_PATH_MAX_CHARS
        || normalized.contains(['\\', '%', '?', '#'])
        || normalized
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(GitHubRepositoryLocationError::UnsafeRepositoryPath);
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments.len() > REPOSITORY_PATH_MAX_COMPONENTS
        || segments
            .iter()
            .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
    {
        return Err(GitHubRepositoryLocationError::UnsafeRepositoryPath);
    }
    Ok(segments.join("/"))
}

fn valid_git_reference(reference: &str) -> bool {
    if reference.is_empty()
        || reference.chars().count() > GIT_REFERENCE_MAX_CHARS
        || reference == "@"
        || reference.starts_with('-')
        || reference.starts_with('/')
        || reference.ends_with('/')
        || reference.ends_with('.')
        || reference.contains("//")
        || reference.contains("..")
        || reference.contains("@{")
        || reference.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || matches!(character, '~' | '^' | ':' | '?' | '*' | '[' | '\\')
        })
    {
        return false;
    }

    reference.split('/').all(|component| {
        !component.is_empty() && !component.starts_with('.') && !component.ends_with(".lock")
    })
}

fn literal_suffix_start(tail: &[&str], suffix: &[&str]) -> Option<usize> {
    if suffix.is_empty() || tail.len() <= suffix.len() {
        return None;
    }
    let start = tail.len() - suffix.len();
    tail.get(start..)
        .filter(|candidate| *candidate == suffix)
        .map(|_| start)
}

pub fn parse_github_repository_location(
    input: &str,
    example_path: Option<&str>,
) -> Result<GitHubRepositoryLocation, GitHubRepositoryLocationError> {
    let path = parse_github_path(input)?;
    let segments = split_path(path)?;
    if segments.len() < 2 {
        return Err(GitHubRepositoryLocationError::UnsupportedPath);
    }

    let username = segments[0];
    let name = segments[1];
    if !valid_owner(username) {
        return Err(GitHubRepositoryLocationError::InvalidOwner);
    }
    if !valid_repository_name(name) {
        return Err(GitHubRepositoryLocationError::InvalidRepository);
    }

    let explicit_path = example_path
        .filter(|path| !path.is_empty())
        .map(normalize_repository_path)
        .transpose()?;

    if segments.len() == 2 {
        return Ok(GitHubRepositoryLocation::NeedsDefaultBranch {
            username: username.to_owned(),
            name: name.to_owned(),
            file_path: explicit_path.unwrap_or_default(),
        });
    }
    if segments.get(2) != Some(&"tree") || segments.len() < 4 {
        return Err(GitHubRepositoryLocationError::UnsupportedPath);
    }

    let tail = &segments[3..];
    let (branch_segments, file_path) = if let Some(explicit_path) = explicit_path {
        let explicit_segments = explicit_path.split('/').collect::<Vec<_>>();
        let branch_end = literal_suffix_start(tail, &explicit_segments).unwrap_or(tail.len());
        (&tail[..branch_end], explicit_path)
    } else {
        (&tail[..1], tail[1..].join("/"))
    };

    let branch = branch_segments.join("/");
    if !valid_git_reference(&branch) {
        return Err(GitHubRepositoryLocationError::InvalidReference);
    }

    Ok(GitHubRepositoryLocation::Resolved(RepoInfo {
        username: username.to_owned(),
        name: name.to_owned(),
        branch,
        file_path,
    }))
}
