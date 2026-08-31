#!/usr/bin/env python3
"""Update migration evidence after the directory-security tranche passes."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/validate-create-turbo-directory-security-v2.yml"
SELF = Path(__file__).resolve()
RED_SHA = "11131d1fc01536c151bdda04ba39fdc4aec5779a"
DIAGNOSTIC_RED_SHA = "6bbef5a95cbc0e6cf908e62083e852db473b2456"
UNICODE_RED_SHA = "beb38232b84dca526a5b86accf40b5dca80a734c"
GREEN_SHA = "e0d5663e51c084f4f25051270ed9bb494df1b21a"
EXPECTED_PARITY = 102
EXPECTED_SECURITY = 81
EXPECTED_CREATE_TOTAL = EXPECTED_PARITY + EXPECTED_SECURITY
EXPECTED_REPOSITORY_TOTAL = 343


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected one reviewed anchor in {path}, found {count}: {old[:120]!r}"
        )
    write(path, text.replace(old, new, 1))


def sub_once(path: str, pattern: str, replacement: str, flags: int = 0) -> None:
    text = read(path)
    updated, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise SystemExit(f"expected one regex anchor in {path}, found {count}: {pattern!r}")
    write(path, updated)


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def count_tests() -> tuple[int, int]:
    test_root = ROOT / "packages/create-turbo/rust/tests"
    parity = sum(
        path.read_text(encoding="utf-8").count("#[test]")
        for path in test_root.glob("*_parity.rs")
    )
    security = sum(
        path.read_text(encoding="utf-8").count("#[test]")
        for path in test_root.glob("*_security.rs")
    )
    if (parity, security) != (EXPECTED_PARITY, EXPECTED_SECURITY):
        raise SystemExit(
            f"unexpected create-turbo test inventory: parity={parity}, security={security}"
        )
    return parity, security


def update_readme(parity: int, security: int) -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "8. the package-manager prompt resolution and installed-choice ordering contract.",
        "8. the package-manager prompt resolution and installed-choice ordering contract.\n"
        "9. the create-command error classification and terminal-safe diagnostic contract.\n"
        "10. the package-install decision and bounded unavailable-manager warning contract.\n"
        "11. the fail-closed project-directory argument and prompt-resolution contract.",
    )
    sections = """### Create-command error and installation policy cores

- classify transform, conversion, download, and unknown failures without letting untrusted message text change fatality or routing;
- escape and bound terminal-visible error fields while preserving safe text and unknown-error rethrow behavior;
- choose the selected or source package manager with JavaScript-compatible version truthiness and exactly-once availability lookup;
- never invoke installation when `package.json` is absent, installation is skipped, or the selected version is absent/empty;
- propagate installer failure after one attempt and render unavailable-manager warnings through a bounded terminal-safe formatter.

The production async/telemetry/process binding remains TypeScript. Exact differences are in `CREATE_ERROR_POLICY_DIVERGENCES.md` and `CREATE_INSTALL_POLICY_DIVERGENCES.md`.

### Project-directory prompt core

- preserves JavaScript argument truthiness: a non-empty direct argument bypasses prompting and is not trimmed, while an absent or empty argument prompts;
- carries the exact prompt message, `./my-turborepo` default, byte bound, and display-only transform policy through `DirectoryPromptRequest`;
- preserves Inquirer's raw accepted answer rather than silently applying its display transformer to the returned directory;
- requires `DirectoryValidator` to return `Result`, making the TypeScript false-success state unrepresentable;
- caps input at 4,096 UTF-8 bytes and rejects terminal-active controls before validation or downstream effects;
- repairs the TypeScript oracle to reject invalid direct paths, unpaired UTF-16 surrogates, and hostile Unicode before path conversion;
- maps known TypeScript directory failures to the existing nonzero notifier exit path with a trusted generic message;
- leaves filesystem identity and interactive terminal I/O behind explicit providers.

