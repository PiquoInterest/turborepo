use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use create_turbo_rs::{
    GitCleanupError, GitDirectoryCleaner, VcsInvocation, VcsRunner, try_git_init_with,
};

#[derive(Default)]
struct FakeRunner {
    results: VecDeque<bool>,
    calls: Vec<VcsInvocation>,
}

impl FakeRunner {
    fn with_results(results: impl IntoIterator<Item = bool>) -> Self {
        Self {
            results: results.into_iter().collect(),
            calls: Vec::new(),
        }
    }
}

impl VcsRunner for FakeRunner {
    fn run(&mut self, invocation: &VcsInvocation) -> bool {
        self.calls.push(invocation.clone());
        self.results.pop_front().unwrap_or(false)
    }
}

#[derive(Default)]
struct FakeCleaner {
    roots: Vec<PathBuf>,
    fail: bool,
}

impl GitDirectoryCleaner for FakeCleaner {
    fn remove_git_directory(&mut self, root: &Path) -> Result<(), GitCleanupError> {
        self.roots.push(root.to_path_buf());
        if self.fail {
            Err(GitCleanupError)
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
fn absolute(path: &str) -> PathBuf {
    PathBuf::from(format!(r"C:\tmp\{path}"))
}

#[cfg(not(windows))]
fn absolute(path: &str) -> PathBuf {
    PathBuf::from(format!("/tmp/{path}"))
}

#[test]
fn rejects_relative_roots_before_any_subprocess() {
    assert_rejected_before_invocation(PathBuf::from("relative/create-turbo-project"));
}

#[test]
fn rejects_filesystem_roots_before_any_subprocess_or_cleanup() {
    #[cfg(windows)]
    let root = PathBuf::from(r"C:\");
    #[cfg(not(windows))]
    let root = PathBuf::from("/");

    assert_rejected_before_invocation(root);
}

#[test]
fn rejects_parent_components_before_any_subprocess() {
    assert_rejected_before_invocation(absolute("safe/../project"));
}

#[test]
fn rejects_control_and_windows_invalid_filename_characters() {
    for root in [absolute("bad?project"), absolute("bad\nproject")] {
        assert_rejected_before_invocation(root);
    }
}

fn assert_rejected_before_invocation(root: PathBuf) {
    let mut runner = FakeRunner::default();
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert!(runner.calls.is_empty());
    assert!(cleaner.roots.is_empty());
}

#[test]
fn shell_metacharacters_are_not_treated_as_injection_without_a_shell() {
    let root = absolute("project-$#;!");
    let mut runner = FakeRunner::with_results([false, false, true, true, true, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls.len(), 6);
    assert!(runner.calls.iter().all(|call| call.cwd.as_deref() == Some(&root)));
    assert!(cleaner.roots.is_empty());
}

#[test]
fn init_failure_does_not_delete_an_unowned_or_ambiguous_git_directory() {
    let root = absolute("create-turbo-project");
    let mut runner = FakeRunner::with_results([false, false, false]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert!(cleaner.roots.is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_roots_do_not_require_lossy_argument_conversion() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let root = PathBuf::from(OsString::from_vec(
        b"/tmp/create-turbo-project-\xff".to_vec(),
    ));
    let mut runner = FakeRunner::with_results([false, false, true, true, true, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls.len(), 6);
    assert!(runner.calls.iter().all(|call| call.cwd.as_deref() == Some(&root)));
    assert!(cleaner.roots.is_empty());
}
