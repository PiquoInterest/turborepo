# TypeScript deprecation program

This document tracks the repository-wide migration of executable TypeScript logic to Rust. It is a migration ledger, not a parity claim. A component is complete only after its public behavior is covered by differential or translated tests, its security-sensitive boundaries are reviewed, and the Rust implementation is the production entry point.

Base revision for the first migration tranche: `813d54ae054923e85269979dfa98fe5e47331070`.

## Current progress

The migration program currently contains three workspace-registered Rust migration cores:

- `packages/turbo-ignore/rust`: 25 translated parity tests and 13 security regression tests.
- `packages/turbo-utils/rust`: 70 translated parity tests and 36 security regression tests. Covered surfaces include case conversion, upward/root/config discovery, folder and directory validation, writability, package-manager discovery, `createProject` orchestration, update notifications, archive entry policy, and GitHub token/proxy selection.
- `packages/create-turbo/rust`: 12 translated parity tests and 9 security regression tests for the `update-commands-in-readme` transform.

That is 165 authored Rust tests on this migration branch. No TypeScript package is removed yet because safe-input differential execution, production bindings, packaging, supported-platform closure, and downstream cutover are still open. Migration CI auto-discovers package-local Rust migration crates, requires their evidence documents and dated advisory records, then compiles, tests, lints, and audits the resolved Rust dependency graph.

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
| `packages/turbo-utils` | TypeScript utilities | `packages/turbo-utils/rust` plus JS/WASM bindings where needed | In progress | Implement and differentially test request execution, GitHub repository resolution, network/archive extraction, and registry update checking; port remaining template/example utilities; close Windows ACL/process/shim gaps; add bindings and migrate callers. |
| `packages/create-turbo` | TypeScript CLI | `packages/create-turbo/rust` | In progress | README command rewriting is ported. Preserve CLI parsing, prompts, templates, package-manager behavior, Git behavior, remaining transforms, network/filesystem failures, telemetry binding, packaging, and downstream cutover. Reuse reviewed `turbo-utils-rs` providers. |
| `packages/turbo-gen` | TypeScript CLI | Rust CLI | Queued | Preserve generator discovery, prompts, template rendering, and workspace mutations. |
| `packages/turbo-codemod` | TypeScript CLI | Rust CLI | Queued | Port transformations with golden fixtures and idempotence tests. |
| `packages/turbo-workspaces` | TypeScript CLI/library | Rust CLI/library | Queued | Preserve package-manager adapters and lock/workspace mutation semantics. |
| `packages/turbo-telemetry` | TypeScript | Rust telemetry client or explicit retirement | Queued | Define consent, persistence, transport, retry, and redaction contract. |
| `packages/eslint-plugin-turbo` and config | TypeScript in a JavaScript host | Rust/WASM rule core with minimal JS adapter | Host-bound | Preserve ESLint node/range/fix semantics and publish compatible packages. |
| VS Code extension and language tooling adapters | TypeScript host layer plus Rust LSP | Rust LSP with minimal extension adapter | Host-bound | Move business logic into Rust; retain only API bootstrap required by VS Code. |
| Factory/web UI and `.tsx` surfaces | TypeScript/React | Rust/WASM only where justified | Host-bound | Define browser/WASM architecture and DOM bindings before deprecation. |
| Test-only TypeScript fixtures | TypeScript | Rust tests or retained cross-language oracle | Later | Remove only after every migrated component has equivalent coverage. |
| npm/native publishing wrappers | JavaScript/TypeScript | Generated platform loaders and signed Rust binaries | Queued | Preserve package names, install behavior, platform selection, provenance, and rollback. |

## Current `create-turbo` tranche

The `update-commands-in-readme` transform is now represented by a package-local Rust crate. The pure transformer preserves the TypeScript alternation order for triple-backtick fences and inline code spans, then performs the same two ordered command substitutions for `pnpm`, `npm`, `yarn`, and `bun`. Prose and `npx` remain untouched.

The filesystem wrapper intentionally tightens unsafe TypeScript behavior:

- README reads are limited to 4 MiB and processed with a linear scanner;
- malformed UTF-8 is rejected without rewriting bytes;
- symlinked roots and README files are rejected;
- Unix builds compare file identity before replacement;
- writes use a same-directory newly created temporary file and preserve Unix mode bits;
- ordinary failure paths leave the original file unchanged and remove temporary files.

