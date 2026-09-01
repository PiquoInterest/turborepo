#!/usr/bin/env python3
"""Record the reviewed Bun workspace-glob Rust migration tranche."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RED_SHA = "7d3657eff34202f912aa0f78437f2dff833c19f3"
FORMAT_SHA = "f05d52f6254043fab656d16ba24311fe6d11b5b1"
GREEN_SHA = "914fd359d5ee5cf86e3b6987e589192609cba957"
RED_RUN = "33555721991"
RED_JOB = "100015929776"
GREEN_RUN = "33556090751"
GREEN_JOB = "100017159744"


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


def update_readme() -> None:
    path = "packages/turbo-workspaces/rust/README.md"
    replace_once(
        path,
        "The first completed core ports the read-only `getWorkspaceDetails` orchestration contract from `src/get-workspace-details.ts`:",
        "The first completed core ports the read-only `getWorkspaceDetails` orchestration contract from `src/get-workspace-details.ts`. The second ports the pure Bun workspace-glob compatibility predicate from `src/utils.ts`:",
    )
    section = f"""## Bun workspace-glob compatibility

The Rust predicate preserves the TypeScript compatibility contract for safe inputs:

- empty lists and literal workspace paths are accepted;
- a single `*` in the final path segment is accepted;
- `**`, a `*` before the final path segment, and `!`, `[`, `]`, `{{`, or `}}` are rejected;
- question marks and safe Unicode remain ordinary characters, matching the source implementation rather than introducing a broader glob grammar.

The Rust boundary intentionally adds limits that the TypeScript predicate lacks: 4,096 UTF-8 bytes per glob, 256 globs, and 65,536 total glob bytes. It also rejects terminal-active and invisible control/format text before compatibility classification. These differences are security hardening and are retained as green `it.failing` TypeScript evidence.

TDD evidence:

- TypeScript focused oracle: 49 of 49 tests green in run `{RED_RUN}`, job `{RED_JOB}`;
- Rust behavioral RED: `{RED_SHA}`;
- formatting-only repair: `{FORMAT_SHA}`;
- Rust GREEN: `{GREEN_SHA}`;
- focused GREEN validation: run `{GREEN_RUN}`, job `{GREEN_JOB}`;
- added evidence: 12 parity tests and 6 security tests.

Production glob discovery and expansion are still TypeScript-owned. A Rust provider must enforce limits before filesystem expansion, confine results to a stable root identity, define symlink/reparse-point behavior, bound result counts and diagnostics, and pass Linux, macOS, and Windows differential tests before cutover."""
    insert_before(path, "## TDD evidence", section)
    replace_once(
        path,
        "- Rust parity tests: 6\n- Rust security tests: 5",
        "- workspace-details Rust parity tests: 6\n- workspace-details Rust security tests: 5\n- Bun workspace-glob Rust parity tests: 12\n- Bun workspace-glob Rust security tests: 6\n- total Rust evidence in this crate: 18 parity and 11 security tests",
    )
    replace_once(
        path,
        "pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-details.test.ts",
        "pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-details.test.ts __tests__/utils.test.ts __tests__/bun-workspace-glob-security.test.ts",
    )
    replace_once(
        path,
        "GitHub Actions remains authoritative for Rust execution. The code and tests are committed, but Rust GREEN execution is still pending while hosted jobs are queued. The integration step must remove the nested `[workspace]`, inherit the root edition/lints, add this crate to the root workspace, regenerate the root lockfile with an exact one-package delta, and rerun all gates.",
        f"GitHub Actions remains authoritative. The Bun workspace-glob tranche passed TypeScript oracle, Rust format, check, all crate tests, Clippy with warnings denied, and no-`unsafe` validation in run `{GREEN_RUN}`, job `{GREEN_JOB}`. Root-workspace integration, lockfile-wide advisory validation, production providers, and supported-platform differentials remain separate gates.",
    )
    replace_once(
        path,
        "`get_workspace_details` owns only deterministic orchestration. `WorkspaceDetailsProvider` owns path inspection, manager detection, and manager-specific reading. The core cannot read a file, traverse a directory, spawn a process, access the network, or broaden the manager registry.",
        "`get_workspace_details` owns only deterministic orchestration. `WorkspaceDetailsProvider` owns path inspection, manager detection, and manager-specific reading. `is_compatible_with_bun_workspaces` owns the bounded pure compatibility predicate and grants no filesystem expansion authority. The crate cannot read a file, traverse a directory, spawn a process, access the network, or broaden the manager registry through these cores.",
    )


def update_parity_matrix() -> None:
    path = "packages/turbo-workspaces/rust/PARITY_MATRIX.md"
    section = f"""## Bun workspace-glob compatibility