Exact type conversions, security divergences, and remaining provider requirements are in [`DIRECTORY_PROMPT_DIVERGENCES.md`](./DIRECTORY_PROMPT_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", sections)
    replace_once(
        path,
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;",
        "- production project-directory prompt and handle-based validation providers, including bounded reads/enumeration, display-only transformation, cancellation, no-follow identity, reparse points, and supported-platform differentials;\n"
        "- native/JavaScript mapping for typed project-directory failures and ill-formed UTF-16 rejection;\n"
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;",
    )
    replace_once(
        path,
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and effect ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation.",
        "`readme_transform` owns the bounded pure Markdown scanner and README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns deterministic VCS decisions behind injected runner and cleanup traits. `default_example` owns exact default-acquisition routing. `create_error_policy` owns typed error classification and terminal-safe fields. `create_install_policy` owns install/no-install decisions and bounded warning rendering. `directory_prompt` owns direct-versus-prompted selection, display metadata, raw-answer preservation, resource bounds, control rejection, and fail-closed validator propagation. `official_starter`, `package_manager_transform`, `transform_pipeline`, and `package_manager_prompt` retain their existing typed provider boundaries.",
    )
    replace_once(
        path,
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c",
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c\n"
        f"Directory prompt RED:    {RED_SHA}\n"
        f"Directory diagnostic RED:{DIAGNOSTIC_RED_SHA}\n"
        f"Directory Unicode RED:   {UNICODE_RED_SHA}\n"
        f"Directory prompt GREEN:  {GREEN_SHA}",
    )
    sub_once(
        path,
        r"The crate contains \d+ translated parity tests and \d+ security regression tests, for \d+ authored focused Rust tests\.",
        f"The crate contains {parity} translated parity tests and {security} security regression tests, for {parity + security} authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    sections = """## Create-command error and installation policy tranches

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| transform/convert/download/unknown error routing | `classify_create_command_error` | implemented core | Fatality, nonfatal continuation, known conversion handling, and unknown rethrow are typed. |
| raw terminal error text | bounded `sanitize_terminal_text` fields | intentional-hardening | Controls, directionality, and oversized values cannot alter terminal layout or classification. |
| selected/source manager installation decision | `apply_create_install_policy` | implemented core | Truthiness, skip branches, availability snapshot, one install attempt, and provider failure are covered. |
| unavailable-manager warning interpolation | `render_unavailable_package_manager_warning` | intentional-hardening | Safe text remains exact; untrusted fields and whole lines are escaped and bounded. |
| async logging, telemetry, installer process, and exit | production host/provider | blocked | Requires exact side effects, safe executable resolution, cleanup, and supported-platform differentials. |

Detailed differences are in `CREATE_ERROR_POLICY_DIVERGENCES.md` and `CREATE_INSTALL_POLICY_DIVERGENCES.md`.

## Project-directory prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| truthy direct `dir` bypasses prompt | non-empty `Option<&str>` branch | implemented core | The direct value is borrowed and not trimmed. |
| absent or empty direct `dir` | `DirectoryPrompter` request | implemented core | Empty string preserves JavaScript falsehood. |
| exact prompt message and `./my-turborepo` default | `DirectoryPromptRequest` constants | implemented core | Exact source text, byte limit, and display policy are tested. |
| Inquirer transformer changes display only | `DirectoryDisplayTransform` plus raw validator input | implemented core | The returned answer is not silently rewritten. |
| invalid direct result was returned and caller ignored `valid` | typed validator error plus repaired TypeScript throw | intentional-hardening | Both implementations now terminate before acquisition. |
| unbounded input and hostile controls | 4,096-byte cap plus early control rejection | intentional-hardening | Providers do not receive oversized or terminal-active text. |
| JavaScript lone surrogates | host rejects unpaired UTF-16; Rust accepts valid UTF-8 `&str` | intentional-hardening | Prevents replacement-encoding aliases at the language boundary. |
| detailed path/conflict diagnostics | fixed trusted TypeScript message and non-reflecting Rust display | intentional-hardening | Raw rejected paths cannot become terminal output. |
| filesystem path resolution and emptiness checks | production `DirectoryValidator` | blocked | Requires no-follow component handling, stable directory identity, bounded enumeration, reparse-point policy, race handling, and supported-platform differentials. |
| Inquirer terminal behavior and native binding | production `DirectoryPrompter` and host | blocked | Requires bounded reads, display-only transformation, cancellation/EOF/signal/non-TTY parity, and safe error mapping. |

Detailed differences are in `DIRECTORY_PROMPT_DIVERGENCES.md`. The legacy TypeScript oracle now has focused security regressions that fail on the pre-repair commits."""
    insert_before(path, "## Existing TypeScript test mapping", sections)


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    replace_once(
        path,
        "- package-manager selection in `packages/create-turbo/src/commands/create/prompts.ts`",
        "- package-manager selection in `packages/create-turbo/src/commands/create/prompts.ts`\n"
        "- project-directory selection in `packages/create-turbo/src/commands/create/prompts.ts` and its CLI caller\n"
        "- create-command error and installation policy in `packages/create-turbo/src/commands/create/index.ts`",
    )
    trust = """The create-command error and install-policy cores receive exception text, transform labels, example names, package-manager metadata, and installer outcomes. Their Rust boundaries separate routing data from terminal rendering, bound every displayed field/line, and keep process effects behind providers.

Project-directory selection receives CLI or terminal text that later controls acquisition and filesystem writes. The reviewed Rust core bounds and filters that text before providers, preserves display-versus-returned semantics, and cannot turn validator failure into success. Filesystem identity, terminal I/O, and native host conversion remain provider-owned."""
    insert_before(
        path,
        "The transform pipeline decides which mutation stages run, whether later stages continue, and whether a failure terminates the command.",
        trust,
    )
    findings = """### CT-RS-028: Unavailable-manager warnings accepted raw terminal text

**Severity:** Medium

The TypeScript warning path interpolated the example name directly into terminal output. The Rust warning renderer escapes terminal-active controls, bounds the untrusted field and both completed lines, preserves exact safe text, and derives the manager from a closed enum. TypeScript retains a failing security oracle until its host rendering is migrated or repaired.

Regression evidence is in `create_install_warning_parity.rs`, `create_install_warning_security.rs`, and `CREATE_INSTALL_POLICY_DIVERGENCES.md`.

### CT-RS-029: Invalid direct directory could continue after validation failure

**Severity:** High

The original TypeScript direct-argument branch returned a validation object with `valid: false`, but the create caller immediately destructured the path fields and continued without checking that flag. A malformed, conflicting, or non-directory path could therefore reach project acquisition after known rejection.

The TypeScript repair throws `InvalidDirectoryError`, the root CLI maps it to the existing nonzero user-error path, and the Rust `DirectoryValidator` returns `Result`, making false success unrepresentable.

Regression evidence is in `directory-security.test.ts` and `invalid_direct_argument_cannot_escape_as_a_false_success`.

### CT-RS-030: Directory input lacked bounds and a complete terminal/Unicode policy

**Severity:** Medium

The legacy path performed path conversion and filesystem checks without an explicit input limit. C0/C1, bidirectional, invisible, line-separator, annotation, and related formatting text could also reach validation or terminal diagnostics. JavaScript can additionally carry unpaired UTF-16 surrogates that Rust strings cannot represent; replacement encoding can collapse distinct ill-formed inputs.

Both decision paths cap input at 4,096 UTF-8 bytes and reject terminal-active text before providers. TypeScript rejects unpaired surrogates before UTF-8/path conversion, while Rust receives only valid UTF-8 `&str`. Public directory errors never include the rejected value.

Regression evidence is in the TypeScript control, Unicode, surrogate, and oversized tests plus `directory_prompt_security.rs`.

### CT-RS-031: Production directory inspection needs stable handle identity

**Severity:** High until provider closure

The TypeScript validator performs separate existence, metadata, and directory-enumeration operations. A writable ancestor can replace a path between those calls, and final-component metadata does not establish no-follow safety for every component.

A production `DirectoryValidator` must use reviewed handle-relative or no-follow operations, bind enumeration to the inspected directory identity, bound entries and name bytes, define concurrent replacement and permission behavior, reject Windows reparse redirection, and pass Linux/macOS/Windows failure-injection and differential tests. The production prompt provider must also enforce limits while reading and preserve exact cancellation/non-TTY/signal behavior."""
    insert_before(path, "## Security invariants", findings)
    replace_once(
        path,
        "- No new `unsafe` or shell command construction is introduced by these tranches.",
        "- No new `unsafe` or shell command construction is introduced by these tranches.\n"
        "- Directory input is bounded and screened before prompt/validator effects; display transformation cannot mutate the returned answer, and validation failure cannot become success.\n"
        "- Ill-formed UTF-16 is rejected in the JavaScript host before conversion to Rust's valid-UTF-8 string boundary.",
    )


def update_program_ledger(parity: int, security: int) -> None:
    path = "docs/typescript-deprecation.md"
    sub_once(
        path,
        r"^- `packages/create-turbo/rust`: \d+ translated parity tests and \d+ security regression tests.*$",
        f"- `packages/create-turbo/rust`: {parity} translated parity tests and {security} security regression tests across README and `.gitignore` transforms, Git orchestration, default routing, error/install/directory/package-manager policies, official-starter orchestration, and transform-pipeline control flow.",
        re.MULTILINE,
    )
    sub_once(
        path,
        r"That is \*\*\d+ authored Rust migration tests\*\* on the integration branch\.",
        f"That is **{EXPECTED_REPOSITORY_TOTAL} authored Rust migration tests** on this focused branch.",
    )
    replace_once(
        path,
        "The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **75%**.",
        "The create-command policy and directory-security tranches advance Rust core and test evidence without completing production binding or cutover stages, so the rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **77%**.",
    )
    replace_once(
        path,
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |",
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git, default routing, typed error/install/directory/package-manager policies, official-starter, and transform-pipeline cores are ported. Production prompt/directory/acquisition/VCS/converter/JSON providers, native binding, telemetry, packaging, callers, platform differentials, and removal proof remain. |",
    )
    sections = """### Create-command error and installation policies

Rust now classifies the create command's transform, conversion, download, and unknown failures through typed outcomes; terminal-visible fields are escaped and bounded without changing fatality. The install policy preserves selected-versus-source manager behavior, JavaScript version truthiness, skip branches, exactly-once availability lookup, noninteractive installation, provider-error propagation, and bounded unavailable-manager warnings. Process execution, telemetry, and async host effects remain binding work. Exact differences are in the two create policy divergence ledgers.

### Project-directory prompt resolution

The RED contract exposed a source bug: direct directory validation could return `valid: false` while the caller ignored that state and continued. TypeScript now throws a known safe user-input error before acquisition. The Rust core makes invalid validation unrepresentable, preserves direct/prompted and display/raw-answer behavior, caps input at 4,096 bytes, rejects terminal-active Unicode, and keeps filesystem/terminal effects behind traits.

JavaScript additionally rejects unpaired UTF-16 surrogates before path conversion because Rust `&str` cannot represent them. Production closure still requires bounded interactive reads, exact cancellation and non-TTY behavior, stable no-follow directory handles, bounded enumeration, Windows reparse handling, native host mapping, and supported-platform differential fixtures. Exact differences are in `packages/create-turbo/rust/DIRECTORY_PROMPT_DIVERGENCES.md`."""
    insert_before(path, "### Package-manager prompt resolution", sections)
    replace_once(
        path,
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.",
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.\n"
        f"- project-directory prompt RED: `{RED_SHA}`.\n"
        f"- project-directory non-reflecting diagnostic RED: `{DIAGNOSTIC_RED_SHA}`.\n"
        f"- project-directory Unicode/UTF-16 RED: `{UNICODE_RED_SHA}`.\n"
        f"- project-directory prompt implementation: `{GREEN_SHA}`.",
    )


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    finding = """### RF-018: Invalid project-directory input could bypass failure and reach acquisition

**Status:** TypeScript oracle repaired and Rust decision core implemented; production prompt/filesystem/native providers remain blocked.

The original direct-directory branch returned `{ valid: false }`, while the create caller ignored `valid` and continued with the returned path fields. The same boundary lacked an explicit size limit, did not define a complete terminal-active Unicode policy, and could receive unpaired UTF-16 surrogates before operating-system path encoding.

The repaired TypeScript path throws a known non-reflecting user-input error and rejects oversized, terminal-active, and ill-formed UTF-16 values before path conversion. Rust uses typed provider errors, a 4,096-byte limit, valid UTF-8 strings, exact raw-answer preservation, and early control rejection. Neither core performs filesystem inspection directly.

Production closure requires a bounded terminal provider, native UTF-16/UTF-8 boundary tests, stable no-follow directory-handle validation, bounded enumeration, Windows reparse-point handling, concurrent replacement failure injection, and Linux/macOS/Windows differential fixtures. Evidence is in the directory TypeScript/Rust tests and `DIRECTORY_PROMPT_DIVERGENCES.md`."""
    insert_before(path, "## Required repository gates", finding)
    replace_once(
        path,
        "- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;",
        "- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;\n"
        "- close the project-directory prompt, native string boundary, and handle-based validator contract, including bounded input/enumeration, display-only transformation, no-follow identity, reparse points, cancellation, and supported-platform differentials;",
    )


def main() -> None:
    parity, security = count_tests()
    update_readme(parity, security)
    update_parity_matrix()
    update_security()
    update_program_ledger(parity, security)
    update_repository_findings()

    for temporary in (WORKFLOW, SELF):
        if not temporary.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {temporary}")
        temporary.unlink()


if __name__ == "__main__":
    main()
