#!/usr/bin/env python3
"""Atomically update consolidated TypeScript-to-Rust migration documentation."""

from __future__ import annotations

import base64
import json
import os
import urllib.parse
import urllib.request


TOKEN = os.environ["GITHUB_TOKEN"]
REPOSITORY = os.environ["GITHUB_REPOSITORY"]
BRANCH = "rust/typescript-deprecation"
SELF_PATH = "tools/migration/apply_consolidated_rust_migration_docs.py"
WORKFLOW_PATH = ".github/workflows/apply-consolidated-rust-migration-docs.yml"
API_ROOT = f"https://api.github.com/repos/{REPOSITORY}"


def api(method: str, path: str, payload: dict[str, object] | None = None) -> dict[str, object]:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        f"{API_ROOT}/{path}",
        data=data,
        method=method,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {TOKEN}",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "turborepo-rust-migration-docs",
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        value = json.load(response)
    if not isinstance(value, dict):
        raise SystemExit(f"unexpected GitHub API response for {path}")
    return value


def read_file(path: str, ref: str) -> str:
    encoded_path = urllib.parse.quote(path, safe="/")
    encoded_ref = urllib.parse.quote(ref, safe="")
    result = api("GET", f"contents/{encoded_path}?ref={encoded_ref}")
    if result.get("encoding") != "base64":
        raise SystemExit(f"unexpected encoding for {path}: {result.get('encoding')}")
    encoded = result.get("content")
    if not isinstance(encoded, str):
        raise SystemExit(f"missing content for {path}")
    return base64.b64decode(encoded).decode("utf-8")


