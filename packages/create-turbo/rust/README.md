# create-turbo Rust migration

This crate is the Rust migration target for executable behavior in `packages/create-turbo`.

Current Rust tranches cover:

1. `update-commands-in-readme` package-manager command rewriting.
2. `git-ignore` creation using the exact TypeScript default content.
3. the dependency-injected core of `tryGitInit`, including Git/Mercurial detection, initialization ordering, post-init cleanup, and root validation.
4. exact `isDefaultExample` routing for the exported `basic` and `default` examples.

The TypeScript package remains the production entry point and differential-test oracle until CLI, prompt, acquisition, process-provider, packaging, and downstream cutover work is complete.

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

### Git initialization core

- returns `false` when the target is already inside a Git or Mercurial repository;
- preserves the TypeScript command order: `git init`, `git checkout -b main`, `git add -A`, then `git commit -m "Initial commit from create-turbo"`;
- uses argument vectors and a separate working directory rather than constructing a shell command;
- preserves the Mercurial `--cwd . root` arguments while carrying the project root as the invocation working directory;
- removes an initialized `.git` directory after checkout, add, or commit failure and swallows cleanup failure like the TypeScript implementation;
- does not remove `.git` when `git init` itself fails, because ownership of a concurrently created or partially existing path is ambiguous;
- requires an absolute non-root path, rejects parent/current-directory components, controls, and filename characters invalid on Windows;
- permits harmless shell metacharacters such as `$`, `#`, `;`, and `!` because the contract never invokes a shell;
- carries roots as `Path`/`PathBuf`, preserving non-UTF-8 Unix paths without lossy conversion.

The current Git tranche is an orchestration core. A production `VcsRunner` and `GitDirectoryCleaner` are deliberately not supplied yet. Their executable resolution, environment/config isolation, timeouts, process cleanup, symlink-safe deletion, and Windows behavior remain cutover blockers.

### Default-example routing

- exports the source-order default example set as `DEFAULT_EXAMPLES = ["basic", "default"]`;
- returns `true` only for exact `basic` or `default` input;
- preserves JavaScript `Set.has` case, whitespace, control-character, and Unicode-normalization behavior;
- rejects prefixes, suffixes, path-like values, Unicode confusables, and oversized arbitrary names without copying the input;
- uses a borrowed `&str` match with no heap allocation or mutable global set.

## Not yet implemented in Rust

- CLI argument parsing, help/version output, and prompts;
- example discovery and secure network/archive acquisition;
- package-manager installation orchestration;
- production Git/Hg process execution and `.git` cleanup providers;
- the remaining source transforms;
- telemetry binding into the production command path;
- native/JavaScript host boundary and npm packaging;
- production entry-point and downstream-caller cutover.

## Architecture

`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate.

The `.gitignore` transform never performs a separate “does not exist, then overwrite-capable write” sequence. It writes the constant to a newly created sibling temporary file, synchronizes it, revalidates the root, and publishes it with a no-overwrite hard link. A concurrent destination wins and is never overwritten.

The Git initialization core never resolves an executable, inherits no process environment by itself, and performs no filesystem deletion directly. Those side effects remain behind explicit provider boundaries so they can be reviewed and tested independently.

The default-example predicate cannot broaden routing through trimming, normalization, regexes, prefixes, suffixes, or mutable collection state. It is intentionally a two-literal borrowed-string match.

## TDD history

```text
README RED:             a0930bc5bd0eee5bc7c6edf09daf8caf38875781
README GREEN:           0af47426b5ef00bbff6dfc7d60aaca23daa71720
.gitignore RED:         f8edbb984cd7255f1d7630689384324009de5ac4
.gitignore GREEN:       c74d664d718691660be969d779d25a76af31fb3e
Git init RED import:    e57cc31afd1d83a015ae49136d71c7daa3217fb7
Git oracle/security:    221586118db79fca2f94cebb15785de4111bde8e
Git init GREEN:         1d7b485d597b70f40bb4aa492f45d1c0638f844e
Default example RED:    edc3b96b106e2c0bebaee299690c7769f9ba6bc2
Default example GREEN:  57f19c56209312fb2d04423fdd86ad239150a753
```

Focused validation:

```sh
cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
pnpm --filter create-turbo test
```

The crate contains 32 translated parity tests and 26 security regression tests, for 58 authored focused Rust tests. The latest tranche is not treated as validated until its merge-head workflow passes the commands above.

## Production status

This is an in-progress migration core, not a production cutover. No TypeScript source is deleted by these tranches. `PARITY_MATRIX.md`, `SECURITY.md`, and `docs/typescript-deprecation.md` contain the exact remaining closure requirements.
