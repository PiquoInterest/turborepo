#!/usr/bin/env python3
"""Record the reviewed project-directory security migration tranche."""

from pathlib import Path

RED_SHA = "a53767309e2ad8fd68a1bdb6501b05961abbe7bd"
INITIAL_GREEN_SHA = "7d890550fffc826781d09b6babcaa0f9738632de"
GREEN_SHA = "3c83fd5e29350102865bc33b0c5177ce25e31093"
ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/validate-create-turbo-directory-security.yml"
SELF = Path(__file__).resolve()


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:120]!r}"
        )
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def write_divergence_ledger() -> None:
    target = ROOT / "packages/create-turbo/rust/DIRECTORY_PROMPT_DIVERGENCES.md"
    if target.exists():
        raise SystemExit(f"refusing to replace existing divergence ledger: {target}")
    target.write_text(
        f"""# Project-directory prompt parity and divergence ledger

## Scope

TypeScript oracles:

- `packages/create-turbo/src/commands/create/prompts.ts`
- `packages/turbo-utils/src/validate-directory.ts`
- `packages/turbo-utils/src/is-folder-empty.ts`
- the `prompts.directory` caller in `packages/create-turbo/src/commands/create/index.ts`

Rust target: `packages/create-turbo/rust/src/directory_prompt.rs`.

The reviewed Rust tranche owns input selection, exact prompt metadata, JavaScript-compatible argument truthiness, display-only transformer metadata, raw-answer preservation, early input hardening, and fail-closed validator propagation. Filesystem resolution and interactive terminal I/O remain typed providers and are not production-approved by this core.

## Confirmed TypeScript defect and repair

For a truthy CLI directory argument, the original `directory` function returned `validateDirectory(dir)` even when `valid` was false. The create command immediately destructured `root` and `projectName` without checking `valid`, so a conflicting or malformed direct path could continue into project acquisition instead of terminating.

The RED TypeScript tests require invalid direct paths to reject. The repaired TypeScript path throws `InvalidDirectoryError` before the caller can continue. The Rust core models validation as `Result`, so an invalid provider result cannot be represented as success.

## Preserved behavior

- A non-empty direct argument is JavaScript-truthy, bypasses prompting, and is not trimmed.
- An absent or empty direct argument invokes the prompt.
- The prompt message remains `Where would you like to create your Turborepo?`.
- The default remains `./my-turborepo`.
- Inquirer's `transformer` trims ECMAScript whitespace for display only; the accepted answer remains raw and is validated unchanged.
- Prompt and validation errors are propagated once without retries or fallback values.
- The validator's successful typed output is returned without reinterpretation.

## Intentional security and representation differences

| Boundary | TypeScript before repair | Rust and repaired TypeScript | Classification and reason |
| --- | --- | --- | --- |
| Invalid direct argument | Returned a false validation object that the caller ignored | Typed rejection before project creation | Security fix: prevents writes or acquisition from continuing with a known-invalid root. |
| Input size | No explicit bound before `path.resolve`, regex work, or filesystem calls | 4,096 UTF-8 byte limit | Security divergence: bounds CPU, allocation, path conversion, and diagnostic work. |
| C0/C1 and format controls | Rejected only indirectly in some basename branches and could be reflected in terminal-facing diagnostics | Rejected before validator/UI effects with a generic non-reflecting error | Security divergence: prevents terminal escape and bidirectional/invisible-text spoofing. |
| Optional CLI argument | JavaScript truthiness | `Option<&str>` plus an exact non-empty check | Representation-only conversion preserving empty-string falsehood. |
| Prompt transformer | `transformer: d => d.trim()` changes rendering, while `done(answer)` returns the raw answer | `DirectoryDisplayTransform::TrimEcmascriptWhitespace` is provider metadata; the core validates the raw answer | Representation-only conversion preserving display-only behavior. |
| Prompt and validation effects | Direct module calls | `DirectoryPrompter` and `DirectoryValidator` traits | Security boundary: terminal and filesystem authority stay outside the reviewed decision core. |
| Public diagnostics | Error strings can contain path text | Core input-error display never includes the rejected input; raw provider causes remain structured | Security divergence: prevents control-text reflection at the Rust boundary. |

## Remaining production requirements

A production directory validator must still prove lexical and platform path semantics, root confinement policy, no-follow handling for every path component, regular-directory requirements, directory-handle identity, bounded enumeration, concurrent replacement behavior, Windows reparse-point handling, permission diagnostics, and Linux/macOS/Windows differential parity. A production prompt provider must enforce the advertised byte limit while reading, apply the display transformer without mutating the returned answer, sanitize terminal rendering, and preserve cancellation, EOF, signal, and non-TTY behavior. The JavaScript host must also classify `InvalidDirectoryError` as a user-input failure rather than an internal bug before cutover.

## TDD evidence

- RED TypeScript and Rust security contract: `{RED_SHA}`.
- Initial fail-closed implementation: `{INITIAL_GREEN_SHA}`.
- Final GREEN implementation after verifying Inquirer's display-only transformer contract: `{GREEN_SHA}`.
- TypeScript regressions: `packages/create-turbo/__tests__/directory-security.test.ts`.
- Rust parity regressions: `packages/create-turbo/rust/tests/directory_prompt_parity.rs`.
- Rust security regressions: `packages/create-turbo/rust/tests/directory_prompt_security.rs`.

This tranche adds no dependency, subprocess, network access, filesystem implementation, logger, unsafe code, or mutable global state.
""",
        encoding="utf-8",
    )


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "8. the package-manager prompt resolution and installed-choice ordering contract.",
        "8. the package-manager prompt resolution and installed-choice ordering contract.\n"
        "9. the fail-closed project-directory argument and prompt-resolution contract.",
    )
    section = """### Project-directory prompt core

- preserves JavaScript truthiness by using a non-empty direct argument without trimming and prompting for an absent or empty argument;
- sends the exact source prompt message, `./my-turborepo` default, input bound, and display-transform policy through `DirectoryPrompter`;
- models Inquirer's ECMAScript trim transformer as display-only metadata and validates the exact raw accepted answer;
- requires `DirectoryValidator` to return `Result`, so a known-invalid direct path cannot escape as a successful value;
- limits directory input to 4,096 UTF-8 bytes before validation or downstream effects;
- rejects C0/C1 controls, terminal escapes, bidirectional controls, soft hyphen, zero-width format controls, and BOM text before providers run;
- returns generic public input errors that never reflect attacker-controlled directory text;
- invokes the prompt and validator at most once and never invents a fallback after provider failure.

The TypeScript oracle is repaired in the same tranche: direct invalid or conflicting paths now throw `InvalidDirectoryError` instead of returning `{ valid: false }` to a caller that ignored the flag. Filesystem validation, terminal interaction, and user-facing host error classification remain provider/binding work. Exact behavior and deliberate security differences are in [`DIRECTORY_PROMPT_DIVERGENCES.md`](./DIRECTORY_PROMPT_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", section)
    replace_once(
        path,
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;",
        "- production project-directory validation and prompt providers, including handle-based path identity, bounded enumeration, display-only transformation, cancellation, signals, and non-TTY behavior;\n"
        "- JavaScript host mapping for typed project-directory failures;\n"
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;",
    )
    replace_once(
        path,
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation.",
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `directory_prompt` owns direct-versus-prompted input selection, display-transform metadata, raw-answer preservation, resource bounds, control rejection, and fail-closed validator propagation. `official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation.",
    )
    replace_once(
        path,
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c",
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c\n"
        f"Directory prompt RED:   {RED_SHA}\n"
        f"Directory prompt GREEN: {GREEN_SHA}",
    )
    replace_once(
        path,
        "The crate contains 73 translated parity tests and 51 security regression tests, for 124 authored focused Rust tests.",
        "The crate contains 81 translated parity tests and 59 security regression tests, for 140 authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    section = """## Project-directory prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| truthy direct `dir` bypasses prompt | non-empty `Option<&str>` branch | implemented core | The direct value is borrowed and not trimmed. |
