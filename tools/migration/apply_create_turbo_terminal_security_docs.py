#!/usr/bin/env python3
"""Record the reviewed create-turbo terminal-diagnostic hardening tranche."""

from __future__ import annotations

import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
WORKFLOW = ROOT / ".github/workflows/validate-create-turbo-terminal-security.yml"

RED_SHA = "069a141aad0ccbc77b4480b0d6811876efa6280a"
GREEN_SHA = "cd8821d201f0cf723ba3081156ce639a061af4c1"
FORMATTED_SHA = "dc7de326654e679bfc9b595e3b1089b9cb4aa0cb"
VALIDATED_SHA = os.environ.get("GITHUB_SHA", "unknown")
VALIDATION_RUN = os.environ.get("GITHUB_RUN_ID", "unknown")


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected one reviewed anchor in {path}, found {count}: {old[:120]!r}"
        )
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "8. the package-manager prompt resolution and installed-choice ordering contract.",
        "8. the package-manager prompt resolution and installed-choice ordering contract.\n"
        "9. bounded, terminal-safe transform diagnostics shared by the TypeScript oracle and Rust error renderer.",
    )
    replace_once(
        path,
        "- returns a typed partial report instead of logging, exiting, or rethrowing inside the core.",
        "- returns a typed partial report instead of logging, exiting, or rethrowing inside the core;\n"
        "- sanitizes terminal-facing transform names and messages with a 512-scalar bound while retaining raw structured values;\n"
        "- escapes C0/C1 controls, bidirectional controls, and invisible format characters without changing ordinary printable Unicode.",
    )
    replace_once(
        path,
        "Logging, telemetry, async adaptation, `process.exit(1)` mapping, and public JavaScript error construction remain binding work. The binding must sanitize terminal control characters for display and must flush telemetry and cleanup before a fatal exit. Exact differences are recorded in [`TRANSFORM_PIPELINE_DIVERGENCES.md`](./TRANSFORM_PIPELINE_DIVERGENCES.md).",
        "The TypeScript `TransformError` path and Rust renderer now share the same bounded terminal-field policy, while retaining raw message and transform values for structured diagnostics. Async adaptation, telemetry, `process.exit(1)` mapping, public JavaScript error construction, and terminal-safe handling of `ConvertError`, `DownloadError`, and other untrusted logger fields remain binding work. Exact differences are recorded in [`TRANSFORM_PIPELINE_DIVERGENCES.md`](./TRANSFORM_PIPELINE_DIVERGENCES.md).",
    )
    replace_once(
        path,
        "- transform-pipeline async binding, terminal-safe logging, telemetry, fatal-exit handling, and public `TransformError` mapping;",
        "- transform-pipeline async binding, terminal-safe handling for non-transform errors and remaining untrusted logger fields, telemetry, fatal-exit handling, and public `TransformError` mapping;",
    )
    replace_once(
        path,
        "`transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow.",
        "`transform_pipeline` owns the fixed transform order, typed fatal/nonfatal control flow, and shared bounded terminal-diagnostic renderer.",
    )
    replace_once(
        path,
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c",
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c\n"
        f"Terminal security RED:   {RED_SHA}\n"
        f"Terminal security GREEN: {GREEN_SHA}\n"
        f"Terminal rustfmt:        {FORMATTED_SHA}\n"
        f"Terminal validation:     {VALIDATED_SHA} (Actions run {VALIDATION_RUN})",
    )
    replace_once(
        path,
        "The crate contains 73 translated parity tests and 51 security regression tests, for 124 authored focused Rust tests.",
        "The crate contains 73 translated parity tests and 54 security regression tests, for 127 authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    replace_once(
        path,
        "| logging, telemetry, exit and async adaptation | production host binding | blocked | Requires exact side effects, cleanup-before-exit, terminal-safe display, and platform differentials. |",
        "| `TransformError` terminal-facing message and transform name | TypeScript `sanitizeTerminalText` and Rust `sanitize_terminal_text` | intentional-hardening | Both paths preserve safe printable Unicode, escape C0/C1, bidi, and invisible format characters, stop after one scalar beyond a 512-scalar limit, append `…`, and retain raw structured values. |\n"
        "| logging, telemetry, exit and async adaptation | production host binding | blocked | Requires exact side effects, cleanup-before-exit, terminal-safe handling for `ConvertError`, `DownloadError`, and remaining untrusted logger fields, and platform differentials. |",
    )
    replace_once(
        path,
        "| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |",
        "| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |\n"
        "| TypeScript transform-diagnostic terminal-control and length regressions | three TypeScript tests and three Rust security tests | intentional-hardening evidence |",
    )


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    old = """### CT-RS-024: Raw error text can inject terminal controls

**Severity:** Medium

The TypeScript handler sends error text through terminal coloring without a control-character policy. The Rust core never logs. The future binding must sanitize controls and directionality characters for terminal display while preserving raw structured diagnostics. Unknown errors must remain unknown and must not inherit nonfatal handling.

The core adds no dependencies or side-effect capability, so it introduces no new advisory surface.
"""
    new = f"""### CT-RS-024: Raw transform errors allowed terminal spoofing and output amplification

**Severity:** Medium

The TypeScript create command previously passed `TransformError.message` and `TransformError.transform` directly through terminal coloring. A transform failure containing escape sequences, C0/C1 controls, line breaks, bidirectional controls, or invisible format characters could alter terminal presentation. Very large fields were also copied and printed without a display bound.

The TypeScript oracle now separates raw and terminal-facing data. `rawMessage` and `rawTransform` retain the original structured values, while `message` and `transform` are rendered by a bounded scalar scanner. The Rust `TransformFailure` retains its raw fields and applies the same renderer only through `Display` and `terminal_transform`.

Both implementations:

- preserve ordinary printable Unicode exactly;
- escape NUL, tab, line feed, carriage return, C0/C1 controls, bidirectional controls, and reviewed invisible-format ranges;
- process at most 513 Unicode scalar values for each field;
- emit at most 512 source scalars plus a single `…` truncation marker;
- do not add a parser, crate, network call, filesystem operation, subprocess, logger, unsafe block, or mutable global state.

The original vulnerability is retained as genuine RED evidence in `{RED_SHA}`. The implementation is `{GREEN_SHA}`, with formatting closure in `{FORMATTED_SHA}` and validation at `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`. Regression coverage is `packages/create-turbo/__tests__/transform-error-security.test.ts` and `packages/create-turbo/rust/tests/transform_diagnostic_security.rs`.

Residual risk remains for raw `ConvertError`, `DownloadError`, project/workspace names, descriptions, and other untrusted logger fields. Those must use the same field-level boundary before terminal coloring. Unknown errors must remain unknown rather than being downgraded into a sanitized recoverable class.
"""
    replace_once(path, old, new)
    replace_once(
        path,
        "- Every intentional incompatibility is recorded here and in `PARITY_MATRIX.md` with regression coverage.",
        "- `TransformError` terminal-facing fields are bounded and escaped, while raw structured values remain separately available.\n"
        "- Every intentional incompatibility is recorded here and in `PARITY_MATRIX.md` with regression coverage.",
    )
    replace_once(
        path,
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
        "- The terminal-diagnostic hardening adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, unsafe code, or mutable global state, and does not change the lockfile.\n"
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
    )
    replace_once(
        path,
        "- map typed failures to the existing JavaScript public contracts;",
        "- map typed failures to the existing JavaScript public contracts and finish terminal-safe handling for `ConvertError`, `DownloadError`, and all remaining untrusted logger fields;",
    )


