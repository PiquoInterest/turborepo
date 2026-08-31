use std::collections::VecDeque;

use create_turbo_rs::{
    PipelineAbortReason, PipelineTransformResponse, TransformExecutor, TransformFailure,
    TransformInvocationError, TransformKind, TransformStatus, TRANSFORM_PIPELINE,
    run_transform_pipeline,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnknownError(&'static str);

#[derive(Debug)]
struct ScriptedExecutor {
    expected: VecDeque<TransformKind>,
    results: VecDeque<Result<PipelineTransformResponse, TransformInvocationError<UnknownError>>>,
    calls: Vec<TransformKind>,
}

impl ScriptedExecutor {
    fn new(
        expected: impl IntoIterator<Item = TransformKind>,
        results: impl IntoIterator<
            Item = Result<PipelineTransformResponse, TransformInvocationError<UnknownError>>,
        >,
    ) -> Self {
        Self {
            expected: expected.into_iter().collect(),
            results: results.into_iter().collect(),
            calls: Vec::new(),
        }
    }
}

impl TransformExecutor<UnknownError> for ScriptedExecutor {
    fn execute(
        &mut self,
        transform: TransformKind,
    ) -> Result<PipelineTransformResponse, TransformInvocationError<UnknownError>> {
        assert_eq!(self.expected.pop_front(), Some(transform));
        self.calls.push(transform);
        self.results
            .pop_front()
            .expect("the test script must provide one result per expected call")
    }
}

fn response(
    name: &'static str,
    result: TransformStatus,
    maintained_by_core_team: Option<&str>,
) -> PipelineTransformResponse {
    PipelineTransformResponse {
        result,
        name,
        maintained_by_core_team: maintained_by_core_team.map(str::to_owned),
    }
}

#[test]
fn exports_the_exact_typescript_transform_order() {
    assert_eq!(
        TRANSFORM_PIPELINE,
        [
            TransformKind::OfficialStarter,
            TransformKind::GitIgnore,
            TransformKind::PackageManager,
            TransformKind::UpdateCommandsInReadme,
        ]
    );
    assert_eq!(
        TRANSFORM_PIPELINE.map(TransformKind::as_str),
        [
            "official-starter",
            "git-ignore",
            "package-manager",
            "update-commands-in-readme",
        ]
    );
}

#[test]
fn skipped_transforms_do_not_invoke_any_step() {
    let mut executor = ScriptedExecutor::new([], []);
    let report = run_transform_pipeline(&mut executor, true).expect("skip should succeed");

    assert!(executor.calls.is_empty());
    assert!(report.responses.is_empty());
    assert!(report.non_fatal_errors.is_empty());
    assert_eq!(report.caught_error_count, 0);
    assert!(!report.is_maintained_by_core_team);
}

#[test]
fn successful_transforms_run_sequentially_and_preserve_responses() {
    let expected = TRANSFORM_PIPELINE;
    let results = [
        Ok(response("official-starter", TransformStatus::Success, None)),
        Ok(response("git-ignore", TransformStatus::Success, None)),
        Ok(response(
            "package-manager",
            TransformStatus::NotApplicable,
            None,
        )),
        Ok(response(
            "update-commands-in-readme",
            TransformStatus::Success,
            None,
        )),
    ];
    let mut executor = ScriptedExecutor::new(expected, results);
    let report = run_transform_pipeline(&mut executor, false).expect("pipeline should succeed");

    assert_eq!(executor.calls, expected);
    assert_eq!(report.responses.len(), 4);
    assert_eq!(report.responses[2].result, TransformStatus::NotApplicable);
    assert_eq!(report.responses[3].name, "update-commands-in-readme");
    assert_eq!(report.caught_error_count, 0);
}

#[test]
fn any_nonempty_maintainer_string_marks_the_project_as_core_maintained() {
    let results = [
        Ok(response("official-starter", TransformStatus::Success, Some(""))),
        Ok(response("git-ignore", TransformStatus::Success, Some("false"))),
        Ok(response("package-manager", TransformStatus::Success, None)),
        Ok(response(
            "update-commands-in-readme",
            TransformStatus::Success,
            None,
        )),
    ];
    let mut executor = ScriptedExecutor::new(TRANSFORM_PIPELINE, results);
    let report = run_transform_pipeline(&mut executor, false).expect("pipeline should succeed");

    assert!(report.is_maintained_by_core_team);
}

#[test]
fn an_empty_maintainer_string_is_javascript_falsy() {
    let results = TRANSFORM_PIPELINE.map(|kind| {
        Ok(response(
            kind.as_str(),
            TransformStatus::Success,
            (kind == TransformKind::OfficialStarter).then_some(""),
        ))
    });
    let mut executor = ScriptedExecutor::new(TRANSFORM_PIPELINE, results);
    let report = run_transform_pipeline(&mut executor, false).expect("pipeline should succeed");

    assert!(!report.is_maintained_by_core_team);
}

#[test]
fn a_nonfatal_transform_error_is_recorded_and_later_steps_continue() {
    let warning = TransformFailure::with_options(
        "unable to remove metadata",
        Some("official-starter"),
        Some(false),
    );
    let results = [
        Err(TransformInvocationError::Transform(warning.clone())),
        Ok(response("git-ignore", TransformStatus::Success, None)),
        Ok(response("package-manager", TransformStatus::Success, None)),
        Ok(response(
            "update-commands-in-readme",
            TransformStatus::Success,
            None,
        )),
    ];
    let mut executor = ScriptedExecutor::new(TRANSFORM_PIPELINE, results);
    let report = run_transform_pipeline(&mut executor, false).expect("nonfatal errors continue");

    assert_eq!(executor.calls, TRANSFORM_PIPELINE);
    assert_eq!(report.responses.len(), 3);
    assert_eq!(report.non_fatal_errors, vec![warning]);
    assert_eq!(report.caught_error_count, 1);
}

#[test]
fn a_fatal_transform_error_stops_before_later_transforms() {
    let fatal = TransformFailure::with_options(
        "unable to write package.json",
        Some("official-starter"),
        None,
    );
    let expected = [TransformKind::OfficialStarter, TransformKind::GitIgnore];
    let results = [
        Ok(response("official-starter", TransformStatus::Success, None)),
        Err(TransformInvocationError::Transform(fatal.clone())),
    ];
    let mut executor = ScriptedExecutor::new(expected, results);
    let abort = run_transform_pipeline(&mut executor, false)
        .expect_err("fatal transform errors must stop the pipeline");

    assert_eq!(executor.calls, expected);
    assert_eq!(abort.report.responses.len(), 1);
    assert_eq!(abort.report.caught_error_count, 1);
    assert!(abort.report.non_fatal_errors.is_empty());
    assert_eq!(abort.reason, PipelineAbortReason::Fatal(fatal));
}

#[test]
fn an_unknown_error_is_rethrown_and_not_downgraded() {
    let expected = [TransformKind::OfficialStarter];
    let error = UnknownError("programming error");
    let results = [Err(TransformInvocationError::Unknown(error.clone()))];
    let mut executor = ScriptedExecutor::new(expected, results);
    let abort = run_transform_pipeline(&mut executor, false)
        .expect_err("unknown errors must abort the pipeline");

    assert_eq!(executor.calls, expected);
    assert_eq!(abort.report.caught_error_count, 1);
    assert_eq!(abort.reason, PipelineAbortReason::Unknown(error));
}

#[test]
fn transform_failure_defaults_match_the_typescript_error_class() {
    let failure = TransformFailure::new("failed");

    assert_eq!(failure.message, "failed");
    assert_eq!(failure.transform, "unknown");
    assert!(failure.fatal);
    assert_eq!(failure.to_string(), "failed");
}

#[test]
fn explicit_empty_transform_and_false_fatal_values_are_preserved() {
    let failure = TransformFailure::with_options("warning", Some(""), Some(false));

    assert_eq!(failure.transform, "");
    assert!(!failure.fatal);
}
