#!/usr/bin/env python3
"""Record the validated bounded package-manager matcher security tranche."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
WORKFLOW = ROOT / ".github/workflows/record-bounded-semver-security.yml"
README = ROOT / "packages/create-turbo/rust/README.md"
PARITY = ROOT / "packages/create-turbo/rust/PARITY_MATRIX.md"
SECURITY = ROOT / "packages/create-turbo/rust/SECURITY.md"
SECURITY_TXT = ROOT / "packages/create-turbo/rust/security.txt"
PROGRAM = ROOT / "docs/typescript-deprecation.md"
REPOSITORY_SECURITY = ROOT / "docs/rust-migration-security-findings.md"

ORACLE_SHA = "3d0d7d63950f21acf4604536fdaffbfffa335798"
RED_SHA = "816216a20b5620ab381842e26ed322d9409b3cec"
GREEN_SHA = "a47192630977ffec2a4208f67d01fbd948a8aa97"
FORMAT_SHA = "149f43f4662d8ab3f44b35a2b21e4e3bfd8c3c31"
DIVERGENCE_SHA = "6fbab195a23fd567891a9c7e31f820534c83a0a6"
INVENTORY_SHA = "1873ad057e82b58f1efdd8dc0614c71ae4bc5f96"
RUN_ID = "33547336164"


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


def update_readme() -> None:
    replace_once(
        README,
        """### Package-manager installation profile core

- preserves all eight source profiles and source-order/default selection;
- keeps Node-semver matching behind a provider boundary;
- represents programs as the closed six-manager enum and arguments as static slices;
- forbids project-local executable preference and shell execution on every platform;
- always ignores standard input.

The production runner remains blocked on canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, Windows shims, and platform differentials. See [`PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`](./PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md).
""",
        """### Package-manager installation profile core

- preserves all eight source profiles and source-order/default selection;
- implements the six committed range literals through a dependency-free bounded matcher;
- rejects version or range text above 256 UTF-8 bytes before parsing;
- rejects non-ASCII, whitespace-bearing, and control-bearing version text rather than normalizing it;
- validates strict three-component versions, identifiers, leading-zero rules, and JavaScript-safe integers;
- preserves canonical build-metadata and prerelease selection behavior covered by the TypeScript oracle;
- represents programs as the closed six-manager enum and arguments as static slices;
- forbids project-local executable preference and shell execution on every platform;
- always ignores standard input.

The production runner remains blocked on canonical executable resolution, environment isolation, deadlines, bounded output, descendant cleanup, Windows shims, and platform differentials. See [`PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`](./PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md) and the repository-wide [test inventory](../../../docs/rust-migration-test-inventory.md).
""",
    )
    replace_once(
        README,
        "- production package-manager workspace conversion plus the no-shell installation runner and Node-semver-compatible matcher;",
        "- production package-manager workspace conversion plus the no-shell installation runner;",
    )
    replace_once(
        README,
        "`package_manager_install_policy` owns static profile selection and no-shell/no-local invocation metadata.",
        "`package_manager_install_policy` owns bounded closed-range profile selection and no-shell/no-local invocation metadata.",
    )
    replace_once(
        README,
        "Install profiles GREEN: a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c\nDirectory prompt RED:",
        f"Install profiles GREEN: a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c\nNode semver oracle:     {ORACLE_SHA}\nNode semver RED:        {RED_SHA}\nNode semver GREEN:      {GREEN_SHA}\nNode semver rustfmt:    {FORMAT_SHA}\nNode semver evidence:   {DIVERGENCE_SHA}\nTest inventory:         {INVENTORY_SHA}\nDirectory prompt RED:",
    )


def update_parity() -> None:
    replace_once(
        PARITY,
        "| `semver.satisfies` | injected matcher | partial | Production binding must prove Node-semver behavior. |",
        "| `semver.satisfies` for the six committed selectors | dependency-free bounded `NodeSemverMatcher` | implemented core | All eight profile records, first-match order, canonical versions, build metadata, prerelease exclusion, malformed versions, and unknown ranges have TypeScript/Rust coverage. |\n| npm edge-whitespace normalization | reject any whitespace or control before parsing | intentional-hardening | The TypeScript oracle stays GREEN by recording current normalization and using `it.failing` for the stricter policy. Canonical safe versions remain equivalent. |\n| arbitrary future range grammar | six reviewed literal selectors only | intentional-hardening | An unreviewed profile edit fails with `InvalidRange` instead of silently expanding parser authority. |",
    )
    replace_once(
        PARITY,
        "| package-manager installation profiles | eight parity and five security tests | implemented core and intentional-hardening evidence |",
        "| package-manager installation profiles and bounded matching | twelve parity and eight security test functions plus the GREEN TypeScript oracle | implemented core and intentional-hardening evidence |",
    )


def update_security() -> None:
    findings = f"""### CT-RS-036: Version matching was delegated to an unbounded provider

