# Transform-pipeline parity and divergence ledger

TypeScript oracles:

- `packages/create-turbo/src/transforms/index.ts`
- `packages/create-turbo/src/transforms/errors.ts`
- `packages/create-turbo/src/transforms/types.ts`
- the transform loop and `handleErrors` in `packages/create-turbo/src/commands/create/index.ts`

Rust target: `packages/create-turbo/rust/src/transform_pipeline.rs`.

## Preserved behavior

- The transform order is exactly `official-starter`, `git-ignore`, `package-manager`, then `update-commands-in-readme`.
- `skipTransforms` invokes no transform.
- Steps execute sequentially and at most once.
- An empty maintainer string is falsey; every non-empty string, including `false` and whitespace, is truthy like JavaScript.
- Nonfatal `TransformError` values are recorded and later transforms continue.
- Fatal `TransformError` values stop the pipeline before later transforms.
- Unknown errors stop the pipeline and are never downgraded.
- Default error metadata is transform `unknown` and `fatal: true`; explicit empty transform and `fatal: false` values are preserved.

## Divergences and type conversions

| Area | TypeScript behavior | Rust behavior | Classification and reason |
| --- | --- | --- | --- |
| Transform collection | Array of function values | Closed `TransformKind` enum and fixed four-element array | Parity plus hardening. Prevents runtime injection or mutation of the reviewed pipeline. |
| Promise loop | Sequential `await` in the CLI | Synchronous dependency-injected core | Representation only. The host binding still owns async adaptation. |
| Maintainer truthiness | Optional JavaScript string truthiness | `Option<String>` and non-empty check | Type conversion. It exactly models the declared string contract without generic coercion. |
| Wrong runtime metadata types | JavaScript could receive an unexpected value despite the TypeScript declaration | Production adapter will reject non-string values | Intentional type-validation hardening. Coercion could mark attacker-controlled metadata as trusted. |
| `instanceof TransformError` | Runtime class check | Closed `TransformInvocationError` enum | Representation plus hardening. Unknown errors cannot inherit nonfatal behavior. |
| Fatal error | `handleErrors` logs, tracks telemetry, then calls `process.exit(1)` | Typed `PipelineAbort` returned to the host | Intentional control-flow hardening. The host can flush telemetry and perform cleanup before returning exit code 1. |
| Unknown error | Re-thrown | `PipelineAbortReason::Unknown` | Representation only. The binding must rethrow or propagate it unchanged. |
| Error output | Raw message is passed through terminal coloring | Core performs no logging | Intentional security boundary. The host must sanitize terminal controls for display while retaining raw structured diagnostics. |
| Telemetry | One error status event per caught error | Bounded `caught_error_count` and typed failures | Internal evidence only. The binding must emit exactly-once telemetry. |
| Intermediate state | Not exposed by the TypeScript function | Internal report contains responses, nonfatal failures, and partial progress | Internal observability divergence. It is not part of the public package contract. |
| Retry and fan-out | No retries in source | Four fixed slots, each invoked at most once | Parity plus resource hardening. |

## Security and production-binding requirements

The core adds no crate, parser, filesystem access, network call, subprocess, logger, unsafe code, or mutable global state. Production activation remains blocked until the host binding proves:

1. exact sequential async invocation and transform argument forwarding;
2. exact JavaScript `TransformError` construction and unknown-error propagation;
3. exactly-once telemetry for every caught error;
4. terminal-control-safe logging without changing structured error data;
5. cleanup and telemetry flush before fatal exit code 1;
6. strict string validation for `maintainedByCoreTeam`;
7. Linux, macOS, and Windows differential fixtures;
8. no executable TypeScript pipeline logic is loaded or shipped after cutover.

## TDD evidence

- RED integration commit: `9d6426ae91f810e093466817ff581f7bc7a5d9cc`.
- GREEN integration commit: `7b208824412f008a942567faa5e37740948a541e`.
- The earlier staging branch was reviewed but not merged because its workflow failed formatting and its helpers violated the repository `expect_used` policy.
- Parity tests: `tests/transform_pipeline_parity.rs`.
- Security tests: `tests/transform_pipeline_security.rs`.
