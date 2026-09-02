use std::path::Path;

use turbo_utils_rs::{ARCHIVE_MAX_PATH_CHARS, ARCHIVE_MAX_PATH_COMPONENTS, is_archive_path_safe};

#[test]
fn nul_bytes_are_rejected() {
    assert!(!is_archive_path_safe(
        Path::new("/tmp/extract"),
        "safe\0/../../etc/passwd",
        None
    ));
}

#[test]
fn backslash_traversal_is_normalized_and_rejected() {
    let root = Path::new("/tmp/extract");
    assert!(!is_archive_path_safe(root, "..\\etc\\passwd", None));
    assert!(!is_archive_path_safe(
        root,
        "safe\\..\\..\\etc\\passwd",
        None
    ));
}

#[test]
fn windows_absolute_and_unc_paths_are_rejected_on_every_platform() {
    let root = Path::new("/tmp/extract");
    assert!(!is_archive_path_safe(root, "C:\\Windows\\win.ini", None));
    assert!(!is_archive_path_safe(root, "C:/Windows/win.ini", None));
    assert!(!is_archive_path_safe(root, "\\\\server\\share\\file", None));
    assert!(!is_archive_path_safe(root, "//server/share/file", None));
}

#[test]
fn windows_alternate_data_stream_syntax_is_rejected() {
    let root = Path::new("/tmp/extract");
    assert!(!is_archive_path_safe(root, "file.txt:payload.exe", None));
    assert!(!is_archive_path_safe(root, "dir/name:stream", None));
}

#[test]
fn path_size_and_component_count_are_bounded() {
    let root = Path::new("/tmp/extract");
    let oversized = "a".repeat(ARCHIVE_MAX_PATH_CHARS + 1);
    assert!(!is_archive_path_safe(root, &oversized, None));

    let too_deep = std::iter::repeat_n("a", ARCHIVE_MAX_PATH_COMPONENTS + 1)
        .collect::<Vec<_>>()
        .join("/");
    assert!(!is_archive_path_safe(root, &too_deep, None));
}

#[test]
fn path_at_the_documented_limits_is_allowed() {
    let root = Path::new("/tmp/extract");
    let at_depth = std::iter::repeat_n("a", ARCHIVE_MAX_PATH_COMPONENTS)
        .collect::<Vec<_>>()
        .join("/");
    assert!(at_depth.chars().count() <= ARCHIVE_MAX_PATH_CHARS);
    assert!(is_archive_path_safe(root, &at_depth, None));
}

#[test]
fn dot_dot_prefixed_normal_names_are_not_parent_traversal() {
    let root = Path::new("/tmp/extract");
    assert!(is_archive_path_safe(root, "..cache", None));
    assert!(is_archive_path_safe(root, "dir/..metadata", None));
}
