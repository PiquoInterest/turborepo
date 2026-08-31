#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RED = "9d6426ae91f810e093466817ff581f7bc7a5d9cc"
GREEN = "7b208824412f008a942567faa5e37740948a541e"


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"expected one anchor in {path}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, addition.rstrip() + "\n\n" + anchor)


def write_ledger() -> None:
    path = ROOT / "packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md"
    if path.exists():
        raise SystemExit("transform pipeline divergence ledger already exists")
    path.write_text(f"""# Transform-pipeline parity and divergence ledger

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

- RED integration commit: `{RED}`.
- GREEN integration commit: `{GREEN}`.
- The earlier staging branch was reviewed but not merged because its workflow failed formatting and its helpers violated the repository `expect_used` policy.
- Parity tests: `tests/transform_pipeline_parity.rs`.
- Security tests: `tests/transform_pipeline_security.rs`.
""", encoding="utf-8")


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(path,
        "6. the dependency-injected `official-starter` transform orchestration contract.",
        "6. the dependency-injected `official-starter` transform orchestration contract.\n7. the fixed-order transform-pipeline and fatal/nonfatal error-control contract.")
    section = """### Transform-pipeline core

- exports the exact four-transform source order as a closed enum array;
- skips every invocation when transforms are disabled;
- executes each step sequentially and at most once;
- preserves JavaScript string truthiness for `maintainedByCoreTeam`;
- continues after nonfatal transform errors, but stops on fatal and unknown errors;
- preserves the source error defaults and explicit false/empty values;
- returns a typed partial report instead of logging, exiting, or rethrowing inside the core.

Logging, telemetry, async adaptation, `process.exit(1)` mapping, and public JavaScript error construction remain binding work. The binding must sanitize terminal control characters for display and must flush telemetry and cleanup before a fatal exit. Exact differences are recorded in [`TRANSFORM_PIPELINE_DIVERGENCES.md`](./TRANSFORM_PIPELINE_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", section)
    replace_once(path,
        "- transform dispatcher binding and public `TransformError` mapping;",
        "- transform-pipeline async binding, terminal-safe logging, telemetry, fatal-exit handling, and public `TransformError` mapping;")
    replace_once(path,
        "`official_starter` owns exact official-repository classification and transform ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`.",
        "`official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow.")
    replace_once(path,
        "Official starter GREEN: cd2ba74b3040e654a63c9799e42c35a12f2c4dbc",
        f"Official starter GREEN: cd2ba74b3040e654a63c9799e42c35a12f2c4dbc\nPipeline RED:            {RED}\nPipeline GREEN:          {GREEN}")
    replace_once(path,
        "The crate contains 55 translated parity tests and 39 security regression tests, for 94 authored focused Rust tests.",
        "The crate contains 65 translated parity tests and 46 security regression tests, for 111 authored focused Rust tests.")


def update_parity() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    section = """## Transform-pipeline tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| exported transform array order | `TRANSFORM_PIPELINE` | implemented core | Exact four names and source order are translated. |
| `skipTransforms` guard | early empty report | implemented core | No executor method can run. |
| sequential `await` loop | one typed call per enum slot | implemented core | Async host bridge remains blocked. |
| optional string maintainer truthiness | non-empty `Option<String>` | implemented core | Empty is falsey; all non-empty strings are truthy. |
| nonfatal `TransformError` | recorded failure then continue | implemented core | Later steps still execute once. |
| fatal `TransformError` | typed fatal abort | implemented core | Later transforms are not invoked. |
| unknown error rethrow | typed unknown abort | implemented core | Unknown errors cannot be downgraded. |
| default and explicit error options | `TransformFailure` constructors | implemented core | `unknown`/true defaults and explicit empty/false values match nullish semantics. |
| logging, telemetry, exit and async adaptation | production host binding | blocked | Requires exact side effects, cleanup-before-exit, terminal-safe display, and platform differentials. |
| internal partial report | `TransformPipelineReport` | intentional-hardening | Bounded internal observability, not a new public API contract. |

