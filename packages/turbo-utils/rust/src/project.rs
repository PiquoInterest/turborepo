use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

use crate::{is_folder_empty, is_writeable};

pub const PROJECT_DOWNLOAD_ATTEMPTS: usize = 4;
const MAX_PACKAGE_JSON_BYTES: u64 = 1_024 * 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRepositoryUrl {
    raw: String,
    path: String,
}

impl GitHubRepositoryUrl {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoInfo {
    pub username: String,
    pub name: String,
    pub branch: String,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectOptions {
    pub app_path: PathBuf,
    pub example: String,
    pub is_default_example: bool,
    pub example_path: Option<String>,
    pub original_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectResult {
    pub cd_path: PathBuf,
    pub has_package_json: bool,
    pub available_scripts: Vec<String>,
    pub repo_info: Option<RepoInfo>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct ProjectSourceError {
    message: String,
}

impl ProjectSourceError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait ProjectSource: Sync {
    fn get_repo_info(
        &self,
        url: &GitHubRepositoryUrl,
        example_path: Option<&str>,
    ) -> Result<Option<RepoInfo>, ProjectSourceError>;

    fn has_repo(&self, repo_info: &RepoInfo) -> Result<bool, ProjectSourceError>;

    fn example_exists(&self, example: &str) -> Result<bool, ProjectSourceError>;

    fn download_example(&self, root: &Path, example: &str) -> Result<(), ProjectSourceError>;

    fn download_repo(&self, root: &Path, repo_info: &RepoInfo) -> Result<(), ProjectSourceError>;
}

#[derive(Debug, Error)]
pub enum CreateProjectError {
    #[error(
        "Invalid URL: \"{input}\". Only GitHub repositories are supported. Please use a GitHub \
         URL and try again."
    )]
    InvalidGitHubUrl { input: String },
    #[error(
        "Unable to fetch repository information from: \"{input}\". Please fix the URL and try \
         again."
    )]
    RepositoryInfoUnavailable { input: String },
    #[error(
        "Could not locate the repository for \"{input}\". Please check that the repository exists \
         and try again."
    )]
    RepositoryNotFound { input: String },
    #[error(
        "Could not locate an example named \"{example}\". It may be misspelled, unavailable, or \
         unreachable through the current network/proxy configuration."
    )]
    ExampleNotFound { example: String },
    #[error("Invalid example name: {example}")]
    InvalidExampleName { example: String },
    #[error("Unsafe repository subpath: {path}")]
    UnsafeRepositoryPath { path: String },
    #[error("The application parent path {path:?} is not writable.")]
    ParentNotWritable { path: PathBuf },
    #[error("Unsafe project path {path:?}: {reason}")]
    UnsafeProjectPath { path: PathBuf, reason: String },
    #[error("Unable to create project directory {path:?}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Unable to inspect project directory {path:?}: {source}")]
    InspectDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{path:?} has {count} conflicting {noun} - please try a different location")]
    ConflictingFiles {
        path: PathBuf,
        count: usize,
        noun: &'static str,
    },
    #[error("Download failed: {0}")]
    Download(#[from] ProjectSourceError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedSource {
    DefaultExample,
    Repository(RepoInfo),
}

fn scheme_end(input: &str) -> Option<usize> {
    let end = input.find(':')?;
    let mut characters = input.get(..end)?.chars();
    if !characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
    {
        return None;
    }
    characters
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.'))
        .then_some(end)
}

fn invalid_github_url(input: &str) -> CreateProjectError {
    CreateProjectError::InvalidGitHubUrl {
        input: input.to_owned(),
    }
}

fn parse_repository_url(input: &str) -> Result<Option<GitHubRepositoryUrl>, CreateProjectError> {
    let Some(scheme_end) = scheme_end(input) else {
        return Ok(None);
    };
    if input
        .chars()
        .any(|character| character.is_ascii_control() || character.is_ascii_whitespace())
    {
        return Err(invalid_github_url(input));
    }

    let scheme = input
        .get(..scheme_end)
        .ok_or_else(|| invalid_github_url(input))?;
    if !scheme.eq_ignore_ascii_case("https") {
        return Err(invalid_github_url(input));
    }
    let after_scheme = input
        .get(scheme_end + 1..)
        .and_then(|value| value.strip_prefix("//"))
        .ok_or_else(|| invalid_github_url(input))?;
    let authority_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let authority = after_scheme
        .get(..authority_end)
        .ok_or_else(|| invalid_github_url(input))?;
    if !authority.eq_ignore_ascii_case("github.com") {
        return Err(invalid_github_url(input));
    }

    let suffix = after_scheme
        .get(authority_end..)
        .ok_or_else(|| invalid_github_url(input))?;
    let path_end = suffix.find(['?', '#']).unwrap_or(suffix.len());
    let path = suffix.get(..path_end).unwrap_or_default();
    let path = if path.is_empty() { "/" } else { path };
    if !path.starts_with('/') {
        return Err(invalid_github_url(input));
    }

    Ok(Some(GitHubRepositoryUrl {
        raw: input.to_owned(),
        path: path.to_owned(),
    }))
}

