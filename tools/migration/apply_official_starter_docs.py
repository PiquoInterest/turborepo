#!/usr/bin/env python3
"""Record the reviewed official-starter Rust migration tranche.

This script is intentionally one-shot. The workflow deletes it and itself in the
same documentation commit after focused Rust validation succeeds.
"""

from pathlib import Path

RED_SHA = "2ca25bd457cbe216f345b5f67cf9ac32f43a2c7a"
GREEN_SHA = "cd2ba74b3040e654a63c9799e42c35a12f2c4dbc"
ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/apply-official-starter-docs.yml"
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
    target = ROOT / "packages/create-turbo/rust/OFFICIAL_STARTER_DIVERGENCES.md"
    if target.exists():
        raise SystemExit(f"refusing to replace existing divergence ledger: {target}")
    target.write_text(
        """# `official-starter` parity and divergence ledger

## Scope and status

TypeScript oracle: `packages/create-turbo/src/transforms/official-starter.ts`.

Rust target: `packages/create-turbo/rust/src/official_starter.rs`.

The reviewed Rust tranche implements the decision and effect-ordering core. It is not yet the production transform. Filesystem access, JSON parsing and serialization, JavaScript host errors, and package publication remain behind typed providers and bindings that are deliberately absent until their safety contracts are proven.

## Preserved observable behavior

- A missing repository, `vercel/turbo`, and `vercel/turborepo` are the only official routes.
- Every other repository returns `not-applicable` before any side-effect provider is called.
- `package.json` existence is observed before the best-effort `meta.json` sequence.
- A metadata read failure is swallowed and prevents the remove attempt.
- A metadata remove failure is swallowed while the already parsed metadata is still returned.
- A missing `package.json` returns success after metadata handling.
- A falsey parsed package value is not written.
- `basic` and `default` rename the package to the requested project name.
- A truthy existing `devDependencies.turbo` receives a truthy explicit version or `^<create-turbo version>`.
- An empty explicit version is JavaScript-falsey and therefore uses the fallback.
- Any truthy parsed package object is written even when neither relevant field changes.
- Package read and write failures retain the exact public messages and `fatal: false` metadata.

## Representation and intentional-divergence ledger

| Area | TypeScript behavior | Rust representation or planned behavior | Classification | Reason |
| --- | --- | --- | --- | --- |
| Missing repository | `example.repo` is absent | `Option::None` | representation-only | Makes absence explicit without changing route selection. |
| Repository matching | Exact string equality and `includes` over two literals | Borrowed `&str` exact comparisons over a fixed array | parity plus hardening | Avoids normalization, fuzzy matching, allocation, and mutable global state. |
| Falsey package result | Runtime falsey guard around parsed JSON | `Option<PackageJson>::None` | representation-only | Prevents null/undefined-style states from leaking into later mutation. |
| Existing Turbo dependency truthiness | JavaScript runtime truthiness | `OfficialStarterPackageJson::turbo_dev_dependency_is_truthy` | type-boundary conversion | The production JSON adapter must reproduce JavaScript truthiness exactly for all supported JSON values rather than guessing from a Rust string type. |
| `create-turbo` version | Runtime `require("../../package.json").version` | Explicit borrowed `create_turbo_version` input | representation-only | Removes hidden module/global lookup and makes the oracle value injectable and testable. |
| Package mutation | Dynamic object property updates | Typed setter methods that preserve all unowned fields in the provider object | representation-only | Narrows mutation authority and makes unknown-field preservation reviewable. |
| Public error | JavaScript `TransformError` object | Typed `OfficialStarterError<E>` with exact message, transform name, and nonfatal flag | partial until binding | Rust must retain the provider cause without inventing JavaScript stack/class behavior; the host binding still has to construct the exact public error. |
| Filesystem and JSON operations | Broad synchronous `fs-extra` reads, forced removal, and in-place JSON write | No production provider yet | intentional security block | Directly cloning the source would retain unbounded parsing, link following, special-file handling, concurrent replacement, partial-write, metadata, and ordering risks. |
| Future production reads | Source follows ordinary path semantics without explicit bounds | Bounded, regular-file, root-confined, no-follow reads | intentional security divergence | Rejects oversized files, links, special files, and redirected roots instead of reading attacker-selected resources. |
| Future package publication | Source rewrites `package.json` directly | Same-directory staged and synchronized atomic replacement with identity checks | intentional security divergence | Prevents partial publication and reduces time-of-check/time-of-use replacement attacks. |
| Metadata/package transaction | Metadata can be removed before a later package failure | Source ordering preserved in the core; production provider requires an explicit rollback or journal decision | compatibility risk, unresolved | Changing this silently would alter behavior, but shipping without recovery can leave partial state. A separate RED contract is required for any transactional hardening. |

## Production-provider security requirements

The Rust binding must not become production-active until one provider proves all of the following with failure injection and Linux, macOS, and Windows differential fixtures:

1. bounded reads for `meta.json` and `package.json`, including nesting, string, collection, and total-byte limits;
2. strict JSON parsing with exact JavaScript-compatible truthiness for the existing Turbo dependency;
3. preservation of unknown fields and the source-observable insertion/serialization order, with deterministic two-space output;
4. root confinement, regular-file requirements, link/reparse-point rejection, and file identity revalidation;
5. same-directory staging, synchronization, atomic replacement, and approved mode/ACL/ownership handling;
6. defined concurrent modification behavior and cleanup of every temporary artifact;
7. an explicit transaction or rollback policy for metadata removal followed by package publication;
8. exact public `TransformError` mapping and no false-success result after package failure;
9. no shell construction, process execution, network access, credential access, or unredacted logging.

## Dependency and advisory impact

This tranche adds no crate, parser, network client, subprocess, logger, unsafe code, or mutable global state. It therefore introduces no new dependency advisory surface. The repository-wide `quick-xml`, `h2`, and `webbrowser` findings remain open and are not ignored or weakened by this change.

## TDD evidence

- RED parity/security contract: `2ca25bd457cbe216f345b5f67cf9ac32f43a2c7a`.
- GREEN orchestration core: `cd2ba74b3040e654a63c9799e42c35a12f2c4dbc`.
- Parity tests: `tests/official_starter_parity.rs`.
- Security tests: `tests/official_starter_security.rs`.
""",
        encoding="utf-8",
    )


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(
        path,
        "5. the dependency-injected `package-manager` transform decision and conversion-request contract.",
        "5. the dependency-injected `package-manager` transform decision and conversion-request contract.\n"
        "6. the dependency-injected `official-starter` transform orchestration contract.",
    )
    section = """### Official-starter transform core

- classifies an example as official only when no repository is supplied or when the repository is exactly `vercel/turbo` or `vercel/turborepo`;
- preserves source ordering by snapshotting `package.json` existence before the best-effort `meta.json` read/removal sequence;
- returns parsed metadata when removal fails, while swallowing metadata read and removal failures like the TypeScript implementation;
- maps `package.json` read and write failures to the exact public messages, transform name, and `fatal: false` contract;
- renames `basic` and `default` package objects to the requested project name;
- updates an existing truthy `devDependencies.turbo` to a non-empty explicit version or to `^<create-turbo version>` when the option is absent or empty;
- still writes a truthy package object when neither relevant field changes, matching `writeJsonSync` ordering and side effects;
- keeps filesystem access, JSON parsing/serialization, truthiness classification, no-follow policy, resource bounds, deterministic ordering, and atomic publication behind `OfficialStarterStore` and `OfficialStarterPackageJson`.

A production store is deliberately absent. It must preserve unknown JSON fields and insertion order, implement JavaScript-compatible truthiness for the existing Turbo dependency, bound both JSON files, reject unsafe links and special files, preserve approved metadata, and stage package writes atomically on Linux, macOS, and Windows before this core can replace the TypeScript transform.

The exact type conversions and intentional security divergences are recorded in [`OFFICIAL_STARTER_DIVERGENCES.md`](./OFFICIAL_STARTER_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", section)
    replace_once(
        path,
        "- the remaining source transforms;",
        "- production filesystem/JSON provider for the `official-starter` transform, including deterministic JSON ordering and atomic no-follow writes;\n"
        "- transform dispatcher binding and public `TransformError` mapping;\n"
        "- the remaining source transforms;",
    )
    replace_once(
        path,
        "`default_example` owns the pure default-acquisition routing predicate. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`.",
        "`default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and transform ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`.",
    )
    insert_before(
        path,
        "The package-manager transform does not receive free-form manager text at its mutation boundary.",
        "The official-starter core cannot open, delete, parse, serialize, or replace a path directly. Its provider boundary makes metadata best-effort behavior, package read/write failures, JSON truthiness, and deterministic serialization independently reviewable instead of silently inheriting broad `fs-extra` behavior.",
    )
    replace_once(
        path,
        "Package manager GREEN:   c7a1776c5f6fa53db4e30d418a9897b56c6263cd",
        "Package manager GREEN:   c7a1776c5f6fa53db4e30d418a9897b56c6263cd\n"
        f"Official starter RED:   {RED_SHA}\n"
        f"Official starter GREEN: {GREEN_SHA}",
    )
    replace_once(
        path,
        "The crate contains 39 translated parity tests and 30 security regression tests, for 69 authored focused Rust tests.",
        "The crate contains 55 translated parity tests and 39 security regression tests, for 94 authored focused Rust tests.",
    )


def update_parity_matrix() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    section = """## Official-starter transform tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| no `example.repo` | `is_official_starter(None)` | implemented core | Preserves the source's built-in official route. |
| repository exactly `vercel/turbo` or `vercel/turborepo` | `ExampleRepository` plus exact borrowed-string matching | implemented core | Case, whitespace, prefixes, suffixes, paths, controls, and Unicode confusables do not broaden the route. |
| non-official repository returns before filesystem access | early `NotApplicable` response | implemented core | An exploding provider test proves no store method can run. |
| `existsSync(package.json)` before metadata handling | `package_json_exists` before `read_meta_json` | implemented core | Provider-call order is translated exactly. |
| `readJsonSync(meta.json)` followed by best-effort forced removal | `read_meta_json` then ignored `remove_meta_json` result | implemented core | Read failure returns no metadata and skips removal; removal failure still returns the parsed metadata. |
| missing `package.json` | source existence snapshot | implemented core | Returns success after metadata handling without package read/write. |
| package read error | `OfficialStarterError::ReadPackageJson` | implemented core | Exact public message, transform name, and nonfatal metadata are covered. |
| falsey parsed package value | `read_package_json` returns `None` | implemented core | Returns success without writing, matching the source guard. |
| `basic`/`default` package rename | `is_default_example` plus `set_name` | implemented core | Exact project name is forwarded as data. |
| truthy existing `devDependencies.turbo` | typed truthiness query plus setter | implemented core | Non-empty explicit version wins; absent or empty option becomes `^<invocation version>`. |
| truthy package object with no relevant field changes | unconditional provider write after successful read | implemented core | Preserves the source's write side effect and ordering. |
| package write error | `OfficialStarterError::WritePackageJson` | implemented core | Cannot become a false success. |
| `fs-extra` JSON parsing, ordering, deletion, and write behavior | production `OfficialStarterStore` | blocked | Requires bounded strict parsing, unknown-field/order preservation, JavaScript truthiness, no-follow paths, atomic publication, metadata policy, and supported-platform differentials. |
| public JavaScript `TransformError` instance | typed Rust error metadata | partial | Native/host binding must construct the exact public error class and stack-facing behavior. |

