#!/usr/bin/env python3
# Apply and document the bounded Node-semver matcher on the single migration branch.

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
WORKFLOW = ROOT / ".github/workflows/apply-create-node-semver-matcher.yml"

RED_SHA = "816216a20b5620ab381842e26ed322d9409b3cec"

SOURCE = ROOT / "packages/create-turbo/rust/src/package_manager_install_policy.rs"
ROOT_MANIFEST = ROOT / "Cargo.toml"
CRATE_MANIFEST = ROOT / "packages/create-turbo/rust/Cargo.toml"
DIVERGENCES = (
    ROOT
    / "packages/create-turbo/rust/PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md"
)


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:160]!r}"
        )
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: Path, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def apply_implementation() -> None:
    replace_once(
        SOURCE,
        '''        if requirement.is_empty() {
            return Err(NodeSemverMatcherError::InvalidRange);
        }

        // RED stub: the concrete Node-compatible parser is added in the
        // following GREEN commit. Keeping this callable makes the behavioral
        // tests compile and fail for the missing matching behavior.
        Ok(false)
''',
        '''        let range = requirement
            .parse::<node_semver::Range>()
            .map_err(|_error| NodeSemverMatcherError::InvalidRange)?;
        let Ok(version) = version.parse::<node_semver::Version>() else {
            return Ok(false);
        };

        Ok(version.satisfies(&range))
''',
    )

    replace_once(
        ROOT_MANIFEST,
        'nix = { version = "0.26.2", default-features = false, features = ["term"] }\n'
        'notify = "6.1.1"',
        'nix = { version = "0.26.2", default-features = false, features = ["term"] }\n'
        'node-semver = "=2.2.0"\n'
        'notify = "6.1.1"',
    )

    replace_once(
        CRATE_MANIFEST,
        '''[lib]
path = "src/lib.rs"

[lints]
workspace = true
''',
        '''[lib]
path = "src/lib.rs"

[dependencies]
node-semver.workspace = true

[lints]
workspace = true
''',
    )


