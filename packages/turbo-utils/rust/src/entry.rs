#[path = "lib.rs"]
mod legacy;

mod archive;
mod config;
mod json5;
mod managers;
mod network;
mod notify;
mod project;
mod repository;
mod root;

pub use archive::{
    ARCHIVE_MAX_PATH_CHARS, ARCHIVE_MAX_PATH_COMPONENTS, is_archive_link_entry,
    is_archive_path_safe,
};
pub use config::{
    ConfigOptions, TurboConfig, TurboConfigError, TurboConfigPathResolution, WorkspaceConfig,
    clear_config_caches, for_each_task_def, get_turbo_configs, get_workspace_configs,
    resolve_turbo_config_path,
};
pub use json5::{Json5Error, parse_json5};
pub use legacy::*;
pub use managers::{
    CommandRequest, PACKAGE_MANAGER_EXEC_TIMEOUT, PACKAGE_MANAGER_MAX_OUTPUT_BYTES, PackageManager,
    PackageManagerCommandRunner, PackageManagerValues, SystemPackageManagerCommandRunner,
    get_available_package_managers, get_available_package_managers_with,
    get_package_managers_bin_paths, get_package_managers_bin_paths_with,
};
pub use network::{
    GITHUB_TOKEN_MAX_CHARS, NetworkEnvironment, NetworkPolicyError, PROXY_URL_MAX_CHARS,
    github_authorization_header, proxy_for_url,
};
pub use notify::{
    ExitCode, NOTIFY_MAX_UNTRUSTED_CHARS, NotifyUpdateOutcome, PackageInfo,
    PreparedUpdateNotification, UpdateCheckError, UpdateChecker, UpdateInfo, UpgradeCommand,
    UpgradeCommandError, UpgradeCommandProvider,
};
pub use project::{
    CreateProjectError, CreateProjectOptions, CreateProjectResult, GitHubRepositoryUrl,
    PROJECT_DOWNLOAD_ATTEMPTS, ProjectSource, ProjectSourceError, RepoInfo, create_project,
    is_valid_github_repo_url,
};
pub use repository::{
    GIT_REFERENCE_MAX_CHARS, GITHUB_REPOSITORY_URL_MAX_CHARS, GitHubRepositoryLocation,
    GitHubRepositoryLocationError, parse_github_repository_location,
};
pub use root::{TurboRootOptions, clear_turbo_root_cache, get_turbo_root};
