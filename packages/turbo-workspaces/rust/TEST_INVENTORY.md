# turbo-workspaces TypeScript test inventory

This file tracks source-suite coverage separately from production cutover. A mapped suite means its observable contract has Rust tests; it does not mean the TypeScript runtime can be removed.

## Source suites

The package currently has ten executable Jest suites and one shared test-support module on the migration branch. Focused migration oracles are retained until the Rust implementation, production binding, platform differentials, and artifact-removal gates are complete.

| TypeScript suite | Current Rust evidence | Status | Remaining work |
| --- | --- | --- | --- |
| `workspace-details.test.ts` | `workspace_details_parity.rs`, `workspace_details_security.rs` | mapped core: 6 parity, 5 security | Production providers/binding, platform differentials, packaging, callers, removal proof. |
| `bun-workspace-glob-security.test.ts` plus compatible cases from `utils.test.ts` | `bun_workspace_glob_parity.rs`, `bun_workspace_glob_security.rs` | mapped pure core: 12 parity, 6 security | Bind to production parser and workspace expansion, then run platform differentials. |
| `workspace-packages.test.ts` plus `parseWorkspacePackages` cases from `utils.test.ts` | `workspace_packages_parity.rs`, `workspace_packages_security.rs` | mapped pure core: 7 parity, 6 security | Bind to bounded package.json parsing and workspace expansion; prove platform and artifact-removal gates. |
| `install-meta.test.ts` | create-turbo installation-profile tests | mapped read-only profile selection | Shared package ownership, process runner, platform differential fixture. |
| `install-security.test.ts` | create-turbo installation-policy security tests | security evidence | Replace TypeScript `preferLocal` and Windows shell execution with production Rust execution. |
| `index.test.ts` | package-manager transform request core only | partial | Full conversion order, dry-run semantics, errors, transaction, rollback. |
| `managers.test.ts` | closed manager enums and workspace-details order only | substantially unported | Discovery, metadata, lockfile, workspace read and mutation behavior for all managers. |
| `utils.test.ts` | Bun compatibility and workspace-package parsing are mapped; declaration work is being migrated separately | partial | Directory/path/package helpers, package manager declaration completion, YAML workspaces, workspace expansion, lockfile probes, mutation and cleanup. |
| `nub.test.ts` | static manager/profile identity only | substantially unported | Nub cleanup, creation, metadata, lockfile, command, and failure behavior. |
| `aube.test.ts` | static manager/profile identity only | substantially unported | Aube cleanup, creation, metadata, lockfile, command, and failure behavior. |
| `test-utils.ts` | retained TypeScript oracle support | support module | Remove only when all dependent differential suites are retired. |

## Current Rust test count

- workspace-details: 6 parity, 5 security;
- Bun workspace-glob compatibility: 12 parity, 6 security;
- workspace-package parsing: 7 parity, 6 security;
- total in this crate: 25 parity, 17 security, 42 tests.

These are authored mappings. The workspace-package tranche is not considered validated until the focused GitHub workflow proves the TypeScript oracle, compiling RED, GREEN Rust tests, formatting, and Clippy.

## Workspace-package parser TDD chain

- TypeScript oracle: `9c8f77deee15c01baba73fdd510960e899756f0e`;
- compiling behavioral Rust RED: `089112a3f85bc2cbaaf864991eb5b6129602ff30`;
- Rust GREEN: `8b4aea45459aa09237aef7d8dd35ccf06503ae28`;
- translated tests: 7 parity and 6 security.

The TypeScript suite stays green. Safe behavior is asserted normally. Legacy acceptance of unbounded or terminal-active input is documented with `it.failing`, while Rust rejects those inputs with bounded typed errors that do not echo attacker-controlled text.

## Remaining test-port sequence

1. Complete package-manager declaration policy and its active TDD chain.
2. Port read-only directory, package metadata, path expansion, and YAML workspace utilities.
3. Port manager discovery and exact metadata precedence from `managers.test.ts`.
4. Port per-manager read models, starting with the smallest bounded contract.
5. Port mutation planning without side effects.
6. Implement staged file publication and rollback at every injected failure point.
7. Implement canonical no-shell process execution with deadlines, bounded output, cancellation, and descendant cleanup.
8. Port Nub and Aube manager-specific behavior.
9. Run Linux, macOS, and Windows TypeScript-versus-Rust differential fixtures.
10. Bind production callers and prove executable TypeScript is neither loaded nor shipped.

Each tranche must keep its TypeScript suite green, commit a compiling behavioral Rust RED first, add the minimal GREEN implementation, and update `SECURITY.md`, `security.txt`, this inventory, and the relevant migration ledger.
