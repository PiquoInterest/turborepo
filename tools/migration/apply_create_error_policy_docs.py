#!/usr/bin/env python3
"""Record the reviewed create-command error-policy Rust migration tranche."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
DOC_WORKFLOW = ROOT / ".github/workflows/apply-create-error-policy-docs.yml"
VALIDATION_WORKFLOW = ROOT / ".github/workflows/validate-create-turbo-error-policy.yml"

RED_SHA = "ae46b703826d866d21b5acd64fd681c0d9313e10"
GREEN_SHA = "de9be3378d3eba70ffd105bdc9692f60c6b9cc48"
FORMAT_SHA = "13b34c6ddebbd938f0985c9201363934a2c5385a"


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:160]!r}"
        )
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def write_divergence_ledger() -> None:
    target = ROOT / "packages/create-turbo/rust/CREATE_ERROR_POLICY_DIVERGENCES.md"
    if target.exists():
        raise SystemExit(f"refusing to replace existing divergence ledger: {target}")
    target.write_text(
        f"""# Create-command error policy parity and divergence ledger

## Scope and evidence

TypeScript oracle:

- `packages/create-turbo/src/commands/create/index.ts`, especially `handleErrors`;
- `packages/create-turbo/src/transforms/errors.ts`;
- the existing download-error behavior in `packages/create-turbo/__tests__/index.test.ts`;
- explicit expected-failure security evidence in `packages/create-turbo/__tests__/create-error-security.test.ts`.

Rust target:

- `packages/create-turbo/rust/src/create_error_policy.rs`;
- `packages/create-turbo/rust/tests/create_error_policy_parity.rs`;
- `packages/create-turbo/rust/tests/create_error_policy_security.rs`.

TDD evidence:

- corrected RED contract: `{RED_SHA}`;
- GREEN implementation: `{GREEN_SHA}`;
- committed-format proof: `{FORMAT_SHA}`.

The Rust module is a decision and display-policy core. It does not log, terminate a process, access telemetry, expose stack traces, or replace the production TypeScript command. Those effects remain host-binding work.

## Preserved observable behavior

- Every caught error requests the create-command error telemetry status.
- A nonfatal transform failure emits one labeled line and continues.
- A fatal transform failure emits one labeled line and requests exit code `1`.
- A known conversion failure emits one unlabeled line and requests exit code `1`.
- An unknown conversion failure is rethrown without being displayed.
- A download failure emits the established generic heading followed by the provider message, then requests exit code `1`.
- An unknown error is rethrown without being displayed.
- Safe printable transform labels and messages are preserved exactly.

## Intentional divergences

| Boundary | TypeScript behavior | Rust behavior | Classification | Reason |
| --- | --- | --- | --- | --- |
| Fatal termination | `process.exit(1)` occurs inside `handleErrors` | returns `CreateCommandErrorAction::Exit(1)` | security and reliability divergence | Lets the host finish logging, cleanup, and telemetry flush before terminating. |
| Error classification | runtime `instanceof`, mutable object properties, and string error types | closed enums and typed fields | representation and type conversion | Prevents message text or malformed runtime objects from changing fatality or error class. |
| Terminal controls | error and transform text is sent through color formatting without a control policy | escapes C0/C1 controls, ESC, BEL, CR/LF/TAB, bidi controls, zero-width format controls, and related terminal-active characters | security divergence | Prevents forged lines, cursor rewrites, OSC hyperlinks, terminal state changes, and directionality spoofing. |
| Output size | provider-controlled text is unbounded | messages are limited to 4096 UTF-8 bytes and labels to 256 UTF-8 bytes | security divergence | Bounds terminal flooding, memory use, and log amplification. |
| Truncation | no explicit policy | removes complete emitted fragments and appends `[truncated]` without splitting UTF-8 or an escape representation | security hardening | Keeps the bound deterministic and output valid. |
| Multiline text | raw newlines, carriage returns, and tabs retain terminal semantics | represented as `\\n`, `\\r`, and `\\t` | security divergence | A single logical error cannot fabricate additional terminal records. |
| Unknown errors | rethrown without the handler displaying them | typed source is returned in `Rethrow` with no display lines | parity plus least disclosure | Preserves propagation and avoids accidental secret or control-text disclosure. |
| Error stack/class | JavaScript `Error.captureStackTrace` and `TransformError` identity | not constructed by the Rust core | binding gap | The host adapter must preserve the established public JavaScript contract where still required. |
| Side effects | handler logs, records telemetry, and may terminate | core returns display lines, telemetry intent, and an action | provider boundary | Makes effect ordering testable and prevents hidden process termination. |