| absent or empty direct `dir` | `DirectoryPrompter` request | implemented core | Empty string preserves JavaScript falsehood. |
| exact prompt message and `./my-turborepo` default | `DirectoryPromptRequest` constants | implemented core | Exact source text, ordering, byte bound, and display policy are tested. |
| Inquirer display transformer trims while `done(answer)` returns raw text | `DirectoryDisplayTransform` plus raw validator input | implemented core | Rendering may trim ECMAScript whitespace; the requested directory is never silently rewritten. |
| direct invalid validation object was returned and caller ignored `valid` | typed validator error plus repaired TypeScript throw | intentional security fix | The RED TypeScript test proves the old false-success route; neither implementation now continues. |
| unbounded path input | 4,096 UTF-8 byte cap | intentional-hardening | Rejects before path resolution, validation, logging, or other providers. |
| control and directionality text could reach diagnostics | early C0/C1/format-control rejection | intentional-hardening | Public core error text never reflects the rejected value. |
| prompt/validator side effects | `DirectoryPrompter` and `DirectoryValidator` | implemented core | Each provider runs at most once; errors propagate without retry. |
| filesystem path resolution and emptiness checks | production validator provider | blocked | Requires no-follow component handling, directory-handle identity, bounded enumeration, reparse-point policy, races, permissions, and supported-platform differentials. |
| Inquirer terminal behavior and CLI error mapping | production prompt provider and host binding | blocked | Requires bounded reads, display-only transformation, cancellation/EOF/signal/non-TTY parity, terminal-safe rendering, and user-input error classification. |