| TypeScript boundary | Rust boundary | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| `Array.every(validator)` over workspace globs | borrowed slice plus `all` | implemented-core | Empty-list truth and early rejection are preserved. |
| `**` rejection | exact substring rejection | implemented-core | Source behavior is translated. |
| `*` before the final slash-delimited segment | prefix check before the final `/` | implemented-core | Intermediate wildcards are rejected; a wildcard in the last segment remains accepted. |
| `!`, `[`, `]`, `{{`, `}}` rejection | fixed byte checks | implemented-core | No regex or external glob parser broadens the grammar. |
| `?` and safe Unicode are ordinary text | UTF-8 borrowed-string handling | implemented-core | Safe-input behavior remains identical. |
| unbounded array and string scanning | 4,096-byte item, 256-item, and 65,536-byte aggregate limits | intentional-hardening | Oversized input fails closed before further interpretation. |
| controls, bidi isolates/overrides, and invisible format text are accepted as ordinary text | explicit terminal-active/invisible rejection | intentional-hardening | Prevents unsafe values from reaching later paths or diagnostics. |
| real `fast-glob` expansion and workspace reads | production Rust discovery provider | blocked | Requires root confinement, stable identities, link/reparse policy, result limits, cancellation, and platform differentials. |

TDD chain: Rust RED `{RED_SHA}`, formatting `{FORMAT_SHA}`, Rust GREEN `{GREEN_SHA}`. TypeScript remained green with 49 of 49 focused tests in run `{RED_RUN}`, job `{RED_JOB}`; the final GREEN gate passed in run `{GREEN_RUN}`, job `{GREEN_JOB}`."""
    insert_before(path, "## Test mapping", section)
    replace_once(
        path,
        "| provider-only failure boundaries | six security/provider tests across both Rust files | added security evidence |",
        "| provider-only failure boundaries | six security/provider tests across both Rust files | added security evidence |\n| `utils.test.ts` Bun compatibility cases | `bun_workspace_glob_parity.rs` | 7 source cases mapped by 12 parity tests |\n| `bun-workspace-glob-security.test.ts` | `bun_workspace_glob_security.rs` | four green expected-failure oracles mapped by six Rust security tests |",
    )
    replace_once(
        path,
        "TDD chain: TypeScript oracle `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`, Rust RED `2d4cc22e6a821c88882a87d604746dabbaa95fe2`, Rust GREEN `263ddc22d5b5f544768f4e089c92892339b0dce8`.",
        f"Workspace-details TDD chain: TypeScript oracle `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`, Rust RED `2d4cc22e6a821c88882a87d604746dabbaa95fe2`, Rust GREEN `263ddc22d5b5f544768f4e089c92892339b0dce8`.\n\nBun workspace-glob TDD chain: TypeScript green run `{RED_RUN}`/job `{RED_JOB}`, Rust RED `{RED_SHA}`, formatting `{FORMAT_SHA}`, Rust GREEN `{GREEN_SHA}`, GREEN run `{GREEN_RUN}`/job `{GREEN_JOB}`.",
    )


def update_security_review() -> None:
    path = "packages/turbo-workspaces/rust/SECURITY.md"
    replace_once(
        path,
        "This review covers only the read-only workspace-details orchestration core. It does not approve production filesystem, parser, process, binding, packaging, or removal behavior.",
        "This review covers the read-only workspace-details orchestration core and the pure Bun workspace-glob compatibility predicate. It does not approve production filesystem expansion, parser, process, binding, packaging, or removal behavior.",
    )
    replace_once(
        path,
        "Detector and parser failures may indicate malformed, conflicting, or adversarial state and must not be converted into permission for another parser.",
        "Detector and parser failures may indicate malformed, conflicting, or adversarial state and must not be converted into permission for another parser. Workspace-glob text is repository-controlled and can influence later filesystem enumeration and diagnostics, so compatibility checks must be bounded before granting expansion authority.",
    )
    findings = f"""### TW-RS-006: Bun compatibility validation accepted unbounded workspace-glob input

