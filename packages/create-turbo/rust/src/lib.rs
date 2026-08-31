use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const TRANSFORM_NAME: &str = "update-commands-in-readme";
pub const MAX_README_BYTES: usize = 4 * 1024 * 1024;

const PACKAGE_MANAGERS: [&[u8]; 4] = [b"pnpm", b"npm", b"yarn", b"bun"];
const TEMPORARY_FILE_ATTEMPTS: usize = 32;
static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

impl PackageManager {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Npm => "npm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformStatus {
    NotApplicable,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformResponse {
    pub result: TransformStatus,
    pub name: &'static str,
}

#[derive(Debug)]
pub enum TransformError {
    Read(std::io::Error),
    Write(std::io::Error),
    InvalidUtf8,
    ReadmeTooLarge,
    UnsafeRoot,
    UnsafeReadme,
    ConcurrentModification,
    TemporaryFileExhausted,
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Read(_) => "unable to read README.md",
            Self::Write(_) => "unable to update README.md",
            Self::InvalidUtf8 => "README.md is not valid UTF-8",
            Self::ReadmeTooLarge => "README.md exceeds the size limit",
            Self::UnsafeRoot => "project root is not a safe directory",
            Self::UnsafeReadme => "README.md is not a safe regular file",
            Self::ConcurrentModification => "README.md changed while it was being updated",
            Self::TemporaryFileExhausted => "unable to allocate a temporary README file",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TransformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) | Self::Write(error) => Some(error),
            Self::InvalidUtf8
            | Self::ReadmeTooLarge
            | Self::UnsafeRoot
            | Self::UnsafeReadme
            | Self::ConcurrentModification
            | Self::TemporaryFileExhausted => None,
        }
    }
}

pub fn replace_package_manager_references(
    target: PackageManager,
    text: &str,
) -> Result<String, TransformError> {
    if text.len() > MAX_README_BYTES {
        return Err(TransformError::ReadmeTooLarge);
    }

    let mut output = String::with_capacity(text.len());
    let mut copied_until = 0;
    let mut search_from = 0;

    while let Some((start, end)) = next_code_region(text, search_from) {
        output.push_str(&text[copied_until..start]);
        let with_run_commands = replace_run_commands(&text[start..end], target.as_str());
        output.push_str(&replace_bare_commands(&with_run_commands, target.as_str()));
        copied_until = end;
        search_from = end;
    }

    output.push_str(&text[copied_until..]);
    Ok(output)
}

pub fn transform_readme(
    root: &Path,
    package_manager: Option<PackageManager>,
) -> Result<TransformResponse, TransformError> {
    let Some(package_manager) = package_manager else {
        return Ok(not_applicable());
    };

    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(not_applicable());
        }
        Err(error) => return Err(TransformError::Read(error)),
    };
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(TransformError::UnsafeRoot);
    }

    let readme_path = root.join("README.md");
    let readme_metadata = match fs::symlink_metadata(&readme_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(not_applicable());
        }
        Err(error) => return Err(TransformError::Read(error)),
    };
    validate_readme_metadata(&readme_metadata)?;

    let original = read_readme(&readme_path, &readme_metadata)?;
    let updated = replace_package_manager_references(package_manager, &original)?;

    validate_root_identity(root, &root_metadata)?;
    validate_readme_identity(&readme_path, &readme_metadata)?;
    write_readme_atomically(
        root,
        &root_metadata,
        &readme_path,
        &readme_metadata,
        updated.as_bytes(),
    )?;

    Ok(TransformResponse {
        result: TransformStatus::Success,
        name: TRANSFORM_NAME,
    })
}

const fn not_applicable() -> TransformResponse {
    TransformResponse {
        result: TransformStatus::NotApplicable,
        name: TRANSFORM_NAME,
    }
}

fn next_code_region(text: &str, mut index: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }

        if bytes[index..].starts_with(b"```") {
            let content_start = index + 3;
            if let Some(relative_end) = find_subslice(&bytes[content_start..], b"```") {
                return Some((index, content_start + relative_end + 3));
            }
        }

        let content_start = index + 1;
        if content_start < bytes.len()
            && bytes[content_start] != b'`'
            && let Some(relative_end) = bytes[content_start..]
                .iter()
                .position(|byte| *byte == b'`')
        {
            return Some((index, content_start + relative_end + 1));
        }

        index += 1;
    }
    None
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn replace_run_commands(input: &str, target: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut copied_until = 0;
    let mut index = 0;

    while index < bytes.len() {
        if let Some(match_len) = match_run_command(bytes, index) {
            output.push_str(&input[copied_until..index]);
            output.push_str(target);
            output.push_str(" run");
            index += match_len;
            copied_until = index;
        } else {
            index += 1;
        }
    }

    output.push_str(&input[copied_until..]);
    output
}

fn match_run_command(bytes: &[u8], start: usize) -> Option<usize> {
    if !has_left_word_boundary(bytes, start) {
        return None;
    }

    for manager in PACKAGE_MANAGERS {
        let manager_end = start.checked_add(manager.len())?;
        let command_end = manager_end.checked_add(4)?;
        if command_end <= bytes.len()
            && &bytes[start..manager_end] == manager
            && &bytes[manager_end..command_end] == b" run"
            && has_right_word_boundary(bytes, command_end)
        {
            return Some(command_end - start);
        }
    }
    None
}