Detailed differences are in `DIRECTORY_PROMPT_DIVERGENCES.md`. The RED contract also adds four TypeScript security regressions so the legacy oracle remains protected until cutover."""
    insert_before(path, "## Existing TypeScript test mapping", section)


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    replace_once(
        path,
        "- package-manager selection in `packages/create-turbo/src/commands/create/prompts.ts`",
        "- package-manager selection in `packages/create-turbo/src/commands/create/prompts.ts`\n"
        "- project-directory selection in `packages/create-turbo/src/commands/create/prompts.ts` and its caller",
    )
    replace_once(
        path,
        "Package-manager prompting receives free-form CLI text, discovered executable versions, terminal input, cancellation, and non-TTY state. The reviewed Rust core accepts only a closed enum after exact parsing and revalidates every selected manager against a non-empty discovered version. Discovery and UI effects remain provider-owned.",
        "Package-manager prompting receives free-form CLI text, discovered executable versions, terminal input, cancellation, and non-TTY state. The reviewed Rust core accepts only a closed enum after exact parsing and revalidates every selected manager against a non-empty discovered version. Discovery and UI effects remain provider-owned.\n\n"
        "Project-directory selection receives a CLI argument or terminal input that later controls repository acquisition and filesystem writes. The reviewed core bounds and filters that text before typed providers, preserves direct-versus-prompted and display-versus-returned semantics, and cannot turn a validator failure into success. Filesystem identity, terminal behavior, and JavaScript host error classification remain provider-owned.",
    )
    findings = """### CT-RS-028: Invalid direct directory could continue after known validation failure

**Severity:** High

The original TypeScript `directory` function returned `validateDirectory(dir)` for a truthy CLI argument. `validateDirectory` represents malformed, conflicting, and non-directory paths as `{ valid: false, ... }`, but the create command immediately destructured `root` and `projectName` and never checked `valid`. A known-invalid direct path could therefore continue into project acquisition and later filesystem operations.

The RED TypeScript tests require rejection for malformed and conflicting direct paths. The repaired TypeScript path throws `InvalidDirectoryError`, and the Rust core requires `DirectoryValidator` to return `Result`, making false success unrepresentable.

Regression tests: TypeScript `directory-security.test.ts` and Rust `invalid_direct_argument_cannot_escape_as_a_false_success`.

### CT-RS-029: Directory input was unbounded and terminal-control text could reach diagnostics

**Severity:** Medium

The legacy path performed resolution, regex checks, filesystem queries, and terminal-facing validation without an explicit input bound. Invalid text could also be included in diagnostics after containing escape, bidirectional, or invisible formatting controls.

Both repaired paths now cap input at 4,096 UTF-8 bytes and reject C0/C1 and selected Unicode format controls before validator or terminal effects. The Rust core's public input error contains only trusted text and numeric lengths, never the rejected directory value. Inquirer's transformer remains display-only, so neither implementation silently rewrites an accepted directory while applying this hardening.

Regression tests: the TypeScript oversized/control cases and Rust `c0_and_c1_controls_are_rejected_before_any_provider`, `invisible_and_bidirectional_format_controls_are_rejected`, `oversized_argument_is_rejected_before_prompt_or_validation`, `unsafe_prompt_response_is_rejected_before_validation`, and `public_input_error_does_not_reflect_attacker_control_text`.

### CT-RS-030: Production directory inspection still needs a handle-based identity contract

**Severity:** High until provider closure

The TypeScript validator uses separate `existsSync`, `lstatSync`, and `readdirSync` operations. A writable adversarial parent can exchange a path between those calls, and checking only the final path does not establish safe identity for every component. The prompt core deliberately does not claim to solve filesystem inspection.

