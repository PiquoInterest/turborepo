use std::path::{Path, PathBuf};

pub const INITIAL_COMMIT_MESSAGE: &str = "Initial commit from Create Turborepo";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsProgram {
    Git,
    Mercurial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsInvocation {
    pub program: VcsProgram,
    pub arguments: Vec<String>,
    pub cwd: Option<PathBuf>,
}

pub trait VcsRunner {
    fn run(&mut self, invocation: &VcsInvocation) -> bool;
}

pub trait GitDirectoryCleaner {
    fn remove_git_directory(&mut self, root: &Path) -> Result<(), GitCleanupError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitCleanupError;

pub fn try_git_init_with<R: VcsRunner, C: GitDirectoryCleaner>(
    _root: &Path,
    _runner: &mut R,
    _cleaner: &mut C,
) -> bool {
    false
}