Detailed representation and security differences are in `OFFICIAL_STARTER_DIVERGENCES.md`."""
    insert_before(path, "## Existing TypeScript test mapping", section)
    replace_once(
        path,
        "| package-manager version/path/provider-boundary regressions | four security tests | intentional-hardening evidence |",
        "| package-manager version/path/provider-boundary regressions | four security tests | intentional-hardening evidence |\n"
        "| `official-starter` source contract without focused Jest coverage | sixteen translated parity tests | implemented core |\n"
        "| official-route confusable/large-input/provider-boundary regressions | nine security tests | intentional-hardening evidence |",
    )
    replace_once(
        path,
        "| `official-starter` transform | not-implemented | Translate package/workspace mutations with deterministic JSON ordering. |",
        "| `official-starter` transform | implemented orchestration core, provider blocked | Add bounded no-follow JSON/filesystem provider, deterministic order-preserving serialization, atomic package publication, native binding, platform differentials, and removal proof. |",
    )


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    replace_once(
        path,
        "- `packages/create-turbo/src/transforms/package-manager.ts`",
        "- `packages/create-turbo/src/transforms/package-manager.ts`\n"
        "- `packages/create-turbo/src/transforms/official-starter.ts`",
    )
    replace_once(
        path,
        "The Git initialization tranche adds decision boundaries for the project-root path, Git and Mercurial executable selection, process working directory, arguments, inherited environment and VCS configuration, template directories, hooks, timeouts, output, child-process cleanup, `.git` ownership, and recursive deletion.",
        "The official-starter tranche adds trust boundaries for exact repository classification, the pre-metadata `package.json` existence snapshot, best-effort `meta.json` read/removal, project-name and version data, JavaScript truthiness, unknown package fields, JSON ordering, parser limits, links, concurrent replacement, and atomic publication. The reviewed core performs only ordering and mutation decisions; all filesystem and JSON effects remain behind typed providers.\n\n"
        "The Git initialization tranche adds decision boundaries for the project-root path, Git and Mercurial executable selection, process working directory, arguments, inherited environment and VCS configuration, template directories, hooks, timeouts, output, child-process cleanup, `.git` ownership, and recursive deletion.",
    )
    findings = """### CT-RS-019: Official-starter repository classification must remain exact