#[must_use]
pub fn is_valid_github_repo_url(input: &str) -> bool {
    matches!(parse_repository_url(input), Ok(Some(_)))
}

fn normalize_repository_path(path: &str) -> Result<String, CreateProjectError> {
    let normalized = path.strip_prefix('/').unwrap_or(path);
    if normalized.is_empty() {
        return Ok(String::new());
    }
    let safe = !normalized.contains(['\\', '%', '?', '#'])
        && !normalized.chars().any(char::is_control)
        && normalized
            .split('/')
            .all(|segment| !segment.is_empty() && !matches!(segment, "." | ".."));
    if !safe {
        return Err(CreateProjectError::UnsafeRepositoryPath {
            path: path.to_owned(),
        });
    }
    Ok(normalized.to_owned())
}

fn is_valid_example_name(example: &str) -> bool {
    !example.is_empty()
        && example
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn select_source<S: ProjectSource + ?Sized>(
    options: &CreateProjectOptions,
    source: &S,
) -> Result<SelectedSource, CreateProjectError> {
    if options.is_default_example {
        return Ok(SelectedSource::DefaultExample);
    }

    if let Some(url) = parse_repository_url(&options.example)? {
        let example_path = options
            .example_path
            .as_deref()
            .map(normalize_repository_path)
            .transpose()?;
        let repo_info = source
            .get_repo_info(&url, example_path.as_deref())?
            .ok_or_else(|| CreateProjectError::RepositoryInfoUnavailable {
                input: options.example.clone(),
            })?;
        if !source.has_repo(&repo_info)? {
            return Err(CreateProjectError::RepositoryNotFound {
                input: options.example.clone(),
            });
        }
        return Ok(SelectedSource::Repository(repo_info));
    }

    if !is_valid_example_name(&options.example) {
        return Err(CreateProjectError::InvalidExampleName {
            example: options.example.clone(),
        });
    }
    if !source.example_exists(&options.example)? {
        return Err(CreateProjectError::ExampleNotFound {
            example: options.example.clone(),
        });
    }

    Ok(SelectedSource::Repository(RepoInfo {
        username: "vercel".to_owned(),
        name: "turborepo".to_owned(),
        branch: "main".to_owned(),
        file_path: format!("examples/{}", options.example),
    }))
}

fn lexical_resolve(path: &Path, base: &Path) -> Result<PathBuf, CreateProjectError> {
    if !base.is_absolute() {
        return Err(CreateProjectError::UnsafeProjectPath {
            path: path.to_path_buf(),
            reason: "the original directory must be absolute".to_owned(),
        });
    }
    if path.to_string_lossy().contains('\0') {
        return Err(CreateProjectError::UnsafeProjectPath {
            path: path.to_path_buf(),
            reason: "NUL bytes are not allowed".to_owned(),
        });
    }

    let combined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in combined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _removed = resolved.pop();
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    if !resolved.is_absolute() {
        return Err(CreateProjectError::UnsafeProjectPath {
            path: resolved,
            reason: "the resolved project path must be absolute".to_owned(),
        });
    }
    Ok(resolved)
}

fn unsafe_project_path(path: &Path, reason: impl Into<String>) -> CreateProjectError {
    CreateProjectError::UnsafeProjectPath {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

fn verify_project_root(root: &Path) -> Result<(), CreateProjectError> {
    let metadata =
        fs::symlink_metadata(root).map_err(|source| CreateProjectError::InspectDirectory {
            path: root.to_path_buf(),
            source,
        })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unsafe_project_path(
            root,
            "the target changed or is no longer a real directory",
        ));
    }
    Ok(())
}

fn prepare_project_root(root: &Path) -> Result<(), CreateProjectError> {
    let parent = root
        .parent()
        .ok_or_else(|| unsafe_project_path(root, "the project path has no parent directory"))?;
    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|_| CreateProjectError::ParentNotWritable {
            path: parent.to_path_buf(),
        })?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(unsafe_project_path(
            parent,
            "the immediate parent must be a real directory, not a symlink",
        ));
    }
    if !is_writeable(parent) {
        return Err(CreateProjectError::ParentNotWritable {
            path: parent.to_path_buf(),
        });
    }

    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(unsafe_project_path(
                root,
                "the target must be a real directory, not a file or symlink",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(root).map_err(|source| CreateProjectError::CreateDirectory {
                path: root.to_path_buf(),
                source,
            })?;
        }
        Err(source) => {
            return Err(CreateProjectError::CreateDirectory {
                path: root.to_path_buf(),
                source,
            });
        }
    }

    verify_project_root(root)?;
    let empty = is_folder_empty(root).map_err(|source| CreateProjectError::InspectDirectory {
        path: root.to_path_buf(),
        source,
    })?;
    if !empty.is_empty {
        let count = empty.conflicts.len();
        return Err(CreateProjectError::ConflictingFiles {
            path: root.to_path_buf(),
            count,
            noun: if count == 1 { "file" } else { "files" },
        });
    }
    Ok(())
}