def update_divergence_ledger() -> None:
    path = "packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md"
    replace_once(
        path,
        "- `packages/create-turbo/src/transforms/errors.ts`",
        "- `packages/create-turbo/src/transforms/errors.ts`\n"
        "- `packages/create-turbo/src/transforms/terminal.ts`\n"
        "- `packages/create-turbo/__tests__/transform-error-security.test.ts`",
    )
    replace_once(
        path,
        "- Default error metadata is transform `unknown` and `fatal: true`; explicit empty transform and `fatal: false` values are preserved.",
        "- Default error metadata is transform `unknown` and `fatal: true`; explicit empty transform and `fatal: false` values are preserved.\n"
        "- Ordinary printable Unicode is unchanged in terminal-facing transform diagnostics.\n"
        "- Raw transform message and name values remain available separately from their bounded terminal rendering.",
    )
    replace_once(
        path,
        "| Error output | Raw message is passed through terminal coloring | Core performs no logging | Intentional security boundary. The host must sanitize terminal controls for display while retaining raw structured diagnostics. |",
        "| Transform-error display fields | `TransformError.message` and `.transform` previously reached terminal coloring raw; they now use `sanitizeTerminalText`, while `rawMessage` and `rawTransform` preserve the source data | `TransformFailure` keeps raw fields and applies `sanitize_terminal_text` through `Display` and `terminal_transform` | Intentional security fix. Safe printable text is preserved; hostile controls and oversized fields deliberately render differently. |",
    )
    replace_once(
        path,
        "4. terminal-control-safe logging without changing structured error data;",
        "4. terminal-control-safe handling for `ConvertError`, `DownloadError`, and every remaining untrusted logger field without changing structured error data;",
    )
    replace_once(
        path,
        "- GREEN integration commit: `7b208824412f008a942567faa5e37740948a541e`.",
        "- GREEN integration commit: `7b208824412f008a942567faa5e37740948a541e`.\n"
        f"- Terminal-diagnostic RED commit: `{RED_SHA}`.\n"
        f"- Terminal-diagnostic GREEN commit: `{GREEN_SHA}`.\n"
        f"- Formatting closure: `{FORMATTED_SHA}`.\n"
        f"- Validated tranche head: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.\n"
        "- TypeScript security tests: `packages/create-turbo/__tests__/transform-error-security.test.ts`.\n"
        "- Rust security tests: `tests/transform_diagnostic_security.rs`.",
    )


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 73 translated parity tests and 51 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, and transform-pipeline control flow.",
        "- `packages/create-turbo/rust`: 73 translated parity tests and 54 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, transform-pipeline control flow, and bounded terminal diagnostics.",
    )
    replace_once(
        path,
        "That is **284 authored Rust migration tests** on the integration branch.",
        "That is **287 authored Rust migration tests** on the integration branch.",
    )
    replace_once(
        path,
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |",
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager prompt/decision, official-starter, transform-pipeline, and bounded transform-diagnostic cores are ported. CLI, discovery/acquisition, production VCS/converter/JSON providers, async binding, remaining error/logger fields, telemetry binding, packaging, callers, and removal proof remain. |",
    )
    replace_once(
        path,
        "The async JavaScript binding remains blocked until it proves exact argument forwarding, exactly-once telemetry, terminal-safe error display, cleanup and flush before fatal exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and removal proof. The full divergence ledger is `packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md`.",
        "The TypeScript `TransformError` path and Rust renderer now share a bounded 512-scalar terminal-field policy that escapes controls and bidirectional/invisible formatting while retaining raw structured values. The async JavaScript binding remains blocked until it proves exact argument forwarding, exactly-once telemetry, terminal-safe handling for `ConvertError`, `DownloadError`, and remaining untrusted fields, cleanup and flush before fatal exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and removal proof. The full divergence ledger is `packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md`.",
    )
    replace_once(
        path,
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.",
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.\n"
        f"- transform terminal-diagnostic RED: `{RED_SHA}`.\n"
        f"- transform terminal-diagnostic implementation: `{GREEN_SHA}`.\n"
        f"- transform terminal-diagnostic formatting closure: `{FORMATTED_SHA}`.\n"
        f"- transform terminal-diagnostic validation: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",
    )


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    old = """### RF-016: Transform-loop termination and terminal output require a secure host boundary

**Status:** Fixed-order Rust core implemented; production binding blocked.

The TypeScript create command runs four transforms sequentially, treats nonfatal `TransformError` values as recoverable, exits immediately on fatal transform errors, and rethrows unknown errors. Raw error text is sent to terminal formatting, while telemetry is a separate side effect.

The Rust core closes routing to four enum variants, bounds each to one invocation, preserves exact error defaults and string truthiness, and returns typed partial progress instead of logging or exiting. Production closure requires exact async forwarding, exactly-once telemetry, terminal-control-safe display, cleanup and flush before exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and TypeScript removal proof.

Regression evidence is in `packages/create-turbo/rust/tests/transform_pipeline_parity.rs` and `transform_pipeline_security.rs`; exact differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md`.
"""
    new = f"""### RF-016: Transform-loop termination and terminal diagnostics require a secure host boundary

**Status:** Fixed-order Rust core and transform-error terminal fields implemented; production binding remains blocked.

The TypeScript create command runs four transforms sequentially, treats nonfatal `TransformError` values as recoverable, exits immediately on fatal transform errors, and rethrows unknown errors. Before this tranche, raw transform names and messages reached terminal coloring with no control-character or display-length policy.

The Rust core closes routing to four enum variants, bounds each to one invocation, preserves exact error defaults and string truthiness, and returns typed partial progress instead of logging or exiting. The TypeScript `TransformError` and Rust renderer now use the same bounded scalar policy: preserve printable Unicode, escape C0/C1, bidirectional, and reviewed invisible-format characters, inspect at most 513 scalars, render at most 512 source scalars plus `…`, and retain the raw structured values separately.

RED evidence is `{RED_SHA}`. The implementation is `{GREEN_SHA}`, formatting closure is `{FORMATTED_SHA}`, and the validated tranche head is `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`. Tests are `packages/create-turbo/__tests__/transform-error-security.test.ts`, `packages/create-turbo/rust/tests/transform_diagnostic_security.rs`, `transform_pipeline_parity.rs`, and `transform_pipeline_security.rs`.

Production closure still requires exact async forwarding, exactly-once telemetry, field-level terminal safety for raw `ConvertError`, `DownloadError`, project/workspace names, descriptions, and every remaining untrusted logger input, cleanup and flush before exit code 1, strict runtime metadata typing, unknown-error propagation, supported-platform differentials, and TypeScript removal proof. Blanket logger sanitization is not acceptable because it would also escape trusted ANSI sequences; untrusted fields must be sanitized before coloring.
"""
    replace_once(path, old, new)
    replace_once(
        path,
        "- close the transform-pipeline async binding, telemetry, terminal-safe logging, cleanup-before-exit, runtime typing, and supported-platform differential contract;",
        "- close the transform-pipeline async binding, telemetry, terminal-safe handling for non-transform errors and every remaining untrusted logger field, cleanup-before-exit, runtime typing, and supported-platform differential contract;",
    )


def main() -> None:
    update_readme()
    update_parity_matrix()
    update_security()
    update_divergence_ledger()
    update_program_ledger()
    update_repository_findings()

    for temporary in (WORKFLOW, SELF):
        if not temporary.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {temporary}")
        temporary.unlink()


if __name__ == "__main__":
    main()