**Severity:** Medium

The TypeScript predicate scans every supplied glob without a per-item, item-count, or aggregate byte limit. A hostile `package.json` or workspace configuration can therefore force disproportionate scanning and carry oversized values into later conversion stages.

**Rust fix:** reject any glob above 4,096 UTF-8 bytes, more than 256 globs, or more than 65,536 aggregate bytes. Aggregate accounting uses checked addition and fails closed on overflow.

**Regression tests:** `rejects_a_workspace_glob_above_the_per_glob_limit`, `rejects_more_workspace_globs_than_the_count_limit`, `rejects_workspace_globs_above_the_total_byte_limit`, and `accepts_a_safe_glob_at_the_exact_per_glob_limit`.

### TW-RS-007: Terminal-active and invisible glob text passed compatibility validation

**Severity:** Medium

The source rejects only `!`, brackets, braces, and selected `*` layouts. C0/C1 controls, line separators, bidi overrides/isolates, zero-width characters, and related format controls remain compatible and can later forge diagnostics or obscure the actual path expression.

**Rust fix:** reject C0/C1 controls plus the reviewed invisible and directionality ranges before wildcard classification. Safe Unicode names remain accepted. This is an intentional security divergence recorded by four green TypeScript `it.failing` tests.

**Regression tests:** `rejects_terminal_active_and_invisible_workspace_globs` and `does_not_reject_safe_unicode_workspace_names`.

### TW-RS-008: Compatibility validation does not close filesystem glob-expansion authority

**Severity:** High until provider closure

This pure predicate does not enumerate the filesystem. The current TypeScript path later uses `fast-glob` against a caller-selected root. A production Rust provider must apply byte/count limits before expansion, hold or revalidate a stable root identity, constrain every result to that root, define symlink and Windows reparse-point behavior, bound match counts and diagnostics, handle cancellation and concurrent mutation, and pass Linux, macOS, and Windows differential fixtures.

The reviewed Rust core deliberately provides no filesystem or process capability, so passing compatibility validation cannot itself read a path.

TDD evidence: TypeScript 49/49 green in run `{RED_RUN}`, job `{RED_JOB}`; Rust RED `{RED_SHA}`; formatting `{FORMAT_SHA}`; Rust GREEN `{GREEN_SHA}`; final focused validation run `{GREEN_RUN}`, job `{GREEN_JOB}`."""
    insert_before(path, "## TDD evidence", findings)
    replace_once(
        path,
        "- Rust evidence: 6 parity tests and 5 security tests",
        "- workspace-details Rust evidence: 6 parity tests and 5 security tests\n- Bun workspace-glob Rust evidence: 12 parity tests and 6 security tests\n- Bun TypeScript/Rust RED/GREEN validation: runs 33555721991 and 33556090751",
    )
    replace_once(
        path,
        "The RED commit compiled by construction around the final public API and deliberately omitted manager iteration. Hosted Rust execution is pending because GitHub currently has no active runner for the queued job; this is recorded as a validation blocker, not a pass.",
        "The workspace-details RED commit compiled around the final provider API and deliberately omitted manager iteration. The Bun workspace-glob RED exported the final bounded API and returned false; run 33555721991 proved that TypeScript stayed green while the Rust tests compiled and failed behaviorally. Run 33556090751 then passed the complete focused GREEN gate.",
    )
    replace_once(
        path,
        "- Known public error type strings and messages remain deterministic.",
        "- Known public error type strings and messages remain deterministic.\n- Bun compatibility work is bounded by per-glob, count, and aggregate byte limits before interpretation.\n- Terminal-active and invisible glob text is rejected while safe Unicode remains accepted.\n- The compatibility predicate adds no filesystem, process, network, dependency, or `unsafe` authority.",
    )
    replace_once(
        path,
        "This tranche adds no third-party dependency. The RustSec advisory database, GitHub Advisory Database, and the existing repository dependency findings therefore introduce no new package-specific disposition for this core.",
        "These tranches add no third-party dependency. The RustSec advisory database, GitHub Advisory Database, and the existing repository dependency findings therefore introduce no new package-specific disposition for these cores.",
    )
    replace_once(
        path,
        "- bounded no-follow directory and metadata reads;",
        "- bounded no-follow directory and metadata reads;\n- bounded root-confined workspace-glob expansion with explicit link/reparse-point, result-count, cancellation, and concurrent-mutation behavior;",
    )


def write_security_index() -> None:
    path = ROOT / "packages/turbo-workspaces/rust/security.txt"
    path.write_text(
        f"""# Rust migration security index for packages/turbo-workspaces