def replace_once(text: str, old: str, new: str, path: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


def insert_before(text: str, anchor: str, addition: str, path: str) -> str:
    return replace_once(text, anchor, addition.rstrip() + "\n\n" + anchor, path)


def update_create_readme(text: str) -> str:
    path = "packages/create-turbo/rust/README.md"
    text = replace_once(
        text,
        "8. the package-manager prompt resolution and installed-choice ordering contract.\n",
        """8. the package-manager prompt resolution and installed-choice ordering contract.
9. the create-command error classification and terminal-safe display policy.
10. the create-command package-install decision and unavailable-manager warning policy.
11. the bounded terminal renderer for workspace summaries, success text, and get-started output.
12. the package-manager installation profile and no-shell/no-local-executable invocation policy.
13. the project-directory argument/prompt decision and fail-closed validator boundary.
""",
        path,
    )
    sections = """### Create-command error policy core

- preserves transform, conversion, download, and unknown-error classification;
- preserves safe printable output order and exit-code decisions;
- returns typed `Continue`, `Exit(1)`, or `Rethrow` actions instead of terminating inside the core;
- escapes terminal controls, directionality and invisible format controls;
- bounds error messages to 4096 UTF-8 bytes and labels to 256 UTF-8 bytes;
- never renders unknown errors.

Exact differences and binding blockers are in [`CREATE_ERROR_POLICY_DIVERGENCES.md`](./CREATE_ERROR_POLICY_DIVERGENCES.md).

### Create installation and warning policy core

- chooses the source manager when transforms are skipped or no selected manager exists;
- preserves resolution order, `skipInstall`, missing-package, unavailable-source, and empty-version behavior;
- issues at most one noninteractive install request and propagates provider errors;
- snapshots availability once to remove a mutable-provider time-of-check/time-of-use ambiguity;
- returns structured warning data and renders bounded terminal-safe warning lines.

Exact differences are in [`CREATE_INSTALL_POLICY_DIVERGENCES.md`](./CREATE_INSTALL_POLICY_DIVERGENCES.md).

### Create output policy core

- preserves safe workspace heading/item, success, and get-started text;
- accepts already-derived ordered workspace records and closed script variants;
- escapes terminal-active project, path, workspace, and description fields;
- bounds fields and lines and caps workspace/script counts.

Path-relative, grouping, locale ordering, coloring, and host emission remain binding work. See [`CREATE_OUTPUT_POLICY_DIVERGENCES.md`](./CREATE_OUTPUT_POLICY_DIVERGENCES.md).

### Package-manager installation profile core

- preserves all eight source profiles and source-order/default selection;
- keeps Node-semver matching behind a provider boundary;
- represents programs as the closed six-manager enum and arguments as static slices;
- forbids project-local executable preference and shell execution on every platform;
- always ignores standard input.

The production runner remains blocked on canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, Windows shims, and platform differentials. See [`PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`](./PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md).

### Project-directory prompt core

- preserves direct-argument versus prompt selection, message/default text, and display-only ECMAScript trimming;
- preserves the raw accepted answer for validation;
- rejects inputs over 4096 UTF-8 bytes and terminal-active or invisible control text before providers;
- represents validator rejection as an error, preventing the confirmed TypeScript fail-open path;
- keeps terminal and filesystem authority behind typed providers.

The repaired TypeScript caller now checks validation, but production Rust prompting and handle-relative filesystem validation remain blocked. See [`DIRECTORY_PROMPT_DIVERGENCES.md`](./DIRECTORY_PROMPT_DIVERGENCES.md)."""
    text = insert_before(text, "## Not yet implemented in Rust", sections, path)
    text = replace_once(
        text,
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;\n",
        "- production directory and package-manager prompt providers, including bounded input, cancellation, EOF, signals, and non-TTY behavior;\n",
        path,
    )
    text = replace_once(
        text,
        "- production package-manager workspace conversion and installation orchestration;\n",
        "- production package-manager workspace conversion plus the no-shell installation runner and Node-semver-compatible matcher;\n",
        path,
    )
    text = replace_once(
        text,
        "- transform-pipeline async binding, terminal-safe logging, telemetry, fatal-exit handling, and public `TransformError` mapping;\n",
        "- transform-pipeline, create-error, install-warning, and final-output host binding, including terminal emission, telemetry, cleanup-before-exit, and public error mapping;\n",
        path,
    )
    old_arch = (
        "`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. "
        "`git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic "
        "VCS decision and command sequence behind injected runner and cleanup traits. `default_example` "
        "owns the pure default-acquisition routing predicate. `official_starter` owns exact "
        "official-repository classification and effect ordering behind typed package/document providers. "
        "`package_manager_transform` owns the no-op decision and typed conversion request while leaving "
        "mutations behind `PackageManagerConverter`. `transform_pipeline` owns the fixed transform order "
        "and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, "
        "discovered-version truthiness, stable choice ordering, and disabled-selection validation."
    )
    new_arch = (
        old_arch
        + " `create_error_policy` owns typed error classification and bounded display fields. "
        "`create_install_policy` owns package-install selection and warning data. "
        "`create_output_policy` owns bounded final terminal lines. "
        "`package_manager_install_policy` owns static profile selection and no-shell/no-local invocation "
        "metadata. `directory_prompt` owns argument/prompt selection, input policy, and typed validator "
        "propagation."
    )
    text = replace_once(text, old_arch, new_arch, path)
    text = replace_once(
        text,
        "Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c\n",
        """Package prompt GREEN:    4f00ff3ebe627acb5a15ead535f27d623d8a9a2c
Create error RED:       ae46b703826d866d21b5acd64fd681c0d9313e10
Create error GREEN:     de9be3378d3eba70ffd105bdc9692f60c6b9cc48
Install policy RED:     ff359432f3b91d1f164c68ed0270d62ec8b15f42
Install policy GREEN:   02eb3f5ba3a8733cf27c5377aaca3fae1ad09f2a
Install warning RED:    39a4ed083dcb021f673d51b599cf58bc7878e7a2
Install warning GREEN:  9423b807e72883f30c3e6bbf83fa918d2d846e34
Create output RED:      68f5ddf67e95b41cf45623a8ada402f9a6a1cd57
Create output GREEN:    f1ea07ef8404321a85fd0091cd612ba64779ef62
Install profiles RED:   b858e98565eb0415c6ab85bb120220529b9a981b
Install profiles GREEN: a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c
Directory prompt RED:   11131d1fc01536c151bdda04ba39fdc4aec5779a
Directory prompt GREEN: e0d5663e51c084f4f25051270ed9bb494df1b21a
Directory consolidation: 3ac9a5c4864602372d1b88f8e39986c700d52508
""",
        path,
    )
    return replace_once(
        text,
        "The crate contains 73 translated parity tests and 51 security regression tests, for 124 authored focused Rust tests.",
        "The crate contains 116 translated parity tests and 92 security regression tests, for 208 authored focused Rust tests.",
        path,
    )


def update_create_parity(text: str) -> str:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    sections = """## Create-command error policy tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| transform, conversion, download, and unknown errors | closed typed classification | implemented core | Safe-input action and display order are translated. |
| immediate fatal `process.exit(1)` | typed `Exit(1)` | intentional-hardening | Allows cleanup and telemetry flush before termination. |
| raw unbounded terminal error text | bounded control-safe display fields | intentional-hardening | Unknown errors are never rendered. |
| logging, telemetry, stack/class identity, and termination | production host binding | blocked | Must consume sanitized fields only and prove exactly-once effects. |

Detailed differences are in `CREATE_ERROR_POLICY_DIVERGENCES.md`.

## Create installation and warning tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| source/selected manager resolution and install gates | `apply_create_install_policy` | implemented core | Preserves ordering, skip, warning, and noninteractive install behavior. |
| repeated mutable availability lookup | one borrowed snapshot | intentional-hardening | Removes a time-of-check/time-of-use ambiguity for unstable providers. |
| raw example name in warning output | bounded terminal-safe renderer | intentional-hardening | Safe wording is preserved; hostile controls and large text are escaped/truncated. |
| real installation and logger effects | typed providers/host binding | blocked | Requires secure runner, exact errors, and supported-platform differentials. |

Detailed differences are in `CREATE_INSTALL_POLICY_DIVERGENCES.md`.

## Create output rendering tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| workspace summary, success, and get-started safe text | pure line renderers | implemented core | Exact safe headings/items and fixed text are translated. |
| raw project/workspace/path/description fields | bounded terminal-safe fields | intentional-hardening | Prevents line forging, OSC/BEL, bidi, and invisible-control spoofing. |
| unbounded workspaces and scripts | 256 workspace and 64 script limits | intentional-hardening | Emits an explicit truncation record. |
| path-relative/group/locale derivation and coloring | host binding | blocked | Requires Linux/macOS/Windows differential fixtures and no raw logger bypass. |

Detailed differences are in `CREATE_OUTPUT_POLICY_DIVERGENCES.md`.

## Package-manager installation profile tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| eight source profiles and order/default selection | static profile tables | implemented core | All six managers and eight profiles are translated. |
| `semver.satisfies` | injected matcher | partial | Production binding must prove Node-semver behavior. |
| `preferLocal: true` | `prefer_local: false` | intentional-hardening | Blocks generated-project executable substitution. |
| Windows `shell: true` | `shell: false` | intentional-hardening | Requires an explicit safe Windows shim adapter or typed unsupported result. |
| process execution | typed invocation metadata only | blocked | Needs canonical resolution, environment policy, deadlines, output bounds, and tree cleanup. |

Detailed differences are in `PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`.

## Project-directory prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| direct non-empty argument bypasses prompt without trimming | exact `Option<&str>` branch | implemented core | Preserves JavaScript truthiness for the empty-string case. |
| prompt message/default and display-only trim | typed prompt request | implemented core | Raw accepted answer is validated unchanged. |
| invalid direct result returned and ignored by caller | typed validator rejection | fixed in Rust and repaired TypeScript | Prevents acquisition against a validator-rejected path. |
| unbounded/control-bearing path input | 4096-byte and terminal-active-text rejection | intentional-hardening | Rejected values are not reflected in public core errors. |
| terminal and filesystem behavior | production `DirectoryPrompter`/`DirectoryValidator` | blocked | Needs bounded reading, cancellation, stable handles, Windows reparse-point behavior, and platform differentials. |

Detailed differences are in `DIRECTORY_PROMPT_DIVERGENCES.md`."""
    text = insert_before(text, "## Existing TypeScript test mapping", sections, path)
    return replace_once(
        text,
        "| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |",
        """| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |
| create-command error routing and terminal bounds | seven parity and eight security tests | implemented core and intentional-hardening evidence |
| create install decision and warning rendering | fourteen parity and thirteen security tests | implemented core and intentional-hardening evidence |
| final create output rendering | six parity and six security tests | implemented core and intentional-hardening evidence |
| package-manager installation profiles | eight parity and five security tests | implemented core and intentional-hardening evidence |
| directory argument/prompt/validator behavior | eight parity and nine security tests plus TypeScript regressions | implemented core, repaired oracle, providers blocked |""",
        path,
    )


def update_create_security(text: str) -> str:
    path = "packages/create-turbo/rust/SECURITY.md"
    text = replace_once(
        text,
        "- `packages/create-turbo/src/utils/is-default-example.ts`\n",
        """- `packages/create-turbo/src/utils/is-default-example.ts`
- the create-command package-install decision and unavailable-manager warning
- final create-command terminal output rendering
- package-manager installation profile selection and invocation policy
- project-directory argument/prompt selection and validator propagation
""",
        path,
    )
    trust = """The create-command error, install-warning, and final-output tranches accept attacker-influenced error, example, path, workspace, and description text at terminal boundaries. Their Rust cores return bounded control-safe strings and typed actions without logging or terminating directly.

Package-manager installation profile selection controls executable identity, arguments, standard input, local executable preference, and shell mediation. The reviewed core produces typed static invocation metadata but does not execute a process.

Project-directory selection accepts CLI text and terminal input before filesystem authority. The Rust core bounds and validates the text and requires validator failure to remain an error; prompting and filesystem inspection stay behind providers."""
    text = insert_before(
        text,
        "The Git initialization tranche adds decision boundaries for the project-root path",
        trust,
        path,
    )
    findings = """### CT-RS-028: Untrusted example names reach unavailable-manager warnings

**Severity:** Medium until production cutover

TypeScript interpolates the example name directly into two warning lines. Rust returns structured warning data and renders the example through bounded terminal-safe fields. Safe wording remains exact; controls, bidi/invisible format text, and oversized input cannot become raw terminal output.

Regression coverage is in `create_install_warning_parity.rs` and `create_install_warning_security.rs`.

### CT-RS-029: Project-local executable substitution during installation

**Severity:** High until TypeScript cutover

The TypeScript installer uses `preferLocal: true`, allowing a generated project to substitute a package-manager-named local executable. Rust installation metadata sets `prefer_local: false`, uses a closed manager enum, and keeps arguments in static profile tables.

A production runner still must prove canonical executable resolution, environment isolation, deadlines, output bounds, descendant cleanup, and platform behavior.

### CT-RS-030: Windows installation uses command-shell mediation

**Severity:** High until TypeScript cutover

The TypeScript installer sets `shell: true` on Windows. Rust sets `shell: false` for every platform. A Windows provider must resolve an approved executable or shim without command-string construction, or return a typed unsupported result.

### CT-RS-031: Generated create output can inject terminal controls

**Severity:** Medium until production cutover

TypeScript sends generated workspace descriptions and path-derived names through terminal coloring without a uniform control policy. Rust sanitizes attacker-influenced fields and complete lines, preserving safe text while escaping terminal, line, bidi, and invisible format controls.

### CT-RS-032: Create output volume is unbounded

**Severity:** Medium until production cutover

Rust caps interpolated fields at 1024 UTF-8 bytes, final lines at 4096 bytes, workspace records at 256, and scripts at 64. TypeScript has no equivalent explicit bounds.

### CT-RS-033: Rejected direct directories could continue into acquisition

**Severity:** High

The original direct-argument path returned `validateDirectory(dir)` even when `valid` was false, and the caller destructured the result without checking `valid`. A malformed, conflicting, or non-directory path could therefore proceed after validator rejection.

The repaired TypeScript caller throws a trusted `InvalidDirectoryError`; Rust represents validator rejection as `Result`, so it cannot be destructured as success. Regression coverage is in `directory-security.test.ts`, `directory_prompt_parity.rs`, and `directory_prompt_security.rs`.

### CT-RS-034: Project-directory input can spoof output or exhaust path work

**Severity:** Medium

Rust rejects directory input over 4096 UTF-8 bytes and rejects C0/C1, ESC/BEL, line separators, bidi overrides/isolates, zero-width and related invisible format controls before providers. Public core errors do not reflect the rejected value.

The production prompt must enforce the bound while reading, not only after submission, and preserve cancellation, EOF, signals, and non-TTY behavior.

### CT-RS-035: Directory validation and later mutation remain a TOCTOU boundary

**Severity:** High until provider closure

The decision core does not own filesystem mutation. A production validator and creator must use stable directory handles or private staging plus atomic promotion, define symlink/reparse-point behavior for every path component, bound enumeration and diagnostics, and pass Linux/macOS/Windows differential tests."""
    return insert_before(text, "## Security invariants", findings, path)


def update_program_ledger(text: str) -> str:
    path = "docs/typescript-deprecation.md"
    text = replace_once(
        text,
        "- `packages/turbo-utils/rust`: 70 translated parity tests and 36 security regression tests.",
        "- `packages/turbo-utils/rust`: 70 translated parity tests and 41 security regression tests.",
        path,
    )
    text = replace_once(
        text,
        "- `packages/create-turbo/rust`: 73 translated parity tests and 51 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, and transform-pipeline control flow.",
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.",
        path,
    )
    text = replace_once(
        text,
        "That is **284 authored Rust migration tests** on the integration branch.",
        "That is **373 authored Rust migration tests** on the integration branch.",
        path,
    )
    text = replace_once(
        text,
        "The latest `create-turbo` tranches remain unvalidated until their merge-head workflow compiles, tests, formats, and lints them successfully.",
        "The consolidated directory and provider-hardening tranches are not treated as reviewable until their merge-head workflow formats, compiles, tests, lints, and audits them successfully.",
        path,
    )
    text = replace_once(
        text,
        "Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **75%**.",
        "Across only the first three stages of the four active surfaces, the recalculated evidence-weighted estimate is about **78%**.",
        path,
    )
    text = replace_once(
        text,
        "| `packages/turbo-utils` | `packages/turbo-utils/rust` plus bindings | In progress | Production network/archive and registry providers, remaining utilities, Windows ACL/process/shim closure, bindings, callers, removal proof. |",
        "| `packages/turbo-utils` | `packages/turbo-utils/rust` plus bindings | In progress | Stable handle-relative directory validation/mutation, production network/archive and registry providers, remaining utilities, Windows ACL/process/shim closure, bindings, callers, removal proof. |",
        path,
    )
    text = replace_once(
        text,
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |",
        "| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README/`.gitignore`, Git, default/official routing, transform/prompt, error/install/output, installation-profile, and directory-selection cores are ported. Production prompt/filesystem/VCS/converter/JSON/process providers, bindings, telemetry, packaging, callers, platform differentials, and removal proof remain. |",
        path,
    )
    new_sections = """### Create-command error, installation, and output policies

The consolidated Rust policies preserve safe create-command error classification, package-install selection, unavailable-manager wording, workspace summaries, success text, and get-started instructions. They intentionally return typed actions and bounded rendered strings rather than logging or terminating inside the core.

Security closure includes terminal-control and directionality escaping, explicit field/line/count bounds, unknown-error non-disclosure, cleanup-before-exit capability, one-shot installer invocation, and a single availability snapshot. Production host bindings must prove exactly-once telemetry and output, error identity, path/group/locale derivation, coloring after sanitization, and no raw logger bypass.

### Package-manager installation profiles

The Rust core preserves all eight npm/pnpm/yarn/bun/nub/aube profiles while forbidding project-local executable preference and shell execution. Node-semver matching and real execution remain provider-owned. Production closure requires canonical executables, environment policy, deadlines, output bounds, descendant cleanup, Windows shim handling, and platform differentials.

### Project-directory selection

The Rust core preserves direct-argument versus prompt behavior, exact prompt metadata, and display-only trimming while validating the raw answer. It fixes the confirmed TypeScript fail-open path by making validator rejection typed, rejects terminal-active or oversized input before providers, and keeps raw rejected values out of public core errors. The TypeScript caller is repaired, but production Rust prompting and stable handle-relative filesystem validation remain blocked."""
    text = insert_before(
        text,
        "### Package-manager transform orchestration",
        new_sections,
        path,
    )
    text = replace_once(
        text,
        '- correct handling of safe names such as `..cache` rather than the TypeScript `startsWith("..")` false positive.\n',
        """- correct handling of safe names such as `..cache` rather than the TypeScript `startsWith("..")` false positive;
- rejection of option-like project basenames and existing symlinked path components;
- symlink-aware allow-listing, strict UTF-8 entry classification, and a 256-entry directory-inspection bound.
""",
        path,
    )
    text = replace_once(
        text,
        "- GitHub policy RED `903d7836a01e6ec47e4df339adc71456b4ecbd0d`, implementation `2e90ea8daa8542aa13cd94ceb981b653756789cb`.\n",
        """- GitHub policy RED `903d7836a01e6ec47e4df339adc71456b4ecbd0d`, implementation `2e90ea8daa8542aa13cd94ceb981b653756789cb`;
- directory-provider RED `53a55eefd92b919824374eb27159ff876e008147`, implementation `c77464a7e6f36813a3b52262e78caa9ee449bb72`, formatting `8ee51022fd84264e0abeee17014802da3afcae20`, and Clippy correction `e47b4994e0d97641c2f976231aa89833aa142913`.
""",
        path,
    )
    return replace_once(
        text,
        "- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.\n",
        """- package-manager prompt implementation: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`;
- create-error RED/GREEN: `ae46b703826d866d21b5acd64fd681c0d9313e10` / `de9be3378d3eba70ffd105bdc9692f60c6b9cc48`;
- install-policy RED/GREEN: `ff359432f3b91d1f164c68ed0270d62ec8b15f42` / `02eb3f5ba3a8733cf27c5377aaca3fae1ad09f2a`;
- create-output RED/GREEN: `68f5ddf67e95b41cf45623a8ada402f9a6a1cd57` / `f1ea07ef8404321a85fd0091cd612ba64779ef62`;
- install-profile RED/GREEN: `b858e98565eb0415c6ab85bb120220529b9a981b` / `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`;
- directory-prompt RED/GREEN: `11131d1fc01536c151bdda04ba39fdc4aec5779a` / `e0d5663e51c084f4f25051270ed9bb494df1b21a`;
- directory consolidation merge: `3ac9a5c4864602372d1b88f8e39986c700d52508`.
""",
        path,
    )


def update_repository_findings(text: str) -> str:
    path = "docs/rust-migration-security-findings.md"
    findings = """### RF-018: Invalid direct project directories could continue after validator rejection

**Status:** Repaired in TypeScript and represented fail-closed in the Rust core; production Rust providers remain blocked.

The original direct-argument path returned a validation object whose `valid: false` result was ignored by the create caller. The repaired TypeScript caller maps this to a trusted known input error, while Rust uses `Result` so rejection cannot be destructured as success.

### RF-019: Create-command error, warning, and final output accept terminal-active untrusted text

**Status:** Fixed in Rust policy/rendering cores; TypeScript production output remains.

Rust escapes terminal, line, directionality, and invisible format controls, applies explicit UTF-8 and record-count limits, and never renders unknown errors. Production bindings must emit only these reviewed strings, apply coloring afterwards, and prove there is no second raw-output path.

### RF-020: Package installation permits project-local substitution and Windows shell mediation

**Status:** Fixed in typed Rust invocation policy; production runner blocked.

The current TypeScript installer uses `preferLocal: true` and `shell: true` on Windows. Rust profile metadata forbids both and uses closed manager/program identities plus static arguments. Canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, and Windows shim behavior remain provider requirements.

### RF-021: Directory allow-listing ignores file type and can alias non-UTF-8 names

**Status:** Fixed in the Rust directory provider; TypeScript production path remains.

Rust treats every symlink entry as a conflict, including allow-listed names, and rejects non-UTF-8 names rather than applying lossy conversion before suffix/allow-list decisions.

### RF-022: Directory validation misses option-like basenames and symlinked ancestors

**Status:** Fixed for stable existing paths in Rust; handle-relative production closure remains blocked.

Rust validates the basename itself and inspects existing path components for symlinks before enumeration. Portable path checks remain raceable and may be conservative on symlink-aliased system paths, so supported-platform differential tests plus Unix directory handles and Windows reparse-point-aware handles are required.

### RF-023: Directory enumeration and conflict collection are unbounded

**Status:** Fixed in Rust with a 256-entry fail-closed limit; TypeScript production path remains.

The Rust validator stops before building an unbounded conflict collection and converts inspection overflow into an invalid-directory result.

### RF-024: Project-directory prompt input is unbounded and can contain invisible terminal controls

**Status:** Fixed in the Rust decision core and partially repaired TypeScript boundary; production prompt provider blocked.

Rust rejects input over 4096 UTF-8 bytes and rejects C0/C1, line separators, bidi controls, zero-width and related format controls before terminal or filesystem providers. The production prompt must enforce the limit while reading and preserve cancellation, EOF, signals, and non-TTY behavior."""
    text = insert_before(text, "## Required repository gates", findings, path)
    return replace_once(
        text,
        "- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;\n",
        """- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;
- close project-directory prompting and validation with bounded reads, trusted diagnostics, stable Unix directory handles, Windows reparse-point-aware handles, atomic/private staging, and platform differentials;
""",
        path,
    )


def main() -> None:
    ref = api("GET", f"git/ref/heads/{urllib.parse.quote(BRANCH, safe='/')}")
    object_value = ref.get("object")
    if not isinstance(object_value, dict) or not isinstance(object_value.get("sha"), str):
        raise SystemExit("unable to resolve integration branch")
    head_sha = object_value["sha"]
    commit = api("GET", f"git/commits/{head_sha}")
    tree_value = commit.get("tree")
    if not isinstance(tree_value, dict) or not isinstance(tree_value.get("sha"), str):
        raise SystemExit("unable to resolve integration tree")
    base_tree = tree_value["sha"]

    file_updaters = {
        "packages/create-turbo/rust/README.md": update_create_readme,
        "packages/create-turbo/rust/PARITY_MATRIX.md": update_create_parity,
        "packages/create-turbo/rust/SECURITY.md": update_create_security,
        "docs/typescript-deprecation.md": update_program_ledger,
        "docs/rust-migration-security-findings.md": update_repository_findings,
    }

    tree_entries: list[dict[str, object]] = []
    for path, updater in file_updaters.items():
        content = updater(read_file(path, head_sha))
        blob = api("POST", "git/blobs", {"content": content, "encoding": "utf-8"})
        blob_sha = blob.get("sha")
        if not isinstance(blob_sha, str):
            raise SystemExit(f"unable to create blob for {path}")
        tree_entries.append(
            {"path": path, "mode": "100644", "type": "blob", "sha": blob_sha}
        )

    for path in (SELF_PATH, WORKFLOW_PATH):
        tree_entries.append({"path": path, "mode": "100644", "type": "blob", "sha": None})

    tree = api("POST", "git/trees", {"base_tree": base_tree, "tree": tree_entries})
    tree_sha = tree.get("sha")
    if not isinstance(tree_sha, str):
        raise SystemExit("unable to create documentation tree")

    new_commit = api(
        "POST",
        "git/commits",
        {
            "message": (
                "docs: Update consolidated Rust migration ledgers\n\n"
                "Record the single-PR consolidation, create-turbo policy cores, "
                "directory fail-closed repair, turbo-utils provider hardening, "
                "test totals, weighted progress, security findings, and exact "
                "remaining production blockers. Remove the one-shot updater in "
                "the same atomic commit."
            ),
            "tree": tree_sha,
            "parents": [head_sha],
        },
    )
    new_sha = new_commit.get("sha")
    if not isinstance(new_sha, str):
        raise SystemExit("unable to create documentation commit")
    api(
        "PATCH",
        f"git/refs/heads/{urllib.parse.quote(BRANCH, safe='/')}",
        {"sha": new_sha, "force": False},
    )
    print(new_sha)


if __name__ == "__main__":
    main()