**Severity:** Medium

The transform treats a missing repository and only the exact repositories `vercel/turbo` and `vercel/turborepo` as official. Trimming, case folding, Unicode normalization, prefix/path matching, or fuzzy matching would broaden a trusted transformation route.

The Rust core uses borrowed exact strings and returns `not-applicable` before any provider access for every other repository. Tests cover case, whitespace, prefixes, suffixes, paths, controls, Unicode confusables, and a multi-megabyte nonmatching name.

Residual risk: production routing still invokes the TypeScript transform until the Rust binding and differential fixtures are complete.

### CT-RS-020: Metadata removal is intentionally best-effort and can leave stale metadata

**Severity:** Low compatibility contract

The TypeScript source reads `meta.json`, then attempts forced removal inside one `try` block. A read failure is ignored and prevents removal. A removal failure is also ignored, while the parsed metadata remains in the successful response. This can leave a stale `meta.json` beside a returned copy.

The Rust orchestration preserves that exact ordering and result contract rather than silently changing observable behavior. The production provider must bound and strictly parse the file, reject links and special files, and make the remaining stale-file possibility explicit in diagnostics or a deliberate compatibility change with its own RED contract.

Regression tests: `package_existence_is_observed_before_meta_processing`, `meta_read_failure_is_swallowed_without_a_remove_attempt`, and `meta_remove_failure_is_swallowed_and_the_parsed_value_is_returned`.

