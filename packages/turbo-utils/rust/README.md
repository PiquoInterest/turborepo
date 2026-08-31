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

The package-manager implementation preserves the existing safe-input results while hardening executable lookup, metadata reads, subprocess deadlines, output limits, and process cleanup. See [`PARITY_MATRIX.md`](./PARITY_MATRIX.md) and [`SECURITY.md`](./SECURITY.md).

## Architecture

`src/entry.rs` is the public crate surface. The original pure utility tranche remains in `src/lib.rs`; root/config and JSON5 behavior live in dedicated modules. `src/managers.rs` separates package-manager policy from command execution through `PackageManagerCommandRunner`, which lets translated tests verify exact argument vectors without executing repository-controlled commands.

The system runner:

- resolves canonical executables from absolute `PATH` entries;
- rejects executable paths inside the inspected project root;
- invokes argument vectors without a shell;
- runs from the temporary directory with `COREPACK_ENABLE_STRICT=0`;
- enforces a five-second deadline and one-MiB output limit;
- kills the process group on Unix when a command exceeds a bound.

## Validation

```sh
cargo fmt --all --check
cargo check --locked -p turbo-utils-rs --all-targets
cargo test --locked -p turbo-utils-rs --all-targets
cargo clippy --locked -p turbo-utils-rs --all-targets -- -D warnings
pnpm --filter @turbo/utils test
```

The package-manager tranche adds 12 translated parity tests and 6 security regression tests. The TypeScript tests remain required until differential host bindings exercise both implementations through the same public API.

## Production cutover status

Blocked. Remaining work includes the network/example/template/project-creation/update-notification surfaces, Windows-native process-tree and ACL parity, JS/WASM or native bindings, npm packaging, downstream caller migration, supported-platform differential tests, and proof that runtime TypeScript is no longer loaded or shipped.
