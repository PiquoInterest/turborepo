# turbo-workspaces Rust migration

This crate is the Rust migration target for executable behavior in `packages/turbo-workspaces`.

The TypeScript package remains the production implementation and behavioral oracle. A Rust core is considered implemented only after the TypeScript oracle is green, a compiling behavioral Rust RED is committed, the minimal Rust GREEN is committed, and focused GitHub Actions validate formatting, compilation, tests, and Clippy.

## Implemented cores

### Workspace-details orchestration

The Rust core ports the read-only `getWorkspaceDetails` control flow from `src/get-workspace-details.ts`:

- inspect the requested directory before any package-manager detector receives authority;
- use the provider-returned absolute path for every detector and reader call;
- detect managers serially in exact TypeScript registry order: `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`;
- read exactly the first manager that detects the workspace;
- propagate detector and selected-reader failures immediately, without trying a fallback parser;
- return the exact `invalid_directory` and `package_manager-unable_to_detect` messages.

TDD evidence:

- TypeScript oracle commit: `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`;
- compiling behavioral Rust RED: `2d4cc22e6a821c88882a87d604746dabbaa95fe2`;
- Rust GREEN: `263ddc22d5b5f544768f4e089c92892339b0dce8`;
- tests: 6 parity and 5 security.

### Bun workspace-glob compatibility

The Rust core preserves the TypeScript-supported Bun glob subset for ordinary inputs while adding explicit count, per-value, and aggregate byte limits. It also rejects terminal-active and invisible control text before any later path or glob consumer receives it.

Tests: 12 parity and 6 security.

### Workspace package parsing

The Rust core ports `parseWorkspacePackages` from `src/utils.ts`:

- a missing `workspaces` value becomes an empty list;
- an array is preserved in source order;
- an object with `packages` yields that array;
- an object without `packages` becomes an empty list;
- duplicates, empty strings, negations, recursive globs, braces, and brackets remain valid input.

The TypeScript oracle deliberately records unsafe legacy behavior with passing `it.failing` cases. Rust intentionally rejects more than 256 globs, values larger than 4096 UTF-8 bytes, aggregate input above 65536 bytes, and terminal-active or invisible control text. Public errors do not echo attacker-controlled glob data.

TDD evidence:

- TypeScript oracle: `9c8f77deee15c01baba73fdd510960e899756f0e`;
- compiling behavioral Rust RED: `089112a3f85bc2cbaaf864991eb5b6129602ff30`;
- Rust GREEN: `8b4aea45459aa09237aef7d8dd35ccf06503ae28`;
- tests: 7 parity and 6 security;
- detailed ledger: [`WORKSPACE_PACKAGES_DIVERGENCES.md`](./WORKSPACE_PACKAGES_DIVERGENCES.md).

## Current test inventory

- parity tests: 25;
- security tests: 17;
- total Rust tests in this crate: 42.

These counts describe authored evidence, not production completion. See [`TEST_INVENTORY.md`](./TEST_INVENTORY.md) for the TypeScript-suite mapping and remaining work.

## Validation

```sh
pnpm exec oxfmt --check packages/turbo-workspaces/__tests__/workspace-packages.test.ts
pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-packages.test.ts
cargo fmt --all --check
cargo check --locked -p turbo-workspaces-rs --all-targets
cargo test --locked -p turbo-workspaces-rs --all-targets
cargo clippy --locked -p turbo-workspaces-rs --all-targets -- -D warnings
```

GitHub Actions is authoritative. A queued, skipped, cancelled, or failing job is a blocker rather than a pass.

## Architecture and production blockers

The current cores are deterministic and provider-oriented. They cannot read a file, traverse a directory, spawn a process, access the network, or expand the parser registry.

Production cutover still requires:

- bounded, no-follow `package.json`, lockfile, and workspace-configuration providers;
- stable filesystem identity across detection, parsing, and mutation;
- parser byte, collection, and nesting limits for every manager;
- staged file publication and rollback after every injected failure point;
- canonical no-shell process execution with deadlines, bounded output, cancellation, and descendant cleanup;
- Linux, macOS, and Windows TypeScript-versus-Rust differential fixtures;
- native or minimal host bindings, npm packaging, provenance, downstream caller cutover, and artifact proof that executable TypeScript is neither loaded nor shipped.
