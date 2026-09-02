# Rust migration test inventory

This inventory maps executable TypeScript test suites to Rust parity and security coverage. It answers two different questions without conflating them:

1. Has source behavior been captured by tests?
2. Has the production TypeScript path actually been replaced and removed?

A mapped suite is behavior evidence. It is not evidence that bindings, packaging, downstream callers, supported platforms, or artifact-removal gates are complete.

## Status vocabulary

- **Mapped:** relevant TypeScript behavior has translated Rust parity tests.
- **Security-evidence:** the TypeScript suite contains a passing `it.failing` or equivalent expectation for behavior intentionally rejected by Rust.
- **Partial:** only a bounded subset of the source suite is represented in Rust.
- **Not ported:** no reviewed Rust test mapping has been committed for the suite.
- **Production complete:** Rust is the shipped path and removal tests prove executable TypeScript is neither loaded nor distributed.

No active surface is production complete yet.

## `packages/create-turbo`

| TypeScript suite | Rust evidence | Mapping status | Remaining closure |
| --- | --- | --- | --- |
| `update-commands-in-readme.test.ts` | README parity, filesystem, and security tests | Mapped | Host binding, Windows metadata and atomicity, package routing, removal proof. |
| `git.test.ts` | Git-init parity and security tests | Mapped core | Canonical Git/Hg runner, environment and hook policy, deadlines, output bounds, descendant cleanup, no-follow `.git` cleaner, platform differentials. |
| `create-error-security.test.ts` | create-error parity and security tests | Mapped plus security-evidence | Terminal host binding, exact JavaScript error identity, cleanup and telemetry before exit, removal proof. |
| `create-install-policy.test.ts` | install-policy and warning tests | Mapped core | Real installer provider, exact output side effects, platform differentials, production routing. |
| `create-output-security.test.ts` | output-policy parity and security tests | Mapped plus security-evidence | Path, group, locale and color binding, artifact proof. |
| `directory-security.test.ts` | directory-prompt parity and security tests | Mapped plus repaired boundary | Interactive reader limits, cancellation, stable filesystem handles, Windows reparse-point behavior, production routing. |
| `index.test.ts` | portions mapped by error, install, output, and pipeline tests | Partial | End-to-end create orchestration through one Rust-backed command. |
| `test-utils.ts` | retained oracle support | Support | Remove only when dependent differential suites retire. |

The create-turbo component ledger remains the source of truth for its detailed test count. Test counts are coverage evidence, not a production percentage.

## `packages/turbo-workspaces`

The migration branch currently contains ten executable Jest suites plus one support module, including focused security and migration oracles.

| TypeScript suite | Rust evidence | Mapping status | Remaining closure |
| --- | --- | --- | --- |
| `workspace-details.test.ts` | `workspace_details_parity.rs`, `workspace_details_security.rs` | Mapped core, 6 parity and 5 security | Production providers, async binding, platform differentials, packaging, callers, removal proof. |
| `bun-workspace-glob-security.test.ts` and compatible `utils.test.ts` cases | `bun_workspace_glob_parity.rs`, `bun_workspace_glob_security.rs` | Mapped pure core, 12 parity and 6 security | Production parser and bounded workspace-expansion binding. |
| `workspace-packages.test.ts` and `parseWorkspacePackages` cases | `workspace_packages_parity.rs`, `workspace_packages_security.rs` | Mapped pure core plus security-evidence, 7 parity and 6 security | Bounded package.json provider, glob expansion, platform differentials, binding, removal proof. |
| `install-meta.test.ts` | create-turbo package-manager profile tests | Mapped profile selection | Shared ownership, production runner, platform differential fixture. |
| `install-security.test.ts` | create-turbo install security tests | Security-evidence | Replace project-local executable preference and Windows shell mediation in the production path. |
| `index.test.ts` | package-manager transform request core | Partial | Full conversion order, dry-run behavior, errors, transaction and rollback. |
| `managers.test.ts` | closed manager enum and orchestration order | Substantially unported | Discovery, metadata, lockfile, workspace read and mutation semantics for every manager. |
| `utils.test.ts` | Bun compatibility and workspace-package parsing mapped; declaration TDD is active | Partial | Directory, package metadata, path expansion, YAML workspaces, lockfile probes, mutation and cleanup. |
| `nub.test.ts` | static manager/profile identity | Partial | Nub cleanup, creation, metadata, lockfile, command and failure behavior. |
| `aube.test.ts` | static manager/profile identity | Partial | Aube cleanup, creation, metadata, lockfile, command and failure behavior. |
| `test-utils.ts` | retained oracle support | Support | Keep until dependent source suites have differential replacements. |

