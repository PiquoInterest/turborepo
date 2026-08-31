# `turbo-utils-rs`

Rust migration core for the executable logic currently exposed by `@turbo/utils`.

This crate is part of the repository-wide TypeScript deprecation program tracked in [`docs/typescript-deprecation.md`](../../../docs/typescript-deprecation.md). It is not yet the production npm entry point, and the TypeScript package remains the compatibility oracle until bindings, packaging, downstream migration, and removal tests are complete.

## Implemented scope

- ASCII camel-case conversion.
- Bounded upward file/config discovery.
- Turborepo root and workspace/config discovery.
- Empty-folder and project-directory validation.
- Unix writability checks, with the Windows ACL gap documented.
- Package-manager version and global-bin discovery for Yarn, npm, pnpm, Bun, Nub, and Aube.
- Yarn version inference from `package.json#packageManager` and conventional `.yarn/releases/yarn-<version>.cjs` paths.
- `createProject` orchestration for default examples, named examples, and GitHub repositories.
- Four-attempt download retry behavior and post-download `package.json` script discovery.
- Update-notification decision and rendering behavior, including static/dynamic commands and exit-code preservation.

The package-manager, project-creation, and notification implementations preserve existing safe-input results while hardening executable lookup, metadata reads, subprocess deadlines, output limits, process cleanup, URL classification, repository subpaths, target symlinks, process-wide working-directory state, and terminal-log rendering. See [`PARITY_MATRIX.md`](./PARITY_MATRIX.md) and [`SECURITY.md`](./SECURITY.md).

## Architecture

`src/entry.rs` is the public crate surface. The original pure utility tranche remains in `src/lib.rs`; root/config and JSON5 behavior live in dedicated modules. `src/managers.rs` separates package-manager policy from command execution through `PackageManagerCommandRunner`, which lets translated tests verify exact argument vectors without executing repository-controlled commands.

`src/project.rs` separates project-creation policy from network and archive acquisition through `ProjectSource`. The coordinator resolves source type, validates the destination, performs the TypeScript-compatible four total download attempts, and inspects generated package metadata. The future production GitHub/example downloader can therefore be differential-tested independently without reintroducing global `chdir` state into the coordinator.

`src/notify.rs` separates notification policy from network/registry lookup through `UpdateChecker` and from dynamic upgrade-command resolution through `UpgradeCommandProvider`. `PreparedUpdateNotification` stores one precomputed update result, matching the TypeScript promise created when `createNotifyUpdate` is called, then returns deterministic stdout/stderr values and the caller-provided exit code for host adapters to apply.

The system package-manager runner:

- resolves canonical executables from absolute `PATH` entries;
- rejects executable paths inside the inspected project root;
- invokes argument vectors without a shell;
- runs from the temporary directory with `COREPACK_ENABLE_STRICT=0`;
- enforces a five-second deadline and one-MiB output limit;
- kills the process group on Unix when a command exceeds a bound.

The project coordinator:

- accepts only credential-free HTTPS URLs whose authority is exactly `github.com`;
- rejects traversal in named examples and explicit repository subpaths;
- rejects target and immediate-parent symlinks;
- never changes the process-wide current directory;
- reads only regular, non-symlink `package.json` files up to one MiB;
- returns script names using JavaScript `Object.keys` array-index ordering.

The notification core:

- performs the injected update check exactly once at preparation time;
- keeps failed or empty update checks silent;
- resolves dynamic commands only after a usable update exists;
- preserves success/failure exit codes even when notification work fails;
- escapes C0/C1 controls, escape sequences, and Unicode directionality controls;
- limits each untrusted rendered field to 1,024 Unicode scalar values.

## Validation

```sh
cargo fmt --all --check
cargo check --locked -p turbo-utils-rs --all-targets
cargo test --locked -p turbo-utils-rs --all-targets
cargo clippy --locked -p turbo-utils-rs --all-targets -- -D warnings
pnpm --filter @turbo/utils test
```

Across this Rust migration core, 56 parity tests and 22 security regression tests are authored. The notification tranche contributes 9 parity tests and 4 security regressions. The TypeScript tests remain required until differential host bindings exercise both implementations through the same public API.

## Production cutover status

Blocked. Remaining work includes the network/example/template extraction implementation behind `ProjectSource`, a bounded production registry/update checker behind `UpdateChecker`, Windows-native process-tree and ACL parity, JavaScript/WASM or native bindings, npm packaging, downstream caller migration, supported-platform differential tests, and proof that runtime TypeScript is no longer loaded or shipped.