## Legacy TypeScript security evidence

The two Jest cases use `it.failing` so they remain executable evidence while TypeScript is still the oracle:

1. download-error text can currently pass terminal-control and directionality characters to stderr;
2. download-error text can currently exceed the 4096-byte Rust display bound.

They must not be converted into ordinary passing legacy tests unless the TypeScript implementation is deliberately hardened. When production routing moves to Rust, shared host-level fixtures must prove the safe-input contract and the documented hostile-input divergence.

## Security invariants

- No raw terminal-control or Unicode directionality/format character leaves the Rust display-policy boundary.
- Every displayed field has an explicit UTF-8 byte bound.
- Truncation never splits a Unicode scalar or an emitted escape fragment.
- Error message content cannot alter fatality, classification, telemetry intent, or exit code.
- Unknown errors are never rendered by this layer.
- The core performs no logging, process exit, process execution, filesystem access, network access, or credential access.
- This tranche adds no dependency, `unsafe` code, parser, subprocess, or mutable global state.

## Production-binding requirements

Before this module can replace `handleErrors`, the host binding must prove:

1. exact mapping from JavaScript `TransformError`, known/unknown `ConvertError`, `DownloadError`, and unknown values into the typed Rust input;
2. exactly-once telemetry status emission;
3. ordered terminal emission using only the already-sanitized display fields;
4. cleanup and telemetry flush before applying a requested exit code;
5. preservation of unknown-error identity and stack behavior where the JavaScript host requires it;
6. no second raw-error logging path that bypasses the Rust policy;
7. Linux, macOS, and Windows differential fixtures for safe text, hostile controls, Unicode, limits, and exit behavior;
8. removal proof showing the TypeScript handler is neither loaded nor shipped after cutover.
""",
        encoding="utf-8",
    )


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "8. the package-manager prompt resolution and installed-choice ordering contract.",
        "8. the package-manager prompt resolution and installed-choice ordering contract.\n"
        "9. the create-command error-classification and terminal-safe display-policy core.",
    )

    section = """### Create-command error policy core

- preserves the TypeScript `handleErrors` split between nonfatal and fatal transform failures, known and unknown conversion failures, download failures, and unknown errors;
- requests error telemetry for every caught value while leaving telemetry emission to the host;
- preserves the established two-line download failure, transform labels, safe printable text, and exit code `1` decisions;
- returns typed `Continue`, `Exit`, or `Rethrow` actions instead of logging or terminating inside the core;
- escapes terminal controls, OSC/BEL content, carriage returns, line breaks, tabs, bidi controls, zero-width format controls, and related terminal-active Unicode;
- bounds error messages to 4096 UTF-8 bytes and transform labels to 256 UTF-8 bytes;
- truncates only at complete emitted-fragment boundaries and appends `[truncated]`;
- never displays unknown errors and does not allow message text to change classification or fatality.

Two Jest `it.failing` cases keep the legacy TypeScript terminal-injection and unbounded-output defects executable as security evidence. The Rust core adds no dependencies, logging, process exit, parser, filesystem, network, or unsafe capability.