def write_divergence_ledger(green_sha: str) -> None:
    DIVERGENCES.write_text(
        f'''# Package-manager installation profile and execution-policy ledger

## Scope

This tranche translates the profile selection, Node-semver matching, and process-invocation policy used by `packages/turbo-workspaces/src/install.ts` into the `create-turbo` Rust migration.

Rust target:

- `packages/create-turbo/rust/src/package_manager_install_policy.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_parity.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_security.rs`

TypeScript oracle and security evidence:

- `packages/turbo-workspaces/__tests__/install-meta.test.ts`
- `packages/turbo-workspaces/__tests__/install-security.test.ts`

## TDD evidence

- profile/execution RED: `b858e98565eb0415c6ab85bb120220529b9a981b`
- profile/execution GREEN: `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`
- concrete matcher RED: `{RED_SHA}`
- concrete matcher GREEN: `{green_sha}`

The matcher RED commit exported a callable bounded matcher that deliberately returned `false`, so the translated source-profile, prerelease, build-metadata, malformed-range, and malformed-version tests compiled and failed for missing behavior rather than for missing symbols.

## Preserved behavior

The Rust constants preserve all eight source profiles and their order:

| Manager | Profile | Semver selector | Install arguments | Default |
| --- | --- | --- | --- | --- |
| npm | `npm` | `*` | `install` | yes |
| pnpm | `pnpm6` | `6.x` | `install` | no |
| pnpm | `pnpm` | `>=7` | `install --fix-lockfile` | yes |
| yarn | `yarn` | `<2` | `install` | yes |
| yarn | `berry` | `>=2` | `install --no-immutable` | no |
| bun | `bun` | `^1.0.1` | `install` | yes |
| nub | `nub` | `*` | `install` | yes |
| aube | `aube` | `*` | `install` | yes |

Additional preserved contracts:

- missing and empty versions use the first profile marked as default;
- supplied versions are tested in source order and the first match wins;
- valid versions use the locked `node-semver` `2.2.0` `Version`/`Range` implementation;
- build metadata is ignored for range satisfaction and prerelease versions are excluded unless the range admits them;
- ordinary malformed versions are unsupported non-matches, matching JavaScript `semver.satisfies`;
- malformed static profile ranges are typed configuration errors;
- unsupported versions return no profile;
- matcher errors are propagated immediately without retry or default fallback;
- the selected command, arguments, project root, and ignored-stdin policy remain typed data;
- the version string never becomes a command or argument.

## Intentional security divergences

### CT-RS-029: Project-local executable substitution during installation

**Severity:** High until TypeScript cutover

TypeScript calls `execa` with `preferLocal: true`. A generated or attacker-influenced project can therefore place a package-manager-named executable in its local binary path and cause that program to run during installation.

The Rust invocation policy sets `prefer_local: false` for every manager and platform. Programs are represented by `WorkspacePackageManager`, a closed six-variant enum. Install arguments come only from static profile tables.

### CT-RS-030: Windows package-manager execution through a command shell

**Severity:** High until TypeScript cutover

TypeScript sets `shell: true` on Windows. Shell mediation expands the interpretation surface for executable resolution, metacharacters, quoting, environment expansion, file associations, and command shims.

The Rust invocation policy sets `shell: false` on every platform. A production Windows runner must resolve an approved package-manager executable or shim explicitly and execute it without constructing a shell command. If a manager cannot be launched safely without a shell, the provider must return a typed unsupported-platform error rather than weakening this policy.

### CT-RS-036: Free-form version matching lacked a bounded production implementation

**Severity:** Medium

An injected matcher left the final profile-selection decision to an unreviewed provider. A permissive provider could normalize hostile text, accept a different range grammar, or select the wrong package-manager profile.

`NodeSemverMatcher` now uses the exact locked `node-semver` `2.2.0` package and rejects version or range text over 256 UTF-8 bytes before parsing. Malformed versions are non-matches, while malformed repository-owned profile ranges fail as typed configuration errors. Tests cover every profile boundary, malformed and oversized text, build metadata, prerelease exclusion, Unicode confusables, controls, and large numeric components.

This is a direct dependency for the migration crate, but the same package identity was already present in the resolved workspace graph. The lockfile gate proves that no package identity changed when the direct edge was added.

## Security invariants

- No install invocation requests shell execution.
- No install invocation permits project-local executable preference.
- Program identity is a closed enum, not free-form text.
- Arguments are static reviewed slices.
- Project roots remain borrowed `Path` values, including non-UTF-8 Unix paths.
- Standard input is always ignored to prevent interactive hangs.
- Version and range text are bounded to 256 UTF-8 bytes before parsing.
- Malformed version text is never coerced, trimmed, normalized, or interpreted as a command.
- Profile scans are bounded to one or two entries per manager.
- The matcher performs no process execution, filesystem mutation, network access, credential access, `unsafe` code, or mutable global state.

## Advisory lookup

**Lookup date: 2026-09-01**

The RustSec advisory repository and the GitHub Advisory Database were searched for `node-semver`; no matching advisory was found for the locked `2.2.0` package at lookup time. This is not a substitute for the repository-wide lockfile audit, which remains authoritative and still reports the separately documented `webbrowser`, `h2`, and `quick-xml` blockers.

Compatibility references:

- `node-semver` Rust crate documentation for `Version`, `Range`, `MAX_LENGTH`, and Node/NPM compatibility;
- npm `node-semver` range, prerelease, and build-metadata semantics;
- the TypeScript `semver.satisfies` oracle in `packages/turbo-workspaces/src/install.ts`.

## Production blockers

The production runner must prove canonical executable resolution, explicit environment and configuration policy, no shell, no project-local substitution, a strict working-directory identity contract, bounded output, deadlines, cancellation and descendant cleanup, signal semantics, Windows shim handling, deterministic error mapping, and Linux/macOS/Windows differential fixtures. Removal proof must show that the TypeScript `install.ts` execution path is no longer loaded or shipped.
''',
        encoding="utf-8",
    )


