use std::{
    env,
    path::{Component, Path, PathBuf},
};

/// Maximum number of Unicode scalar values accepted in one archive entry path.
pub const ARCHIVE_MAX_PATH_CHARS: usize = 4_096;
/// Maximum number of non-empty archive path components processed per entry.
pub const ARCHIVE_MAX_PATH_COMPONENTS: usize = 256;

/// Returns true for tar entry types that can redirect later writes outside the
/// intended extraction tree.
#[must_use]
pub fn is_archive_link_entry(entry_type: &str) -> bool {
    matches!(entry_type, "SymbolicLink" | "Link")
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn lexical_absolute(path: &Path) -> Option<PathBuf> {
    let combined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().ok()?.join(path)
    };

    let mut resolved = PathBuf::new();
    for component in combined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _removed = resolved.pop();
            }
            Component::Prefix(prefix) => resolved.push(prefix.as_os_str()),
            Component::RootDir => resolved.push(component.as_os_str()),
            Component::Normal(segment) => resolved.push(segment),
        }
    }
    resolved.is_absolute().then_some(resolved)
}

/// Validates an archive entry path without consulting the destination
/// filesystem.
///
/// The safe TypeScript behavior is preserved for ordinary relative paths:
/// mixed separators are normalized, internal `..` components are allowed when
/// they remain below the root, and any component that would escape is rejected.
/// The Rust implementation uses component semantics rather than
/// `relativePath.startsWith("..")`, so safe names such as `..cache` are not
/// misclassified as traversal.
///
/// Additional cross-platform hardening rejects NULs, absolute/UNC/drive paths,
/// Windows alternate-data-stream syntax, and oversized/deep entry names.
#[must_use]
pub fn is_archive_path_safe(
    root: &Path,
    stripped_path: &str,
    resolved_root: Option<&Path>,
) -> bool {
    if stripped_path.contains('\0') || stripped_path.chars().count() > ARCHIVE_MAX_PATH_CHARS {
        return false;
    }

    let normalized = stripped_path.replace('\\', "/");
    if normalized.starts_with('/')
        || has_windows_drive_prefix(&normalized)
        || normalized.contains(':')
    {
        return false;
    }

    let Some(root_path) = lexical_absolute(resolved_root.unwrap_or(root)) else {
        return false;
    };
    let mut destination = root_path.clone();
    let mut depth = 0_usize;
    let mut components_seen = 0_usize;

    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        components_seen += 1;
        if components_seen > ARCHIVE_MAX_PATH_COMPONENTS {
            return false;
        }

        if segment == ".." {
            if depth == 0 || !destination.pop() {
                return false;
            }
            depth -= 1;
            continue;
        }

        destination.push(segment);
        depth += 1;
    }

    destination.starts_with(root_path)
}