### CT-RS-021: Package JSON mutation lacks a reviewed bounded, no-follow, atomic provider

**Severity:** High until the provider contract is closed

The TypeScript transform uses broad synchronous `fs-extra` operations. It does not establish explicit byte limits, no-follow path handling, file identity checks, atomic replacement, or complete metadata preservation. It also relies on JavaScript truthiness and JSON serialization order. Because metadata removal happens before package read/write, a later package failure can leave `meta.json` removed while `package.json` remains unchanged, with no shared rollback contract.

The Rust tranche deliberately keeps these effects behind `OfficialStarterStore` and `OfficialStarterPackageJson`. The core proves exact classification, call order, rename/version decisions, nonfatal public errors, and provider-failure propagation, but it cannot touch a path or parser directly.

A production provider must prove bounded strict JSON parsing, unknown-field and insertion-order preservation, JavaScript-compatible truthiness for the existing Turbo dependency, safe root/file identity, no-follow behavior, synchronized same-directory staging, atomic package replacement, a transaction or rollback journal covering metadata removal plus package publication, approved mode/ACL/ownership handling, concurrent-path behavior, deterministic output, and Linux/macOS/Windows differential parity.

Regression tests prove that non-official input cannot reach a provider, metadata failures alone are swallowed, package failures remain fatal to the transform result, large fallback text is not copied when the dependency is falsey, and data strings are never interpreted as commands by the core."""
    insert_before(path, "## Security invariants", findings)
    replace_once(
        path,
        "- README, `.gitignore`, default-example, and package-manager orchestration add no network or credential behavior.",
        "- README, `.gitignore`, default-example, official-starter, and package-manager orchestration add no network or credential behavior.\n"
        "- The official-starter core performs no filesystem access, JSON parsing, serialization, deletion, process execution, or logging; those effects remain behind typed providers.",
    )
    replace_once(
        path,
        "- The package-manager orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.",
        "- The official-starter orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, logging operation, or mutable global state.\n"
        "- A production official-starter store remains blocked until its JSON truthiness, ordering, resource bounds, no-follow identity, atomic write, metadata, and supported-platform contracts are proven.\n"
        "- The package-manager orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.",
    )


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/create-turbo/rust`: 39 translated parity tests and 30 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, and package-manager transform orchestration.",
        "- `packages/create-turbo/rust`: 55 translated parity tests and 39 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager transform orchestration, and official-starter orchestration.",
    )
    replace_once(path, "That is **229 authored Rust migration tests** on the integration branch.", "That is **254 authored Rust migration tests** on the integration branch.")
    replace_once(
        path,
        "The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. Across only the first three stages of those four active surfaces, the evidence-weighted estimate is about **70%**. Across the complete repository production program it remains about **8%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
        "The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **72%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.",
    )
    replace_once(
        path,
        "README, `.gitignore`, Git orchestration, default-example routing, and package-manager decision/request cores are ported. CLI, prompts, discovery/acquisition, production VCS and converter providers, remaining transforms, telemetry binding, packaging, callers, and removal proof remain.",
        "README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain.",
    )
    section = """### Official-starter transform orchestration

