# AGENTS.md

Instructions for AI agents working on this repository.

## Architecture

See [ARCHITECTURE.md](./crates/turborepo/ARCHITECTURE.md) for an overview of the `turbo run` command architecture.

## Keeping Documentation Up to Date

When making changes to the codebase, check if the following docs need updates:

- **[ARCHITECTURE.md](./crates/turborepo/ARCHITECTURE.md)** - Update when changing core `turbo run` components:
  - Run builder, package graph, task graph/engine
  - Task visitor, caching system, task hashing
  - Run tracking and summary generation
  - Any files in `crates/turborepo-lib/src/run/`, `crates/turborepo-lib/src/engine/`, `crates/turborepo-lib/src/task_graph/`, or `crates/turborepo-cache/`

- **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Update when changing:
  - Build process or development setup
  - Testing procedures or requirements
  - Project structure or tooling

- **This file (AGENTS.md)** - Update when changing:
  - PR requirements or CI workflows
  - Repository conventions or policies

## TypeScript-to-Rust Deprecation Workflow

The long-lived integration branch for this program is `rust/typescript-deprecation`, represented by the single repository-wide integration pull request. Read [docs/typescript-deprecation.md](./docs/typescript-deprecation.md) before changing a migration tranche. Do not declare repository-wide parity while its ledger still contains executable TypeScript runtime entries.

### Non-negotiable migration program contract

- The objective is a repository-wide replacement of executable TypeScript business logic with Rust. Completing one helper, package, test group, or focused PR is not a stopping condition. Continue in reviewable tranches until every production-cutover and removal gate in this file is satisfied.
- Keep one integration line: `rust/typescript-deprecation` and its repository-wide pull request. Focused branches must target that branch, must be merged into it in dependency order, and must be closed once their exact history and reviewed content are consolidated. Do not open competing repository-wide migration PRs and do not push migration work directly to `main`.
- Keep the integration pull request in draft and do not merge it to `main` while any required test, lint, differential, platform, packaging, provenance, security, advisory, caller-cutover, or removal gate is red, skipped, unknown, or blocked.
- Use RED-first TDD for every behavior tranche. Commit or otherwise retain the failing translated test contract before adding the Rust implementation. Keep the TypeScript tests and implementation as the behavioral oracle until the production cutover is proven.
- Migrate all relevant tests, fixtures, failure contracts, ordering rules, side effects, and platform branches. A passing subset, matching line count, successful compilation, or broad smoke test is not evidence of complete parity.
- Security takes precedence over blindly reproducing unsafe TypeScript behavior. Research trust boundaries and current advisories before implementation and repeat that review before merge. Fix discovered TypeScript logic or security flaws in Rust, add a regression test, and record the exact incompatibility and rationale.
- Audit the complete resolved Rust lockfile. Never add a broad advisory ignore, warning suppression, or CI bypass to make the migration appear green. When an upstream constraint blocks a patched version, keep the gate red and document the exact advisory, affected version, reverse dependency chain, attempted remediation, and required upstream or direct-dependency change.
- Do not use `--no-verify`, force-push shared migration history, weaken required checks, remove tests merely because they fail, or relabel a skipped or unavailable validation as passing.
- Every tranche must update its package `README.md`, `PARITY_MATRIX.md`, and `SECURITY.md`, plus `docs/typescript-deprecation.md` and `docs/rust-migration-security-findings.md`, in the same reviewed change.
- Production completion includes native/WASM or minimal host bindings, npm/native packaging, signing and provenance, release and rollback behavior, all downstream callers, supported-platform differential tests, and removal tests proving that executable TypeScript is neither loaded nor shipped. Only then may the corresponding runtime TypeScript be deleted.
- Report migration progress from an explicit inventory of executable TypeScript surfaces and weighted completion stages. Report Rust core implementation progress separately from production cutover/removal progress. Never infer a percentage from lines of code or raw test counts, and never round an estimate up to suggest completion.
- Human-facing implementation guidance and requested code output must use complete, copy-paste-ready files or commands rather than omitted sections or unexplained fragments.
- Agents must perform the work they report in the current execution and leave exact commits, tests, documentation, and blockers in the repository. Do not promise asynchronous or background completion.
- After each successful consolidation or tranche, immediately select the next highest-impact unblocked migration surface and continue. Stop only for a concrete tool, evidence, safety, or CI blocker, and record that blocker precisely without claiming the migration is complete.

### 1. Inventory the complete behavior surface

Before writing Rust code:

