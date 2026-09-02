use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use create_turbo_rs::{
    GitCleanupError, GitDirectoryCleaner, INITIAL_COMMIT_MESSAGE, VcsInvocation, VcsProgram,
    VcsRunner, try_git_init_with,
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
fn root() -> PathBuf {
    PathBuf::from(r"C:\tmp\create-turbo-project")
}

#[cfg(not(windows))]
fn root() -> PathBuf {
    PathBuf::from("/tmp/create-turbo-project")
}

fn invocation(program: VcsProgram, arguments: &[&str], cwd: Option<&Path>) -> VcsInvocation {
    VcsInvocation {
        program,
        arguments: arguments.iter().map(|value| (*value).to_string()).collect(),
        cwd: cwd.map(Path::to_path_buf),
    }
}

#[test]
fn initial_commit_message_matches_the_typescript_source() {
    assert_eq!(INITIAL_COMMIT_MESSAGE, "Initial commit from create-turbo");
}

#[test]
fn returns_false_when_already_inside_git_repository() {
    let root = root();
    let mut runner = FakeRunner::with_results([true]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(
        runner.calls,
        [invocation(
            VcsProgram::Git,
            &["rev-parse", "--is-inside-work-tree"],
            Some(&root),
        )]
    );
    assert!(cleaner.roots.is_empty());
}

#[test]
fn returns_false_when_inside_mercurial_repository() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(
        runner.calls,
        [
            invocation(
                VcsProgram::Git,
                &["rev-parse", "--is-inside-work-tree"],
                Some(&root),
            ),
            invocation(VcsProgram::Mercurial, &["--cwd", ".", "root"], Some(&root),),
        ]
    );
    assert!(cleaner.roots.is_empty());
}

#[test]
fn returns_false_when_git_init_is_unavailable_or_fails() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, false, false]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(
        runner.calls,
        [
            invocation(
                VcsProgram::Git,
                &["rev-parse", "--is-inside-work-tree"],
                Some(&root),
            ),
            invocation(VcsProgram::Mercurial, &["--cwd", ".", "root"], Some(&root),),
            invocation(VcsProgram::Git, &["init"], Some(&root)),
        ]
    );
    assert!(cleaner.roots.is_empty());
}

#[test]
fn runs_the_exact_typescript_command_sequence_on_success() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, false, true, true, true, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(
        runner.calls,
        [
            invocation(
                VcsProgram::Git,
                &["rev-parse", "--is-inside-work-tree"],
                Some(&root),
            ),
            invocation(VcsProgram::Mercurial, &["--cwd", ".", "root"], Some(&root),),
            invocation(VcsProgram::Git, &["init"], Some(&root)),
            invocation(VcsProgram::Git, &["checkout", "-b", "main"], Some(&root),),
            invocation(VcsProgram::Git, &["add", "-A"], Some(&root)),
            invocation(
                VcsProgram::Git,
                &["commit", "-m", INITIAL_COMMIT_MESSAGE],
                Some(&root),
            ),
        ]
    );
    assert!(cleaner.roots.is_empty());
}

#[test]
fn checkout_failure_triggers_cleanup_and_returns_false() {
    assert_failure_after_init([false, false, true, false], 4);
}

#[test]
fn add_failure_triggers_cleanup_and_returns_false() {
    assert_failure_after_init([false, false, true, true, false], 5);
}

#[test]
fn commit_failure_triggers_cleanup_and_returns_false() {
    assert_failure_after_init([false, false, true, true, true, false], 6);
}

fn assert_failure_after_init<const N: usize>(results: [bool; N], expected_calls: usize) {
    let root = root();
    let mut runner = FakeRunner::with_results(results);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls.len(), expected_calls);
    assert_eq!(cleaner.roots, [root]);
}

#[test]
fn cleanup_failure_is_swallowed_like_the_typescript_implementation() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, false, true, false]);
    let mut cleaner = FakeCleaner {
        roots: Vec::new(),
        fail: true,
    };

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(cleaner.roots, [root]);
}
