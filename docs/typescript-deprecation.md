# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is complete only after public behavior is covered by translated and differential tests, security-sensitive boundaries are reviewed, and production packaging and callers use the Rust implementation.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The migration program currently contains these workspace-registered Rust migration cores:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 70 translated parity tests and 36 security regression tests.
- `packages/create-turbo/rust`: 17 translated parity tests and 14 security regression tests across README command rewriting and `.gitignore` creation.

That is 175 authored Rust migration tests on this stacked branch. The package telemetry Rust tranche is developed separately in PR #4 and is not included in that total until it lands on `rust/typescript-deprecation`.

No TypeScript package is removed yet. Safe-input differential execution, production bindings, packaging, supported-platform closure, downstream cutover, and removal proof remain open. Migration CI auto-discovers package-local Rust crates, requires current evidence documents and advisory records, and compiles, tests, lints, and audits the resolved dependency graph.

The mandatory workflow is in `AGENTS.md`. Every tranche must use RED-first translated tests, retain TypeScript as an oracle until cutover, perform current advisory review, and update `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, this ledger, and the repository security index in the same change.

## Completion rules

Executable TypeScript may be deleted only after all of the following are true:

1. Existing tests are retained as an oracle or translated into Rust.
2. Rust tests cover every documented branch, failure mode, exit code, serialized interface, ordering rule, and side effect.
3. Differential fixtures compare TypeScript and Rust on every supported platform.
4. Security deviations are explicit, tested, and recorded rather than hidden behind a parity claim.
5. Native/WASM or minimal JavaScript host bindings are production-ready.
6. npm/native packaging, signing, provenance, rollback, and install behavior use Rust artifacts.
7. All downstream callers are migrated.
8. Removal tests prove that the old runtime is neither loaded nor shipped.

Test-only TypeScript and host-required JavaScript adapters are tracked separately from executable business logic.

## Migration ledger

| Surface | Rust target | Status | Required closure |
| --- | --- | --- | --- |
| Core `turbo` engine and CLI | existing Rust crates | Existing | Continue removing legacy wrappers and retain compatibility tests. |
| `packages/turbo-ignore` | `packages/turbo-ignore/rust` | In progress | Differential CLI tests, Windows process-tree handling, telemetry integration, native npm packaging, caller cutover, removal proof. |
| `packages/turbo-utils` | `packages/turbo-utils/rust` plus bindings | In progress | Production network/archive and registry providers, remaining utilities, Windows ACL/process/shim closure, bindings, callers, removal proof. |
| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README and `.gitignore` transform cores are ported. CLI, prompts, acquisition, Git behavior, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |
| `packages/turbo-gen` | Rust CLI | Queued | Generator discovery, prompts, template rendering, workspace mutations, packaging. |
| `packages/turbo-codemod` | Rust CLI | Queued | Golden fixtures, idempotence, parser/rewriter boundaries, packaging. |
| `packages/turbo-workspaces` | Rust CLI/library | Queued | Package-manager adapters and lock/workspace mutation semantics. |
| `packages/turbo-telemetry` | existing telemetry Rust crate plus package contract | In progress in PR #4 | Consent/config persistence, transport, retry/redaction parity, JS binding, caller cutover, removal proof. |
| ESLint plugin/config | Rust/WASM rule core with minimal JS adapter | Host-bound | Preserve ESLint node/range/fix semantics and package compatibility. |
| VS Code and language-tool adapters | Rust LSP plus minimal extension bootstrap | Host-bound | Move business logic to Rust while retaining host-required JavaScript only. |
| Factory/web `.tsx` surfaces | Rust/WASM where justified | Host-bound | Define browser architecture and DOM boundary before deprecation. |
| Test-only TypeScript fixtures | Rust tests or retained oracle | Later | Remove only after migrated components have equivalent coverage. |
| npm/native wrappers | generated loaders and signed Rust binaries | Queued | Preserve package names, platform selection, provenance, install behavior, and rollback. |

## Current `create-turbo` tranches

### README package-manager command transform

The Rust scanner preserves the TypeScript precedence for triple-backtick fences and inline code, then performs the same ordered compound and bare-manager substitutions for `pnpm`, `npm`, `yarn`, and `bun`. Prose and `npx` remain untouched.

Security closure in the Rust core:

- 4 MiB input bound and linear scanning;
- strict UTF-8 rather than silent replacement decoding;
- symlink rejection and Unix identity checks;
- synchronized sibling temporary writes and permission preservation;
- ordinary failure cleanup.

Windows atomic replacement and complete metadata/ACL preservation remain blockers.

### `.gitignore` transform

The Rust core preserves the exact `DEFAULT_IGNORE` bytes and the TypeScript success/not-applicable/public-error contract. Unlike the TypeScript `existsSync` plus overwrite-capable write sequence, Rust publishes a fully written temporary inode through a no-overwrite hard link.

Security closure in the Rust core:

- a concurrent destination is never overwritten;
- broken and existing `.gitignore` symlinks are rejected;
- symlinked roots are rejected;
- temporary creation is bounded and uses `create_new`;
- ordinary success and failure paths remove the temporary name.

Handle-relative publication is still required to close every malicious concurrent root-replacement race.

TDD history:

- README RED: `a0930bc5bd0eee5bc7c6edf09daf8caf38875781`.
- README implementation: `0af47426b5ef00bbff6dfc7d60aaca23daa71720`.
- `.gitignore` RED: `f8edbb984cd7255f1d7630689384324009de5ac4`.

## Current `turbo-utils` tranche

Implemented cores cover case conversion, bounded upward/root/config discovery, folder and directory validation, writability, package-manager discovery, project orchestration, update notifications, archive-entry policy, and GitHub token/proxy selection.

Network/archive writes remain behind `ProjectSource`; registry lookup remains behind `UpdateChecker`. Production providers must close TLS, redirect, proxy, credential, timeout, size, extraction, staging, cleanup, and atomic-promotion contracts before caller cutover.

Notable intentional fixes include:

- no process-wide `chdir` mutation;
- no project-local executable substitution;
- bounded process output/deadlines in implemented runners;
- control-safe terminal output;
- traversal/link/archive-entry rejection;
- correct handling of safe names such as `..cache` rather than the TypeScript `startsWith("..")` false positive.

TDD history includes:

- project creation RED `0468eda3829e5b1bb98f96b86a7f0817ac542f51`, implementation `b2992a27dbf44c5ab8bc7405dc088236eb53c70e`;
- notification RED `7a446b29f3e6054a58e891b898d3f8c4f85854ce`, implementation `cabec01820809f34d8f42cf1adbbff50c3307e68`;
- archive policy RED `5ab1da42327a85e4c026e8531953fb108b56434d`, implementation `bdeb6760d41d5f9d72d2b9fb8042339b55011923`;
- GitHub policy RED `903d7836a01e6ec47e4df339adc71456b4ecbd0d`, implementation `2e90ea8daa8542aa13cd94ceb981b653756789cb`.

## Security review method

Each tranche maintains a colocated `SECURITY.md` with attacker inputs, trust boundaries, filesystem/process/network behavior, resource limits, logging/redaction, advisory lookup, findings, fixes, regression names, and residual risks. Memory safety alone is not completion.

The repository security index records the affected `webbrowser` dependency under `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`. The observed current call uses a constant HTTP URL, but the dependency remains in an affected range and must be upgraded or removed. Migration CI temporarily ignores only that documented advisory so any additional advisory still fails.

## Branch and pull-request policy

The integration branch is `rust/typescript-deprecation`. Work lands in reviewable conventional-commit tranches or stacked focused PRs. Shared history is never force-pushed. Repository-wide parity is declared only when this ledger contains no executable TypeScript runtime entries and production packaging points to Rust for every supported target.
