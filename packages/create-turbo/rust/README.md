# create-turbo Rust migration

This crate is the Rust migration target for executable behavior in `packages/create-turbo`.

Current Rust tranches cover:

1. `update-commands-in-readme` package-manager command rewriting.
2. `git-ignore` creation using the exact TypeScript default content.

The TypeScript package remains the production entry point and differential-test oracle until CLI, prompt, acquisition, Git, packaging, and downstream cutover work is complete.

## Implemented behavior

### README command transform

- package-manager command rewriting for `pnpm`, `npm`, `yarn`, and `bun`;
- exact distinction between `<manager> run <script>` and bare manager subcommands;
- inline-code and triple-backtick fenced-code discovery;
- prose and `npx` isolation;
- `not-applicable` behavior when no package manager or README is present;
- deterministic success metadata after a completed write;
- bounded UTF-8 reads and linear-time rewriting;
- no-follow checks, Unix identity checks, synchronized temporary writes, and permission preservation.

### `.gitignore` transform

- exact `DEFAULT_IGNORE` bytes, including `.turbo` and the leading/trailing newline contract;
- `success` after creating a missing `.gitignore`;
- `not-applicable` for an existing regular file or directory, without modification;
- the public `Unable to write .gitignore` error text when the project root cannot be written;
- no-overwrite publication through a fully written temporary file and `hard_link`;
- rejection of symlinked project roots and existing or broken `.gitignore` symlinks;
- bounded temporary-name retries and ordinary failure cleanup.

## Not yet implemented in Rust

- CLI argument parsing, help/version output, and prompts;
- example discovery and secure network/archive acquisition;
- package-manager installation orchestration;
- Git and Mercurial repository detection, Git initialization, staging, and commit behavior;
- the remaining source transforms;
- telemetry binding;
- native/JavaScript host boundary and npm packaging;
- production entry-point and downstream-caller cutover.

## Architecture

`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication.

The `.gitignore` transform never performs a separate “does not exist, then overwrite-capable write” sequence. It writes the constant to a newly created sibling temporary file, synchronizes it, revalidates the root, and publishes it with a no-overwrite hard link. A concurrent destination wins and is never overwritten.

## TDD history

```text
README RED:       a0930bc5bd0eee5bc7c6edf09daf8caf38875781
README GREEN:     0af47426b5ef00bbff6dfc7d60aaca23daa71720
.gitignore RED:   f8edbb984cd7255f1d7630689384324009de5ac4
```

Focused validation:

```sh
cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
pnpm --filter create-turbo test
```

The crate contains 17 translated parity tests and 14 security regression tests, for 31 focused Rust tests.

## Production status

This is an in-progress migration core, not a production cutover. No TypeScript source is deleted by these tranches. `PARITY_MATRIX.md`, `SECURITY.md`, and `docs/typescript-deprecation.md` contain the exact remaining closure requirements.
