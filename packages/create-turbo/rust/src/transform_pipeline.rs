use std::fmt;

use crate::TransformStatus;

pub const MAX_TERMINAL_DIAGNOSTIC_SCALARS: usize = 512;

const TERMINAL_TRUNCATION_MARKER: char = '…';
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

pub const TRANSFORM_PIPELINE: [TransformKind; 4] = [
    TransformKind::OfficialStarter,
    TransformKind::GitIgnore,
    TransformKind::PackageManager,
    TransformKind::UpdateCommandsInReadme,
];

#[must_use]
pub fn sanitize_terminal_text(input: &str) -> String {
    let mut output = String::with_capacity(
        input
            .len()
            .min(MAX_TERMINAL_DIAGNOSTIC_SCALARS.saturating_mul(4)),
    );
    let mut characters = input.chars();

    for _ in 0..MAX_TERMINAL_DIAGNOSTIC_SCALARS {
        let Some(character) = characters.next() else {
            return output;
        };

        if is_unsafe_terminal_scalar(character) {
            push_terminal_escape(&mut output, character);
        } else {
            output.push(character);
        }
    }

    if characters.next().is_some() {
        output.push(TERMINAL_TRUNCATION_MARKER);
    }

    output
}

fn is_unsafe_terminal_scalar(character: char) -> bool {
    matches!(
        u32::from(character),
        0x0000..=0x001f
            | 0x007f..=0x009f
            | 0x061c
            | 0x070f
            | 0x180e
            | 0x200b..=0x200f
            | 0x2028..=0x202e
            | 0x2060..=0x206f
            | 0xfeff
            | 0xfff9..=0xfffb
    )
}

fn push_terminal_escape(output: &mut String, character: char) {
    match character {
        '\0' => output.push_str("\\0"),
        '\t' => output.push_str("\\t"),
        '\n' => output.push_str("\\n"),
        '\r' => output.push_str("\\r"),
        _ => push_unicode_escape(output, u32::from(character)),
    }
}

fn push_unicode_escape(output: &mut String, mut value: u32) {
    let mut digits = [0_u8; 6];
    let mut first_digit = digits.len();

    loop {
        first_digit -= 1;
        digits[first_digit] = HEX_DIGITS[(value & 0x0f) as usize];
        value >>= 4;
        if value == 0 {
            break;
        }
    }

    output.push_str("\\u{");
    for digit in &digits[first_digit..] {
        output.push(char::from(*digit));
    }
    output.push('}');
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformKind {
    OfficialStarter,
    GitIgnore,
    PackageManager,
    UpdateCommandsInReadme,
}

impl TransformKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OfficialStarter => "official-starter",
            Self::GitIgnore => "git-ignore",
            Self::PackageManager => "package-manager",
            Self::UpdateCommandsInReadme => "update-commands-in-readme",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineTransformResponse {
    pub result: TransformStatus,
    pub name: &'static str,
    pub maintained_by_core_team: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformFailure {
    pub message: String,
    pub transform: String,
    pub fatal: bool,
}

impl TransformFailure {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self::with_options(message, None, None)
    }

    #[must_use]
    pub fn with_options(
        message: impl Into<String>,
        transform: Option<&str>,
        fatal: Option<bool>,
    ) -> Self {
        Self {
            message: message.into(),
            transform: transform.unwrap_or("unknown").to_owned(),
            fatal: fatal.unwrap_or(true),
        }
    }

    #[must_use]
    pub fn terminal_transform(&self) -> String {
        sanitize_terminal_text(&self.transform)
    }
}

impl fmt::Display for TransformFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&sanitize_terminal_text(&self.message))
    }
}

impl std::error::Error for TransformFailure {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformInvocationError<E> {
    Transform(TransformFailure),
    Unknown(E),
}

pub trait TransformExecutor<E> {
    fn execute(
        &mut self,
        transform: TransformKind,
    ) -> Result<PipelineTransformResponse, TransformInvocationError<E>>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransformPipelineReport {
    pub responses: Vec<PipelineTransformResponse>,
    pub non_fatal_errors: Vec<TransformFailure>,
    pub caught_error_count: usize,
    pub is_maintained_by_core_team: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineAbortReason<E> {
    Fatal(TransformFailure),
    Unknown(E),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineAbort<E> {
    pub report: TransformPipelineReport,
    pub reason: PipelineAbortReason<E>,
}

pub fn run_transform_pipeline<E>(
    executor: &mut impl TransformExecutor<E>,
    skip_transforms: bool,
) -> Result<TransformPipelineReport, PipelineAbort<E>> {
    let mut report = TransformPipelineReport::default();
    if skip_transforms {
        return Ok(report);
    }

    for transform in TRANSFORM_PIPELINE {
        match executor.execute(transform) {
            Ok(response) => {
                if response
                    .maintained_by_core_team
                    .as_deref()
                    .is_some_and(|maintainer| !maintainer.is_empty())
                {
                    report.is_maintained_by_core_team = true;
                }
                report.responses.push(response);
            }
            Err(TransformInvocationError::Transform(failure)) => {
                report.caught_error_count += 1;
                if failure.fatal {
                    return Err(PipelineAbort {
                        report,
                        reason: PipelineAbortReason::Fatal(failure),
                    });
                }
                report.non_fatal_errors.push(failure);
            }
            Err(TransformInvocationError::Unknown(error)) => {
                report.caught_error_count += 1;
                return Err(PipelineAbort {
                    report,
                    reason: PipelineAbortReason::Unknown(error),
                });
            }
        }
    }

    Ok(report)
}