Host binding remains blocked until it proves exact runtime error conversion, ordered sanitized output, exactly-once telemetry, cleanup and flush before applying exit code `1`, unknown-error identity, supported-platform differentials, and TypeScript removal. Exact differences are recorded in [`CREATE_ERROR_POLICY_DIVERGENCES.md`](./CREATE_ERROR_POLICY_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", section)

    replace_once(
        path,
        "- transform-pipeline async binding, terminal-safe logging, telemetry, fatal-exit handling, and public `TransformError` mapping;",
        "- transform-pipeline and create-error-policy host binding, including async adaptation, terminal emission, exactly-once telemetry, cleanup/flush before applying exit code `1`, and public `TransformError` mapping;",
    )

    replace_once(
        path,
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation.",
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation. `create_error_policy` owns error classification, bounded terminal-safe display fields, telemetry intent, and typed continue/exit/rethrow decisions without performing those effects.",
    )

    replace_once(
        path,
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c",
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c\n"
        f"Create error policy RED: {RED_SHA}\n"
        f"Create error policy GREEN: {GREEN_SHA}\n"
        f"Create error rustfmt proof: {FORMAT_SHA}",
    )

    replace_once(
        path,
        "The crate contains 73 translated parity tests and 51 security regression tests, for 124 authored focused Rust tests.",
        "The crate contains 80 translated parity tests and 59 security regression tests, for 139 authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    section = """## Create-command error policy tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| every caught error records create-command error telemetry | `track_error_status: true` | implemented core | The host still owns exactly-once telemetry emission. |
| nonfatal `TransformError` logs one labeled line and continues | typed display line plus `Continue` | implemented core | Safe text, label, and control flow are translated. |
| fatal `TransformError` logs then calls `process.exit(1)` | typed display line plus `Exit(1)` | intentional-hardening | Preserves the exit decision while allowing cleanup and telemetry flush first. |
| known `ConvertError` logs its message and exits | unlabeled display line plus `Exit(1)` | implemented core | Unknown conversion errors remain a separate branch. |
| unknown `ConvertError` is rethrown | `Rethrow(source)` without display | implemented core | Cannot inherit known/nonfatal handling. |
| `DownloadError` logs a fixed heading then provider message and exits | two ordered display lines plus `Exit(1)` | implemented core | Existing safe-input output order and text are covered. |
| unknown errors are rethrown | typed `Rethrow(source)` without display | implemented core | Unknown payloads are not exposed by this layer. |
| raw terminal-colored error text | control and format characters escaped | intentional-hardening | Blocks cursor rewrites, forged lines, OSC hyperlinks, BEL, bidi spoofing, and zero-width formatting. |
| unbounded provider error text | 4096-byte message and 256-byte label limits | intentional-hardening | Bounds memory, terminal flooding, and log amplification without splitting UTF-8. |
| JavaScript runtime classes and mutable fields | closed Rust enums and typed fatality | representation-only for valid inputs | Message content cannot alter error class or fatality. |
| logging, telemetry, stack/class construction, cleanup, and termination | production host binding | blocked | Must consume only sanitized fields, flush before exit, preserve unknown identity, and pass platform differentials. |

Detailed behavior and security differences are in `CREATE_ERROR_POLICY_DIVERGENCES.md`."""
    insert_before(path, "## Existing TypeScript test mapping", section)

    replace_once(
        path,
        "| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |",
        "| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |\n"
        "| existing create-command download-error test and `handleErrors` source contract | seven create-error-policy parity tests | implemented core |\n"
        "| TypeScript terminal-control and unbounded-error `it.failing` evidence | eight create-error-policy security tests | intentional-hardening evidence |",
    )

    replace_once(
        path,
        "| transform pipeline and error handling | implemented core, binding blocked | Add async host bridge, telemetry, terminal-safe logging, fatal-exit cleanup, JavaScript error mapping, platform differentials, and removal proof. |",
        "| transform pipeline and error handling | pipeline and terminal-safe error-policy cores implemented, binding blocked | Add async host bridge, ordered display emission, exactly-once telemetry, cleanup/flush before applying exit code `1`, JavaScript error mapping, platform differentials, and removal proof. |",
    )


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"

    insert_before(
        path,
        "The Git initialization tranche adds decision boundaries for the project-root path, Git and Mercurial executable selection, process working directory, arguments, inherited environment and VCS configuration, template directories, hooks, timeouts, output, child-process cleanup, `.git` ownership, and recursive deletion.",
        "The create-command error-policy tranche accepts attacker-influenced transform names and provider messages at a terminal boundary. The Rust core converts them into bounded, control-safe display fields and typed telemetry/continue/exit/rethrow decisions. It cannot log, expose unknown errors, or terminate the process directly.",
    )

    old_findings = """### CT-RS-023: Fatal `process.exit` can bypass cleanup and telemetry flush

**Severity:** Medium

The TypeScript handler logs and calls `process.exit(1)` for fatal transform errors. Immediate process termination can bypass caller cleanup or buffered telemetry. Rust returns a typed fatal abort. The production binding must emit the exact user-visible failure and telemetry once, flush and clean up, then return exit code 1. This is an intentional security and reliability divergence.

### CT-RS-024: Raw error text can inject terminal controls

**Severity:** Medium

The TypeScript handler sends error text through terminal coloring without a control-character policy. The Rust core never logs. The future binding must sanitize controls and directionality characters for terminal display while preserving raw structured diagnostics. Unknown errors must remain unknown and must not inherit nonfatal handling.
"""
    new_findings = """### CT-RS-023: Fatal `process.exit` can bypass cleanup and telemetry flush

**Severity:** Medium

The TypeScript handler logs and calls `process.exit(1)` for fatal transform, known conversion, and download failures. Immediate termination can bypass caller cleanup or buffered telemetry.

The transform pipeline and create-error-policy cores now return typed abort or `Exit(1)` decisions rather than terminating. `CreateCommandErrorOutcome` also carries explicit error-telemetry intent, so the production binding can emit sanitized lines, record telemetry exactly once, flush and clean up, and only then apply the exit code.

Regression tests cover every continue/exit/rethrow branch and prove that hostile message text cannot alter fatality. The remaining risk is entirely at the host boundary: a binding that calls `process.exit` early or logs the raw source error would reintroduce the defect.

### CT-RS-024: Raw and unbounded error text can control or flood the terminal

**Severity:** Medium

The TypeScript handler sends transform labels and provider error messages through terminal coloring without a control-character or size policy. Two Jest `it.failing` cases prove that download failures currently pass OSC/ESC/BEL, carriage-return, and directionality controls to stderr and can exceed 4096 bytes.

Rust escapes line controls, C0/C1 controls, OSC/BEL bytes, Unicode bidi controls, zero-width format controls, and related terminal-active characters. Messages are bounded to 4096 UTF-8 bytes and labels to 256 UTF-8 bytes. Truncation removes complete emitted fragments and appends `[truncated]`, so it cannot split UTF-8 or an escape representation. Unknown errors produce no display fields.

Regression tests cover controls, directionality, bounds, multibyte truncation, classification integrity, and least disclosure. The production binding must consume only the sanitized fields and must not add a second raw-error logging path.
"""
    replace_once(path, old_findings, new_findings)

    replace_once(
        path,
        "- No new `unsafe` or shell command construction is introduced by these tranches.",
        "- No new `unsafe` or shell command construction is introduced by these tranches.\n"
        "- Create-command display fields contain no raw terminal-control or Unicode directionality/format characters and have explicit UTF-8 byte limits.\n"
        "- Error text cannot change typed fatality, classification, telemetry intent, or exit code.\n"
        "- Unknown errors are rethrown without being displayed by the Rust policy core.",
    )

    replace_once(
        path,
        "Disposition:\n\n- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
        "Disposition:\n\n"
        "- The create-command error-policy tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, process exit, unsafe code, or mutable global state.\n"
        "- The TypeScript terminal-control and unbounded-output findings remain executable as expected-failure tests until production cutover or deliberate TypeScript hardening.\n"
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
    )

    replace_once(
        path,
        "- map typed failures to the existing JavaScript public contracts;",
        "- bind typed create-command failures to the existing JavaScript public contracts without bypassing sanitized fields, exactly-once telemetry, cleanup, or flush-before-exit ordering;",
    )


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 73 translated parity tests and 51 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, and transform-pipeline control flow.",
        "- `packages/create-turbo/rust`: 80 translated parity tests and 59 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, transform-pipeline control flow, and create-command error policy.",
    )
    replace_once(
        path,
        "That is **284 authored Rust migration tests** on the integration branch.",
        "That is **299 authored Rust migration tests** on the integration branch.",
    )

    replace_once(
        path,
        "The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **75%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
        "The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The create-command error-policy tranche advances create-turbo core and security-test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **76%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
    )

    replace_once(
        path,
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |",
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, official-starter orchestration, transform-pipeline, and create-error-policy cores are ported. CLI, discovery/acquisition, production VCS/converter/JSON providers, host binding, terminal emission/telemetry/exit ordering, remaining orchestration, packaging, callers, and removal proof remain. |",
    )

    section = """### Create-command error classification and terminal display policy

The Rust core preserves `handleErrors` classification for nonfatal/fatal transform failures, known/unknown conversion failures, download failures, and unknown errors. It returns typed display lines, telemetry intent, and `Continue`, `Exit(1)`, or `Rethrow` actions rather than performing hidden side effects.

Security closure in the Rust core:

- terminal controls, OSC/BEL content, carriage returns, line breaks, tabs, bidi controls, zero-width format controls, and related terminal-active Unicode are escaped;
- messages are bounded to 4096 UTF-8 bytes and labels to 256 UTF-8 bytes;
- truncation cannot split UTF-8 or an emitted escape representation;
- message text cannot alter classification, fatality, telemetry intent, or exit code;
- unknown errors are not rendered;
- no logging, telemetry, process exit, process execution, filesystem, network, dependency, or unsafe capability is added.

Two Jest `it.failing` tests retain executable proof that the legacy TypeScript path currently permits terminal-control injection and unbounded error output. Production host binding still must map runtime classes exactly, emit only sanitized fields, record telemetry once, flush and clean up before applying exit code `1`, preserve unknown-error identity, run Linux/macOS/Windows differentials, and prove the TypeScript handler is no longer loaded or shipped. Exact differences are in `packages/create-turbo/rust/CREATE_ERROR_POLICY_DIVERGENCES.md`."""
    insert_before(path, "### Package-manager transform orchestration", section)

    replace_once(
        path,
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.",
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.\n"
        f"- create-command error-policy corrected RED: `{RED_SHA}`.\n"
        f"- create-command error-policy implementation: `{GREEN_SHA}`.\n"
        f"- create-command error-policy committed-format proof: `{FORMAT_SHA}`.",
    )


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    replace_once(
        path,
        "**Status:** Fixed-order Rust core implemented; production binding blocked.",
        "**Status:** Fixed-order and terminal-safe error-policy Rust cores implemented; production binding blocked.",
    )
    replace_once(
        path,
        "The Rust core closes routing to four enum variants, bounds each to one invocation, preserves exact error defaults and string truthiness, and returns typed partial progress instead of logging or exiting. Production closure requires exact async forwarding, exactly-once telemetry, terminal-control-safe display, cleanup and flush before exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and TypeScript removal proof.",
        "The Rust pipeline closes routing to four enum variants, bounds each to one invocation, and preserves exact error defaults and string truthiness. The create-error-policy core now also preserves the source classification while returning bounded terminal-safe display fields, telemetry intent, and typed continue/exit/rethrow actions instead of logging or terminating. Production closure requires exact async/runtime conversion, exactly-once telemetry, emission of sanitized fields only, cleanup and flush before exit code 1, unknown-error identity, supported-platform differentials, and TypeScript removal proof.",
    )
    replace_once(
        path,
        "Regression evidence is in `packages/create-turbo/rust/tests/transform_pipeline_parity.rs` and `transform_pipeline_security.rs`; exact differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md`.",
        "Regression evidence is in the transform-pipeline and create-error-policy parity/security tests. The legacy terminal-injection and unbounded-output defects remain executable in `packages/create-turbo/__tests__/create-error-security.test.ts`. Exact differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md` and `CREATE_ERROR_POLICY_DIVERGENCES.md`.",
    )

    finding = """### RF-018: Create-command error text permits terminal injection and unbounded output

**Status:** Fixed in the Rust display-policy core; TypeScript production path and host binding remain.

The TypeScript create command sends transform labels and provider messages through terminal coloring without escaping terminal-control, carriage-return, directionality, or zero-width format characters and without an explicit size limit. It also calls `process.exit(1)` inside the handler for several fatal classes. This permits forged or rewritten terminal output, OSC hyperlink/state manipulation, log amplification, and cleanup or telemetry loss on immediate termination.

Two Jest `it.failing` cases preserve executable evidence of the legacy control-character and unbounded-output behavior. The Rust core:

- escapes terminal-active controls and Unicode format/directionality characters;
- limits messages to 4096 UTF-8 bytes and labels to 256 bytes;
- truncates without splitting UTF-8 or emitted escape fragments;
- keeps fatality and classification in typed fields;
- never displays unknown errors;
- returns telemetry intent and `Continue`, `Exit(1)`, or `Rethrow` instead of performing side effects.

The implementation adds no dependency, unsafe code, logger, subprocess, filesystem, network, or credential capability. Production closure requires an exact JavaScript/Rust error adapter, sanitized-field-only output, exactly-once telemetry, cleanup and flush before exit, platform differentials, downstream routing, and artifact/removal proof."""
    insert_before(path, "## Required repository gates", finding)

    replace_once(
        path,
        "- close the transform-pipeline async binding, telemetry, terminal-safe logging, cleanup-before-exit, runtime typing, and supported-platform differential contract;",
        "- close the transform-pipeline and create-error-policy host binding, runtime typing, sanitized-field-only output, exactly-once telemetry, cleanup/flush-before-exit, and supported-platform differential contract;",
    )


def remove_one_shot_automation() -> None:
    for path in (DOC_WORKFLOW, VALIDATION_WORKFLOW, SELF):
        if not path.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {path}")
        path.unlink()


def main() -> None:
    write_divergence_ledger()
    update_readme()
    update_parity_matrix()
    update_security()
    update_program_ledger()
    update_repository_findings()
    remove_one_shot_automation()


if __name__ == "__main__":
    main()
