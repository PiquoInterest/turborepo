use std::{error::Error, fs, path::Path};

use pretty_assertions::assert_eq;
use turbo_utils_rs::{
    CaseStyle, convert_case, is_folder_empty, is_writeable, search_up, validate_directory,
};

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn camel_case_matches_existing_table() -> Result<(), Box<dyn Error>> {
    for (input, expected) in [
        ("hello_world", "helloWorld"),
        ("hello-world", "helloWorld"),
        ("helloWorld", "helloWorld"),
        ("helloworld", "helloworld"),
    ] {
        assert_eq!(convert_case(input, CaseStyle::Camel)?, expected);
    }
    Ok(())
}

#[test]
fn unimplemented_case_styles_remain_errors() {
    for style in [CaseStyle::Pascal, CaseStyle::Kebab, CaseStyle::Snake] {
        assert!(convert_case("hello-world", style).is_err());
    }
}

#[test]
fn search_up_finds_current_and_parent_directories() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let nested = directory.path().join("project/src/components");
    fs::create_dir_all(&nested)?;
    write(&nested.join("package.json"), "{}")?;
    assert_eq!(
        search_up(Path::new("package.json"), &nested, None)?,
        Some(nested.clone())
    );

    write(&directory.path().join("project/turbo.json"), "{}")?;
    assert_eq!(
        search_up(Path::new("turbo.json"), &nested, None)?,
        Some(directory.path().join("project"))
    );
    Ok(())
}

#[test]
fn search_up_returns_none_when_missing() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let nested = directory.path().join("a/b/c");
    fs::create_dir_all(&nested)?;
    assert_eq!(search_up(Path::new("missing.json"), &nested, None)?, None);
    Ok(())
}

#[test]
fn search_up_content_check_skips_nonmatching_candidates() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let nested = directory.path().join("project/apps/web");
    fs::create_dir_all(&nested)?;
    write(&nested.join("turbo.json"), r#"{"extends":["//"]}"#)?;
    write(&directory.path().join("project/turbo.json"), r#"{"tasks":{}}"#)?;
    let is_root = |content: &str| !content.contains("extends");
    assert_eq!(
        search_up(Path::new("turbo.json"), &nested, Some(&is_root))?,
        Some(directory.path().join("project"))
    );
    Ok(())
}

#[test]
fn folder_empty_contract_matches_existing_tests() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    assert_eq!(
        is_folder_empty(directory.path())?,
        turbo_utils_rs::FolderEmptyResult {
            is_empty: true,
            conflicts: Vec::new()
        }
    );

    write(&directory.path().join("LICENSE"), "MIT")?;
    write(&directory.path().join("idea.iml"), "{}")?;
    assert!(is_folder_empty(directory.path())?.is_empty);

    write(&directory.path().join("README.md"), "project")?;
    let result = is_folder_empty(directory.path())?;
    assert!(!result.is_empty);
    assert_eq!(result.conflicts, vec!["README.md"]);
    Ok(())
}

#[test]
fn writeable_directory_and_missing_path_match_existing_tests() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    assert!(is_writeable(directory.path()));
    assert!(!is_writeable(&directory.path().join("does-not-exist")));
    Ok(())
}

#[test]
fn validate_directory_accepts_empty_or_missing_directory() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let empty = directory.path().join("project");
    fs::create_dir_all(&empty)?;
    let existing = validate_directory(empty.to_string_lossy().as_ref(), directory.path());
    assert!(existing.valid);
    assert_eq!(existing.project_name, "project");

    let missing = validate_directory("new-project", directory.path());
    assert!(missing.valid);
    assert_eq!(missing.root, directory.path().join("new-project"));
    assert_eq!(missing.project_name, "new-project");
    Ok(())
}

#[test]
fn validate_directory_rejects_file_and_conflicts() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let file = directory.path().join("file.txt");
    write(&file, "file")?;
    let result = validate_directory(file.to_string_lossy().as_ref(), directory.path());
    assert!(!result.valid);
    assert!(result.error.is_some_and(|error| error.contains("is not a directory")));

    let project = directory.path().join("existing");
    fs::create_dir_all(&project)?;
    write(&project.join("package.json"), "{}")?;
    write(&project.join("src"), "conflict")?;
    let result = validate_directory(project.to_string_lossy().as_ref(), directory.path());
    assert!(!result.valid);
    assert!(result.error.is_some_and(|error| error.contains("has 2 conflicting files")));
    Ok(())
}

#[test]
fn validate_directory_uses_singular_conflict_word() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = directory.path().join("existing");
    fs::create_dir_all(&project)?;
    write(&project.join("package.json"), "{}")?;
    let result = validate_directory(project.to_string_lossy().as_ref(), directory.path());
    assert!(!result.valid);
    assert!(result.error.is_some_and(|error| error.contains("has 1 conflicting file")));
    Ok(())
}
