use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

pub const INITIAL_COMMIT_MESSAGE: &str = "Initial commit from create-turbo";

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
    root: &Path,
    runner: &mut R,
    cleaner: &mut C,
) -> bool {
    if !root_is_safe(root) {
        return false;
    }

    if runner.run(&invocation(
        VcsProgram::Git,
        &["rev-parse", "--is-inside-work-tree"],
        Some(root),
    )) {
        return false;
    }

    if runner.run(&invocation(
        VcsProgram::Mercurial,
        &["--cwd", ".", "root"],
        Some(root),
    )) {
        return false;
    }

    if !runner.run(&invocation(VcsProgram::Git, &["init"], Some(root))) {
        return false;
    }

    for command in [
        invocation(
            VcsProgram::Git,
            &["checkout", "-b", "main"],
            Some(root),
        ),
        invocation(VcsProgram::Git, &["add", "-A"], Some(root)),
        invocation(
            VcsProgram::Git,
            &["commit", "-m", INITIAL_COMMIT_MESSAGE],
            Some(root),
        ),
    ] {
        if !runner.run(&command) {
            let _ = cleaner.remove_git_directory(root);
            return false;
        }
    }

    true
}

fn invocation(program: VcsProgram, arguments: &[&str], cwd: Option<&Path>) -> VcsInvocation {
    VcsInvocation {
        program,
        arguments: arguments
            .iter()
            .map(|argument| (*argument).to_string())
            .collect(),
        cwd: cwd.map(Path::to_path_buf),
    }
}

fn root_is_safe(root: &Path) -> bool {
    if !root.is_absolute() || root.file_name().is_none() {
        return false;
    }

    for component in root.components() {
        match component {
            Component::CurDir | Component::ParentDir => return false,
            Component::Normal(value) if component_has_unsafe_characters(value) => return false,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {}
        }
    }

    true
}

#[cfg(unix)]
fn component_has_unsafe_characters(value: &OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt;

    value.as_bytes().iter().copied().any(|byte| {
        byte < b' ' || matches!(byte, b'"' | b'*' | b'<' | b'>' | b'?' | b'|')
    })
}

#[cfg(windows)]
fn component_has_unsafe_characters(value: &OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt;

    value.encode_wide().any(|unit| {
        unit < 0x20 || matches!(unit, 0x22 | 0x2a | 0x3a | 0x3c | 0x3e | 0x3f | 0x7c)
    })
}

#[cfg(not(any(unix, windows)))]
fn component_has_unsafe_characters(value: &OsStr) -> bool {
    value.to_string_lossy().chars().any(|character| {
        character.is_control() || matches!(character, '"' | '*' | '<' | '>' | '?' | '|')
    })
}