Detailed differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md`."""
    insert_before(path, "## Existing TypeScript test mapping", section)
    replace_once(path,
        "| official-route confusable/large-input/provider-boundary regressions | nine security tests | intentional-hardening evidence |",
        "| official-route confusable/large-input/provider-boundary regressions | nine security tests | intentional-hardening evidence |\n| create command transform-loop source contract | ten translated parity tests | implemented core |\n| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |")
    replace_once(path,
        "| `official-starter` transform | implemented orchestration core, provider blocked |",
        "| transform pipeline and error handling | implemented core, binding blocked | Add async host bridge, telemetry, terminal-safe logging, fatal-exit cleanup, JavaScript error mapping, platform differentials, and removal proof. |\n| `official-starter` transform | implemented orchestration core, provider blocked |")


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    replace_once(path,
        "- `packages/create-turbo/src/transforms/official-starter.ts`",
        "- `packages/create-turbo/src/transforms/official-starter.ts`\n- the transform loop and `handleErrors` in `packages/create-turbo/src/commands/create/index.ts`")
    trust = """The transform pipeline decides which mutation stages run, whether later stages continue, and whether a failure terminates the command. The Rust core uses a closed four-step enum and typed error classes. It performs no logging, telemetry, process exit, filesystem access, or transform side effect directly."""
    insert_before(path, "The Git initialization tranche adds decision boundaries", trust)
    findings = """### CT-RS-022: Mutable or extensible transform routing can broaden trusted execution

**Severity:** Medium

The source exports a fixed array, but a loose port could accept arbitrary names, duplicate stages, or retries. The Rust core uses a closed four-variant enum and fixed array. Tests prove exact order, no calls when skipped, at-most-once invocation, and a four-failure upper bound.

### CT-RS-023: Fatal `process.exit` can bypass cleanup and telemetry flush

**Severity:** Medium

The TypeScript handler logs and calls `process.exit(1)` for fatal transform errors. Immediate process termination can bypass caller cleanup or buffered telemetry. Rust returns a typed fatal abort. The production binding must emit the exact user-visible failure and telemetry once, flush and clean up, then return exit code 1. This is an intentional security and reliability divergence.

### CT-RS-024: Raw error text can inject terminal controls

**Severity:** Medium

The TypeScript handler sends error text through terminal coloring without a control-character policy. The Rust core never logs. The future binding must sanitize controls and directionality characters for terminal display while preserving raw structured diagnostics. Unknown errors must remain unknown and must not inherit nonfatal handling.

The core adds no dependencies or side-effect capability, so it introduces no new advisory surface."""
    insert_before(path, "## Security invariants", findings)


def update_repository_docs() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(path,
        "- `packages/create-turbo/rust`: 55 translated parity tests and 39 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager transform orchestration, and official-starter orchestration.",
        "- `packages/create-turbo/rust`: 65 translated parity tests and 46 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager and official-starter orchestration, and transform-pipeline control flow.")
    replace_once(path, "That is **254 authored Rust migration tests** on the integration branch.", "That is **271 authored Rust migration tests** on the integration branch.")
    replace_once(path, "the evidence-weighted estimate is now about **72%**", "the evidence-weighted estimate is now about **74%**")
    section = """### Transform-pipeline orchestration

The Rust core ports the exact fixed transform order, skip behavior, JavaScript string truthiness, and fatal/nonfatal/unknown error control from the create command. It uses a closed enum and bounded internal report and cannot log, exit, execute a process, access the network, or mutate files.

The async JavaScript binding remains blocked until it proves exact argument forwarding, exactly-once telemetry, terminal-safe error display, cleanup and flush before fatal exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and removal proof. The full divergence ledger is `packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md`.
"""
    insert_before(path, "### Package-manager transform orchestration", section)
    replace_once(path,
        "- official-starter transform implementation: `cd2ba74b3040e654a63c9799e42c35a12f2c4dbc`.",
        f"- official-starter transform implementation: `cd2ba74b3040e654a63c9799e42c35a12f2c4dbc`.\n- transform-pipeline RED: `{RED}`.\n- transform-pipeline implementation: `{GREEN}`.")

    path = "docs/rust-migration-security-findings.md"
    finding = """### RF-016: Transform-loop termination and terminal output require a secure host boundary

**Status:** Fixed-order Rust core implemented; production binding blocked.

The TypeScript create command runs four transforms sequentially, treats nonfatal `TransformError` values as recoverable, exits immediately on fatal transform errors, and rethrows unknown errors. Raw error text is sent to terminal formatting, while telemetry is a separate side effect.

The Rust core closes routing to four enum variants, bounds each to one invocation, preserves exact error defaults and string truthiness, and returns typed partial progress instead of logging or exiting. Production closure requires exact async forwarding, exactly-once telemetry, terminal-control-safe display, cleanup and flush before exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and TypeScript removal proof.

Regression evidence is in `packages/create-turbo/rust/tests/transform_pipeline_parity.rs` and `transform_pipeline_security.rs`; exact differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md`."""
    insert_before(path, "## Required repository gates", finding)
    replace_once(path,
        "- close the official-starter bounded JSON, truthiness, no-follow identity, deterministic ordering, atomic publication, and supported-platform provider contract;",
        "- close the official-starter bounded JSON, truthiness, no-follow identity, deterministic ordering, atomic publication, and supported-platform provider contract;\n- close the transform-pipeline async binding, telemetry, terminal-safe logging, cleanup-before-exit, runtime typing, and supported-platform differential contract;")


def main() -> None:
    write_ledger()
    update_readme()
    update_parity()
    update_security()
    update_repository_docs()
    (ROOT / "tools/migration/apply_transform_pipeline_docs.py").unlink()


if __name__ == "__main__":
    main()
