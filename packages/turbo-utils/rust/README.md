# `turbo-utils-rs`

Rust migration core for executable logic currently exposed by `@turbo/utils`.

This crate belongs to the repository-wide TypeScript deprecation program tracked in [`docs/typescript-deprecation.md`](../../../docs/typescript-deprecation.md). It is not yet the production npm entry point. TypeScript remains the compatibility oracle until bindings, packaging, downstream migration, and removal tests are complete.

## Implemented scope

- ASCII camel-case conversion.
- Bounded upward file/config discovery.
- Turborepo root and workspace/config discovery.
- Empty-folder and project-directory validation.
- Unix writability checks, with the Windows ACL gap documented.
- Package-manager version and global-bin discovery for Yarn, npm, pnpm, Bun, Nub, and Aube.
- Yarn inference from `package.json#packageManager` and conventional `.yarn/releases/yarn-<version>.cjs` paths.
- `createProject` orchestration for default examples, named examples, and GitHub repositories.
- Four-attempt retry behavior and post-download `package.json` script discovery.
- Update-notification decisions/rendering, including static/dynamic commands and exit-code preservation.
- Archive entry path validation and symbolic/hard-link classification.

The Rust core preserves existing safe-input results while hardening executable lookup, metadata reads, subprocess deadlines, output limits, process cleanup, URL classification, repository subpaths, target symlinks, process-wide current-directory state, terminal rendering, and archive path handling. See [`PARITY_MATRIX.md`](./PARITY_MATRIX.md) and [`SECURITY.md`](./SECURITY.md).

## Architecture

`src/entry.rs` is the public crate surface. The original pure utility tranche remains in `src/lib.rs`; root/config and JSON5 behavior live in dedicated modules.

`src/managers.rs` separates package-manager policy from command execution through `PackageManagerCommandRunner`, allowing exact argument-vector tests without running repository-controlled commands.

`src/project.rs` separates project-creation policy from network and archive acquisition through `ProjectSource`. The coordinator resolves source type, validates the destination, performs four attempts, and inspects generated metadata without global `chdir` state.

`src/notify.rs` separates notification policy from registry lookup through `UpdateChecker` and dynamic command resolution through `UpgradeCommandProvider`. `PreparedUpdateNotification` stores one precomputed update result and returns deterministic output plus the caller-provided exit code.

`src/archive.rs` contains the pure archive entry policy. It normalizes mixed separators, resolves lexical parent components relative to the extraction root, rejects cross-platform absolute/prefix/alternate-stream forms, bounds path size/depth, and rejects tar symbolic and hard links. It deliberately does not write files; the future provider must combine this policy with descriptor-relative or private-staging extraction.

## Security properties

The system package-manager runner:

- resolves canonical executables from absolute `PATH` entries;
- rejects executable paths inside the inspected project root;
- invokes argument vectors without a shell;
- runs from the temporary directory with `COREPACK_ENABLE_STRICT=0`;
- enforces a five-second deadline and one-MiB output limit;
- kills the process group on Unix when a command exceeds a bound.

The project coordinator:

- accepts credential-free HTTPS URLs whose authority is exactly `github.com`;
- rejects traversal in named examples and repository subpaths;
- rejects target and immediate-parent symlinks;
- never changes process-wide current-directory state;
- reads only regular non-symlink `package.json` files up to one MiB;
- reproduces JavaScript `Object.keys` array-index ordering.

The notification core:

- performs the injected update check exactly once at preparation time;
- keeps failed or empty checks silent;
- resolves dynamic commands only when an update exists;
- preserves success/failure exit codes;
- escapes terminal and Unicode directionality controls;
- limits every untrusted rendered field to 1,024 Unicode scalar values.

The archive entry policy:

- accepts ordinary relative paths and internal parent cancellation that stays inside the root;
- rejects NULs, traversal above the root, absolute/UNC/drive paths, and Windows alternate data streams;
- limits paths to 4,096 Unicode scalar values and 256 non-empty components;
- rejects symbolic and hard-link tar entries;
- corrects the TypeScript false positive that treats safe names such as `..cache` as traversal merely because their relative string starts with two dots.

## Validation

```sh
cargo fmt --all --check
cargo check --locked -p turbo-utils-rs --all-targets
cargo test --locked -p turbo-utils-rs --all-targets
cargo clippy --locked -p turbo-utils-rs --all-targets -- -D warnings
pnpm --filter @turbo/utils test
```

This Rust migration core now has 63 parity tests and 29 security regression tests. The archive tranche contributes 7 parity tests and 7 security/logic regressions. TypeScript tests remain required until differential host bindings exercise both implementations through the same public API.

## Production cutover status

Blocked. Remaining work includes the production GitHub/network/archive provider behind `ProjectSource`, a bounded registry update checker behind `UpdateChecker`, safe extraction writes and atomic promotion, Windows-native process-tree and ACL parity, JavaScript/WASM or native bindings, npm packaging, downstream caller migration, supported-platform differential tests, and proof that executable TypeScript is no longer loaded or shipped.
