# turbo-workspaces TypeScript test inventory

This file tracks source-suite coverage separately from production cutover. A mapped suite means its observable contract has Rust tests; it does not mean the TypeScript runtime can be removed.

## Source suites

The package currently has eight executable Jest suites and one shared test-support module.

| TypeScript suite | Current Rust evidence | Status | Remaining work |
| --- | --- | --- | --- |
| `workspace-details.test.ts` | `workspace_details_parity.rs`, `workspace_details_security.rs` | mapped core: 6 parity, 5 security | Run authoritative Rust gates, implement production providers/binding, platform differentials, packaging, callers, removal proof. |
| `install-meta.test.ts` | create-turbo installation-profile tests | mapped read-only profile selection | Shared package ownership, process runner, platform differential fixture. |
| `install-security.test.ts` | create-turbo installation-policy security tests | security evidence | Replace TypeScript `preferLocal`/Windows shell path with production Rust execution. |
| `index.test.ts` | package-manager transform request core only | partial | Full conversion order, dry-run semantics, errors, transaction, rollback. |
| `managers.test.ts` | closed manager enums and workspace-details order only | substantially unported | Discovery, metadata, lockfile, workspace read/mutation behavior for all managers. |
| `utils.test.ts` | no dedicated Rust port | not ported | Directory/path/package utility behavior and malformed-input boundaries. |
| `nub.test.ts` | static manager/profile identity only | substantially unported | Nub cleanup, creation, metadata, lockfile, command, and failure behavior. |
| `aube.test.ts` | static manager/profile identity only | substantially unported | Aube cleanup, creation, metadata, lockfile, command, and failure behavior. |
| `test-utils.ts` | retained TypeScript oracle support | support module | Remove only when all dependent differential suites are retired. |

## Current Rust test count

- workspace-details parity tests: 6
- workspace-details security tests: 5
- total in this crate: 11
- repository authored migration-test total after this tranche: 393

The previous repository ledger contained 382 authored tests. These 11 tests are committed and mapped, but their hosted Rust execution remains pending; the count must not be presented as 393 validated tests until CI runs.

## Next RED-first tranches

1. Manager discovery and exact metadata precedence from `managers.test.ts`.
2. Read-only utility behavior from `utils.test.ts`.
3. Per-manager read models, starting with the smallest bounded manager contract.
4. Mutation planning without side effects.
5. Staged file publication and rollback at every injected failure point.
6. Canonical no-shell process execution with deadlines, bounded output, and descendant cleanup.
7. Linux, macOS, and Windows TypeScript-versus-Rust differential fixtures.

Each tranche must keep its TypeScript suite green, commit a compiling behavioral Rust RED first, add the minimal GREEN implementation, and update `SECURITY.md`, `security.txt`, this inventory, and the repository ledgers.
