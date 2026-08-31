# create-turbo Rust migration

This crate is the Rust migration target for executable behavior in `packages/create-turbo`.

Current Rust tranches cover:

1. `update-commands-in-readme` package-manager command rewriting.
2. `git-ignore` creation using the exact TypeScript default content.
3. the dependency-injected core of `tryGitInit`, including Git/Mercurial detection, initialization ordering, post-init cleanup, and root validation.
4. exact `isDefaultExample` routing for the exported `basic` and `default` examples.
5. the dependency-injected `package-manager` transform decision and conversion-request contract.
6. the dependency-injected `official-starter` transform orchestration contract.

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

### Package-manager transform core

- exports the exact transform name `package-manager`;
- represents all six repository package-manager variants: `yarn`, `npm`, `pnpm`, `bun`, `nub`, and `aube`;
- returns `not-applicable` without invoking the mutation provider when the prompt has no selection or the selected manager already matches the project;
- requests exactly one conversion when the manager changes;
- forwards the project root as a borrowed `Path`, the target manager as a closed enum, and `skip_install: true`;
- intentionally does not forward the prompt's version string, matching the TypeScript transform;
- propagates provider failure instead of reporting a false success;
- preserves non-UTF-8 Unix roots and never constructs a shell command or executable name in the orchestration core.

A production `PackageManagerConverter` is deliberately absent. The existing TypeScript `@turbo/workspaces` converter performs broad package and lockfile mutation, so its Rust provider requires its own translated tests, rollback model, atomic-write policy, supported-manager matrix, and platform review before cutover.

### Official-starter transform core

- classifies an example as official only when no repository is supplied or when the repository is exactly `vercel/turbo` or `vercel/turborepo`;
- preserves source ordering by snapshotting `package.json` existence before the best-effort `meta.json` read/removal sequence;
- returns parsed metadata when removal fails, while swallowing metadata read and removal failures like the TypeScript implementation;
- maps `package.json` read and write failures to the exact public messages, transform name, and `fatal: false` contract;
- renames `basic` and `default` package objects to the requested project name;
- updates an existing truthy `devDependencies.turbo` to a non-empty explicit version or to `^<create-turbo version>` when the option is absent or empty;
- still writes a truthy package object when neither relevant field changes, matching `writeJsonSync` ordering and side effects;
- keeps filesystem access, JSON parsing/serialization, truthiness classification, no-follow policy, resource bounds, deterministic ordering, and atomic publication behind `OfficialStarterStore` and `OfficialStarterPackageJson`.

A production store is deliberately absent. It must preserve unknown JSON fields and insertion order, implement JavaScript-compatible truthiness for the existing Turbo dependency, bound both JSON files, reject unsafe links and special files, preserve approved metadata, and stage package writes atomically on Linux, macOS, and Windows before this core can replace the TypeScript transform.

The exact type conversions and intentional security divergences are recorded in [`OFFICIAL_STARTER_DIVERGENCES.md`](./OFFICIAL_STARTER_DIVERGENCES.md).

## Not yet implemented in Rust

- CLI argument parsing, help/version output, and prompts;
- example discovery and secure network/archive acquisition;
- production package-manager workspace conversion and installation orchestration;
- production Git/Hg process execution and `.git` cleanup providers;
- production filesystem/JSON provider for the `official-starter` transform, including deterministic JSON ordering and atomic no-follow writes;
- transform dispatcher binding and public `TransformError` mapping;
- the remaining source transforms;
- telemetry binding into the production command path;
- native/JavaScript host boundary and npm packaging;
- production entry-point and downstream-caller cutover.

## Architecture

`readme_transform` owns the bounded pure Markdown scanner and the README replacement policy. `git_ignore` owns creation-only `.gitignore` publication. `git_init` owns the deterministic VCS decision and command sequence behind injected runner and cleanup traits. `default_example` owns the pure default-acquisition routing predicate. `official_starter` owns exact official-repository classification and transform ordering behind typed package/document providers. `package_manager_transform` owns the no-op decision and typed conversion request while leaving mutations behind `PackageManagerConverter`.

The `.gitignore` transform never performs a separate “does not exist, then overwrite-capable write” sequence. It writes the constant to a newly created sibling temporary file, synchronizes it, revalidates the root, and publishes it with a no-overwrite hard link. A concurrent destination wins and is never overwritten.

The Git initialization core never resolves an executable, inherits no process environment by itself, and performs no filesystem deletion directly. Those side effects remain behind explicit provider boundaries so they can be reviewed and tested independently.

The default-example predicate cannot broaden routing through trimming, normalization, regexes, prefixes, suffixes, or mutable collection state. It is intentionally a two-literal borrowed-string match.

The official-starter core cannot open, delete, parse, serialize, or replace a path directly. Its provider boundary makes metadata best-effort behavior, package read/write failures, JSON truthiness, and deterministic serialization independently reviewable instead of silently inheriting broad `fs-extra` behavior.

The package-manager transform does not receive free-form manager text at its mutation boundary. It also does not copy, log, or forward the prompt version. That preserves the source decision contract while keeping destructive conversion outside the reviewed core.

## TDD history

```text
README RED:              a0930bc5bd0eee5bc7c6edf09daf8caf38875781
README GREEN:            0af47426b5ef00bbff6dfc7d60aaca23daa71720
.gitignore RED:          f8edbb984cd7255f1d7630689384324009de5ac4
.gitignore GREEN:        c74d664d718691660be969d779d25a76af31fb3e
Git init RED import:     e57cc31afd1d83a015ae49136d71c7daa3217fb7
Git oracle/security:     221586118db79fca2f94cebb15785de4111bde8e
Git init GREEN:          1d7b485d597b70f40bb4aa492f45d1c0638f844e
Default example RED:     edc3b96b106e2c0bebaee299690c7769f9ba6bc2
Default example GREEN:   57f19c56209312fb2d04423fdd86ad239150a753
Package manager RED:     9f9b33f889d92e5b61a484ac445b4e297110f6f0
Package manager GREEN:   c7a1776c5f6fa53db4e30d418a9897b56c6263cd
Official starter RED:   2ca25bd457cbe216f345b5f67cf9ac32f43a2c7a
Official starter GREEN: cd2ba74b3040e654a63c9799e42c35a12f2c4dbc
```

Focused validation:

```sh
cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
pnpm --filter create-turbo test
```

The crate contains 55 translated parity tests and 39 security regression tests, for 94 authored focused Rust tests. The latest tranche is not treated as validated until its merge-head workflow passes the commands above.

## Production status

This is an in-progress migration core, not a production cutover. No TypeScript source is deleted by these tranches. `PARITY_MATRIX.md`, `SECURITY.md`, and `docs/typescript-deprecation.md` contain the exact remaining closure requirements.
