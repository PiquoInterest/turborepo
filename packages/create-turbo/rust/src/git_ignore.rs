use std::{fmt, path::Path};

use crate::{TransformResponse, TransformStatus};

pub const GIT_IGNORE_TRANSFORM_NAME: &str = "git-ignore";
pub const DEFAULT_IGNORE: &str = r#"
# See https://help.github.com/articles/ignoring-files/ for more about ignoring files.

# dependencies
node_modules
.pnp
.pnp.js

# testing
coverage

# misc
.DS_Store
*.pem

# debug
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# turbo
.turbo

# vercel
.vercel
"#;

#[derive(Debug)]
pub enum GitIgnoreError {
    Write(std::io::Error),
    UnsafeRoot,
    UnsafeIgnore,
}

impl fmt::Display for GitIgnoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write(_) => formatter.write_str("Unable to write .gitignore"),
            Self::UnsafeRoot => formatter.write_str("project root is not a safe directory"),
            Self::UnsafeIgnore => formatter.write_str(".gitignore is not a safe path"),
        }
    }
}

impl std::error::Error for GitIgnoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write(error) => Some(error),
            Self::UnsafeRoot | Self::UnsafeIgnore => None,
        }
    }
}

pub fn create_git_ignore(_root: &Path) -> Result<TransformResponse, GitIgnoreError> {
    Ok(TransformResponse {
        result: TransformStatus::NotApplicable,
        name: GIT_IGNORE_TRANSFORM_NAME,
    })
}
