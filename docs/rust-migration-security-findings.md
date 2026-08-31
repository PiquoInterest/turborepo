# Rust migration security findings

This is the repository-level index for security findings discovered while moving executable TypeScript logic to Rust. Package-specific details, regression tests, residual risks, and intentional incompatibilities remain in each migration crate's `SECURITY.md`.

This index is evidence-based but not exhaustive. A component is not considered audited merely because it appears here.

## Package reviews

- [`packages/turbo-ignore/rust/SECURITY.md`](../packages/turbo-ignore/rust/SECURITY.md)
- [`packages/turbo-utils/rust/SECURITY.md`](../packages/turbo-utils/rust/SECURITY.md)
- [`packages/create-turbo/rust/SECURITY.md`](../packages/create-turbo/rust/SECURITY.md)

## Repository findings

### RF-001: Affected `webbrowser` dependency remains in the workspace

**Status:** Open, upgrade or removal required before integration merge.

The workspace declares `webbrowser = "0.8.7"` and currently resolves an affected `0.8.x` release. RustSec advisory `RUSTSEC-2026-0257`, also published as `GHSA-2ph8-5cr8-hr33`, affects relevant browser-opening APIs through version `1.2.1`; version `1.2.2` or later is patched.

The observed repository call uses a constant HTTP URL, which limits current reachability, but the dependency remains in an affected range. Required closure is to upgrade or remove it, run the affected query tests/lints, retain an HTTP(S)-only input policy, and remove the existing temporary migration-CI exception. No additional advisory exception may be added to make the integration branch appear green.

References:

- <https://rustsec.org/advisories/RUSTSEC-2026-0257.html>
- <https://github.com/advisories/GHSA-2ph8-5cr8-hr33>

### RF-002: Project archive paths do not share one proven extraction contract

**Status:** Production cutover blocked.

The TypeScript archive paths apply different explicit checks. A production Rust provider must use one extractor with tested limits for entry count, file size, total expansion, decompression ratio, depth, links, device nodes, permissions, time, redirects, staging, and cleanup.

### RF-003: TypeScript project creation mutates process-wide working-directory state

**Status:** Fixed in the Rust coordinator; TypeScript production path remains.

The TypeScript coordinator calls `process.chdir(root)` without restoration. The Rust core passes absolute destinations and does not mutate global current-directory state.

### RF-004: Decision metadata has symlink and resource-exhaustion boundaries

**Status:** Fixed in current Rust cores; TypeScript production paths remain.

Rust migration cores use bounded regular-file reads, reject or ignore symlinked decision metadata, and fail conservatively where uncertainty changes a build or generation decision.

### RF-005: CLI notification text can spoof terminal output

**Status:** Fixed in the Rust notification core; TypeScript production path remains.

Rust escapes terminal controls and Unicode directionality controls and bounds each untrusted field while preserving safe printable message order and exit behavior.

### RF-006: README transforms follow links, process unbounded content, and write in place

**Status:** Fixed in the `create-turbo` Rust transform core; TypeScript production path remains.

Rust limits input to 4 MiB, scans linearly, rejects malformed UTF-8 and symlinked roots/files, checks Unix file identity, writes through a synchronized sibling temporary file, preserves mode bits, and replaces only after revalidation.

Windows atomic replacement, complete metadata/ACL preservation, and descriptor-relative concurrent-path handling remain open.

### RF-007: `.gitignore` check/write race and broken-link following

**Status:** Fixed in the `create-turbo` Rust transform core; TypeScript production path remains.

The TypeScript transform checks `existsSync` and then performs an overwrite-capable write. A destination can appear between those operations. More seriously, `existsSync` reports a broken `.gitignore` symlink as absent, after which the write follows the link and can create an external target.

The Rust core:

- rejects symlinked roots and broken or existing `.gitignore` symlinks;
- writes the exact default bytes to a newly created sibling file;
- synchronizes the file before publication;
- publishes through `hard_link`, which never overwrites an existing destination;
- treats a concurrent regular destination as `not-applicable`;
- bounds temporary-name retries and cleans up ordinary failure paths.

Regression tests prove that external symlink targets are not created or modified and customer-owned existing content is not overwritten.

Residual risk: path-based standard-library APIs cannot close every malicious root-component exchange. Descriptor-relative Unix operations and reviewed Windows handle-based publication remain required before production cutover in attacker-writable roots.

### RF-008: TypeScript Git root validation models shell injection instead of path safety

**Status:** Fixed in the injected Rust orchestration core; production provider remains blocked.

The TypeScript `tryGitInit` path rejects `$`, `#`, `;`, and `!` even though it passes an argument vector to `spawnSync` rather than a shell string. It does not reject relative roots, filesystem roots, parent components, controls, or all filename characters invalid on Windows.