fn download_with_retry<S: ProjectSource + ?Sized>(
    source: &S,
    selected: &SelectedSource,
    root: &Path,
) -> Result<(), CreateProjectError> {
    let mut last_error = None;
    for _attempt in 0..PROJECT_DOWNLOAD_ATTEMPTS {
        verify_project_root(root)?;
        let result = match selected {
            SelectedSource::DefaultExample => source.download_example(root, "basic"),
            SelectedSource::Repository(repo_info) => source.download_repo(root, repo_info),
        };
        verify_project_root(root)?;
        match result {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }

    match last_error {
        Some(error) => Err(CreateProjectError::Download(error)),
        None => Err(CreateProjectError::Download(ProjectSourceError::new(
            "download attempt limit was zero",
        ))),
    }
}

fn javascript_array_index(key: &str) -> Option<u32> {
    if key.is_empty()
        || (key.len() > 1 && key.starts_with('0'))
        || !key.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let value = key.parse::<u64>().ok()?;
    if value > u64::from(u32::MAX) - 1 || value.to_string() != key {
        return None;
    }
    u32::try_from(value).ok()
}

fn javascript_object_keys(object: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let mut array_indices = Vec::new();
    let mut other_keys = Vec::new();
    for key in object.keys() {
        if let Some(index) = javascript_array_index(key) {
            array_indices.push((index, key.clone()));
        } else {
            other_keys.push(key.clone());
        }
    }
    array_indices.sort_unstable_by_key(|(index, _key)| *index);
    array_indices
        .into_iter()
        .map(|(_index, key)| key)
        .chain(other_keys)
        .collect()
}

fn open_package_json(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    }
    options.open(path)
}

fn inspect_package_json(root: &Path) -> (bool, Vec<String>) {
    let path = root.join("package.json");
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return (false, Vec::new());
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_PACKAGE_JSON_BYTES
    {
        return (true, Vec::new());
    }

    let Ok(file) = open_package_json(&path) else {
        return (true, Vec::new());
    };
    let Ok(opened_metadata) = file.metadata() else {
        return (true, Vec::new());
    };
    if !opened_metadata.is_file() || opened_metadata.len() > MAX_PACKAGE_JSON_BYTES {
        return (true, Vec::new());
    }

    let mut contents = String::new();
    if file
        .take(MAX_PACKAGE_JSON_BYTES + 1)
        .read_to_string(&mut contents)
        .is_err()
        || contents.len() as u64 > MAX_PACKAGE_JSON_BYTES
    {
        return (true, Vec::new());
    }
    let Ok(package_json) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return (true, Vec::new());
    };
    let scripts = package_json
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .map(javascript_object_keys)
        .unwrap_or_default();
    (true, scripts)
}

fn cd_path(options: &CreateProjectOptions, root: &Path) -> PathBuf {
    let Some(app_name) = root.file_name() else {
        return options.app_path.clone();
    };
    if options.original_directory.join(app_name) == options.app_path {
        PathBuf::from(app_name)
    } else {
        options.app_path.clone()
    }
}

pub fn create_project<S: ProjectSource + ?Sized>(
    options: &CreateProjectOptions,
    source: &S,
) -> Result<CreateProjectResult, CreateProjectError> {
    let selected = select_source(options, source)?;
    let root = lexical_resolve(&options.app_path, &options.original_directory)?;
    prepare_project_root(&root)?;
    download_with_retry(source, &selected, &root)?;
    let (has_package_json, available_scripts) = inspect_package_json(&root);
    let repo_info = match selected {
        SelectedSource::DefaultExample => None,
        SelectedSource::Repository(repo_info) => Some(repo_info),
    };

    Ok(CreateProjectResult {
        cd_path: cd_path(options, &root),
        has_package_json,
        available_scripts,
        repo_info,
    })
}
