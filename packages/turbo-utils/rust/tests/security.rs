use std::{error::Error, fs, path::Path};

use turbo_utils_rs::{is_folder_empty, search_up, validate_directory};

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

#[test]
fn option_like_project_names_are_rejected_before_filesystem_inspection()
-> Result<(), Box<dyn Error>> {
    let current_directory = tempfile::tempdir()?;
    for (directory, project_name) in [("-rf", "-rf"), ("--help", "--help"), ("nested/-C", "-C")] {
        let result = validate_directory(directory, current_directory.path());

        assert!(!result.valid, "accepted {directory:?}");
        assert_eq!(result.project_name, project_name);
        assert!(
            result
                .error
                .is_some_and(|error| error.contains("is not a valid directory"))
        );
    }
    Ok(())
}

#[test]
fn ordinary_hyphenated_project_name_remains_valid() -> Result<(), Box<dyn Error>> {
    let current_directory = tempfile::tempdir()?;
    let result = validate_directory("app-name", current_directory.path());

    assert!(result.valid);
    assert_eq!(result.project_name, "app-name");
    assert_eq!(result.root, current_directory.path().join("app-name"));
    Ok(())
}

#[test]
fn folder_scan_is_bounded_before_collecting_untrusted_entries() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    for index in 0..257 {
        write(
            &directory.path().join(format!("conflict-{index:03}")),
            "conflict",
        )?;
    }

    let Err(error) = is_folder_empty(directory.path()) else {
        panic!("an oversized directory scan must fail closed");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
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

#[cfg(unix)]
#[test]
fn symlinked_ancestor_is_rejected_before_directory_enumeration() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    fs::create_dir_all(outside.path().join("project"))?;
    symlink(outside.path(), directory.path().join("redirect"))?;

    let result = validate_directory("redirect/project", directory.path());
    assert!(
        !result.valid,
        "accepted a directory through a symlinked ancestor"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn allowlisted_symlink_is_never_treated_as_an_empty_directory() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    symlink(outside.path(), directory.path().join(".git"))?;

    let result = validate_directory(
        directory.path().to_string_lossy().as_ref(),
        directory.path(),
    );
    assert!(
        !result.valid,
        "accepted an allowlisted name backed by a symlink"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn non_utf8_iml_name_is_not_silently_allowlisted() -> Result<(), Box<dyn Error>> {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let directory = tempfile::tempdir()?;
    let mut bytes = vec![0xff];
    bytes.extend_from_slice(b".iml");
    write(
        &directory.path().join(OsString::from_vec(bytes)),
        "conflict",
    )?;

    assert!(
        is_folder_empty(directory.path()).is_err(),
        "lossy filename conversion hid a non-UTF-8 conflict"
    );
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