The Rust core requires an absolute non-root path, rejects current/parent components, controls, and Windows-invalid filename characters, and permits harmless shell metacharacters because no shell is used. It also carries the root as `PathBuf`, preserving non-UTF-8 Unix paths.

Regression coverage is in `packages/create-turbo/rust/tests/git_init_security.rs`.

### RF-009: Git initialization can inherit executable, template, configuration, and hook execution

**Status:** Production cutover blocked; no production runner is implemented.

The TypeScript code resolves `git` and `hg` by command name and inherits the caller environment and VCS configuration. Git documents that `git init` may copy templates selected by environment or configuration and that `git commit` may execute commit-related hooks. A production Rust provider must therefore prove canonical executable resolution, an explicit environment/configuration policy, no shell, deadlines, bounded output, descendant cleanup, and deliberate hook/template behavior.

The Rust tranche keeps these effects behind `VcsRunner`. The integration branch must not treat the injected orchestration tests as evidence that process execution is production-safe.

References:

- <https://git-scm.com/docs/git-init>
- <https://git-scm.com/docs/git-commit>
- <https://git-scm.com/docs/githooks>

### RF-010: Recursive `.git` cleanup lacks a proven no-follow ownership contract

**Status:** Production cutover blocked; no production cleaner is implemented.

A naive recursive delete can cross links or Windows reparse points, race with path replacement, or remove a repository the current operation did not create. The Rust orchestration core requests cleanup only after `git init` returned success and a later command failed. It does not request cleanup after an ambiguous init failure.

A production `GitDirectoryCleaner` must prove root identity, `.git` ownership, no-follow traversal, bounded work, cleanup failure behavior, and supported Windows semantics.

### RF-011: `h2 0.4.5` is affected by unbounded empty DATA frame handling

**Status:** Open, patched lockfile version required.

The current audited lockfile contains `h2 0.4.5`. `RUSTSEC-2026-0258` / `GHSA-q83h-524g-xf6h` describes unbounded queuing of empty HTTP/2 DATA frames and is patched in `h2 0.4.16`.

A temporary dependency-refresh workflow proved that Cargo can select `h2 0.4.16`, but the workflow did not commit any lockfile because a later `quick-xml` constraint failed and validation was intentionally fail-closed. The permanent remediation must update the lockfile in a reviewed branch and pass all affected workspace checks before integration.

References:

- <https://rustsec.org/advisories/RUSTSEC-2026-0258.html>
- <https://github.com/advisories/GHSA-q83h-524g-xf6h>

### RF-012: `quick-xml 0.38.4` has two denial-of-service advisories

**Status:** Open, direct dependency-chain change required.

The current audited lockfile contains `quick-xml 0.38.4`, affected by:

- `RUSTSEC-2026-0194`: quadratic duplicate-attribute checking;
- `RUSTSEC-2026-0195`: unbounded namespace-declaration allocation.

Both are patched in `quick-xml 0.41.0` or later. A precise lockfile update was rejected because `opendal 0.55.0` requires `quick-xml ^0.38`. The observed reverse chain is `quick-xml -> opendal -> sccache -> turbo` and the `turborepo-sccache-proxy` development path.

Required closure is to update or remove the constraining `opendal`/`sccache` dependency path, then regenerate the lockfile and run the affected workspace build, test, lint, and audit gates. An advisory ignore is not an acceptable substitute.

References:

- <https://rustsec.org/advisories/RUSTSEC-2026-0194.html>
- <https://rustsec.org/advisories/RUSTSEC-2026-0195.html>

### RF-013: Default-example acquisition routing must remain exact

**Status:** Fixed in the Rust predicate core; production caller remains TypeScript.

`create-turbo` passes `isDefaultExample(exampleName)` into project acquisition. Broadening that predicate through trimming, case folding, Unicode normalization, prefixes, suffixes, path matching, or fuzzy matching could classify attacker-controlled names as built-in defaults.

The Rust core exports the exact source-order values `basic` and `default` and matches only those two borrowed ASCII strings. Regression tests reject case variants, whitespace, controls, NUL, prefixes, suffixes, path-like values, Unicode confusables, normalization variants, joiners, and a 4 MiB arbitrary input.

Required closure is to bind the Rust predicate into production acquisition orchestration and run shared TypeScript/Rust routing fixtures before the TypeScript helper is removed.

## Required repository gates

Before declaring repository-wide TypeScript deprecation complete:

- keep lockfile-wide RustSec auditing enabled and remove every temporary exception after remediation;
- resolve `webbrowser`, `h2`, and `quick-xml` rather than suppressing them;
- run npm advisory and provenance checks for retained host adapters;
- execute differential fixtures on Linux, macOS, and Windows;
- prove that published artifacts do not load executable TypeScript at runtime;
- retain minimal JavaScript host shims only where host APIs require them, with business logic behind reviewed Rust/native/WASM boundaries.
