# Security review

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

## TU-001: Ancestor-search target traversal

The TypeScript `searchUp` accepts an arbitrary target and joins it beneath every ancestor. Parent components can make each probe escape the ancestor being searched. The Rust API accepts only a non-empty relative target without parent, root, or platform-prefix components.

## TU-002: Unbounded content predicates

`searchUp` reads every matching candidate fully before running `contentCheck`. A large file can consume deployment-process memory. Rust content checks only read regular files up to 4 MiB and treat larger or unreadable candidates as non-matches.

## TU-003: Symlinked project roots

The TypeScript directory validator uses `lstat`, which currently rejects a symlink because it is not reported as a directory. Rust preserves and tests this behavior explicitly so later refactors cannot silently begin following project-root symlinks.

## TU-004: Metadata/read uncertainty

The TypeScript function can throw for permission and I/O failures even though callers generally expect a validation result. Rust treats uncertain metadata and directory enumeration as invalid, preventing creation logic from continuing against a path it could not inspect.

## TU-005: Time-of-check/time-of-use remains

Directory validation and later project creation are separate operations. Another process can replace entries after validation. The final migration must use descriptor-relative creation and no-follow semantics where supported. This tranche documents but does not eliminate that cross-operation race.

## TU-006: Platform writability differences

On Unix, Rust uses `access(W_OK)`, matching Node's effective access check closely. The non-Unix fallback uses directory metadata and the readonly flag, which is not complete ACL parity. Windows cutover remains blocked until native access checks and dedicated parity tests are added.

## TU-007: Package-manager executable substitution

**Boundary:** `PATH` and every executable named `yarnpkg`, `yarn`, `npm`, `pnpm`, `bun`, `nub`, or `aube`.

The TypeScript implementation passes bare executable names to `execa`, and invokes a separate bare `which` process for Nub and Aube. A repository, wrapper, or deployment environment that can prepend a directory to `PATH` can substitute code that runs during package-manager detection.

The Rust system runner accepts only a single normal executable name, scans only absolute `PATH` entries, canonicalizes the selected file, rejects files that resolve inside the inspected project root, and invokes it with an argument vector rather than a shell. Nub and Aube paths are resolved directly, so no separate `which` binary is executed.

**Residual risk:** an attacker who controls a writable absolute directory already trusted in `PATH` can still replace an executable. Production packaging should prefer explicitly provisioned tool paths or an allow-listed resolver where the host supplies one.

**Regression:** `resolver_skips_relative_and_project_local_path_entries`.

## TU-008: Unbounded package-manager output

The TypeScript code sets a five-second timeout but does not set an explicit stdout/stderr bound for these probes. A command can emit substantial output before it exits or is terminated.

Rust limits each stream to 1 MiB and returns an unavailable-manager result when the limit is exceeded. Readers run concurrently so a full stderr pipe cannot deadlock stdout collection.

**Regression:** `command_output_is_bounded`.

## TU-009: Process-tree cleanup

A timeout must not leave a package-manager descendant running after detection returns. Rust creates a new Unix process group and sends `SIGKILL` to the group before killing and waiting for the direct child.

**Residual risk:** Windows process-tree cleanup is not yet equivalent to a Job Object. Windows production cutover is blocked until the native runner assigns the child to a kill-on-close Job Object and exercises it in integration tests.

**Regression:** `command_execution_has_a_deadline`.

## TU-010: Project metadata symlinks and resource exhaustion

The TypeScript helper reads `package.json` and `.yarnrc.yml` without an explicit size limit and follows symlinks. This can read attacker-selected files or allocate excessive memory during detection.

Rust accepts only non-symlink regular files of at most 1 MiB. Malformed, oversized, missing, or unsafe metadata is treated as unavailable. A custom Yarn path remains a configuration marker but is never executed.

**Regressions:** `symlinked_package_metadata_is_not_followed`, `oversized_package_metadata_is_not_parsed`, and `custom_yarn_path_is_never_executed`.

## TU-011: Windows command-shim boundary

Windows package managers are commonly exposed through `.cmd` shims. Executing a command script safely without reintroducing shell parsing requires a separately reviewed Windows adapter. The current hardened runner resolves `.exe` and `.com` files only, so Windows `.cmd`/`.bat` parity is intentionally blocked rather than implemented through `cmd.exe` implicitly.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and its package/advisory index.
- GitHub Advisory Database, which imports RustSec data and exposes GitHub-reviewed advisories.
- Official upstream repository/security information for existing direct dependencies used by `turbo-utils-rs`.

This package-manager tranche adds **no new Rust dependency**. The manual lookup found no advisory specific to the new standard-library-only manager code. It did identify the existing YAML-stack maintenance boundary: `serde_yaml_ng` is presented as a maintained `serde_yaml` fork but still uses `unsafe-libyaml`; `RUSTSEC-2023-0075` affects `unsafe-libyaml` versions before `0.2.10`, and RustSec's `RUSTSEC-2025-0068` notes the maintenance concern. The committed lockfile version must be checked by an automated full dependency audit before merge.

A manual package-name search is not equivalent to auditing the complete resolved graph. `cargo audit` or an equivalent RustSec/OSV lockfile scan was unavailable in the connected local environment and remains a required CI/review gate. Any finding must be recorded here with the affected path, disposition, and remediation before production cutover.
