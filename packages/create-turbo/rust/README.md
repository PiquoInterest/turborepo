# create-turbo Rust migration

This crate is the Rust migration target for executable behavior in `packages/create-turbo`.

The current tranche ports the `update-commands-in-readme` transform. It preserves the TypeScript transform's safe-input behavior for package-manager command references inside inline Markdown code and triple-backtick fenced blocks. Prose outside code regions remains unchanged.

## Current scope

Implemented:

- package-manager command rewriting for `pnpm`, `npm`, `yarn`, and `bun`;
- exact distinction between `<manager> run <script>` and bare manager subcommands;
- inline-code and triple-backtick fenced-code discovery;
- `not-applicable` behavior when no package manager or README is present;
- deterministic `success` result and transform name after a completed write;
- bounded UTF-8 reads and linear-time rewriting;
- regular-file and symlink checks for the project root and README;
- same-file checks on Unix before replacement;
- same-directory temporary writes, permission preservation, synchronization, and replacement;
- cleanup of temporary files on ordinary failure paths.

Not yet implemented in Rust:

- the `create-turbo` CLI and prompts;
- example discovery and network acquisition;
- package-manager installation orchestration;
- Git initialization and commit behavior;
- the remaining source transforms;
- telemetry binding;
- npm/native packaging and production entry-point cutover.

The TypeScript implementation remains the production entry point and test oracle until those boundaries are closed.

## Architecture

`replace_package_manager_references` is a pure bounded transformer. It scans for the same two Markdown regions as the TypeScript regular expression, then applies the same ordered replacements:

1. `<manager> run` becomes `<selected-manager> run`;
2. a bare manager becomes the selected manager unless JavaScript whitespace plus `run` follows it.

The implementation uses a linear scanner rather than a backtracking regular expression. `transform_readme` owns filesystem policy and writes through a same-directory temporary file.

## Tests

The RED contract was committed before the implementation:

```text
a0930bc5bd0eee5bc7c6edf09daf8caf38875781
```

Focused validation:

```sh
cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
pnpm --filter create-turbo test
```

The Rust crate currently contains 12 translated parity tests and 9 security regression tests.

## Production status

This is an in-progress migration core, not a production cutover. No TypeScript source is deleted by this tranche. `PARITY_MATRIX.md`, `SECURITY.md`, and `docs/typescript-deprecation.md` record the exact remaining closure work.
