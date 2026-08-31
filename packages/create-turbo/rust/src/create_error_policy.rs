pub const CREATE_COMMAND_ERROR_EXIT_CODE: u8 = 1;
pub const CREATE_COMMAND_ERROR_MESSAGE_LIMIT: usize = 4096;
pub const CREATE_COMMAND_ERROR_TRANSFORM_LIMIT: usize = 256;
pub const DOWNLOAD_ERROR_HEADING: &str = "Unable to download template from GitHub";

const TRUNCATION_MARKER: &str = "[truncated]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertErrorType {
    Known,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateCommandError<'a, E> {
    Transform {
        transform: &'a str,
        message: &'a str,
        fatal: bool,
    },
    Convert {
        source: E,
        error_type: ConvertErrorType,
        message: &'a str,
    },
    Download {
        message: &'a str,
    },
    Unknown(E),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCommandErrorLine {
    pub label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateCommandErrorAction<E> {
    Continue,
    Exit(u8),
    Rethrow(E),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCommandErrorOutcome<E> {
    pub track_error_status: bool,
    pub lines: Vec<CreateCommandErrorLine>,
    pub action: CreateCommandErrorAction<E>,
}

#[must_use]
pub fn classify_create_command_error<E>(
    error: CreateCommandError<'_, E>,
) -> CreateCommandErrorOutcome<E> {
    match error {
        CreateCommandError::Transform {
            transform,
            message,
            fatal,
        } => {
            let action = if fatal {
                CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE)
            } else {
                CreateCommandErrorAction::Continue
            };
            outcome(
                vec![display_line(Some(transform), message)],
                action,
            )
        }
        CreateCommandError::Convert {
            source,
            error_type,
            message,
        } => match error_type {
            ConvertErrorType::Known => outcome(
                vec![display_line(None, message)],
                CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE),
            ),
            ConvertErrorType::Unknown => {
                outcome(Vec::new(), CreateCommandErrorAction::Rethrow(source))
            }
        },
        CreateCommandError::Download { message } => outcome(
            vec![
                display_line(None, DOWNLOAD_ERROR_HEADING),
                display_line(None, message),
            ],
            CreateCommandErrorAction::Exit(CREATE_COMMAND_ERROR_EXIT_CODE),
        ),
        CreateCommandError::Unknown(source) => {
            outcome(Vec::new(), CreateCommandErrorAction::Rethrow(source))
        }
    }
}

#[must_use]
pub fn sanitize_terminal_text(input: &str, max_bytes: usize) -> String {
    let mut output = String::with_capacity(input.len().min(max_bytes));
    let mut chunk_lengths = Vec::with_capacity(input.len().min(max_bytes).min(256));
    let mut truncated = false;

    for character in input.chars() {
        let fragment = terminal_fragment(character);
        if fragment.len() > max_bytes.saturating_sub(output.len()) {
            truncated = true;
            break;
        }
        output.push_str(&fragment);
        chunk_lengths.push(fragment.len());
    }

    if truncated {
        append_truncation_marker(&mut output, &mut chunk_lengths, max_bytes);
    }

    output
}

fn display_line(label: Option<&str>, message: &str) -> CreateCommandErrorLine {
    CreateCommandErrorLine {
        label: label.map(|value| sanitize_terminal_text(value, CREATE_COMMAND_ERROR_TRANSFORM_LIMIT)),
        message: sanitize_terminal_text(message, CREATE_COMMAND_ERROR_MESSAGE_LIMIT),
    }
}

fn outcome<E>(
    lines: Vec<CreateCommandErrorLine>,
    action: CreateCommandErrorAction<E>,
) -> CreateCommandErrorOutcome<E> {
    CreateCommandErrorOutcome {
        track_error_status: true,
        lines,
        action,
    }
}

fn terminal_fragment(character: char) -> String {
    match character {
        '\n' => "\\n".to_owned(),
        '\r' => "\\r".to_owned(),
        '\t' => "\\t".to_owned(),
        _ if character.is_control() || is_terminal_format_control(character) => {
            character.escape_unicode().to_string()
        }
        _ => character.to_string(),
    }
}

fn append_truncation_marker(
    output: &mut String,
    chunk_lengths: &mut Vec<usize>,
    max_bytes: usize,
) {
    if max_bytes < TRUNCATION_MARKER.len() {
        output.clear();
        output.push_str(&TRUNCATION_MARKER[..max_bytes]);
        return;
    }

    let content_limit = max_bytes - TRUNCATION_MARKER.len();
    while output.len() > content_limit {
        let Some(chunk_length) = chunk_lengths.pop() else {
            output.clear();
            break;
        };
        output.truncate(output.len() - chunk_length);
    }
    output.push_str(TRUNCATION_MARKER);
}

fn is_terminal_format_control(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{2028}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
    )
}