# Canonical narrative review: SECURITY.md
# Vulnerability disclosure policy: ../../../SECURITY.md

Component: turbo-workspaces-rs
Status: implemented-core-validated
Production-Cutover: blocked
TypeScript-Removal: not-started
Workspace-Details-TypeScript-Oracle: 4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff
Workspace-Details-TypeScript-Result: 5/5 green; workflow 33551576871; job 100002027683
Workspace-Details-Rust-RED: 2d4cc22e6a821c88882a87d604746dabbaa95fe2
Workspace-Details-Rust-GREEN: 263ddc22d5b5f544768f4e089c92892339b0dce8
Bun-Glob-TypeScript-Result: 49/49 focused green; workflow {RED_RUN}; job {RED_JOB}
Bun-Glob-Rust-RED: {RED_SHA}
Bun-Glob-Rust-Format: {FORMAT_SHA}
Bun-Glob-Rust-GREEN: {GREEN_SHA}
Bun-Glob-GREEN-Validation: workflow {GREEN_RUN}; job {GREEN_JOB}
Rust-Parity-Tests: 18
Rust-Security-Tests: 11
Rust-Execution: focused-green

Security-Fix: TW-RS-001 fixed closed six-manager detection order
Security-Fix: TW-RS-002 no parser fallback after detector or reader error
Security-Fix: TW-RS-003 managers receive only provider-validated absolute path
Security-Fix: TW-RS-004 at most six detections and one read
Security-Blocker: TW-RS-005 stable bounded no-follow filesystem provider required
Security-Fix: TW-RS-006 Bun glob compatibility bounded to 4096 bytes per item, 256 items, and 65536 aggregate bytes
Security-Fix: TW-RS-007 terminal-active and invisible glob text rejected before compatibility classification
Security-Blocker: TW-RS-008 root-confined bounded filesystem expansion and platform closure required
New-Dependencies: none
Unsafe-Code: none
Shell-Execution: none
Network-Authority: none
""",
        encoding="utf-8",
    )


def update_package_test_inventory() -> None:
    path = "packages/turbo-workspaces/rust/TEST_INVENTORY.md"
    replace_once(
        path,
        "The package currently has eight executable Jest suites and one shared test-support module.",
        "The package currently has nine executable Jest suites and one shared test-support module after adding the Bun workspace-glob security oracle.",
    )
    replace_once(
        path,
        "| `utils.test.ts` | no dedicated Rust port | not ported | Directory/path/package utility behavior and malformed-input boundaries. |",
        "| `utils.test.ts` | `bun_workspace_glob_parity.rs` | partial: 7 of 45 source tests mapped by 12 Rust parity tests | 38 utility tests remain, including package declarations, paths, workspace parsing/expansion, and mutation helpers. |\n| `bun-workspace-glob-security.test.ts` | `bun_workspace_glob_security.rs` | mapped security evidence: 4 green `it.failing` cases mapped by 6 Rust tests | Production glob expansion, root identity, link/reparse behavior, and platform differentials remain. |",
    )
    replace_once(
        path,
        "- workspace-details parity tests: 6\n- workspace-details security tests: 5\n- total in this crate: 11\n- repository authored migration-test total after this tranche: 393\n\nThe previous repository ledger contained 382 authored tests. These 11 tests are committed and mapped, but their hosted Rust execution remains pending; the count must not be presented as 393 validated tests until CI runs.",
        "- workspace-details parity tests: 6\n- workspace-details security tests: 5\n- Bun workspace-glob parity tests: 12\n- Bun workspace-glob security tests: 6\n- total in this crate: 29\n- repository authored migration-test total after this tranche: 411\n\nThe focused Bun tranche is validated. The 411 count describes authored migration evidence, not production readiness or repository-wide validation. Thirty-eight cases in `utils.test.ts` remain unported, and no TypeScript package has passed binding, packaging, caller-cutover, or removal gates.",
    )
    replace_once(
        path,
        "1. Manager discovery and exact metadata precedence from `managers.test.ts`.\n2. Read-only utility behavior from `utils.test.ts`.",
        "1. The remaining 38 read-only and mutation utility cases from `utils.test.ts`, split by trust boundary.\n2. Manager discovery and exact metadata precedence from `managers.test.ts`.",
    )


def update_repository_test_inventory() -> None:
    path = "docs/rust-migration-test-inventory.md"
    replace_once(
        path,
        "The source package currently has seven executable Jest suites plus one support module.",
        "The source package currently has nine executable Jest suites plus one support module, including the workspace-details and Bun workspace-glob security suites added for this migration.",
    )
    replace_once(
        path,
        "| `__tests__/install-meta.test.ts` | `package_manager_install_policy_parity.rs` | Mapped for the eight profile records and six current range literals | Shared platform differential fixture and production runner binding. |",
        "| `__tests__/workspace-details.test.ts` | `workspace_details_parity.rs`, `workspace_details_security.rs` | Mapped core: 6 parity and 5 security tests | Production filesystem/parser providers, async binding, platforms, packaging, callers, and removal proof. |\n| `__tests__/install-meta.test.ts` | `package_manager_install_policy_parity.rs` | Mapped for the eight profile records and six current range literals | Shared platform differential fixture and production runner binding. |",
    )
    replace_once(
        path,
        "| `__tests__/utils.test.ts` | none | Not ported | Port utility behavior, malformed-input cases, path and serialization boundaries. |",
        "| `__tests__/utils.test.ts` | `bun_workspace_glob_parity.rs` | Partial: 7 of 45 source tests mapped by 12 Rust parity tests | 38 cases remain across declaration parsing, paths, workspace parsing/expansion, mutation, and error boundaries. |\n| `__tests__/bun-workspace-glob-security.test.ts` | `bun_workspace_glob_security.rs` | Security-evidence: four green `it.failing` cases mapped by six Rust tests | Production root-confined expansion and platform closure remain. |",
    )
    replace_once(
        path,
        "The next high-impact test migration is the `@turbo/workspaces` conversion surface. Five source suites remain wholly or substantially unported: `index.test.ts`, `managers.test.ts`, `utils.test.ts`, `nub.test.ts`, and `aube.test.ts`.",
        "The next high-impact test migration is the remaining `@turbo/workspaces` conversion surface. Four suites remain wholly or substantially unported: `index.test.ts`, `managers.test.ts`, `nub.test.ts`, and `aube.test.ts`; `utils.test.ts` is partial with 38 of 45 cases remaining.",
    )
    bun_section = f"""## Bun workspace-glob TDD chain

