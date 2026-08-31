use std::fmt;

use crate::TransformStatus;

pub const TRANSFORM_PIPELINE: [TransformKind; 4] = [
    TransformKind::OfficialStarter,
    TransformKind::GitIgnore,
    TransformKind::PackageManager,
    TransformKind::UpdateCommandsInReadme,
];

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
}

impl fmt::Display for TransformFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
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