def update_readme(green_sha: str) -> None:
    path = ROOT / "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "12. the package-manager installation profile and no-shell/no-local-executable invocation policy.",
        "12. the package-manager installation profile, bounded Node-semver matcher, and no-shell/no-local-executable invocation policy.",
    )
    replace_once(
        path,
        '''### Package-manager installation profile core

- preserves all eight source profiles and source-order/default selection;
- keeps Node-semver matching behind a provider boundary;
- represents programs as the closed six-manager enum and arguments as static slices;
- forbids project-local executable preference and shell execution on every platform;
- always ignores standard input.

The production runner remains blocked on canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, Windows shims, and platform differentials. See [`PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`](./PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md).
''',
        '''### Package-manager installation profile core

- preserves all eight source profiles and source-order/default selection;
- uses the locked `node-semver` 2.2.0 parser for Node/NPM-compatible matching;
- bounds both version and range text to 256 UTF-8 bytes before parsing;
- treats malformed versions as unsupported non-matches and malformed static ranges as typed configuration errors;
- preserves build-metadata and prerelease selection behavior through TypeScript oracle fixtures;
- represents programs as the closed six-manager enum and arguments as static slices;
- forbids project-local executable preference and shell execution on every platform;
- always ignores standard input.

The production runner remains blocked on canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, Windows shims, and platform differentials. See [`PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`](./PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md).
''',
    )
    replace_once(
        path,
        "- production package-manager workspace conversion plus the no-shell installation runner and Node-semver-compatible matcher;",
        "- production package-manager workspace conversion plus the no-shell installation runner;",
    )
    replace_once(
        path,
        "`package_manager_install_policy` owns static profile selection and no-shell/no-local invocation metadata.",
        "`package_manager_install_policy` owns bounded Node-semver profile selection and no-shell/no-local invocation metadata.",
    )
    replace_once(
        path,
        "Install profiles GREEN: a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c",
        "Install profiles GREEN: a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c\n"
        f"Node semver RED:        {RED_SHA}\n"
        f"Node semver GREEN:      {green_sha}",
    )
    replace_once(
        path,
        "The crate contains 116 translated parity tests and 92 security regression tests, for 208 authored focused Rust tests.",
        "The crate contains 120 translated parity tests and 95 security regression tests, for 215 authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = ROOT / "packages/create-turbo/rust/PARITY_MATRIX.md"
    replace_once(
        path,
        "| `semver.satisfies` | injected matcher | partial | Production binding must prove Node-semver behavior. |",
        "| `semver.satisfies` | bounded `NodeSemverMatcher` using locked `node-semver` 2.2.0 | implemented core | Profile boundaries, malformed versions, build metadata, and prerelease behavior have TypeScript/Rust oracle coverage. |\n"
        "| unbounded or permissively normalized version/range text | 256-byte pre-parse limits and strict parsing | intentional-hardening | Oversized input is typed failure; malformed versions are non-matches and malformed static ranges are configuration errors. |",
    )
    replace_once(
        path,
        "| package-manager installation profiles | eight parity and five security tests | implemented core and intentional-hardening evidence |",
        "| package-manager installation profiles and Node-semver matching | twelve parity and eight security tests plus TypeScript oracle cases | implemented core and intentional-hardening evidence |",
    )


def update_security() -> None:
    path = ROOT / "packages/create-turbo/rust/SECURITY.md"
    finding = '''### CT-RS-036: Free-form version matching lacked a bounded production implementation

**Severity:** Medium

The profile-selection core previously delegated `semver.satisfies` to an injected matcher. A permissive or incompatible provider could trim, normalize, coerce, or interpret untrusted version text differently and select the wrong package-manager profile.

Rust now uses the locked `node-semver` 2.2.0 implementation. Version and range inputs are rejected above 256 UTF-8 bytes before parsing; malformed versions are unsupported non-matches; malformed repository-owned profile ranges are typed configuration errors. Regression coverage includes every source profile boundary, build metadata, prerelease exclusion, Unicode confusables, terminal controls, oversized text, and unsafe numeric components.

The direct dependency edge does not add a new package identity to the lockfile because `node-semver` 2.2.0 was already resolved transitively. The lockfile validation gate rejects any unrelated package identity or dependency-record change.'''
    insert_before(path, "## Security invariants", finding)
    replace_once(
        path,
        "- The package-manager core accepts a closed enum, preserves the root as a path, does not forward version text, and cannot mutate files or execute a process directly.",
        "- The package-manager core accepts a closed enum, preserves the root as a path, does not forward version text, and cannot mutate files or execute a process directly.\n"
        "- Package-manager version and range matching is limited to 256 UTF-8 bytes, performs no trimming or Unicode normalization, and cannot influence program or argument construction.",
    )
    replace_once(path, "**Lookup date: 2026-08-31**", "**Lookup date: 2026-09-01**")
    replace_once(
        path,
        "- The package-manager orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.",
        "- The package-manager orchestration tranche adds no network call, filesystem operation, subprocess, or mutable global state.\n"
        "- The installation-profile tranche now directly uses the already-resolved `node-semver` 2.2.0 parser; no matching RustSec or GitHub advisory was found for that package at the 2026-09-01 lookup, and the complete lockfile audit remains authoritative.",
    )


def update_program_ledger(green_sha: str) -> None:
    path = ROOT / "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.",
        "- `packages/create-turbo/rust`: 120 translated parity tests and 95 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, bounded Node-semver installation profiles, and project-directory selection.",
    )
    replace_once(
        path,
        "That is **382 authored Rust migration tests** on the integration branch.",
        "That is **389 authored Rust migration tests** on the integration branch.",
    )
    replace_once(
        path,
        "The Rust core preserves all eight npm/pnpm/yarn/bun/nub/aube profiles while forbidding project-local executable preference and shell execution. Node-semver matching and real execution remain provider-owned. Production closure requires canonical executables, environment policy, deadlines, output bounds, descendant cleanup, Windows shim handling, and platform differentials.",
        "The Rust core preserves all eight npm/pnpm/yarn/bun/nub/aube profiles, implements bounded Node/NPM-compatible matching through locked `node-semver` 2.2.0, and forbids project-local executable preference and shell execution. Real execution remains provider-owned. Production closure requires canonical executables, environment policy, deadlines, output bounds, descendant cleanup, Windows shim handling, and platform differentials.",
    )
    replace_once(
        path,
        "- install-profile RED/GREEN: `b858e98565eb0415c6ab85bb120220529b9a981b` / `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`;",
        "- install-profile RED/GREEN: `b858e98565eb0415c6ab85bb120220529b9a981b` / `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`;\n"
        f"- Node-semver matcher RED/GREEN: `{RED_SHA}` / `{green_sha}`;",
    )