Windows atomic replacement and metadata/ACL preservation are not closed and therefore block production cutover. The TypeScript implementation remains the production oracle and entry point.

TDD history:

- README command transform RED contract: `a0930bc5bd0eee5bc7c6edf09daf8caf38875781`.
- Lockfile-only bootstrap commit: `ba93b4772e15fa1211bc7b70ae5eb1f223d66e67`.

## Current `turbo-utils` tranche

The project coordinator is separated from network/archive execution through `ProjectSource`. It covers source selection, four acquisition attempts, generated `package.json` inspection, JavaScript script-key ordering, missing sources, and strict URL/path/destination/metadata boundaries.

The notification core is separated from registry lookup through `UpdateChecker` and dynamic command resolution through `UpgradeCommandProvider`. It covers one stored update decision, failed/no-update silence, command rendering, exit-code preservation, TypeScript-compatible debug handling, and bounded control-safe output.

The archive entry policy translates `isPathSafe` and `isLinkEntry` for safe paths and rejects NULs, escaping traversal, absolute/UNC/drive paths, alternate streams, excessive length/depth, symbolic links, and hard links. It also fixes the TypeScript `relativePath.startsWith("..")` false positive for safe names such as `..cache`.

The GitHub network policy translates environment precedence without performing I/O:

- `GITHUB_TOKEN` takes precedence over `GH_TOKEN`; an invalid selected primary token does not silently fall back to secondary credentials;
- bearer credentials are emitted only for credential-free HTTPS requests to exact `api.github.com` or `codeload.github.com` authorities with no explicit port;
- look-alike domains, malformed URLs, plaintext HTTP, userinfo, ports, non-ASCII/control-bearing tokens, and oversized tokens receive no credentials;
- HTTPS proxy precedence remains lowercase HTTPS, uppercase HTTPS, lowercase HTTP, uppercase HTTP; non-HTTPS uses only HTTP proxy values;
- an invalid selected proxy is an error rather than permission to bypass proxy policy with a direct connection.

Actual HTTP execution, redirect handling, proxy agent construction, `NO_PROXY` policy, GitHub repository/default-branch resolution, sparse Git fallback, tar streaming/writes, registry lookup, staging, and atomic promotion remain outside these pure cores until their complete contracts are differential-tested.

TDD history:

- Project creation: RED `0468eda3829e5b1bb98f96b86a7f0817ac542f51`, implementation `b2992a27dbf44c5ab8bc7405dc088236eb53c70e`.
- Update notification: RED `7a446b29f3e6054a58e891b898d3f8c4f85854ce`, implementation `cabec01820809f34d8f42cf1adbbff50c3307e68`, bidi RED `28f7ec2f0388c93a92d6ed7261c10832e29f5925`, fix `84e5ed11155af39a151d56b1e6b4580eaf6f057e`.
- Archive entry policy: RED `5ab1da42327a85e4c026e8531953fb108b56434d`, `..`-prefix RED `37f6b4caaea9d06e87ec82f550560d989a6af872`, implementation `bdeb6760d41d5f9d72d2b9fb8042339b55011923`.
- GitHub token/proxy policy: RED `903d7836a01e6ec47e4df339adc71456b4ecbd0d`, implementation `2e90ea8daa8542aa13cd94ceb981b653756789cb`.

## Security review method

Each tranche gets a colocated `SECURITY.md` containing attacker inputs and trust boundaries, subprocess/executable behavior, filesystem/race considerations, resource limits, network/package acquisition, logging/redaction, a dated authoritative advisory lookup, and intentional incompatibilities where exact parity would preserve unsafe behavior.

Security fixes require regression tests. Memory safety alone is not completion: semantic injection, package acquisition, path trust, authorization, denial of service, and fail-open/fail-closed behavior require explicit review.

The repository-level index records affected `webbrowser` under `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`. The observed call uses a constant HTTP URL, limiting current reachability, but the dependency must be upgraded or removed before merge. Migration CI temporarily ignores only that documented advisory so additional findings still fail.

## Branch and pull-request policy

The integration branch is `rust/typescript-deprecation`. Work lands in reviewable conventional-commit tranches. Repository-wide parity is declared only when this ledger has no executable TypeScript runtime entries and production packaging points to Rust for all supported targets.