- Identify the TypeScript runtime files, public exports, CLI flags, environment variables, serialized formats, filesystem/network/process side effects, platform-specific behavior, package metadata, release jobs, and downstream callers.
- Locate the existing tests and fixtures that define the current behavior. Run them before changing implementation and record any pre-existing failures instead of silently accepting them as the new baseline.
- Add or update the component row in `docs/typescript-deprecation.md`, including the exact remaining closure work.

### 2. Use test-driven and differential development

- Translate the existing behavioral tests into Rust before implementing the corresponding behavior. The new Rust test must fail for the missing behavior before the implementation is added.
- Keep the TypeScript suite as an oracle until production cutover. Add differential fixtures that run both implementations over the same inputs and compare outputs, exit codes, error classes, ordering, mutations, and side effects.
- Cover success, failure, malformed input, boundary sizes, cancellation, concurrency, platform differences, and deterministic behavior. Compilation alone is never parity evidence.
- Preserve exact safe-input behavior unless a documented security fix intentionally changes it.

### 3. Perform a security review and current advisory lookup

Perform this review at the start of each tranche and repeat it before merge:

- Map attacker-controlled inputs, privilege boundaries, credentials, network destinations, package acquisition, executable resolution, subprocess arguments, filesystem roots, parsers, caches, and persistence.
- Check current authoritative security sources for every new or materially changed dependency and externally executed tool. At minimum, consult the RustSec Advisory Database, GitHub Security Advisories, and the dependency or tool's official security notices and release notes. Record the lookup date, affected versions, sources checked, and disposition in the component `SECURITY.md`.
- Review command and argument injection, shell use, package-spec execution, `PATH` substitution, path traversal, symlink/hardlink and TOCTOU races, archive extraction, SSRF and redirects, TLS validation, parser size/depth limits, denial of service, secret logging, telemetry/redaction, authorization, unsafe code, integer/encoding boundaries, process cleanup, and fail-open/fail-closed behavior.
- Do not preserve a known unsafe behavior merely to claim parity. Every intentional security incompatibility must be explicit, covered by a regression test, and recorded in both `SECURITY.md` and `PARITY_MATRIX.md`.
- Use the fewest new dependencies possible, prefer workspace-managed versions, and do not add an unreviewed Git, path, URL, prerelease, or unpinned executable acquisition path.

### 4. Populate the migration documentation in the same change

Every migrated component must contain and keep current:

- `README.md`: scope, architecture, test commands, packaging/binding status, production entry-point status, and remaining work.
- `PARITY_MATRIX.md`: each TypeScript source/test boundary mapped to Rust, with implemented, intentional-deviation, partial, blocked, or not-implemented status.
- `SECURITY.md`: trust boundaries, findings, severity, impact, fix, regression tests, advisory lookup record, and residual risk.
- `docs/typescript-deprecation.md`: repository ledger status, test totals, production cutover state, and exact blockers.
- `docs/rust-migration-security-findings.md`: repository-level finding index, exact remediation status, regression coverage, and unresolved dependency or platform blockers.

Do not leave these files as placeholders. Behavior, test, security, dependency, packaging, or cutover changes must update the relevant Markdown files in the same tranche. Update `AGENTS.md` whenever this workflow or its required gates change.

### 5. Rust implementation rules

- Follow the workspace panic-extraction policy. Production code must not introduce `.unwrap()`, `.unwrap_err()`, `.unwrap_none()`, or `.expect()`.
- Avoid `unsafe`. When it is unavoidable, keep the block minimal, document the safety invariants next to it, and add tests around the boundary.
- Invoke subprocesses with argument vectors rather than shell command construction. Bound execution time and output, validate semantic mini-languages, and clean up descendant processes.
- Bound untrusted reads and parser depth, canonicalize or otherwise constrain filesystem access, and define symlink behavior deliberately.
- Make errors deterministic and preserve exact public error/exit/serialization contracts where safe.

### 6. Required validation before cutover

Run the narrow component gates while developing, then the repository-required gates before merge. At minimum:

```sh
cargo fmt --all --check
cargo check --locked -p <rust-package> --all-targets
cargo test --locked -p <rust-package> --all-targets
cargo clippy --locked -p <rust-package> --all-targets -- -D warnings
pnpm --filter <typescript-package> test
```

Also run the tranche's differential tests and any supported dependency/advisory scanner. A missing tool, skipped job, or unavailable platform is a recorded blocker, not a pass.

Executable TypeScript may be deleted only after all of these are true:

- Safe-input differential parity passes on every supported platform.
- Security deviations and residual risks are reviewed and tested.
- Native/WASM bindings or the minimal required JavaScript host adapter are production-ready.
- npm/native packaging, provenance, signing, release, rollback, and install behavior point to the Rust implementation.
- All downstream callers are migrated.
- Removal tests prove that the old TypeScript runtime is no longer loaded or shipped.
- The component ledger and colocated Markdown files state complete with evidence.

