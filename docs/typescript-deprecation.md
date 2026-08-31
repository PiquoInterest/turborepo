# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is complete only after public behavior is covered by translated and differential tests, security-sensitive boundaries are reviewed, and production packaging and callers use the Rust implementation.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The migration program currently contains these Rust migration cores on the single `rust/typescript-deprecation` integration branch:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 70 translated parity tests and 36 security regression tests.
- `packages/create-turbo/rust`: 55 translated parity tests and 39 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager transform orchestration, and official-starter orchestration.
- `crates/turborepo-telemetry::events::package`: 9 translated parity tests and 7 security regression tests for the package-facing telemetry contract.

That is **254 authored Rust migration tests** on the integration branch. Test count is evidence coverage, not a completion percentage. The latest `create-turbo` tranches remain unvalidated until their merge-head workflow compiles, tests, formats, and lints them successfully.

No TypeScript package is removed yet. Safe-input differential execution, production bindings, packaging, supported-platform closure, downstream cutover, and removal proof remain open. Migration CI auto-discovers package-local Rust crates, requires current evidence documents and advisory records, and compiles, tests, lints, and audits the resolved dependency graph.

The mandatory workflow is in `AGENTS.md`. Every tranche must use RED-first translated tests, retain TypeScript as an oracle until cutover, perform current advisory review, and update `README.md`, `PARITY_MATRIX.md`, `SECURITY.md`, this ledger, and the repository security index in the same change.

## Weighted progress estimate

The repository-wide rewrite is currently estimated at **about 8% complete**, with a conservative credible range of **8% to 10%**. The estimate is deliberately not based on line count or raw test count.

The denominator is 12 tracked migration surfaces multiplied by eight equally weighted production stages:

1. complete inventory and TypeScript oracle;
2. Rust core implementation;
3. translated parity and security tests;
4. Linux/macOS/Windows differential execution;
5. native/WASM or minimal host binding;
6. packaging, signing, provenance, release, install, and rollback;
7. downstream caller cutover;
8. artifact/removal proof and executable TypeScript deletion.

The four active surfaces have strong inventory plus partial core/test credit, but stages 4 through 8 are almost entirely open. The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**. Across only the first three stages of the four active surfaces, the evidence-weighted estimate is now about **72%**. Final package cutover and executable-TypeScript removal remain **0%**, because no package yet meets every deletion gate.

This estimate must be revised from the inventory as surfaces are split, added, or proven complete. It must never be rounded upward to imply production readiness.

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
| `packages/create-turbo` | `packages/create-turbo/rust` | In progress | README, `.gitignore`, Git orchestration, default-example routing, package-manager decision/request, and official-starter orchestration cores are ported. CLI, prompts, discovery/acquisition, production VCS/converter/JSON providers, transform binding, remaining transforms, telemetry binding, packaging, callers, and removal proof remain. |
| `packages/turbo-gen` | Rust CLI | Queued | Generator discovery, prompts, template rendering, workspace mutations, packaging. |
| `packages/turbo-codemod` | Rust CLI | Queued | Golden fixtures, idempotence, parser/rewriter boundaries, packaging. |
| `packages/turbo-workspaces` | Rust CLI/library | Queued and partially exposed through provider boundary | Package-manager adapters, complete six-manager conversion, lock/workspace mutation semantics, rollback, process policy, and packaging. |
| `packages/turbo-telemetry` | existing telemetry Rust crate plus package contract | In progress | Package event core is consolidated. Consent/config persistence, production binding, transport integration, caller cutover, and removal proof remain. |
| ESLint plugin/config | Rust/WASM rule core with minimal JS adapter | Host-bound | Preserve ESLint node/range/fix semantics and package compatibility. |
| VS Code and language-tool adapters | Rust LSP plus minimal extension bootstrap | Host-bound | Move business logic to Rust while retaining host-required JavaScript only. |
| Factory/web `.tsx` surfaces | Rust/WASM where justified | Host-bound | Define browser architecture and DOM boundary before deprecation. |
| Test-only TypeScript fixtures | Rust tests or retained oracle | Later | Remove only after migrated components have equivalent coverage. |
| npm/native wrappers | generated loaders and signed Rust binaries | Queued | Preserve package names, platform selection, provenance, install behavior, and rollback. |

The weighted denominator groups test-only fixtures and npm/native wrappers as program stages rather than double-counting them as independent production applications; the ledger retains them as explicit work rows because they have distinct owners and removal gates.

## Current `create-turbo` tranches

### README package-manager command transform

The Rust scanner preserves the TypeScript precedence for triple-backtick fences and inline code, then performs the same ordered compound and bare-manager substitutions for `pnpm`, `npm`, `yarn`, and `bun`. Prose and `npx` remain untouched.

Security closure in the Rust core:

- 4 MiB input bound and linear scanning;
- strict UTF-8 rather than silent replacement decoding;
- symlink rejection and Unix identity checks;
- synchronized sibling temporary writes and permission preservation;
- ordinary failure cleanup.

The shared source target type also admits the repository's `nub` and `aube` test managers while the replacement regex scans only four real command spellings. That wider target behavior remains a differential-test gap. Windows atomic replacement and complete metadata/ACL preservation also remain blockers.

### `.gitignore` transform

The Rust core preserves the exact `DEFAULT_IGNORE` bytes and the TypeScript success/not-applicable/public-error contract. Unlike the TypeScript `existsSync` plus overwrite-capable write sequence, Rust publishes a fully written temporary inode through a no-overwrite hard link.

