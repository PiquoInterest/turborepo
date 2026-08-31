use std::{error::Error, fs, path::Path};

use turbo_utils_rs::{search_up, validate_directory};

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn search_target_cannot_escape_each_ancestor() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let nested = directory.path().join("a/b");
    fs::create_dir_all(&nested)?;
    assert!(search_up(Path::new("../secret"), &nested, None).is_err());
    assert!(search_up(Path::new("/tmp/secret"), &nested, None).is_err());
    Ok(())
}

#[test]
fn relative_search_start_is_rejected() {
    assert!(search_up(Path::new("package.json"), Path::new("relative/path"), None).is_err());
}

#[test]
fn invalid_or_control_character_project_names_are_rejected() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    for name in ["", "   ", "bad name", "bad\nname", "bad/name/../?"] {
        assert!(
            !validate_directory(name, directory.path()).valid,
            "accepted {name:?}"
        );
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_project_root_is_rejected() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    write(&outside.path().join("LICENSE"), "MIT")?;
    let link = directory.path().join("project");
    symlink(outside.path(), &link)?;
    let result = validate_directory(link.to_string_lossy().as_ref(), directory.path());
    assert!(!result.valid);
    Ok(())
}

#[test]
fn metadata_uncertainty_does_not_validate_the_directory() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = directory.path().join("project");
    fs::create_dir_all(&project)?;
    write(&project.join("README.md"), "conflict")?;
    let result = validate_directory(project.to_string_lossy().as_ref(), directory.path());
    assert!(!result.valid);
    Ok(())
}
