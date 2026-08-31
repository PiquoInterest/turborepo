use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use create_turbo_rs::{
    DEFAULT_IGNORE, GIT_IGNORE_TRANSFORM_NAME, TransformStatus, create_git_ignore,
};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(20_000);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Result<Self, std::io::Error> {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "create-turbo-git-ignore-{label}-{}-{sequence}",
            std::process::id()
        ));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn default_ignore_matches_the_typescript_constant() {
    assert_eq!(
        DEFAULT_IGNORE,
        concat!(
            "\n",
            "# See https://help.github.com/articles/ignoring-files/ for more about ignoring files.\n",
            "\n",
            "# dependencies\n",
            "node_modules\n",
            ".pnp\n",
            ".pnp.js\n",
            "\n",
            "# testing\n",
            "coverage\n",
            "\n",
            "# misc\n",
            ".DS_Store\n",
            "*.pem\n",
            "\n",
            "# debug\n",
            "npm-debug.log*\n",
            "yarn-debug.log*\n",
            "yarn-error.log*\n",
            "\n",
            "# turbo\n",
            ".turbo\n",
            "\n",
            "# vercel\n",
            ".vercel\n",
        )
    );
    assert!(DEFAULT_IGNORE.contains(".turbo"));
}

#[test]
fn creates_missing_gitignore_with_exact_content() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("create")?;

    let response = create_git_ignore(directory.path())?;

    assert_eq!(response.result, TransformStatus::Success);
    assert_eq!(response.name, GIT_IGNORE_TRANSFORM_NAME);
    assert_eq!(
        fs::read_to_string(directory.path().join(".gitignore"))?,
        DEFAULT_IGNORE
    );
    Ok(())
}

#[test]
fn existing_gitignore_is_not_applicable_and_is_unchanged() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::new("existing")?;
    let path = directory.path().join(".gitignore");
    fs::write(&path, "custom\n")?;

    let response = create_git_ignore(directory.path())?;

    assert_eq!(response.result, TransformStatus::NotApplicable);
    assert_eq!(response.name, GIT_IGNORE_TRANSFORM_NAME);
    assert_eq!(fs::read_to_string(path)?, "custom\n");
    Ok(())
}

#[test]
fn existing_directory_at_gitignore_path_is_not_applicable() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = TestDirectory::new("existing-directory")?;
    fs::create_dir(directory.path().join(".gitignore"))?;

    let response = create_git_ignore(directory.path())?;

    assert_eq!(response.result, TransformStatus::NotApplicable);
    assert_eq!(response.name, GIT_IGNORE_TRANSFORM_NAME);
    Ok(())
}

#[test]
fn missing_project_root_returns_the_public_write_error() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("missing-root-parent")?;
    let missing = directory.path().join("missing");

    let error = create_git_ignore(&missing).expect_err("missing root must fail");

    assert_eq!(error.to_string(), "Unable to write .gitignore");
    assert!(!missing.exists());
    Ok(())
}