### 7. Commits and pull requests

- Work in reviewable tranches on `rust/typescript-deprecation` or a focused branch targeting it. Never force-push shared migration history.
- Keep tests, implementation, security notes, parity notes, ledger updates, and the repository finding index together so the branch never loses its evidence trail.
- Merge only a tranche whose applicable focused Rust checks, translated tests, security regressions, TypeScript oracle tests, and differential fixtures pass. A focused branch may remain blocked while the integration pull request continues with an independent unblocked tranche.
- Use Conventional Commit titles and never bypass hooks with `--no-verify`.
- PR descriptions must list exact commands and results, test counts, security findings and intentional deviations, advisory sources checked, production cutover status, and remaining blockers.
- Never use “1:1 parity”, “complete”, “100%”, or “TypeScript deprecated” unless the evidence and repository ledger satisfy every gate above.

## Pull Request Guidelines

### Always run pre-commit/pre-push hooks

- You are not allowed to use `--no-verify` when making a commit or push.
- If you do not have dependencies available, you can download them with `pnpm install --frozen-lockfile`.

### Rust panic extraction policy

- Workspace Clippy lints deny `.unwrap()`, `.unwrap_err()`, `.unwrap_none()`, and `.expect()` in Rust targets covered by `cargo lint`.
- Crates with existing implementation-code violations may temporarily allow `clippy::unwrap_used` and `clippy::expect_used` at the crate root; remove those allows as each crate is cleaned up.
- Tests are exempt from this panic-extraction policy, but still linted by `cargo lint` with panic-extraction lints allowed under `cfg(test)`.

### CI task scheduling

- Test and lint workflows do not pre-classify changed paths. PR jobs run consistently and use the Turborepo task graph and cache where applicable.
- Same-repository PRs authenticate to Remote Cache through OIDC; fork PRs remain local-only.
- Rust CI restores full Cargo target state on Ubuntu, macOS, and Windows from trusted `main` snapshots; only `main` writes. Repository sccache dogfooding is disabled.
- Linux Rust shards include `terminal-control` black-box TUI integration tests; known regressions remain explicitly ignored.
- Rust test shards pin Node.js 18.20.2, so any `packageManager` version an integration fixture declares has to run on that Node.
- Rust test shards and the `@turbo/repository` native-package matrix install the pinned uv version because Python workspace discovery invokes `uv workspace metadata`.
- Rust integration tests read fixtures from `turborepo-tests/integration/fixtures/`, never from `examples/`. Examples are version-bumped on their own cadence and are not declared inputs of the Rust test task.
- Example validation remains push-only because it requires Vercel credentials and project state.

### PR Title Format

PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/). See [`.github/workflows/lint-pr-title.yml`](./.github/workflows/lint-pr-title.yml) for the enforced constraints.

Format: `<type>: <Description>`

Key rules:

- Description must start with uppercase letter
- Scopes are not allowed

Examples:

```
feat: Add new cache configuration option
fix: Resolve race condition in task scheduling
docs: Update installation instructions
```

## Release Workflow Notes

- The `LSP` workflow packages `packages/turbo-vsc` VSIX artifacts for release. Stable and canary Turborepo versions are mapped to Marketplace-safe `major.minor.patch` versions before packaging.
- Canary VS Code extension packages use `--pre-release`.
- Non-dry-run releases publish the VS Code extension through the `LSP` workflow using `publish=true`, `dry_run=false`, and a `VSCE_PAT` secret on the protected `vscode-marketplace` environment. This publish path must not block release PR creation. Once npm publishing starts, preserve the staging branch and release tag so partial releases can be resumed safely.
- npm publishing is resumable per package: existing versions are skipped only when registry integrity and the requested dist-tag match the local release and provenance is present, and `turbo` publishes last after the native and supporting packages.
- Release PRs are created by `github-actions[bot]` using the ephemeral workflow token. After exact generated release changes are validated from immutable SHAs, the release job approves only the three required PR workflows, waits for their checks, and squash-merges the validated head SHA. Turborepo release PR titles use `chore: Release Turborepo <version>`; the author and title are part of the trusted release-PR validation contract.
- The `Release` workflow signs and notarizes macOS `turbo` binaries during `build-rust` using static GitHub secrets and `apple-codesign`/`rcodesign`.
- The `Release` and `LSP` workflows install Zig during `build-rust` because `turbo` and `turborepo-lsp` link `libghostty-vt` through `libghostty-vt-sys`.