**Severity:** Medium

The first Rust profile tranche delegated `semver.satisfies` to an injected matcher. A permissive or incompatible provider could normalize hostile text, accept an unreviewed range grammar, or select the wrong installation profile.

The committed Rust matcher limits version and range text to 256 UTF-8 bytes, rejects non-ASCII, whitespace-bearing, and control-bearing input, validates strict three-component versions and identifiers, caps numeric components at JavaScript's maximum safe integer, and accepts only the six selectors currently present in the source profile table. Unknown selectors are typed `InvalidRange` errors rather than permissive fallbacks.

Regression coverage is in `package_manager_install_policy_parity.rs` and `package_manager_install_policy_security.rs`. TDD evidence is TypeScript oracle `{ORACLE_SHA}`, compiling RED `{RED_SHA}`, GREEN `{GREEN_SHA}`, and formatter proof `{FORMAT_SHA}`. GitHub Actions run `{RUN_ID}` compiled the exact formatted implementation, passed all migration parity/security tests, and passed Clippy with warnings denied.

### CT-RS-037: npm edge-whitespace normalization is intentionally rejected

**Severity:** Low intentional hardening

The TypeScript npm `semver` path accepts leading and trailing ASCII whitespace around an otherwise valid version. Package-manager versions normally come from executable discovery and have no legitimate need for hidden line or spacing characters. Normalizing those bytes can turn ambiguous terminal-derived text into a trusted installation profile.

Rust rejects any ASCII whitespace or control before parsing. The TypeScript suite remains GREEN: ordinary tests document current normalization and `it.failing` tests preserve the desired security expectation. Rust security tests require rejection. Canonical safe versions retain the same profile selection.

### CT-RS-038: Matcher grammar expansion requires explicit review

**Severity:** Low defense in depth

A general SemVer range parser would accept substantially more syntax than the six repository-owned selectors used by the installer. That authority is unnecessary for this boundary and could make a future profile edit silently executable without a dedicated TDD and security review.

Rust matches only `*`, `6.x`, `>=7`, `<2`, `>=2`, and `^1.0.1`. Any other range is a typed configuration error. Adding a selector requires a RED test, implementation change, parity entry, advisory review, and security record.
"""
    insert_before(SECURITY, "## Security invariants", findings)
    replace_once(
        SECURITY,
        "- The package-manager core accepts a closed enum, preserves the root as a path, does not forward version text, and cannot mutate files or execute a process directly.",
        "- The package-manager core accepts a closed enum, preserves the root as a path, does not forward version text, and cannot mutate files or execute a process directly.\n- Package-manager version and range input is bounded before parsing and cannot influence program or argument construction.\n- Only the six reviewed profile selectors are accepted; unknown range syntax fails typed and closed.\n- Whitespace, controls, Unicode lookalikes, invalid identifiers, leading-zero core components, and integers above JavaScript's safe range cannot select an install profile.",
    )
    replace_once(SECURITY, "**Lookup date: 2026-08-31**", "**Lookup date: 2026-09-01**")
    replace_once(
        SECURITY,
        "- The package-manager orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.",
        "- The package-manager orchestration tranche adds no dependency, network call, filesystem operation, subprocess, or mutable global state.\n- The bounded matcher implementation is dependency-free and therefore adds no transitive advisory surface.\n- The TypeScript oracle uses npm `semver` 7.6.2; the historical regular-expression denial-of-service issue fixed after 7.5.2 does not affect that installed version.\n- The lockfile-wide audit remains authoritative and the existing `webbrowser`, `h2`, and `quick-xml` findings remain blockers rather than exceptions.",
    )


def update_program() -> None:
    replace_once(
        PROGRAM,
        "The mandatory workflow is in `AGENTS.md`. Every tranche must use RED-first translated tests, retain TypeScript as an oracle until cutover, perform current advisory review, and update `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, this ledger, and the repository security index in the same change.",
        "The mandatory workflow is in `AGENTS.md`. Every tranche must use RED-first translated tests, retain TypeScript as an oracle until cutover, perform current advisory review, and update `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, this ledger, and the repository security index in the same change. The executable suite-by-suite mapping and remaining test debt are tracked in [`docs/rust-migration-test-inventory.md`](./rust-migration-test-inventory.md).",
    )
    replace_once(
        PROGRAM,
        "The Rust core preserves all eight npm/pnpm/yarn/bun/nub/aube profiles while forbidding project-local executable preference and shell execution. Node-semver matching and real execution remain provider-owned. Production closure requires canonical executables, environment policy, deadlines, output bounds, descendant cleanup, Windows shim handling, and platform differentials.",
        "The Rust core preserves all eight npm/pnpm/yarn/bun/nub/aube profiles, implements a dependency-free bounded matcher for the six committed selectors, and forbids project-local executable preference and shell execution. The TypeScript oracle remains GREEN and records npm whitespace normalization as an intentional Rust security divergence. Real execution remains provider-owned. Production closure requires canonical executables, environment policy, deadlines, output bounds, descendant cleanup, Windows shim handling, and platform differentials.",
    )


def update_repository_security() -> None:
    text = REPOSITORY_SECURITY.read_text(encoding="utf-8")
    if "### RF-027:" in text:
        raise SystemExit("RF-027 is already allocated; reconcile the repository finding index")
    finding = f"""### RF-027: Package-manager profile matching accepted unbounded provider authority

