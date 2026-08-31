use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use create_turbo_rs::{DEFAULT_IGNORE, GitIgnoreError, create_git_ignore};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(30_000);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Result<Self, std::io::Error> {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "create-turbo-git-ignore-security-{label}-{}-{sequence}",
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
fn regular_existing_file_is_never_overwritten() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("no-overwrite")?;
    let path = directory.path().join(".gitignore");
    let original = "customer-owned\n";
    fs::write(&path, original)?;

    let _ = create_git_ignore(directory.path())?;

    assert_eq!(fs::read_to_string(path)?, original);
    Ok(())
}

#[cfg(unix)]
#[test]
fn broken_symlink_is_rejected_without_creating_its_external_target(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new("broken-link")?;
    let outside = TestDirectory::new("broken-link-outside")?;
    let target = outside.path().join("created-outside");
    symlink(&target, directory.path().join(".gitignore"))?;

    assert!(matches!(
        create_git_ignore(directory.path()),
        Err(GitIgnoreError::UnsafeIgnore)
    ));
    assert!(!target.exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn existing_symlink_is_rejected_without_modifying_its_target(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new("existing-link")?;
    let outside = TestDirectory::new("existing-link-outside")?;
    let target = outside.path().join("target");
    fs::write(&target, "outside\n")?;
    symlink(&target, directory.path().join(".gitignore"))?;

    assert!(matches!(
        create_git_ignore(directory.path()),
        Err(GitIgnoreError::UnsafeIgnore)
    ));
    assert_eq!(fs::read_to_string(target)?, "outside\n");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_project_root_is_rejected_without_writing_through_it(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let parent = TestDirectory::new("root-link-parent")?;
    let outside = TestDirectory::new("root-link-outside")?;
    let link = parent.path().join("project");
    symlink(outside.path(), &link)?;

    assert!(matches!(
        create_git_ignore(&link),
        Err(GitIgnoreError::UnsafeRoot)
    ));
    assert!(!outside.path().join(".gitignore").exists());
    Ok(())
}

#[test]
fn successful_creation_has_only_the_expected_file() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("single-file")?;

    let _ = create_git_ignore(directory.path())?;

    let entries = fs::read_dir(directory.path())?
        .map(|entry| entry.map(|value| value.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(entries, [std::ffi::OsString::from(".gitignore")]);
    assert_eq!(
        fs::read_to_string(directory.path().join(".gitignore"))?,
        DEFAULT_IGNORE
    );
    Ok(())
}
