mod create_install_policy;
mod default_example;
mod git_ignore;
mod git_init;
mod official_starter;
mod package_manager_prompt;
mod package_manager_transform;
mod readme_transform;
mod transform_pipeline;

pub use create_install_policy::{
    CreateInstallInput, CreateInstallOutcome, CreateInstallRequest, CreateInstaller,
    UnavailablePackageManagerWarning, apply_create_install_policy,
};
pub use default_example::{DEFAULT_EXAMPLES, is_default_example};
pub use git_ignore::{
    DEFAULT_IGNORE, GIT_IGNORE_TRANSFORM_NAME, GitIgnoreError, create_git_ignore,
};
pub use git_init::{
    GitCleanupError, GitDirectoryCleaner, INITIAL_COMMIT_MESSAGE, VcsInvocation, VcsProgram,
    VcsRunner, try_git_init_with,
};
pub use official_starter::{
    ExampleRepository, OFFICIAL_REPOSITORIES, OFFICIAL_STARTER_TRANSFORM_NAME,
    OfficialStarterError, OfficialStarterInput, OfficialStarterPackageJson,
    OfficialStarterResponse, OfficialStarterStore, is_official_starter, transform_official_starter,
};
pub use package_manager_prompt::{
    PACKAGE_MANAGER_PROMPT_ORDER, PackageManagerAvailability, PackageManagerPromptChoice,
    PackageManagerPromptError, PackageManagerSelector, resolve_package_manager_prompt,
};
pub use package_manager_transform::{
    PACKAGE_MANAGER_TRANSFORM_NAME, PackageManagerConversion, PackageManagerConverter,
    PackageManagerSelection, WorkspacePackageManager, transform_package_manager,
};
pub use readme_transform::*;
pub use transform_pipeline::{
    PipelineAbort, PipelineAbortReason, PipelineTransformResponse, TRANSFORM_PIPELINE,
    TransformExecutor, TransformFailure, TransformInvocationError, TransformKind,
    TransformPipelineReport, run_transform_pipeline,
};
