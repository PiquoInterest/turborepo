use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use create_turbo_rs::{
    MAX_README_BYTES, PackageManager, TransformError, replace_package_manager_references,
    transform_readme,
};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(10_000);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Result<Self, std::io::Error> {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "create-turbo-rs-security-{label}-{}-{sequence}",
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
fn rejects_oversized_in_memory_markdown() {
    let input = "a".repeat(MAX_README_BYTES + 1);
    assert!(matches!(
        replace_package_manager_references(PackageManager::Npm, &input),
        Err(TransformError::ReadmeTooLarge)
    ));
}

#[test]
fn rejects_oversized_readme_without_modifying_it() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("oversized")?;
    let readme = directory.path().join("README.md");
    let original = vec![b'a'; MAX_README_BYTES + 1];
    fs::write(&readme, &original)?;

    assert!(matches!(
        transform_readme(directory.path(), Some(PackageManager::Npm)),
        Err(TransformError::ReadmeTooLarge)
    ));
    assert_eq!(fs::read(readme)?, original);
    Ok(())
}

#[test]
fn rejects_invalid_utf8_without_modifying_it() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("invalid-utf8")?;
    let readme = directory.path().join("README.md");
    let original = [0xff, 0xfe, b'`', b'n', b'p', b'm', b'`'];
    fs::write(&readme, original)?;

    assert!(matches!(
        transform_readme(directory.path(), Some(PackageManager::Npm)),
        Err(TransformError::InvalidUtf8)
    ));
    assert_eq!(fs::read(readme)?, original);
    Ok(())
}

#[test]
fn rejects_non_regular_readme_paths() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("readme-directory")?;
    fs::create_dir(directory.path().join("README.md"))?;
    assert!(matches!(
        transform_readme(directory.path(), Some(PackageManager::Npm)),
        Err(TransformError::UnsafeReadme)
    ));
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_readme_without_touching_target() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new("readme-symlink")?;
    let outside = TestDirectory::new("readme-symlink-target")?;
    let target = outside.path().join("target.md");
    fs::write(&target, "Run `pnpm install`.")?;
    symlink(&target, directory.path().join("README.md"))?;

    assert!(matches!(
        transform_readme(directory.path(), Some(PackageManager::Npm)),
        Err(TransformError::UnsafeReadme)
    ));
    assert_eq!(fs::read_to_string(target)?, "Run `pnpm install`.");
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_project_root() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let parent = TestDirectory::new("root-symlink-parent")?;
    let real = TestDirectory::new("root-symlink-real")?;
    fs::write(real.path().join("README.md"), "Run `pnpm install`.")?;
    let link = parent.path().join("project");
    symlink(real.path(), &link)?;

    assert!(matches!(
        transform_readme(&link, Some(PackageManager::Npm)),
        Err(TransformError::UnsafeRoot)
    ));
    assert_eq!(
        fs::read_to_string(real.path().join("README.md"))?,
        "Run `pnpm install`."
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn preserves_existing_readme_permissions() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let directory = TestDirectory::new("permissions")?;
    let readme = directory.path().join("README.md");
    fs::write(&readme, "Run `pnpm install`.")?;
    fs::set_permissions(&readme, fs::Permissions::from_mode(0o640))?;

    transform_readme(directory.path(), Some(PackageManager::Npm))?;

    assert_eq!(fs::read_to_string(&readme)?, "Run `npm install`.");
    assert_eq!(fs::metadata(readme)?.permissions().mode() & 0o777, 0o640);
    Ok(())
}

#[test]
fn successful_write_leaves_no_temporary_files() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("temp-cleanup")?;
    let readme = directory.path().join("README.md");
    fs::write(&readme, "Run `pnpm install`.")?;

    transform_readme(directory.path(), Some(PackageManager::Npm))?;

    let entries = fs::read_dir(directory.path())?
        .map(|entry| entry.map(|value| value.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(entries, vec![std::ffi::OsString::from("README.md")]);
    assert_eq!(fs::read_to_string(readme)?, "Run `npm install`.");
    Ok(())
}

#[test]
fn unmatched_fence_is_bounded_and_left_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let input = format!("```{}", "pnpm ".repeat(100_000));
    assert_eq!(
        replace_package_manager_references(PackageManager::Npm, &input)?,
        input
    );
    Ok(())
}