A production `DirectoryValidator` must use reviewed no-follow or handle-relative operations, validate every relevant component, bind enumeration to the inspected directory identity, bound entry count and name bytes, define concurrent replacement and permission behavior, reject Windows reparse redirection, and pass Linux/macOS/Windows differential and failure-injection tests. Until then, the TypeScript validator remains an oracle and a cutover blocker rather than a production Rust provider."""
    insert_before(path, "## Security invariants", findings)
    replace_once(
        path,
        "- No new `unsafe` or shell command construction is introduced by these tranches.",
        "- No new `unsafe` or shell command construction is introduced by these tranches.\n"
        "- Project-directory input is bounded and screened before prompt/validator side effects, validator failure cannot become success, and display transformation cannot mutate the returned answer.",
    )
    replace_once(
        path,
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
        "- The project-directory prompt tranche adds no dependency, parser, network call, filesystem operation, subprocess, logger, unsafe code, or mutable global state.\n"
        "- Its production terminal, filesystem, and JavaScript host providers remain blocked until input, identity, race, display, cancellation, error-classification, and supported-platform contracts are proven.\n"
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.",
    )
    replace_once(
        path,
        "- map typed failures to the existing JavaScript public contracts;",
        "- map typed failures to the existing JavaScript public contracts;\n"
        "- implement the production project-directory prompt and handle-based validator providers with display-only transformation, bounded enumeration, and supported-platform differential tests;",
    )


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 73 translated parity tests and 51 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, and transform-pipeline control flow.",
        "- `packages/create-turbo/rust`: 81 translated parity tests and 59 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, project-directory and package-manager prompts, package-manager/official-starter transforms, and transform-pipeline control flow.",
    )
    replace_once(
        path,
        "That is **284 authored Rust migration tests** on the integration branch.",
        "That is **300 authored Rust migration tests** on the integration branch.",
    )
    replace_once(
        path,
        "The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **75%**.",
        "The project-directory security tranche advances create-turbo core and test evidence without completing a production binding or cutover stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **76%**.",
    )
    replace_once(
        path,
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |",
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, project-directory/package-manager prompt decisions, package-manager decision/request, and official-starter orchestration cores are ported. CLI binding, production prompt/directory/discovery/acquisition/VCS/converter/JSON providers, remaining transforms, telemetry, packaging, callers, and removal proof remain. |",
    )
    section = """### Project-directory prompt resolution

The RED contract exposed a source security bug: direct CLI arguments returned a false validation object, while the create caller ignored `valid` and continued. The TypeScript oracle is repaired so malformed or conflicting direct paths throw before project creation. The Rust core makes that state unrepresentable through a typed validator result.

The Rust decision layer preserves direct-argument truthiness, exact prompt text/default, Inquirer's display-only ECMAScript trim transformer, and the raw accepted answer while adding a 4,096-byte cap and early control/directionality rejection. Prompting, filesystem validation, and JavaScript host error mapping remain outside the core. Production cutover requires bounded terminal input, cancellation/non-TTY/signal behavior, no-follow component inspection, directory-handle identity, bounded enumeration, Windows reparse handling, typed user-error classification, and shared platform fixtures. Exact differences are in `packages/create-turbo/rust/DIRECTORY_PROMPT_DIVERGENCES.md`.
"""
    insert_before(path, "### Package-manager prompt resolution", section)
    replace_once(
        path,
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.",
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.\n"
        f"- project-directory prompt RED: `{RED_SHA}`.\n"
        f"- project-directory prompt implementation: `{GREEN_SHA}`.",
    )


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    finding = """### RF-018: Invalid project-directory arguments could bypass validation failure

**Status:** Fixed in the TypeScript oracle and Rust decision core; production filesystem/prompt/host providers remain blocked.

The original `create-turbo` direct-argument branch returned `{ valid: false }` for malformed, conflicting, or non-directory paths, but the create caller ignored the flag and continued with the returned root. The same boundary also lacked an explicit input-size limit and could reflect control or bidirectional text into terminal-facing diagnostics.

The RED TypeScript tests demonstrate the false-success route. The repaired TypeScript implementation throws before acquisition. The Rust core uses a typed validator result, caps input at 4,096 UTF-8 bytes, rejects C0/C1 and selected Unicode format controls before providers, preserves exact prompt/default and Inquirer's display-only transformer behavior, validates the raw answer, and never includes rejected input in its public error display.

Production closure still requires a prompt provider with bounded reads and exact display/cancellation/non-TTY/signal behavior, a directory validator that binds no-follow component inspection and bounded enumeration to a stable directory handle on Linux, macOS, and Windows, and a JavaScript host mapping that treats typed directory rejection as user input rather than an internal bug. Regression evidence and the complete divergence ledger are in `packages/create-turbo/rust/tests/directory_prompt_*` and `DIRECTORY_PROMPT_DIVERGENCES.md`."""
    insert_before(path, "## Required repository gates", finding)
    replace_once(
        path,
        "- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;",
        "- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;\n"
        "- close the project-directory prompt, host-error, and validator provider contract, including bounded input/enumeration, display-only transformation, no-follow handle identity, reparse-point behavior, cancellation, and supported-platform differentials;",
    )


def main() -> None:
    write_divergence_ledger()
    update_readme()
    update_parity_matrix()
    update_security()
    update_program_ledger()
    update_repository_findings()

    for temporary in (WORKFLOW, SELF):
        if not temporary.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {temporary}")
        temporary.unlink()


if __name__ == "__main__":
    main()
