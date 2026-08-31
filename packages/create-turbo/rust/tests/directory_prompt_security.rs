use std::{error::Error, fmt};

use create_turbo_rs::{
    DirectoryInputRejection, DirectoryPromptError, DirectoryPromptRequest, DirectoryPrompter,
    DirectoryValidator, MAX_DIRECTORY_INPUT_BYTES, resolve_directory_prompt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PromptFailure;

impl fmt::Display for PromptFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("prompt failure")
    }
}

impl Error for PromptFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ValidationFailure;

impl fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("validation failure")
    }
}

impl Error for ValidationFailure {}

#[derive(Debug)]
struct CountingPrompter {
    calls: Vec<DirectoryPromptRequest>,
    response: Result<String, PromptFailure>,
}

impl CountingPrompter {
    fn returning(value: String) -> Self {
        Self {
            calls: Vec::new(),
            response: Ok(value),
        }
    }

    fn failing() -> Self {
        Self {
            calls: Vec::new(),
            response: Err(PromptFailure),
        }
    }
}

impl DirectoryPrompter for CountingPrompter {
    type Error = PromptFailure;

    fn prompt(&mut self, request: DirectoryPromptRequest) -> Result<String, Self::Error> {
        self.calls.push(request);
        self.response.clone()
    }
}

#[derive(Debug)]
struct CountingValidator {
    calls: Vec<String>,
    reject: bool,
}

impl CountingValidator {
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

impl DirectoryValidator for CountingValidator {
    type Error = ValidationFailure;
    type Output = String;

    fn validate(&mut self, directory: &str) -> Result<Self::Output, Self::Error> {
        self.calls.push(directory.to_owned());
        if self.reject {
            Err(ValidationFailure)
        } else {
            Ok(directory.to_owned())
        }
    }
}

#[test]
fn invalid_direct_argument_cannot_escape_as_a_false_success() {
    let mut prompter = CountingPrompter::failing();
    let mut validator = CountingValidator::rejecting();

    let result = resolve_directory_prompt(Some("conflicting-project"), &mut prompter, &mut validator);

    assert_eq!(
        result,
        Err(DirectoryPromptError::Validation(ValidationFailure))
    );
    assert!(prompter.calls.is_empty());
    assert_eq!(validator.calls, ["conflicting-project"]);
}

#[test]
fn c0_and_c1_controls_are_rejected_before_any_provider() {
    for input in ["project\0", "project\nname", "project\u{001b}[31m", "project\u{0085}"] {
        let mut prompter = CountingPrompter::failing();
        let mut validator = CountingValidator::accepting();

        let result = resolve_directory_prompt(Some(input), &mut prompter, &mut validator);

        assert_eq!(
            result,
            Err(DirectoryPromptError::Input(
                DirectoryInputRejection::UnsafeControl
            ))
        );
        assert!(prompter.calls.is_empty());
        assert!(validator.calls.is_empty());
    }
}

#[test]
fn invisible_and_bidirectional_format_controls_are_rejected() {
    for input in [
        "project\u{061c}",
        "project\u{200d}",
        "project\u{202e}txt",
        "project\u{2066}txt\u{2069}",
        "project\u{feff}suffix",
    ] {
        let mut prompter = CountingPrompter::failing();
        let mut validator = CountingValidator::accepting();

        let result = resolve_directory_prompt(Some(input), &mut prompter, &mut validator);

        assert_eq!(
            result,
            Err(DirectoryPromptError::Input(
                DirectoryInputRejection::UnsafeControl
            ))
        );
        assert!(prompter.calls.is_empty());
        assert!(validator.calls.is_empty());
    }
}

#[test]
fn oversized_argument_is_rejected_before_prompt_or_validation() {
    let oversized = "a".repeat(MAX_DIRECTORY_INPUT_BYTES + 1);
    let mut prompter = CountingPrompter::failing();
    let mut validator = CountingValidator::accepting();

    let result = resolve_directory_prompt(Some(&oversized), &mut prompter, &mut validator);

    assert_eq!(
        result,
        Err(DirectoryPromptError::Input(
            DirectoryInputRejection::TooLong {
                actual_bytes: MAX_DIRECTORY_INPUT_BYTES + 1,
                max_bytes: MAX_DIRECTORY_INPUT_BYTES,
            }
        ))
    );
    assert!(prompter.calls.is_empty());
    assert!(validator.calls.is_empty());
}

#[test]
fn prompt_request_advertises_the_same_enforced_byte_limit() {
    let oversized = "a".repeat(MAX_DIRECTORY_INPUT_BYTES + 1);
    let mut prompter = CountingPrompter::returning(oversized);
    let mut validator = CountingValidator::accepting();

    let result = resolve_directory_prompt(None, &mut prompter, &mut validator);

    assert_eq!(prompter.calls.len(), 1);
    assert_eq!(
        prompter.calls[0].max_input_bytes,
        MAX_DIRECTORY_INPUT_BYTES
    );
    assert!(matches!(
        result,
        Err(DirectoryPromptError::Input(
            DirectoryInputRejection::TooLong { .. }
        ))
    ));
    assert!(validator.calls.is_empty());
}

#[test]
fn unsafe_prompt_response_is_rejected_before_validation() {
    let mut prompter = CountingPrompter::returning("project\u{202e}txt".to_owned());
    let mut validator = CountingValidator::accepting();

    let result = resolve_directory_prompt(None, &mut prompter, &mut validator);

    assert_eq!(
        result,
        Err(DirectoryPromptError::Input(
            DirectoryInputRejection::UnsafeControl
        ))
    );
    assert_eq!(prompter.calls.len(), 1);
    assert!(validator.calls.is_empty());
}

#[test]
fn public_input_error_does_not_reflect_attacker_control_text() {
    let attacker_input = "project\u{001b}[31msecret";
    let mut prompter = CountingPrompter::failing();
    let mut validator = CountingValidator::accepting();

    let result = resolve_directory_prompt(Some(attacker_input), &mut prompter, &mut validator);
    let message = match result {
        Err(error) => error.to_string(),
        Ok(_) => String::from("unexpected success"),
    };

    assert!(!message.contains(attacker_input));
    assert!(!message.contains('\u{001b}'));
    assert!(message.contains("unsafe control"));
}

#[test]
fn valid_input_reaches_the_validator_exactly_once() {
    let mut prompter = CountingPrompter::failing();
    let mut validator = CountingValidator::accepting();

    let result = resolve_directory_prompt(Some("project"), &mut prompter, &mut validator);

    assert_eq!(result, Ok("project".to_owned()));
    assert_eq!(validator.calls, ["project"]);
}