def update_repository_findings() -> None:
    path = ROOT / "docs/rust-migration-security-findings.md"
    finding = '''### RF-026: Package-manager profile selection depended on an unbounded matcher provider

**Status:** Fixed in the Rust profile core; production execution remains blocked.

The TypeScript installer selects profiles through JavaScript `semver.satisfies`. The first Rust profile tranche preserved that decision behind an injected matcher, leaving room for an incompatible provider to trim, coerce, normalize, or accept a different range grammar.

`NodeSemverMatcher` now uses the locked `node-semver` 2.2.0 implementation, rejects version and range text above 256 UTF-8 bytes before parsing, treats malformed versions as unsupported, and treats malformed repository-owned ranges as typed configuration errors. The direct dependency edge reused the package identity already present in the workspace lockfile; validation rejects unrelated lockfile changes.

The remaining installation risk is process authority, not profile parsing: canonical executable resolution, explicit environment policy, no project-local substitution, no shell, time/output bounds, cancellation, descendant cleanup, Windows shim handling, platform differentials, binding, packaging, and TypeScript removal proof remain required.'''
    insert_before(path, "## Required repository gates", finding)


def remove_one_shot_automation() -> None:
    for path in (WORKFLOW, SELF):
        if not path.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {path}")
        path.unlink()


def apply_documentation(green_sha: str) -> None:
    if len(green_sha) != 40 or any(character not in "0123456789abcdef" for character in green_sha):
        raise SystemExit(f"invalid GREEN commit SHA: {green_sha!r}")
    write_divergence_ledger(green_sha)
    update_readme(green_sha)
    update_parity_matrix()
    update_security()
    update_program_ledger(green_sha)
    update_repository_findings()
    remove_one_shot_automation()


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: apply_create_node_semver_matcher.py implement|document [green-sha]")
    command = sys.argv[1]
    if command == "implement" and len(sys.argv) == 2:
        apply_implementation()
        return
    if command == "document" and len(sys.argv) == 3:
        apply_documentation(sys.argv[2])
        return
    raise SystemExit("usage: apply_create_node_semver_matcher.py implement|document [green-sha]")


if __name__ == "__main__":
    main()
