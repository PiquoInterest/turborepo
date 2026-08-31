# `turbo-utils-rs`

Rust migration core for executable logic currently exposed by `@turbo/utils`.

This crate belongs to the repository-wide TypeScript deprecation program tracked in [`docs/typescript-deprecation.md`](../../../docs/typescript-deprecation.md). It is not yet the production npm entry point. TypeScript remains the compatibility oracle until bindings, packaging, downstream migration, and removal tests are complete.

## Implemented scope

- ASCII camel-case conversion.
- Bounded upward file/config discovery.
- Turborepo root and workspace/config discovery.
- Empty-folder and project-directory validation.
- Fail-closed directory inspection for option-like basenames, existing symlinked path components, symlinked allow-list entries, non-UTF-8 filenames, and directories exceeding 256 entries.
- Unix writability checks, with the Windows ACL gap documented.
- Package-manager version and global-bin discovery for Yarn, npm, pnpm, Bun, Nub, and Aube.
- Yarn inference from `package.json#packageManager` and conventional `.yarn/releases/yarn-<version>.cjs` paths.
- `createProject` orchestration for default examples, named examples, and GitHub repositories.
- Four-attempt retry behavior and post-download `package.json` script discovery.
- Update-notification decisions/rendering, including static/dynamic commands and exit-code preservation.
- Archive entry path validation and symbolic/hard-link classification.
- GitHub token allow-listing and HTTP/HTTPS proxy precedence policy.

The Rust core preserves existing safe-input results while hardening executable lookup, metadata reads, subprocess deadlines, output limits, process cleanup, URL classification, credentials, proxy failure behavior, repository subpaths, target symlinks, process-wide current-directory state, terminal rendering, archive paths, and project-directory inspection. See [`PARITY_MATRIX.md`](./PARITY_MATRIX.md) and [`SECURITY.md`](./SECURITY.md).

## Architecture

`src/entry.rs` is the public crate surface. The original pure utility tranche remains in `src/lib.rs`; root/config and JSON5 behavior live in dedicated modules.

The directory-validation core resolves the requested root lexically, validates the basename before downstream use, rejects existing symlinked path components, rejects symlink directory entries even when their names are normally allowed, requires exact UTF-8 entry names, and stops after 256 entries. The implementation deliberately fails closed when metadata or enumeration is uncertain. It does not claim to close malicious concurrent path replacement; production mutation still needs stable directory handles or private staging followed by atomic promotion.

`src/managers.rs` separates package-manager policy from command execution through `PackageManagerCommandRunner`, allowing exact argument-vector tests without running repository-controlled commands.

`src/project.rs` separates project-creation policy from network/archive acquisition through `ProjectSource`. The coordinator validates source/destination behavior, performs four attempts, and inspects generated metadata without global `chdir` state.

`src/notify.rs` separates notification policy from registry lookup through `UpdateChecker` and dynamic command resolution through `UpgradeCommandProvider`. `PreparedUpdateNotification` stores one precomputed update result and returns deterministic output plus the caller-provided exit code.

`src/archive.rs` contains pure archive-entry policy. It normalizes mixed separators, resolves lexical parent components, rejects cross-platform absolute/prefix/alternate-stream forms, bounds path size/depth, and classifies tar symbolic/hard links. It deliberately performs no writes.

`src/network.rs` snapshots GitHub token and proxy environment policy without performing network I/O. It preserves TypeScript precedence while ensuring credentials can be attached only to exact credential-free HTTPS GitHub API/codeload authorities. Invalid selected proxies are errors rather than silent direct-connection fallbacks.

## Security properties

Directory inspection rejects option-like project basenames before later command or display boundaries, rejects existing symlinked components and symlink entries, never uses lossy filename conversion for the allow-list decision, and bounds directory enumeration to 256 entries. These are intentional hardenings over the current TypeScript provider. The portable path walk reduces known link traversal, but descriptor-relative Unix operations and reviewed Windows handle/reparse-point behavior remain cutover blockers.

The package-manager runner resolves canonical executables from absolute `PATH` entries, rejects project-local executables, invokes argument vectors without a shell, uses a temporary working directory, sets `COREPACK_ENABLE_STRICT=0`, bounds time/output, and kills Unix process groups on failure.

The project coordinator accepts exact credential-free GitHub HTTPS URLs, rejects unsafe example/subpaths and target symlinks, never changes process-wide current-directory state, bounds metadata, and reproduces JavaScript script-key ordering.

The notification core performs one injected update check at preparation time, keeps failed/empty checks silent, resolves dynamic commands only when needed, preserves exit codes, escapes terminal and directionality controls, and bounds untrusted fields.

The archive policy accepts ordinary relative paths and safe internal parent cancellation, rejects NULs/traversal/absolute/UNC/drive/alternate-stream forms, limits paths to 4,096 scalar values and 256 components, rejects symbolic/hard links, and fixes the TypeScript `..cache` false positive.

The network policy:

- preserves `GITHUB_TOKEN` over `GH_TOKEN` precedence;
- rejects empty, control-bearing, non-ASCII, whitespace-containing, or oversized selected tokens;
- emits bearer credentials only for HTTPS `api.github.com` and `codeload.github.com` with no userinfo or explicit port;
- rejects look-alike hosts and malformed URLs;
- preserves lower/uppercase HTTPS/HTTP proxy precedence;
- accepts only bounded HTTP(S) proxy URLs;
- returns an error for an invalid winning proxy value instead of connecting directly.

## Directory-provider TDD record

- Consolidated create-directory prompt history: merge commit `3ac9a5c4864602372d1b88f8e39986c700d52508`.
- Directory-provider RED test commit: `53a55eefd92b919824374eb27159ff876e008147`.
- GREEN implementation: `c77464a7e6f36813a3b52262e78caa9ee449bb72`.
- Committed formatting proof: `8ee51022fd84264e0abeee17014802da3afcae20`.
- Clippy lifetime correction: `e47b4994e0d97641c2f976231aa89833aa142913`.

The first RED workflow was formatting-blocked, so it is retained as test-first history rather than described as a clean behavioral RED execution. The final merge-head workflow is the authoritative compile, test, Clippy, and advisory evidence.

## Validation

```sh
cargo fmt --all --check
cargo check --locked -p turbo-utils-rs --all-targets
cargo test --locked -p turbo-utils-rs --all-targets
cargo clippy --locked -p turbo-utils-rs --all-targets -- -D warnings
pnpm --filter @turbo/utils test
```

This Rust migration core now has 70 parity tests and 41 security regression tests. The directory-provider tranche contributes five new security tests. The network-policy tranche contributes 7 parity and 7 security tests. TypeScript tests remain required until differential host bindings exercise both implementations through the same API.

## Production cutover status

Blocked. Remaining work includes stable handle-relative directory validation and mutation, request execution and response bounds, GitHub repository/default-branch resolution, the production archive provider and safe writes behind `ProjectSource`, a bounded registry update checker behind `UpdateChecker`, explicit `NO_PROXY` semantics, Windows-native process-tree/ACL/reparse-point parity, native/WASM or JavaScript bindings, npm packaging, downstream migration, supported-platform differential tests, and proof that executable TypeScript is no longer loaded or shipped.