### Current turbo-workspaces Rust inventory

- workspace-details: 6 parity and 5 security;
- Bun workspace-glob compatibility: 12 parity and 6 security;
- workspace-package parsing: 7 parity and 6 security;
- current crate total: 25 parity and 17 security, 42 tests.

The workspace-package parser uses this committed sequence:

- TypeScript oracle: `9c8f77deee15c01baba73fdd510960e899756f0e`;
- initial RED `089112a3f85bc2cbaaf864991eb5b6129602ff30` rejected after GitHub proved the security test did not compile;
- corrected compiling Rust RED: `72aa20cf4e17f528b46111f9681f06d522994655`;
- corrected Rust GREEN: `d997c57b66b4d10710ecee8c98b8a72ff61f2eef`;
- exact TypeScript formatter output: `aaa354bf2a808039bdff461dc65dd5e7507a8aec`.

Safe-input behavior remains ordinary TypeScript assertions. Unsafe legacy acceptance remains executable as expected-failure evidence while Rust enforces count, byte-volume, checked-arithmetic, unsafe-text, and no-echo error rules. The rejected RED remains visible but is not counted as behavioral TDD proof.

### Immediate remaining turbo-workspaces work

1. Finish the active package-manager declaration TDD chain without overlapping its workflow.
2. Port read-only directory, package metadata, path expansion, and YAML workspace utilities.
3. Port manager discovery and exact metadata precedence.
4. Port per-manager read models.
5. Port mutation planning without side effects.
6. Implement staged file publication and rollback after every injected failure point.
7. Implement canonical no-shell execution with deadlines, bounded output, cancellation, and descendant cleanup.
8. Complete Nub and Aube behavior.
9. Run Linux, macOS, and Windows TypeScript-versus-Rust differential fixtures.
10. Bind production callers and prove executable TypeScript is neither loaded nor shipped.

## Bounded matcher TDD chain

The package-manager version matcher is backed by this committed sequence:

- TypeScript oracle correction: `3d0d7d63950f21acf4604536fdaffbfffa335798`;
- compiling Rust RED: `816216a20b5620ab381842e26ed322d9409b3cec`;
- Rust GREEN: `a47192630977ffec2a4208f67d01fbd948a8aa97`;
- exact Rustfmt output: `149f43f4662d8ab3f44b35a2b21e4e3bfd8c3c31`;
- parity and divergence evidence: `6fbab195a23fd567891a9c7e31f820534c83a0a6`.

The lockfile-wide advisory audit remains a separate required repository gate.

## Security rules for test migration

- Keep the TypeScript suite green. Unsafe source behavior is represented with `it.failing` or equivalent expected-failure evidence.
- Make the Rust test compile and fail for missing behavior before adding implementation.
- Preserve canonical safe-input behavior unless an intentional security divergence is documented.
- Add hostile-input tests for paths, links, archives, process arguments, environment variables, terminal text, parsers, network authorities, redirects, credentials, size and depth limits, cancellation, and concurrency where applicable.
- Do not count a provider interface as production behavior. Real side effects need failure injection, platform differentials, cleanup, and rollback proof.
- Never convert an unavailable tool, cancelled job, skipped platform, or advisory exception into a passing status.

## Repository-level remaining work

The migration ledger tracks multiple production surfaces across inventory, Rust core, differential execution, binding, packaging, caller cutover, and removal stages. The active surfaces have substantial core and test evidence, but supported-platform differentials, production host bindings, packaging and provenance, downstream cutover, and TypeScript artifact-removal proof remain mostly open.

The weighted repository estimate remains approximately 8%. This file must be updated whenever a TypeScript suite becomes mapped, partial coverage becomes complete, a production binding lands, or a removal gate closes.
