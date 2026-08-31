use std::{collections::VecDeque, path::{Path, PathBuf}};

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
        if self.fail { Err(GitCleanupError) } else { Ok(()) }
    }
}

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
fn returns_false_when_already_inside_git_repository() {
    let root = root();
    let mut runner = FakeRunner::with_results([true]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls, [invocation(VcsProgram::Git, &["rev-parse", "--is-inside-work-tree"], Some(&root))]);
    assert!(cleaner.roots.is_empty());
}

#[test]
fn returns_false_when_inside_mercurial_repository() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls, [
        invocation(VcsProgram::Git, &["rev-parse", "--is-inside-work-tree"], Some(&root)),
        invocation(VcsProgram::Mercurial, &["--cwd", root.to_str().unwrap(), "root"], None),
    ]);
    assert!(cleaner.roots.is_empty());
}

#[test]
fn returns_false_when_git_is_unavailable() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, false, false]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls.last(), Some(&invocation(VcsProgram::Git, &["--version"], None)));
    assert!(cleaner.roots.is_empty());
}

#[test]
fn rejects_invalid_path_characters_before_initialization() {
    let root = PathBuf::from("/tmp/bad?project");
    let mut runner = FakeRunner::with_results([false, false, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls.len(), 3);
    assert!(cleaner.roots.is_empty());
}

#[test]
fn runs_the_exact_typescript_command_sequence_on_success() {
    let root = root();
    let mut runner = FakeRunner::with_results([false, false, true, true, true, true, true]);
    let mut cleaner = FakeCleaner::default();

    assert!(try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(runner.calls, [
        invocation(VcsProgram::Git, &["rev-parse", "--is-inside-work-tree"], Some(&root)),
        invocation(VcsProgram::Mercurial, &["--cwd", root.to_str().unwrap(), "root"], None),
        invocation(VcsProgram::Git, &["--version"], None),
        invocation(VcsProgram::Git, &["init"], Some(&root)),
        invocation(VcsProgram::Git, &["checkout", "-b", "main"], Some(&root)),
        invocation(VcsProgram::Git, &["add", "-A"], Some(&root)),
        invocation(VcsProgram::Git, &["commit", "-m", INITIAL_COMMIT_MESSAGE], Some(&root)),
    ]);
    assert!(cleaner.roots.is_empty());
}

#[test]
fn init_failure_triggers_cleanup_and_returns_false() {
    assert_failure_at([false, false, true, false], 4);
}

#[test]
fn checkout_failure_triggers_cleanup_and_returns_false() {
    assert_failure_at([false, false, true, true, false], 5);
}

#[test]
fn add_failure_triggers_cleanup_and_returns_false() {
    assert_failure_at([false, false, true, true, true, false], 6);
}

#[test]
fn commit_failure_triggers_cleanup_and_returns_false() {
    assert_failure_at([false, false, true, true, true, true, false], 7);
}

fn assert_failure_at<const N: usize>(results: [bool; N], expected_calls: usize) {
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
    let mut runner = FakeRunner::with_results([false, false, true, true, false]);
    let mut cleaner = FakeCleaner { roots: Vec::new(), fail: true };

    assert!(!try_git_init_with(&root, &mut runner, &mut cleaner));
    assert_eq!(cleaner.roots, [root]);
}
