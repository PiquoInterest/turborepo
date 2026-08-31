# Rust migration security findings

This is the repository-level index for security findings discovered while moving executable TypeScript logic to Rust. Package-specific details, regression tests, residual risks, and intentional incompatibilities remain in each migration crate's `SECURITY.md`.

This index is evidence-based but not exhaustive. A component is not considered audited merely because it appears here.

## Package reviews

- [`packages/turbo-ignore/rust/SECURITY.md`](../packages/turbo-ignore/rust/SECURITY.md)
- [`packages/turbo-utils/rust/SECURITY.md`](../packages/turbo-utils/rust/SECURITY.md)

## Repository findings

### RF-001: Affected `webbrowser` dependency remains in the workspace

**Status:** Open, upgrade required before merge.

The workspace declares `webbrowser = "0.8.7"`. RustSec advisory `RUSTSEC-2026-0257`, also published as `GHSA-2ph8-5cr8-hr33`, affects `webbrowser::open`, `open_browser`, and `open_browser_with_options` through version `1.2.1`; version `1.2.2` or later is patched.

Observed repository usage is currently:

```rust
webbrowser::open("http://localhost:8000")?;
```

That call uses a constant HTTP URL, so the reviewed path does not supply the attacker-controlled non-HTTP(S) text required by the advisory. The dependency is still inside the affected range and must be upgraded or removed so a later call site cannot silently become reachable.

Required closure:

1. update the workspace requirement and lockfile to `webbrowser >= 1.2.2`;
2. run the `turborepo-query` test and lint targets on Unix;
3. keep browser-launch inputs restricted to explicit HTTP(S) URLs;
4. remove the temporary `RUSTSEC-2026-0257` exception from migration CI.

Migration CI now audits the complete resolved Rust dependency graph. It temporarily ignores only this documented advisory so additional findings still fail the gate.

References:

- <https://rustsec.org/advisories/RUSTSEC-2026-0257.html>
- <https://github.com/advisories/GHSA-2ph8-5cr8-hr33>

### RF-002: Project archive paths do not yet share one proven extraction contract

**Status:** Production cutover blocked.

The TypeScript `streamingExtract` path explicitly performs destination containment checks and rejects symbolic and hard links. The `downloadAndExtractRepo` path uses a separate extraction flow and does not apply those same explicit checks in its own code. Library defaults may mitigate some archive classes, so this is recorded as an unclosed security contract rather than a confirmed exploit.

The Rust `createProject` coordinator intentionally leaves network and archive acquisition behind `ProjectSource`. A production provider must use a single extractor with tested limits for entry count, individual file size, total expanded bytes, decompression ratio, path depth, links, device nodes, permissions, time, redirects, staging, and cleanup.

### RF-003: TypeScript project creation mutates process-wide working-directory state

**Status:** Fixed in the Rust coordinator; TypeScript production path remains.

`createProject` calls `process.chdir(root)` and does not restore the original directory before return. Concurrent operations and later relative paths can therefore observe state from a previous project creation. The Rust coordinator passes absolute destinations explicitly and never changes process-wide current-directory state.

Production risk remains until callers use the Rust implementation and the TypeScript runtime path is removed.

### RF-004: Decision metadata has symlink and resource-exhaustion boundaries

**Status:** Fixed in current Rust cores; TypeScript production paths remain.

Observed TypeScript helpers read decision-critical project metadata without consistent no-follow and explicit-size policies. The Rust migration cores use bounded regular-file reads, reject or ignore symlinked metadata, and fail conservatively where uncertainty could change build or project-generation decisions.

The detailed instances and regression names are recorded in the package security reviews.

### RF-005: CLI notification text can spoof terminal output

**Status:** Fixed in the Rust notification core; TypeScript production path remains.

The TypeScript update notification renders package names, upgrade commands, and debug error values without a uniform control-character or length policy. The Rust core escapes terminal controls and Unicode directionality controls and bounds each untrusted field. Safe printable values retain the same message ordering and exit behavior.

Production risk remains until the Rust notification core is bound into the affected CLIs and the TypeScript path is removed.

## Required repository gates

Before declaring repository-wide TypeScript deprecation complete:

- keep the lockfile-wide RustSec audit enabled and remove every temporary advisory exception after remediation;
- run npm ecosystem advisory and provenance checks for retained host adapters;
- execute differential fixtures on Linux, macOS, and Windows;
- prove that published artifacts do not load executable TypeScript at runtime;
- retain minimal JavaScript host shims only where the host API requires JavaScript, with all business logic behind reviewed Rust/native/WASM boundaries.
