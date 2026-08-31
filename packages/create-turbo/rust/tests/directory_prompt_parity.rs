use std::{error::Error, fmt};

use create_turbo_rs::{
    DEFAULT_PROJECT_DIRECTORY, DIRECTORY_PROMPT_MESSAGE, DirectoryDisplayTransform,
    DirectoryPromptError, DirectoryPromptRequest, DirectoryPrompter, DirectoryValidator,
    MAX_DIRECTORY_INPUT_BYTES, resolve_directory_prompt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedDirectory(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptFailure {
    Cancelled,
}

impl fmt::Display for PromptFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("prompt cancelled")
    }
}

impl Error for PromptFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationFailure {
    Invalid,
}

impl fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid directory")
    }
}

impl Error for ValidationFailure {}

#[derive(Debug)]
struct RecordingPrompter {
    calls: Vec<DirectoryPromptRequest>,
    result: Result<String, PromptFailure>,
}

impl RecordingPrompter {
    fn succeeds(value: &str) -> Self {
        Self {
            calls: Vec::new(),
            result: Ok(value.to_owned()),
        }
    }

    fn fails() -> Self {
        Self {
            calls: Vec::new(),
            result: Err(PromptFailure::Cancelled),
        }
    }
}

impl DirectoryPrompter for RecordingPrompter {
    type Error = PromptFailure;

    fn prompt(&mut self, request: DirectoryPromptRequest) -> Result<String, Self::Error> {
        self.calls.push(request);
        self.result.clone()
    }
}

#[derive(Debug)]
struct RecordingValidator {
    calls: Vec<String>,
    reject: bool,
}

impl RecordingValidator {
    fn accepting() -> Self {
        Self {
            calls: Vec::new(),
            reject: false,
        }
    }

    fn rejecting() -> Self {
        Self {
            calls: Vec::new(),
            reject: true,
        }
    }
}

impl DirectoryValidator for RecordingValidator {
    type Error = ValidationFailure;
    type Output = ValidatedDirectory;

    fn validate(&mut self, directory: &str) -> Result<Self::Output, Self::Error> {
        self.calls.push(directory.to_owned());
        if self.reject {
            Err(ValidationFailure::Invalid)
        } else {
            Ok(ValidatedDirectory(directory.to_owned()))
        }
    }
}

#[test]
fn prompt_constants_match_the_typescript_source() {
    assert_eq!(
        DIRECTORY_PROMPT_MESSAGE,
        "Where would you like to create your Turborepo?"
    );
    assert_eq!(DEFAULT_PROJECT_DIRECTORY, "./my-turborepo");
    assert_eq!(MAX_DIRECTORY_INPUT_BYTES, 4_096);
}

#[test]
fn truthy_argument_bypasses_prompt_and_is_not_trimmed() {
    let mut prompter = RecordingPrompter::fails();
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(Some("  project  "), &mut prompter, &mut validator);

    assert_eq!(result, Ok(ValidatedDirectory("  project  ".to_owned())));
    assert!(prompter.calls.is_empty());
    assert_eq!(validator.calls, ["  project  "]);
}

#[test]
fn empty_argument_is_javascript_falsy_and_uses_the_prompt() {
    let mut prompter = RecordingPrompter::succeeds("project");
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(Some(""), &mut prompter, &mut validator);

    assert_eq!(result, Ok(ValidatedDirectory("project".to_owned())));
    assert_eq!(prompter.calls.len(), 1);
    assert_eq!(validator.calls, ["project"]);
}

#[test]
fn missing_argument_uses_the_exact_prompt_request() {
    let mut prompter = RecordingPrompter::succeeds("project");
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(None, &mut prompter, &mut validator);

    assert_eq!(result, Ok(ValidatedDirectory("project".to_owned())));
    assert_eq!(
        prompter.calls,
        [DirectoryPromptRequest {
            message: DIRECTORY_PROMPT_MESSAGE,
            default: DEFAULT_PROJECT_DIRECTORY,
            max_input_bytes: MAX_DIRECTORY_INPUT_BYTES,
            display_transform: DirectoryDisplayTransform::TrimEcmascriptWhitespace,
        }]
    );
}

#[test]
fn prompt_transform_is_display_only_and_raw_answer_is_validated() {
    let raw_answer = "  project  ";
    let mut prompter = RecordingPrompter::succeeds(raw_answer);
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(None, &mut prompter, &mut validator);

    assert_eq!(result, Ok(ValidatedDirectory(raw_answer.to_owned())));
    assert_eq!(validator.calls, [raw_answer]);
    assert_eq!(prompter.calls.len(), 1);
    assert_eq!(
        prompter.calls[0]
            .display_transform
            .apply("\u{feff}\u{00a0}project\u{3000}"),
        "project"
    );
}

#[test]
fn prompt_failure_is_propagated_without_validation() {
    let mut prompter = RecordingPrompter::fails();
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(None, &mut prompter, &mut validator);

    assert_eq!(
        result,
        Err(DirectoryPromptError::Prompt(PromptFailure::Cancelled))
    );
    assert_eq!(prompter.calls.len(), 1);
    assert!(validator.calls.is_empty());
}

#[test]
fn validation_failure_is_propagated_without_prompt_retry() {
    let mut prompter = RecordingPrompter::fails();
    let mut validator = RecordingValidator::rejecting();

    let result = resolve_directory_prompt(Some("project"), &mut prompter, &mut validator);

    assert_eq!(
        result,
        Err(DirectoryPromptError::Validation(ValidationFailure::Invalid))
    );
    assert!(prompter.calls.is_empty());
    assert_eq!(validator.calls, ["project"]);
}

#[test]
fn validated_output_is_returned_without_reinterpretation() {
    let mut prompter = RecordingPrompter::fails();
    let mut validator = RecordingValidator::accepting();

    let result = resolve_directory_prompt(Some("project-01"), &mut prompter, &mut validator);

    assert_eq!(result, Ok(ValidatedDirectory("project-01".to_owned())));
}
