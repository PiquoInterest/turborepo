# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is marked complete only after its public behavior is covered by differential or translated tests, its security-sensitive boundaries are reviewed, and the Rust implementation is the production entry point.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The integration branch currently contains two workspace-registered Rust migration cores:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 10 translated parity tests and 5 security regression tests for case conversion, upward search, folder-conflict detection, writability, and directory validation.

That is 53 authored Rust tests. Neither TypeScript package is removed yet because compiled CI, differential execution, production bindings, packaging, and downstream cutover are still open.

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

| Surface                                         | Current implementation              | Rust target                                                    | Status      | Required closure                                                                                                                                                                 |
| ----------------------------------------------- | ----------------------------------- | -------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core `turbo` engine and CLI                     | Predominantly Rust                  | Existing Rust crates                                           | Existing    | Continue removing legacy wrappers and keep compatibility tests.                                                                                                                  |
| `packages/turbo-ignore` decision engine         | TypeScript                          | `packages/turbo-ignore/rust`                                   | In progress | Compile/lint, differential CLI tests, Windows process-tree handling, telemetry decision, native npm packaging, production cutover, then remove runtime TS.                       |
| `packages/turbo-utils`                          | TypeScript utilities                | `packages/turbo-utils/rust` plus JS/WASM bindings where needed | In progress | Compile/lint the migrated pure core; port root/config, package-manager, network, template, update-notification, and project-creation behavior; add bindings and migrate callers. |
| `packages/create-turbo`                         | TypeScript CLI                      | Rust CLI                                                       | Queued      | Preserve templates, prompts, package-manager behavior, network and filesystem failure modes.                                                                                     |
| `packages/turbo-gen`                            | TypeScript CLI                      | Rust CLI                                                       | Queued      | Preserve generator discovery, prompts, template rendering, and workspace mutations.                                                                                              |
| `packages/turbo-codemod`                        | TypeScript CLI                      | Rust CLI                                                       | Queued      | Port transformations with golden fixtures and idempotence tests.                                                                                                                 |
| `packages/turbo-workspaces`                     | TypeScript CLI/library              | Rust CLI/library                                               | Queued      | Preserve package-manager adapters and lock/workspace mutation semantics.                                                                                                         |
| `packages/turbo-telemetry`                      | TypeScript                          | Rust telemetry client or explicit retirement                   | Queued      | Define consent, persistence, transport, retry, and redaction contract.                                                                                                           |
| `packages/eslint-plugin-turbo` and config       | TypeScript in a JavaScript host     | Rust/WASM rule core with minimal JS adapter                    | Host-bound  | Preserve ESLint node/range/fix semantics and publish compatible packages.                                                                                                        |
| VS Code extension and language tooling adapters | TypeScript host layer plus Rust LSP | Rust LSP with minimal extension adapter                        | Host-bound  | Move business logic into Rust; retain only API bootstrap required by VS Code.                                                                                                    |
| Factory/web UI and `.tsx` surfaces              | TypeScript/React                    | Rust/WASM only where technically justified                     | Host-bound  | This is not a mechanical native rewrite. Define browser/WASM architecture and DOM bindings before deprecation.                                                                   |
| Test-only TypeScript fixtures                   | TypeScript                          | Rust tests or retained cross-language oracle                   | Later       | Remove only after every migrated component has equivalent coverage.                                                                                                              |
| npm/native publishing wrappers                  | JavaScript/TypeScript               | Generated platform loaders and signed Rust binaries            | Queued      | Preserve package names, install behavior, platform selection, provenance, and rollback.                                                                                          |

## Security review method

Each tranche gets a colocated `SECURITY.md` containing:

- attacker-controlled inputs and trust boundaries;
- subprocess and executable-resolution behavior;
- filesystem traversal, symlink, and race considerations;
- parser size/depth limits;
- network acquisition and package-spec behavior;
- logging, telemetry, and secret-redaction behavior;
- intentional incompatibilities where exact parity would preserve an unsafe behavior.

Security fixes must have regression tests. A migration is not complete merely because memory safety improves: semantic injection, package acquisition, path trust, authorization, denial of service, and fail-open/fail-closed behavior still require explicit review.

## Branch and pull-request policy

The long-lived integration branch is `rust/typescript-deprecation`. Work lands in reviewable tranches with conventional-commit titles. The branch must remain buildable; repository-wide parity will be declared only when the ledger has no executable TypeScript runtime entries and production packaging points to Rust for all supported targets.