- TypeScript oracle: 49 of 49 focused tests green in run `{RED_RUN}`, job `{RED_JOB}`.
- Rust compiling behavioral RED: `{RED_SHA}`.
- Formatting-only repair: `{FORMAT_SHA}`.
- Rust GREEN implementation: `{GREEN_SHA}`.
- Focused GREEN validation: run `{GREEN_RUN}`, job `{GREEN_JOB}`.

Seven `utils.test.ts` cases are mapped by twelve Rust parity tests. Four expected-failure TypeScript security cases remain green and are mapped by six Rust security tests. The repository authored migration-test count is now 411, while 38 `utils.test.ts` cases remain unported."""
    insert_before(path, "## Bounded matcher TDD chain", bun_section)


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.\n- `crates/turborepo-telemetry::events::package`: 9 translated parity tests and 7 security regression tests for the package-facing telemetry contract.",
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.\n- `packages/turbo-workspaces/rust`: 18 translated parity tests and 11 security regression tests across workspace-details orchestration and Bun workspace-glob compatibility.\n- `crates/turborepo-telemetry::events::package`: 9 translated parity tests and 7 security regression tests for the package-facing telemetry contract.",
    )
    replace_once(
        path,
        "That is **382 authored Rust migration tests** on the integration branch.",
        "That is **411 authored Rust migration tests** on the integration branch.",
    )
    replace_once(
        path,
        "The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The bounded `NO_PROXY` tranche advances turbo-utils network-policy core and test evidence without completing the production request-execution, binding, packaging, caller, platform, or removal stages, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the recalculated evidence-weighted estimate is about **78%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
        "The five active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The Bun workspace-glob tranche advances turbo-workspaces core and test evidence without completing filesystem expansion, binding, packaging, caller, platform, or removal stages, so the rounded repository score remains about **8%**, meaning roughly **92%** of the production migration program remains. Across only the first three stages of the five active surfaces, the recalculated evidence-weighted estimate is about **77%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
    )
    replace_once(
        path,
        "| `packages/turbo-workspaces` | Rust CLI/library | Queued and partially exposed through provider boundary | Package-manager adapters, complete six-manager conversion, lock/workspace mutation semantics, rollback, process policy, and packaging. |",
        "| `packages/turbo-workspaces` | `packages/turbo-workspaces/rust` plus existing shared Rust policies | In progress | Workspace-details and bounded Bun compatibility cores are ported. Remaining utilities, manager adapters, root-confined expansion, complete six-manager conversion, lock/workspace mutation semantics, rollback, process policy, bindings, packaging, callers, platforms, and removal proof remain. |",
    )
    section = f"""## Current `turbo-workspaces` tranches

