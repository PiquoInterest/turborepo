# Rust migration security findings

This is the repository-level index for security findings discovered while moving executable TypeScript logic to Rust. Package-specific details, regression tests, residual risks, and intentional incompatibilities remain in each migration crate's `SECURITY.md`.

This index is evidence-based but not exhaustive. A component is not considered audited merely because it appears here.

## Package reviews

- [`packages/turbo-ignore/rust/SECURITY.md`](../packages/turbo-ignore/rust/SECURITY.md)
- [`packages/turbo-utils/rust/SECURITY.md`](../packages/turbo-utils/rust/SECURITY.md)
- [`packages/create-turbo/rust/SECURITY.md`](../packages/create-turbo/rust/SECURITY.md)

## Repository findings

### RF-001: Affected `webbrowser` dependency remains in the workspace

**Status:** Open, upgrade required before merge.

The workspace declares `webbrowser = "0.8.7"`. RustSec advisory `RUSTSEC-2026-0257`, also published as `GHSA-2ph8-5cr8-hr33`, affects relevant browser-opening APIs through version `1.2.1`; version `1.2.2` or later is patched.

The observed repository call currently uses a constant HTTP URL, which limits current reachability, but the dependency remains in an affected range. Required closure is to upgrade or remove it, run the affected query tests/lints, retain an HTTP(S)-only input policy, and remove the temporary migration-CI advisory exception.

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

## Required repository gates

Before declaring repository-wide TypeScript deprecation complete:

- keep lockfile-wide RustSec auditing enabled and remove every temporary exception after remediation;
- run npm advisory and provenance checks for retained host adapters;
- execute differential fixtures on Linux, macOS, and Windows;
- prove that published artifacts do not load executable TypeScript at runtime;
- retain minimal JavaScript host shims only where host APIs require them, with business logic behind reviewed Rust/native/WASM boundaries.
