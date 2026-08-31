use std::collections::VecDeque;

use create_turbo_rs::{
    PipelineAbortReason, PipelineTransformResponse, TRANSFORM_PIPELINE, TransformExecutor,
    TransformFailure, TransformInvocationError, TransformKind, TransformStatus,
    run_transform_pipeline,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnknownError(&'static str);

#[derive(Debug)]
struct CountingExecutor {
    results: VecDeque<Result<PipelineTransformResponse, TransformInvocationError<UnknownError>>>,
    calls: Vec<TransformKind>,
}

impl CountingExecutor {
    fn new(
        results: impl IntoIterator<
            Item = Result<PipelineTransformResponse, TransformInvocationError<UnknownError>>,
        >,
    ) -> Self {
        Self {
            results: results.into_iter().collect(),
            calls: Vec::new(),
        }
    }
}

impl TransformExecutor<UnknownError> for CountingExecutor {
    fn execute(
        &mut self,
        transform: TransformKind,
    ) -> Result<PipelineTransformResponse, TransformInvocationError<UnknownError>> {
        self.calls.push(transform);
        let result = self.results.pop_front();
        let Some(result) = result else {
            panic!("the security script must provide a result for every call");
        };
        result
    }
}

fn success(kind: TransformKind) -> PipelineTransformResponse {
    PipelineTransformResponse {
        result: TransformStatus::Success,
        name: kind.as_str(),
        maintained_by_core_team: None,
    }
}

#[test]
fn every_pipeline_slot_is_invoked_at_most_once() {
    let mut executor = CountingExecutor::new(TRANSFORM_PIPELINE.map(|kind| Ok(success(kind))));
    let report = run_transform_pipeline(&mut executor, false).expect("pipeline should succeed");

    assert_eq!(executor.calls, TRANSFORM_PIPELINE);
    assert_eq!(report.responses.len(), TRANSFORM_PIPELINE.len());
}

#[test]
fn four_nonfatal_failures_are_bounded_by_the_closed_pipeline() {
    let results = TRANSFORM_PIPELINE.map(|kind| {
        Err(TransformInvocationError::Transform(
            TransformFailure::with_options("warning", Some(kind.as_str()), Some(false)),
        ))
    });
    let mut executor = CountingExecutor::new(results);
    let report = run_transform_pipeline(&mut executor, false).expect("nonfatal failures continue");

    assert_eq!(executor.calls.len(), 4);
    assert_eq!(report.non_fatal_errors.len(), 4);
    assert_eq!(report.caught_error_count, 4);
    assert!(report.responses.is_empty());
}

#[test]
fn a_failure_is_never_retried_after_it_returns() {
    let results = [
        Err(TransformInvocationError::Transform(
            TransformFailure::with_options("one warning", Some("official-starter"), Some(false)),
        )),
        Ok(success(TransformKind::GitIgnore)),
        Ok(success(TransformKind::PackageManager)),
        Ok(success(TransformKind::UpdateCommandsInReadme)),
    ];
    let mut executor = CountingExecutor::new(results);
    let _ = run_transform_pipeline(&mut executor, false).expect("nonfatal failure should continue");

    assert_eq!(
        executor
            .calls
            .iter()
            .filter(|kind| **kind == TransformKind::OfficialStarter)
            .count(),
        1
    );
}

#[test]
fn an_unknown_error_after_a_nonfatal_failure_still_aborts() {
    let warning =
        TransformFailure::with_options("recoverable", Some("official-starter"), Some(false));
    let unknown = UnknownError("unexpected type");
    let results = [
        Err(TransformInvocationError::Transform(warning.clone())),
        Err(TransformInvocationError::Unknown(unknown.clone())),
    ];
    let mut executor = CountingExecutor::new(results);
    let abort = run_transform_pipeline(&mut executor, false)
        .expect_err("unknown errors must not inherit nonfatal behavior");

    assert_eq!(executor.calls.len(), 2);
    assert_eq!(abort.report.non_fatal_errors, vec![warning]);
    assert_eq!(abort.report.caught_error_count, 2);
    assert_eq!(abort.reason, PipelineAbortReason::Unknown(unknown));
}

#[test]
fn arbitrary_error_text_is_carried_as_data_without_affecting_control_flow() {
    let message = "\u{001b}[31mfailed\nunknown\u{202e}txt";
    let transform = "../../official-starter\0";
    let failure = TransformFailure::with_options(message, Some(transform), Some(false));
    let results = [
        Err(TransformInvocationError::Transform(failure.clone())),
        Ok(success(TransformKind::GitIgnore)),
        Ok(success(TransformKind::PackageManager)),
        Ok(success(TransformKind::UpdateCommandsInReadme)),
    ];
    let mut executor = CountingExecutor::new(results);
    let report =
        run_transform_pipeline(&mut executor, false).expect("nonfatal failure should continue");

    assert_eq!(report.non_fatal_errors, vec![failure]);
    assert_eq!(executor.calls, TRANSFORM_PIPELINE);
}

#[test]
fn whitespace_only_maintainer_metadata_is_truthy_like_javascript() {
    let results = TRANSFORM_PIPELINE.map(|kind| {
        Ok(PipelineTransformResponse {
            result: TransformStatus::Success,
            name: kind.as_str(),
            maintained_by_core_team: (kind == TransformKind::OfficialStarter)
                .then(|| " \t\n".to_owned()),
        })
    });
    let mut executor = CountingExecutor::new(results);
    let report = run_transform_pipeline(&mut executor, false).expect("pipeline should succeed");

    assert!(report.is_maintained_by_core_team);
}

#[test]
fn a_fatal_failure_cannot_be_hidden_in_the_nonfatal_collection() {
    let fatal = TransformFailure::with_options("fatal", Some("official-starter"), Some(true));
    let mut executor =
        CountingExecutor::new([Err(TransformInvocationError::Transform(fatal.clone()))]);
    let abort =
        run_transform_pipeline(&mut executor, false).expect_err("fatal failures must abort");

    assert!(abort.report.non_fatal_errors.is_empty());
    assert_eq!(abort.reason, PipelineAbortReason::Fatal(fatal));
}
