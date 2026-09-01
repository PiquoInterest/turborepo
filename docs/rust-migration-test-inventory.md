# Rust migration test inventory

This inventory maps executable TypeScript test suites to Rust parity and security coverage. It exists to answer two different questions without conflating them:

1. Has the source behavior been captured by tests?
2. Has the production TypeScript path actually been replaced and removed?

A mapped test suite is evidence for behavior. It is not evidence that bindings, packaging, downstream callers, supported platforms, or artifact-removal gates are complete.

## Status vocabulary

- **Mapped:** the relevant TypeScript behavior has translated Rust parity tests.
- **Security-evidence:** the TypeScript suite contains a passing `it.failing` or equivalent expectation that demonstrates behavior intentionally rejected by Rust.
- **Partial:** only a bounded subset of the source suite is represented in Rust.
- **Not ported:** no reviewed Rust test mapping has been committed for the suite.
- **Production complete:** the Rust implementation is the shipped path and removal tests prove that executable TypeScript is neither loaded nor distributed.

No active surface is production complete yet.

## `packages/create-turbo`

The source package currently has seven executable Jest suites plus one support module.

| TypeScript suite | Rust evidence | Mapping status | Remaining closure |
| --- | --- | --- | --- |
| `__tests__/update-commands-in-readme.test.ts` | `readme_parity.rs`, `readme_security.rs`, `readme_fs_parity.rs`, `readme_fs_security.rs` | Mapped | Host binding, Windows metadata/atomicity, package routing, removal proof. |
| `__tests__/git.test.ts` | `git_init_parity.rs`, `git_init_security.rs` | Mapped core | Canonical Git/Hg runner, environment/config/hook policy, deadlines, output bounds, descendant cleanup, no-follow `.git` cleaner, platform differentials. |
| `__tests__/create-error-security.test.ts` | `create_error_policy_parity.rs`, `create_error_policy_security.rs` | Mapped plus security-evidence | Terminal host binding, exact JavaScript error identity, telemetry flush, cleanup-before-exit, removal proof. |
| `__tests__/create-install-policy.test.ts` | `create_install_policy_parity.rs`, `create_install_policy_security.rs`, warning tests | Mapped core | Real installer provider, exact output side effects, platform differentials, production routing. |
| `__tests__/create-output-security.test.ts` | `create_output_policy_parity.rs`, `create_output_policy_security.rs` | Mapped plus security-evidence | Path/group/locale derivation, coloring after sanitization, output host binding, artifact proof. |
| `__tests__/directory-security.test.ts` | `directory_prompt_parity.rs`, `directory_prompt_security.rs` | Mapped plus repaired source boundary | Interactive reader limits, cancellation/EOF/signals, stable filesystem handles, Windows reparse-point behavior, production routing. |
| `__tests__/index.test.ts` | portions mapped by create error/install/output/pipeline tests | Partial | End-to-end create orchestration, acquisition, workspace inspection, Git, transforms, install, telemetry, exact console order, exits, and platform fixtures must run through one Rust-backed command. |
| `__tests__/test-utils.ts` | test support only | Not an independent behavior suite | Replace or retain only as long as TypeScript remains the differential oracle. |

### Current create-turbo Rust test inventory

The existing component ledger records 116 parity tests and 92 security tests across the currently reviewed create-turbo cores. The bounded matcher tranche uses 12 parity test functions and 8 security test functions within those totals. Test counts remain evidence coverage, not a production percentage.

## `packages/turbo-workspaces`

The source package currently has seven executable Jest suites plus one support module.

| TypeScript suite | Rust evidence | Mapping status | Remaining closure |
| --- | --- | --- | --- |
| `__tests__/install-meta.test.ts` | `package_manager_install_policy_parity.rs` | Mapped for the eight profile records and six current range literals | Shared platform differential fixture and production runner binding. |
| `__tests__/install-security.test.ts` | `package_manager_install_policy_security.rs` | Security-evidence | TypeScript still uses project-local executable preference and Windows shell mediation; production Rust execution is not yet bound. |
| `__tests__/index.test.ts` | package-manager transform request core only | Partial | Full conversion orchestration, error order, dry-run behavior, manager lifecycle, transaction and rollback. |
| `__tests__/managers.test.ts` | none | Not ported | Port package-manager discovery, metadata, lockfile and workspace semantics for every manager. |
| `__tests__/utils.test.ts` | none | Not ported | Port utility behavior, malformed-input cases, path and serialization boundaries. |
| `__tests__/nub.test.ts` | static manager/profile enum coverage only | Partial | Port Nub-specific cleanup, creation, lockfile, package metadata, command, and failure behavior. |
| `__tests__/aube.test.ts` | static manager/profile enum coverage only | Partial | Port Aube-specific cleanup, creation, lockfile, package metadata, command, and failure behavior. |
| `__tests__/test-utils.ts` | test support only | Not an independent behavior suite | Keep until all dependent source suites have differential replacements. |

### Immediate remaining test work

The next high-impact test migration is the `@turbo/workspaces` conversion surface. Five source suites remain wholly or substantially unported: `index.test.ts`, `managers.test.ts`, `utils.test.ts`, `nub.test.ts`, and `aube.test.ts`.

Those suites must not be translated as one large mutation. Split them into RED-first tranches by observable transaction boundary:

1. manager discovery and exact metadata;
2. read-only workspace inspection;
3. per-manager mutation plan;
4. staged file publication;
5. external process execution;
6. rollback after each injected failure point;
7. Linux, macOS, and Windows differential fixtures.

## Bounded matcher TDD chain

The package-manager version matcher is backed by this committed sequence:

- TypeScript oracle correction: `3d0d7d63950f21acf4604536fdaffbfffa335798`
- compiling Rust RED: `816216a20b5620ab381842e26ed322d9409b3cec`
- Rust GREEN: `a47192630977ffec2a4208f67d01fbd948a8aa97`
- exact Rustfmt output: `149f43f4662d8ab3f44b35a2b21e4e3bfd8c3c31`
- parity and divergence evidence: `6fbab195a23fd567891a9c7e31f820534c83a0a6`

GitHub Actions compiled the exact formatted implementation, passed the migration parity and security tests, and passed Clippy with warnings denied. The lockfile-wide advisory audit is a separate repository gate and remains required.

## Security rules for test migration

- Keep the TypeScript suite green. Unsafe source behavior is represented with `it.failing` or an equivalent expected-failure oracle rather than by making the source suite red.
- Make the Rust test compile and fail for the missing behavior before adding implementation.
- Preserve canonical safe-input behavior unless an intentional security divergence is documented.
- Add hostile-input tests for paths, links, archives, process arguments, environment variables, terminal text, parsers, network authorities, redirects, credentials, size/depth limits, cancellation, and concurrency wherever applicable.
- Do not count a provider interface as production behavior. Real side effects need failure injection, platform differentials, and cleanup/rollback proof.
- Never convert an unavailable tool, cancelled job, skipped platform, or advisory exception into a passing status.

## Repository-level remaining work

The repository migration ledger currently tracks twelve production surfaces across eight completion stages. The four active surfaces have significant core and test evidence, but supported-platform differential execution, native or minimal host bindings, packaging and provenance, downstream caller cutover, and TypeScript artifact-removal proof remain mostly open.

The weighted repository estimate therefore remains approximately 8%, despite hundreds of authored Rust tests. This file must be updated whenever a TypeScript test suite becomes mapped, partial coverage becomes complete, a production binding lands, or a removal gate closes.