fn replace_bare_commands(input: &str, target: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut copied_until = 0;
    let mut index = 0;

    while index < bytes.len() {
        if let Some(match_len) = match_bare_manager(input, index) {
            output.push_str(&input[copied_until..index]);
            output.push_str(target);
            index += match_len;
            copied_until = index;
        } else {
            index += 1;
        }
    }

    output.push_str(&input[copied_until..]);
    output
}

fn match_bare_manager(input: &str, start: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if !has_left_word_boundary(bytes, start) {
        return None;
    }

    for manager in PACKAGE_MANAGERS {
        let end = start.checked_add(manager.len())?;
        if end <= bytes.len()
            && &bytes[start..end] == manager
            && has_right_word_boundary(bytes, end)
            && !is_followed_by_whitespace_run(&input[end..])
        {
            return Some(manager.len());
        }
    }
    None
}

fn has_left_word_boundary(bytes: &[u8], start: usize) -> bool {
    start == 0 || !is_javascript_word_byte(bytes[start - 1])
}

fn has_right_word_boundary(bytes: &[u8], end: usize) -> bool {
    end == bytes.len() || !is_javascript_word_byte(bytes[end])
}

const fn is_javascript_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_followed_by_whitespace_run(value: &str) -> bool {
    let mut whitespace_end = 0;
    for (index, character) in value.char_indices() {
        if !is_javascript_whitespace(character) {
            break;
        }
        whitespace_end = index + character.len_utf8();
    }
    whitespace_end > 0 && value[whitespace_end..].starts_with("run")
}

const fn is_javascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

fn validate_readme_metadata(metadata: &fs::Metadata) -> Result<(), TransformError> {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(TransformError::UnsafeReadme);
    }
    if metadata.len() > MAX_README_BYTES as u64 {
        return Err(TransformError::ReadmeTooLarge);
    }
    Ok(())
}

fn read_readme(path: &Path, expected_metadata: &fs::Metadata) -> Result<String, TransformError> {
    let file = File::open(path).map_err(TransformError::Read)?;
    let opened_metadata = file.metadata().map_err(TransformError::Read)?;
    validate_readme_metadata(&opened_metadata)?;
    if !same_file(expected_metadata, &opened_metadata) {
        return Err(TransformError::ConcurrentModification);
    }

    let expected_capacity = usize::try_from(opened_metadata.len())
        .unwrap_or(MAX_README_BYTES)
        .min(MAX_README_BYTES);
    let mut bytes = Vec::with_capacity(expected_capacity);
    file.take((MAX_README_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(TransformError::Read)?;
    if bytes.len() > MAX_README_BYTES {
        return Err(TransformError::ReadmeTooLarge);
    }
    String::from_utf8(bytes).map_err(|_| TransformError::InvalidUtf8)
}

fn validate_root_identity(root: &Path, expected: &fs::Metadata) -> Result<(), TransformError> {
    let current = fs::symlink_metadata(root).map_err(TransformError::Write)?;
    if current.file_type().is_symlink() || !current.is_dir() {
        return Err(TransformError::UnsafeRoot);
    }
    if !same_file(expected, &current) {
        return Err(TransformError::ConcurrentModification);
    }
    Ok(())
}

fn validate_readme_identity(path: &Path, expected: &fs::Metadata) -> Result<(), TransformError> {
    let current = fs::symlink_metadata(path).map_err(TransformError::Write)?;
    validate_readme_metadata(&current)?;
    if !same_file(expected, &current) {
        return Err(TransformError::ConcurrentModification);
    }
    Ok(())
}

fn write_readme_atomically(
    root: &Path,
    root_metadata: &fs::Metadata,
    readme_path: &Path,
    readme_metadata: &fs::Metadata,
    contents: &[u8],
) -> Result<(), TransformError> {
    let (temporary_path, mut temporary_file) = create_temporary_file(root)?;
    let write_result = (|| {
        temporary_file
            .write_all(contents)
            .map_err(TransformError::Write)?;
        temporary_file
            .set_permissions(readme_metadata.permissions())
            .map_err(TransformError::Write)?;
        temporary_file.sync_all().map_err(TransformError::Write)?;
        drop(temporary_file);

        validate_root_identity(root, root_metadata)?;
        validate_readme_identity(readme_path, readme_metadata)?;
        replace_file(&temporary_path, readme_path).map_err(TransformError::Write)
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    write_result
}

fn create_temporary_file(root: &Path) -> Result<(PathBuf, File), TransformError> {
    for _ in 0..TEMPORARY_FILE_ATTEMPTS {
        let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!(
            ".README.md.{}.{}.tmp",
            std::process::id(),
            sequence
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(TransformError::Write(error)),
        }
    }
    Err(TransformError::TemporaryFileExhausted)
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::rename(temporary, target)
}

#[cfg(windows)]
fn replace_file(temporary: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::remove_file(target)?;
    fs::rename(temporary, target)
}

#[cfg(unix)]
fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_left: &fs::Metadata, _right: &fs::Metadata) -> bool {
    true
}
