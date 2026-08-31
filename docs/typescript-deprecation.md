# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is complete only after its public behavior is covered by differential or translated tests, its security-sensitive boundaries are reviewed, and the Rust implementation is the production entry point.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The integration branch currently contains two workspace-registered Rust migration cores:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 63 translated parity tests and 29 security regression tests. Covered surfaces include case conversion, upward/root/config discovery, folder and directory validation, writability, package-manager version/global-bin discovery, `createProject` orchestration, update-notification behavior, and archive entry path/link policy.

That is 130 authored Rust tests. Neither TypeScript package is removed yet because safe-input differential execution, production bindings, packaging, supported-platform closure, and downstream cutover are still open. Migration CI auto-discovers package-local Rust migration crates, requires their evidence documents and dated advisory records, then compiles, tests, lints, and audits the resolved Rust dependency graph.

The mandatory workflow is recorded in `AGENTS.md`: every tranche must use TDD and differential tests, perform a current authoritative advisory lookup, and update its `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, and this ledger in the same change. Repository-level findings are indexed in [`rust-migration-security-findings.md`](./rust-migration-security-findings.md).

## Completion rules

A runtime component may leave TypeScript only when all of the following are true:

1. Existing tests are retained as an oracle or translated into Rust.
2. Rust tests cover every documented branch, error mode, exit code, and serialized interface.
3. Differential fixtures compare TypeScript and Rust behavior for safe inputs.
4. Security deviations are explicit, tested, and recorded rather than hidden behind “parity”.
5. Packaging and release jobs ship the Rust implementation on every supported platform.
6. TypeScript runtime files are removed only after downstream packages use the Rust implementation.

Tests, declarations, build metadata, and host adapters are tracked separately from executable runtime logic. TypeScript required by a JavaScript host, such as ESLint or VS Code APIs, needs a native/WASM boundary or an intentionally retained minimal JavaScript adapter.

## Migration ledger

| Surface | Current implementation | Rust target | Status | Required closure |
| --- | --- | --- | --- | --- |
| Core `turbo` engine and CLI | Predominantly Rust | Existing Rust crates | Existing | Continue removing legacy wrappers and keep compatibility tests. |
| `packages/turbo-ignore` decision engine | TypeScript | `packages/turbo-ignore/rust` | In progress | Differential CLI tests, Windows process-tree handling, telemetry decision, native npm packaging, production cutover, then remove runtime TS. |
| `packages/turbo-utils` | TypeScript utilities | `packages/turbo-utils/rust` plus JS/WASM bindings where needed | In progress | Implement and differentially test the production GitHub/network/archive provider and registry update checker; port remaining template/example utilities; close Windows ACL/process/shim gaps; add bindings and migrate callers. |
| `packages/create-turbo` | TypeScript CLI | Rust CLI | Queued | Preserve templates, prompts, package-manager behavior, network and filesystem failure modes. Reuse reviewed `turbo-utils-rs` providers. |
| `packages/turbo-gen` | TypeScript CLI | Rust CLI | Queued | Preserve generator discovery, prompts, template rendering, and workspace mutations. |
| `packages/turbo-codemod` | TypeScript CLI | Rust CLI | Queued | Port transformations with golden fixtures and idempotence tests. |
| `packages/turbo-workspaces` | TypeScript CLI/library | Rust CLI/library | Queued | Preserve package-manager adapters and lock/workspace mutation semantics. |
| `packages/turbo-telemetry` | TypeScript | Rust telemetry client or explicit retirement | Queued | Define consent, persistence, transport, retry, and redaction contract. |
| `packages/eslint-plugin-turbo` and config | TypeScript in a JavaScript host | Rust/WASM rule core with minimal JS adapter | Host-bound | Preserve ESLint node/range/fix semantics and publish compatible packages. |
| VS Code extension and language tooling adapters | TypeScript host layer plus Rust LSP | Rust LSP with minimal extension adapter | Host-bound | Move business logic into Rust; retain only API bootstrap required by VS Code. |
| Factory/web UI and `.tsx` surfaces | TypeScript/React | Rust/WASM only where justified | Host-bound | Define browser/WASM architecture and DOM bindings before deprecation. |
| Test-only TypeScript fixtures | TypeScript | Rust tests or retained cross-language oracle | Later | Remove only after every migrated component has equivalent coverage. |
| npm/native publishing wrappers | JavaScript/TypeScript | Generated platform loaders and signed Rust binaries | Queued | Preserve package names, install behavior, platform selection, provenance, and rollback. |

## Current `turbo-utils` tranche

The project coordinator is separated from network and archive execution through `ProjectSource`. The translated contract covers default and named examples, GitHub repository selection, four acquisition attempts, generated `package.json` inspection, JavaScript script-key ordering, missing sources, and strict URL/path/destination/metadata boundaries.

The update notification core is separated from registry lookup through `UpdateChecker` and dynamic command resolution through `UpgradeCommandProvider`. It covers one stored update decision, failed/no-update silence, static and dynamic command rendering, exit-code preservation, TypeScript-compatible debug handling, and bounded control-safe output.

The archive entry policy now translates the existing `isPathSafe` and `isLinkEntry` behavior for safe paths and adds cross-platform safeguards:

- normal and nested relative paths remain valid;
- `..` is allowed only while the lexical destination remains below the extraction root;
- NULs, absolute paths, UNC paths, Windows drive prefixes, alternate data streams, excessive path length, excessive component counts, symbolic links, and hard links are rejected;
- safe names beginning with two dots, such as `..cache`, are accepted. The TypeScript `relativePath.startsWith("..")` check incorrectly rejects them even though they are not parent components.

The actual HTTP, proxy, GitHub authentication, sparse Git fallback, tar streaming, extraction writes, registry lookup, staging, and atomic promotion implementations remain outside these pure coordinators until each has one shared differential-tested security contract.

TDD history:

- Project creation: RED `0468eda3829e5b1bb98f96b86a7f0817ac542f51`, implementation `b2992a27dbf44c5ab8bc7405dc088236eb53c70e`.
- Update notification: RED `7a446b29f3e6054a58e891b898d3f8c4f85854ce`, implementation `cabec01820809f34d8f42cf1adbbff50c3307e68`, bidi regression RED `28f7ec2f0388c93a92d6ed7261c10832e29f5925`, fix `84e5ed11155af39a151d56b1e6b4580eaf6f057e`.
- Archive entry policy: RED `5ab1da42327a85e4c026e8531953fb108b56434d`, corrected `..`-prefix regression `37f6b4caaea9d06e87ec82f550560d989a6af872`, implementation `bdeb6760d41d5f9d72d2b9fb8042339b55011923`.

## Security review method

Each tranche gets a colocated `SECURITY.md` containing attacker inputs and trust boundaries, subprocess and executable behavior, filesystem and race considerations, parser/resource limits, network/package acquisition, logging and redaction, a dated authoritative advisory lookup, and every intentional incompatibility where exact parity would preserve unsafe behavior.

Security fixes require regression tests. Memory safety alone is not completion: semantic injection, package acquisition, path trust, authorization, denial of service, and fail-open/fail-closed behavior require explicit review.

The repository-level index records affected `webbrowser` under `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`. The observed call site uses a constant HTTP URL, limiting current reachability, but the dependency must be upgraded or removed before merge. Migration CI temporarily ignores only that documented advisory so additional findings still fail.

## Branch and pull-request policy

The long-lived integration branch is `rust/typescript-deprecation`. Work lands in reviewable conventional-commit tranches. The branch must remain buildable; repository-wide parity is declared only when this ledger has no executable TypeScript runtime entries and production packaging points to Rust for all supported targets.