The Rust core preserves the source's exact official route and side-effect ordering behind `OfficialStarterStore` and `OfficialStarterPackageJson`:

- no repository, `vercel/turbo`, and `vercel/turborepo` are the only official inputs;
- non-official inputs return before any provider access;
- `package.json` existence is captured before best-effort metadata processing;
- metadata read failure skips removal and is swallowed;
- metadata removal failure is swallowed while the parsed metadata is still returned;
- package read/write failures retain the exact nonfatal public messages and cannot become success;
- `basic` and `default` rename the package;
- a truthy existing Turbo development dependency receives a non-empty explicit version or the `^<create-turbo version>` fallback;
- an empty explicit version follows JavaScript falsey behavior and uses the fallback;
- any truthy package object is written even when no relevant field changes.

The production provider remains blocked. It must implement bounded strict JSON parsing, exact JavaScript truthiness, unknown-field and insertion-order preservation, no-follow identity checks, synchronized atomic publication, approved metadata/ACL handling, deterministic output, and Linux/macOS/Windows differential fixtures before binding or TypeScript removal. Exact representation and intentional security differences are catalogued in `packages/create-turbo/rust/OFFICIAL_STARTER_DIVERGENCES.md`.
"""
    insert_before(path, "### Package-manager transform orchestration", section)
    replace_once(
        path,
        "- package-manager transform implementation: `c7a1776c5f6fa53db4e30d418a9897b56c6263cd`.",
        "- package-manager transform implementation: `c7a1776c5f6fa53db4e30d418a9897b56c6263cd`.\n"
        f"- official-starter transform RED: `{RED_SHA}`.\n"
        f"- official-starter transform implementation: `{GREEN_SHA}`.",
    )


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    finding = """### RF-015: Official-starter JSON mutation lacks a bounded atomic production provider

**Status:** Orchestration core implemented; production cutover blocked.

The TypeScript `official-starter` transform trusts only a missing repository or the exact repositories `vercel/turbo` and `vercel/turborepo`, reads and best-effort removes `meta.json`, then may rewrite `package.json`. Broadening repository matching would widen a trusted route, while directly reproducing `fs-extra` would retain unbounded parsing, link following, in-place write, metadata, ordering, and concurrent-path uncertainty.

The Rust core now proves exact borrowed-string classification, source call order, metadata failure behavior, package rename/version decisions, public nonfatal errors, and failure propagation. It cannot access a filesystem, parser, serializer, process, network, or logger directly.

Required closure is a production `OfficialStarterStore` with bounded strict parsing, JavaScript-compatible truthiness, unknown-field and insertion-order preservation, no-follow root/file identity, synchronized atomic package publication, transaction or rollback coverage for metadata removal plus package mutation, approved metadata/ACL behavior, deterministic serialization, public binding, and Linux/macOS/Windows differential fixtures.

Regression coverage is in `packages/create-turbo/rust/tests/official_starter_parity.rs` and `official_starter_security.rs`. The complete representation and intentional-divergence ledger is `packages/create-turbo/rust/OFFICIAL_STARTER_DIVERGENCES.md`."""
    insert_before(path, "## Required repository gates", finding)
    replace_once(
        path,
        "- close the package-manager conversion transaction, rollback, and supported-platform contract;",
        "- close the package-manager conversion transaction, rollback, and supported-platform contract;\n"
        "- close the official-starter bounded JSON, truthiness, no-follow identity, deterministic ordering, atomic publication, and supported-platform provider contract;",
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
