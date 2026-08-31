# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is marked complete only after its public behavior is covered by differential or translated tests, its security-sensitive boundaries are reviewed, and the Rust implementation is the production entry point.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The integration branch currently contains two workspace-registered Rust migration cores:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 32 translated parity tests and 18 security regression tests. Covered surfaces now include case conversion, upward/root/config discovery, folder and directory validation, writability, package-manager version/global-bin discovery, and `createProject` orchestration.

That is 88 authored Rust tests. Neither TypeScript package is removed yet because compiled CI for the latest tranche, differential execution, production bindings, packaging, supported-platform closure, and downstream cutover are still open.

The mandatory agent workflow is recorded in `AGENTS.md`: every tranche must use TDD and differential tests, perform a current authoritative advisory lookup, and update its `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, and this ledger in the same change. Repository-level findings are indexed in [`rust-migration-security-findings.md`](./rust-migration-security-findings.md).

## Completion rules

A runtime component may leave TypeScript only when all of the following are true:

1. Existing tests are retained as an oracle or translated into Rust.
2. Rust tests cover every documented branch, error mode, exit code, and serialized interface.
3. Differential fixtures compare TypeScript and Rust behavior for safe inputs.
4. Security deviations are explicit, tested, and recorded rather than hidden behind “parity”.
5. Packaging and release jobs ship the Rust implementation on every supported platform.
6. TypeScript runtime files are removed only after downstream packages use the Rust implementation.

Tests, type declarations, build metadata, and host-specific adapters are tracked separately from executable runtime logic. TypeScript that is required by a JavaScript host, such as ESLint or VS Code extension APIs, needs either a native/WASM boundary or an intentionally retained JavaScript adapter.

## Migration ledger

| Surface | Current implementation | Rust target | Status | Required closure |
| --- | --- | --- | --- | --- |
| Core `turbo` engine and CLI | Predominantly Rust | Existing Rust crates | Existing | Continue removing legacy wrappers and keep compatibility tests. |
| `packages/turbo-ignore` decision engine | TypeScript | `packages/turbo-ignore/rust` | In progress | Compile/lint, differential CLI tests, Windows process-tree handling, telemetry decision, native npm packaging, production cutover, then remove runtime TS. |
| `packages/turbo-utils` | TypeScript utilities | `packages/turbo-utils/rust` plus JS/WASM bindings where needed | In progress | Compile/lint the project-creation tranche; implement and differentially test the `ProjectSource` GitHub/network/archive provider; port template/example, update-notification, and remaining project utilities; close Windows ACL/process/shim gaps; add bindings and migrate callers. |
| `packages/create-turbo` | TypeScript CLI | Rust CLI | Queued | Preserve templates, prompts, package-manager behavior, network and filesystem failure modes. Reuse the reviewed `turbo-utils-rs` project/source provider rather than duplicating acquisition logic. |
| `packages/turbo-gen` | TypeScript CLI | Rust CLI | Queued | Preserve generator discovery, prompts, template rendering, and workspace mutations. |
| `packages/turbo-codemod` | TypeScript CLI | Rust CLI | Queued | Port transformations with golden fixtures and idempotence tests. |
| `packages/turbo-workspaces` | TypeScript CLI/library | Rust CLI/library | Queued | Preserve package-manager adapters and lock/workspace mutation semantics. |
| `packages/turbo-telemetry` | TypeScript | Rust telemetry client or explicit retirement | Queued | Define consent, persistence, transport, retry, and redaction contract. |
| `packages/eslint-plugin-turbo` and config | TypeScript in a JavaScript host | Rust/WASM rule core with minimal JS adapter | Host-bound | Preserve ESLint node/range/fix semantics and publish compatible packages. |
| VS Code extension and language tooling adapters | TypeScript host layer plus Rust LSP | Rust LSP with minimal extension adapter | Host-bound | Move business logic into Rust; retain only API bootstrap required by VS Code. |
| Factory/web UI and `.tsx` surfaces | TypeScript/React | Rust/WASM only where technically justified | Host-bound | This is not a mechanical native rewrite. Define browser/WASM architecture and DOM bindings before deprecation. |
| Test-only TypeScript fixtures | TypeScript | Rust tests or retained cross-language oracle | Later | Remove only after every migrated component has equivalent coverage. |
| npm/native publishing wrappers | JavaScript/TypeScript | Generated platform loaders and signed Rust binaries | Queued | Preserve package names, install behavior, platform selection, provenance, and rollback. |

## Current `turbo-utils` tranche

The project-creation coordinator is separated from network and archive execution through `ProjectSource`. The translated contract now covers:

- default example acquisition through the sparse-example path;
- named examples through the repository tarball path used by the TypeScript implementation;
- custom GitHub repository discovery and existence checks;
- four total acquisition attempts, matching `async-retry({ retries: 3 })`;
- generated `package.json` presence and script discovery;
- JavaScript `Object.keys` ordering for integer-like script names;
- missing examples, missing repository information, and missing repository contents;
- strict URL, example-name, repository-subpath, destination, symlink, and metadata boundaries.

The actual HTTP, proxy, GitHub authentication, sparse Git fallback, tar streaming, archive extraction, staging, and atomic promotion implementation remains deliberately outside the coordinator until it has one shared, differential-tested security contract.

## Security review method

Each tranche gets a colocated `SECURITY.md` containing:

- attacker-controlled inputs and trust boundaries;
- subprocess and executable-resolution behavior;
- filesystem traversal, symlink, and race considerations;
- parser size/depth limits;
- network acquisition and package-spec behavior;
- logging, telemetry, and secret-redaction behavior;
- current authoritative advisory sources, lookup date, affected versions, and disposition;
- intentional incompatibilities where exact parity would preserve an unsafe behavior.

Security fixes must have regression tests. A migration is not complete merely because memory safety improves: semantic injection, package acquisition, path trust, authorization, denial of service, and fail-open/fail-closed behavior still require explicit review.

The repository-level index currently records an affected `webbrowser` dependency under `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`. The observed call site uses a constant HTTP URL, limiting current reachability, but the dependency must be upgraded or removed before this migration can merge.

## Branch and pull-request policy

The long-lived integration branch is `rust/typescript-deprecation`. Work lands in reviewable tranches with conventional-commit titles. The branch must remain buildable; repository-wide parity will be declared only when the ledger has no executable TypeScript runtime entries and production packaging points to Rust for all supported targets.