The crate now has two validated Rust cores:

1. workspace-details orchestration, with exact six-manager order and no parser fallback after provider errors;
2. Bun workspace-glob compatibility, preserving safe source behavior while adding 4,096-byte per-item, 256-item, and 65,536-byte aggregate limits plus terminal-active/invisible-text rejection.

The focused TypeScript oracle remained green with 49 of 49 tests. Rust TDD is RED `{RED_SHA}`, formatting `{FORMAT_SHA}`, GREEN `{GREEN_SHA}`, and final validation run `{GREEN_RUN}`/job `{GREEN_JOB}`. The crate now contains 18 parity and 11 security tests. Seven of 45 `utils.test.ts` cases are mapped; 38 remain.

Production expansion and conversion remain blocked on stable root identity, root confinement, link/reparse-point behavior, result and parser bounds, transactional mutation/rollback, process policy, bindings, supported-platform differentials, packaging, callers, and removal proof."""
    insert_before(path, "## Current `create-turbo` tranches", section)


def update_repository_security_findings() -> None:
    path = ROOT / "docs/rust-migration-security-findings.md"
    text = path.read_text(encoding="utf-8")
    ids = [int(value) for value in re.findall(r"^### RF-(\d+):", text, re.MULTILINE)]
    finding_id = max(ids, default=0) + 1
    title = "Bun workspace compatibility accepted unbounded terminal-active glob text"
    if title in text:
        raise SystemExit("repository Bun workspace-glob finding already exists")
    finding = f"""### RF-{finding_id:03d}: {title}

**Status:** Fixed in the pure Rust compatibility core; production filesystem expansion remains blocked.

The TypeScript predicate scans every workspace glob without explicit per-item, item-count, or aggregate byte limits and accepts control, directionality, and invisible format text as ordinary characters. Rust preserves safe wildcard and fancy-pattern behavior while rejecting input above 4,096 bytes per glob, 256 globs, or 65,536 aggregate bytes and rejecting terminal-active/invisible text before interpretation.

TypeScript remains green through four expected-failure security oracles. TDD evidence is RED `{RED_SHA}`, formatting `{FORMAT_SHA}`, GREEN `{GREEN_SHA}`, with final focused validation in run `{GREEN_RUN}`, job `{GREEN_JOB}`. No dependency, filesystem operation, process, network access, credential source, mutable global state, or `unsafe` code was added.

Production closure still requires a root-confined filesystem-expansion provider with stable identities, symlink and Windows reparse-point policy, bounded matches and diagnostics, cancellation, concurrent-mutation behavior, supported-platform differential tests, binding, packaging, callers, and TypeScript-removal proof."""
    marker = "## Required repository gates"
    if text.count(marker) != 1:
        raise SystemExit("repository security gate marker changed")
    path.write_text(text.replace(marker, f"{finding}\n\n{marker}", 1), encoding="utf-8")


def main() -> None:
    update_readme()
    update_parity_matrix()
    update_security_review()
    write_security_index()
    update_package_test_inventory()
    update_repository_test_inventory()
    update_program_ledger()
    update_repository_security_findings()

    for relative in [
        ".github/workflows/validate-bun-workspace-glob-red.yml",
        ".github/workflows/document-bun-workspace-glob.yml",
        "tools/migration/document_bun_workspace_glob_tranche.py",
    ]:
        target = ROOT / relative
        if not target.exists():
            raise SystemExit(f"expected one-shot file is missing: {relative}")
        target.unlink()


if __name__ == "__main__":
    main()
