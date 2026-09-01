# turbo-workspaces Rust migration

This crate is the Rust migration target for executable behavior in `packages/turbo-workspaces`.

## Current tranche

The first completed core ports the read-only `getWorkspaceDetails` orchestration contract from `src/get-workspace-details.ts`:

- inspect the requested directory before any package-manager detector receives authority;
- use the provider-returned absolute path for every detector and reader call;
- detect managers serially in exact TypeScript registry order: `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`;
- read exactly the first manager that detects the workspace;
- propagate detector and selected-reader failures immediately, without trying a fallback parser;
- return the exact `invalid_directory` and `package_manager-unable_to_detect` messages.

The TypeScript package remains the production implementation and behavioral oracle. Filesystem providers, manager-specific parsers, bindings, packaging, downstream callers, supported-platform differential execution, and TypeScript-removal proof remain open.

## TDD evidence

- TypeScript oracle commit: `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`
- TypeScript oracle: `packages/turbo-workspaces/__tests__/workspace-details.test.ts`
- TypeScript result: 5 of 5 focused Jest tests passed in workflow run `33551576871`, job `100002027683`
- compiling behavioral RED: `2d4cc22e6a821c88882a87d604746dabbaa95fe2`
- Rust GREEN implementation: `263ddc22d5b5f544768f4e089c92892339b0dce8`
- Rust parity tests: 6
- Rust security tests: 5

The RED commit exported the final provider API and known errors but deliberately returned `UnableToDetect` after a successful directory check. The GREEN commit adds only the fixed manager loop and selected read, preserving a reviewable test-first history.

## Validation

The crate is temporarily self-contained so the RED/GREEN history and local lockfile remain reproducible while the repository runner queue is unavailable:

```sh
cargo fmt --manifest-path packages/turbo-workspaces/rust/Cargo.toml --all --check
cargo check --manifest-path packages/turbo-workspaces/rust/Cargo.toml --locked --all-targets
cargo test --manifest-path packages/turbo-workspaces/rust/Cargo.toml --locked --all-targets
cargo clippy --manifest-path packages/turbo-workspaces/rust/Cargo.toml --locked --all-targets -- -D warnings
pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-details.test.ts
```

GitHub Actions remains authoritative for Rust execution. The code and tests are committed, but Rust GREEN execution is still pending while hosted jobs are queued. The integration step must remove the nested `[workspace]`, inherit the root edition/lints, add this crate to the root workspace, regenerate the root lockfile with an exact one-package delta, and rerun all gates.

## Architecture

`get_workspace_details` owns only deterministic orchestration. `WorkspaceDetailsProvider` owns path inspection, manager detection, and manager-specific reading. The core cannot read a file, traverse a directory, spawn a process, access the network, or broaden the manager registry.

The provider boundary is deliberate. A production provider must use stable filesystem identities, reject unsafe links and special files, bound reads and parser depth, preserve exact manager semantics, and pass Linux, macOS, and Windows differential fixtures before cutover.
