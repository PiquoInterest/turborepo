mod default_example;
mod git_ignore;
mod git_init;
mod readme_transform;

pub use default_example::{DEFAULT_EXAMPLES, is_default_example};
pub use git_ignore::{DEFAULT_IGNORE, GIT_IGNORE_TRANSFORM_NAME, GitIgnoreError, create_git_ignore};
pub use git_init::{
    GitCleanupError, GitDirectoryCleaner, INITIAL_COMMIT_MESSAGE, VcsInvocation, VcsProgram,
    VcsRunner, try_git_init_with,
};
pub use readme_transform::*;
