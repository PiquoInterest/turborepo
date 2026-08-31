use std::{error::Error, fmt};

pub const DIRECTORY_PROMPT_MESSAGE: &str = "Where would you like to create your Turborepo?";
pub const DEFAULT_PROJECT_DIRECTORY: &str = "./my-turborepo";
pub const MAX_DIRECTORY_INPUT_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryPromptRequest {
    pub message: &'static str,
    pub default: &'static str,
    pub max_input_bytes: usize,
}

pub trait DirectoryPrompter {
    type Error;

    fn prompt(&mut self, request: DirectoryPromptRequest) -> Result<String, Self::Error>;
}

pub trait DirectoryValidator {
    type Error;
    type Output;

    fn validate(&mut self, directory: &str) -> Result<Self::Output, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryInputRejection {
    TooLong {
        actual_bytes: usize,
        max_bytes: usize,
    },
    UnsafeControl,
}

impl fmt::Display for DirectoryInputRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong {
                actual_bytes,
                max_bytes,
            } => write!(
                formatter,
                "project directory input is {actual_bytes} bytes and exceeds the {max_bytes}-byte \
                 limit"
            ),
            Self::UnsafeControl => {
                formatter.write_str("project directory input contains unsafe control characters")
            }
        }
    }
}

impl Error for DirectoryInputRejection {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryPromptError<P, V> {
    Prompt(P),
    Input(DirectoryInputRejection),
    Validation(V),
}

impl<P, V> fmt::Display for DirectoryPromptError<P, V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prompt(_) => formatter.write_str("unable to read the project directory"),
            Self::Input(error) => error.fmt(formatter),
            Self::Validation(_) => formatter.write_str("project directory validation failed"),
        }
    }
}

impl<P, V> Error for DirectoryPromptError<P, V>
where
    P: Error + 'static,
    V: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Prompt(error) => Some(error),
            Self::Input(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}

pub fn resolve_directory_prompt<P, V>(
    argument: Option<&str>,
    prompter: &mut P,
    validator: &mut V,
) -> Result<V::Output, DirectoryPromptError<P::Error, V::Error>>
where
    P: DirectoryPrompter,
    V: DirectoryValidator,
{
    if let Some(directory) = argument
        && !directory.is_empty()
    {
        validate_directory_input(directory).map_err(DirectoryPromptError::Input)?;
        return validator
            .validate(directory)
            .map_err(DirectoryPromptError::Validation);
    }

    let prompted = prompter
        .prompt(DirectoryPromptRequest {
            message: DIRECTORY_PROMPT_MESSAGE,
            default: DEFAULT_PROJECT_DIRECTORY,
            max_input_bytes: MAX_DIRECTORY_INPUT_BYTES,
        })
        .map_err(DirectoryPromptError::Prompt)?;
    let directory = trim_ecmascript_whitespace(&prompted);
    validate_directory_input(directory).map_err(DirectoryPromptError::Input)?;
    validator
        .validate(directory)
        .map_err(DirectoryPromptError::Validation)
}

fn validate_directory_input(directory: &str) -> Result<(), DirectoryInputRejection> {
    let actual_bytes = directory.len();
    if actual_bytes > MAX_DIRECTORY_INPUT_BYTES {
        return Err(DirectoryInputRejection::TooLong {
            actual_bytes,
            max_bytes: MAX_DIRECTORY_INPUT_BYTES,
        });
    }
    if directory.chars().any(is_unsafe_control) {
        return Err(DirectoryInputRejection::UnsafeControl);
    }
    Ok(())
}

fn is_unsafe_control(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{00ad}'
                | '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{206f}'
                | '\u{feff}'
        )
}

fn trim_ecmascript_whitespace(value: &str) -> &str {
    value.trim_matches(is_ecmascript_whitespace)
}

fn is_ecmascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'
            | '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
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
