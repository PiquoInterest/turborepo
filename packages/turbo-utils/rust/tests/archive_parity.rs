use std::path::Path;

use turbo_utils_rs::{is_archive_link_entry, is_archive_path_safe};

#[test]
fn paths_inside_the_extraction_root_are_allowed() {
    let root = Path::new("/tmp/extract");
    assert!(is_archive_path_safe(root, "file.txt", None));
    assert!(is_archive_path_safe(root, "subdir/file.txt", None));
    assert!(is_archive_path_safe(root, "a/b/c/file.txt", None));
}

#[test]
fn parent_traversal_outside_the_root_is_blocked() {
    let root = Path::new("/tmp/extract");
    assert!(!is_archive_path_safe(root, "../etc/passwd", None));
    assert!(!is_archive_path_safe(root, "../../etc/passwd", None));
    assert!(!is_archive_path_safe(root, "../../../etc/passwd", None));
}

#[test]
fn nested_parent_traversal_outside_the_root_is_blocked() {
    let root = Path::new("/tmp/extract");
    assert!(!is_archive_path_safe(
        root,
        "foo/../../../etc/passwd",
        None
    ));
    assert!(!is_archive_path_safe(
        root,
        "foo/bar/../../../etc/passwd",
        None
    ));
}

#[test]
fn internal_parent_components_that_remain_inside_are_allowed() {
    let root = Path::new("/tmp/extract");
    assert!(is_archive_path_safe(root, "foo/../bar", None));
    assert!(is_archive_path_safe(root, "a/b/../c", None));
}

#[test]
fn absolute_entry_paths_are_blocked() {
    assert!(!is_archive_path_safe(
        Path::new("/tmp/extract"),
        "/etc/passwd",
        None
    ));
}

#[test]
fn a_pre_resolved_root_uses_the_same_contract() {
    let root = Path::new("relative-is-not-used");
    let resolved = Path::new("/tmp/extract");
    assert!(is_archive_path_safe(root, "file.txt", Some(resolved)));
    assert!(!is_archive_path_safe(
        root,
        "../etc/passwd",
        Some(resolved)
    ));
}

#[test]
fn symbolic_and_hard_links_are_rejected() {
    assert!(is_archive_link_entry("SymbolicLink"));
    assert!(is_archive_link_entry("Link"));
    assert!(!is_archive_link_entry("File"));
    assert!(!is_archive_link_entry("Directory"));
}