Security closure in the Rust core:

- a concurrent destination is never overwritten;
- broken and existing `.gitignore` symlinks are rejected;
- symlinked roots are rejected;
- temporary creation is bounded and uses `create_new`;
- ordinary success and failure paths remove the temporary name.

Handle-relative publication is still required to close every malicious concurrent root-replacement race.

### Git initialization orchestration

The Rust core preserves the exact safe command sequence and boolean failure contract behind injected runner and cleaner traits. It corrects the first RED draft against the actual TypeScript source before implementation:

- exact message `Initial commit from create-turbo`;
- exact Mercurial `--cwd . root` arguments with the project root as process cwd;
- no invented `git --version` call;
- no cleanup after ambiguous `git init` failure;
- structural path validation rather than a shell-metacharacter blacklist on a shell-free call;
- lossless non-UTF-8 Unix roots.

Production Git/Hg execution and `.git` cleanup remain blocked until executable resolution, environment/config/template/hook isolation, deadlines, bounded output, process cleanup, no-follow deletion, and Windows behavior are proven.

### Default-example routing

The Rust predicate preserves the exported source-order values `basic` and `default` and returns true only for exact borrowed-string membership. It intentionally performs no trimming, case folding, Unicode normalization, substring/path matching, mutable set lookup, or input copy.

This closes the pure routing core but not the production route. The TypeScript `create` command still owns acquisition orchestration, so binding and shared differential fixtures remain required.

### Official-starter transform orchestration

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

### Package-manager transform orchestration

The Rust core preserves the source no-op and conversion-request behavior behind `PackageManagerConverter`:

- absent selection and unchanged manager return `not-applicable` without mutation;
- all six repository manager variants are represented by a closed enum;
- changed manager issues exactly one request with the borrowed root, target enum, and `skip_install: true`;
- the optional prompt version is deliberately not forwarded, matching the source transform;
- provider errors propagate and cannot become success;
- non-UTF-8 Unix roots remain lossless;
- no filesystem or process effect exists in the reviewed orchestration core.

The production converter remains blocked. The TypeScript `@turbo/workspaces.convert` path performs manager-specific cleanup, creation, package metadata, lockfile, and configuration mutation across multiple steps without a proven shared atomic transaction. Rust cutover requires a complete six-manager source/target matrix, failure injection, atomic promotion or rollback, no-follow filesystem handling, bounded process behavior, exact errors, and Linux/macOS/Windows differential tests.

TDD history:

- README RED: `a0930bc5bd0eee5bc7c6edf09daf8caf38875781`.
- README implementation: `0af47426b5ef00bbff6dfc7d60aaca23daa71720`.
- `.gitignore` RED: `f8edbb984cd7255f1d7630689384324009de5ac4`.
- `.gitignore` implementation: `c74d664d718691660be969d779d25a76af31fb3e`.
- Git RED import: `e57cc31afd1d83a015ae49136d71c7daa3217fb7`.
- corrected Git oracle/security RED: `221586118db79fca2f94cebb15785de4111bde8e`.
- Git implementation: `1d7b485d597b70f40bb4aa492f45d1c0638f844e`.
- default-example RED: `edc3b96b106e2c0bebaee299690c7769f9ba6bc2`.
- default-example implementation: `57f19c56209312fb2d04423fdd86ad239150a753`.
- package-manager transform RED: `9f9b33f889d92e5b61a484ac445b4e297110f6f0`.
- package-manager transform implementation: `c7a1776c5f6fa53db4e30d418a9897b56c6263cd`.
- official-starter transform RED: `2ca25bd457cbe216f345b5f67cf9ac32f43a2c7a`.
- official-starter transform implementation: `cd2ba74b3040e654a63c9799e42c35a12f2c4dbc`.

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

## Current package telemetry tranche

The consolidated Rust package event core covers the `create-turbo` and `turbo-ignore` event envelope, batching, close-time flush, disabled behavior, salted hashing, package/runtime metadata, coarse option classifications, endpoint restrictions, bounded inputs, and no-follow configuration handling.

The core is not yet the production package entry point. Binding, full consent/config differential behavior, production transport integration, caller cutover, packaging, and removal proof remain.

TDD history:

- package telemetry RED: `67475ffb7616bc78bc4c6759ab63558d751e588e`;
- package telemetry implementation: `86a879c6987a2160bed820cbfff9b54a6ad8284f`;
- integration merge: `7482a57576fb2fb85efe976620ce910295a4feda`.

## Security review method

Each tranche maintains a colocated `SECURITY.md` with attacker inputs, trust boundaries, filesystem/process/network behavior, resource limits, logging/redaction, advisory lookup, findings, fixes, regression names, and residual risks. Memory safety alone is not completion.

The repository security index records unresolved `webbrowser`, `h2`, and `quick-xml` advisories. No tranche may suppress those findings to claim green. Exact reverse dependencies and remediation blockers are recorded in `docs/rust-migration-security-findings.md`.

## Branch and pull-request policy

The integration branch is `rust/typescript-deprecation`, represented by pull request #1. Focused PR histories #2 through #5 are merged into it. PR #6 is closed with its exact integration merge recorded because GitHub refused to retarget a head already fully contained by the integration branch. Shared history is never force-pushed. Repository-wide parity is declared only when this ledger contains no executable TypeScript runtime entries and production packaging points to Rust for every supported target.