**Status:** Fixed in the Rust profile core; production process execution remains blocked.

The TypeScript installer selects profiles through npm `semver.satisfies`. The first Rust profile tranche left that decision behind an injected provider, which could normalize hostile input, accept a different grammar, or choose a different profile.

The Rust core now enforces 256-byte version and range limits, strict ASCII three-component versions, JavaScript-safe numeric components, validated prerelease/build identifiers, default prerelease exclusion, and a closed six-selector grammar. Whitespace and control characters are intentionally rejected instead of normalized. Unknown range syntax is a typed configuration error.

The TypeScript oracle remains GREEN and documents current whitespace normalization while retaining `it.failing` security expectations. TDD evidence is oracle `{ORACLE_SHA}`, RED `{RED_SHA}`, GREEN `{GREEN_SHA}`, formatter `{FORMAT_SHA}`, and divergence record `{DIVERGENCE_SHA}`. The source implementation adds no dependency or side-effect authority.

Production closure still requires canonical executable resolution, no shell or project-local substitution, explicit environment policy, deadlines, bounded output, cancellation, descendant cleanup, Windows shim behavior, supported-platform differential fixtures, host binding, packaging, caller cutover, and removal proof.
"""
    insert_before(REPOSITORY_SECURITY, "## Required repository gates", finding)


def write_security_txt() -> None:
    if SECURITY_TXT.exists():
        raise SystemExit(f"refusing to overwrite existing {SECURITY_TXT}")
    SECURITY_TXT.write_text(
        f"""# Rust migration security index for packages/create-turbo
# Canonical narrative review: SECURITY.md
# Vulnerability disclosure policy: ../../../SECURITY.md

Component: create-turbo-rs
Status: migration-core-only
Production-Cutover: blocked
TypeScript-Removal: not-started

Tranche: package-manager-profile-matching
TypeScript-Oracle: {ORACLE_SHA}
Rust-RED: {RED_SHA}
Rust-GREEN: {GREEN_SHA}
Rustfmt-Proof: {FORMAT_SHA}
Divergence-Record: {DIVERGENCE_SHA}
Test-Inventory: {INVENTORY_SHA}
GitHub-Actions-Run: {RUN_ID}

Security-Fix: CT-RS-029 project-local executable substitution rejected
Security-Fix: CT-RS-030 Windows shell mediation rejected
Security-Fix: CT-RS-036 bounded strict version matching implemented
Security-Fix: CT-RS-037 npm edge-whitespace normalization intentionally rejected
Security-Fix: CT-RS-038 matcher grammar restricted to six reviewed selectors

Input-Limit-Version-Bytes: 256
Input-Limit-Range-Bytes: 256
Allowed-Ranges: *,6.x,>=7,<2,>=2,^1.0.1
Shell-Execution: forbidden
Project-Local-Executable-Preference: forbidden
Unsafe-Code: none
New-Dependencies: none

Remaining-Security-Blockers: canonical executable resolution; environment isolation; deadlines; bounded output; cancellation; descendant cleanup; Windows shim handling; platform differentials; host binding; package provenance; caller cutover; TypeScript artifact removal; repository lockfile advisories
""",
        encoding="utf-8",
    )


def remove_automation() -> None:
    for path in (SELF, WORKFLOW):
        if not path.exists():
            raise SystemExit(f"expected one-shot automation path is missing: {path}")
        path.unlink()


def main() -> None:
    update_readme()
    update_parity()
    update_security()
    update_program()
    update_repository_security()
    write_security_txt()
    remove_automation()
    print("recorded bounded matcher parity, security, and test inventory evidence")


if __name__ == "__main__":
    main()
