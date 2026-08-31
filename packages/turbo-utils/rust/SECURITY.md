# Security review

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

This document records observed trust-boundary defects and hardening gaps in the migrated `@turbo/utils` surfaces. It is not an assertion that the rest of the repository has been exhaustively audited.

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

## TU-012: Hostname-only repository URL policy

**Boundary:** the `example` value accepted by `createProject`.

The TypeScript function constructs a `URL` and accepts it whenever `url.hostname === "github.com"`. That admits non-HTTPS schemes, credential-bearing URLs, and explicit ports. Some of those forms are not an immediate SSRF because the downstream helpers construct fixed GitHub API/codeload origins, but they create an unnecessarily broad and inconsistent trust contract and can leak credentials through logs or future refactors.

Rust accepts only credential-free HTTPS URLs whose authority is exactly `github.com`, case-insensitively, with no explicit port and no control or whitespace characters. Query and fragment text may be present but do not participate in repository-path selection.

**Regression:** `github_url_validation_rejects_scheme_host_credential_and_port_confusion`.

## TU-013: Unvalidated named examples and repository subpaths

The TypeScript non-URL path calls `existsInRepo` with an encoded example name, while `examplePath` is only stripped of one leading slash before it is used in repository-path construction. Slash, parent, encoded-delimiter, and URL-delimiter handling is not defined as a security contract at this layer.

Rust restricts named examples to ASCII letters, digits, hyphens, and underscores. Explicit repository subpaths reject backslashes, percent signs, URL delimiters, controls, empty segments, and `.`/`..` components before any provider or network operation.

**Regressions:** `unsafe_named_example_is_rejected_before_any_source_operation` and `unsafe_repository_subpath_is_rejected_before_network_resolution`.

## TU-014: Process-wide current-directory mutation

`createProject` stores `process.cwd()` and then calls `process.chdir(root)` without restoring it before returning. This mutates global process state, affects concurrent work, and makes later relative filesystem and subprocess operations depend on call order. A failed download also leaves the process in the project directory until an outer caller repairs it.

Rust never changes the process-wide working directory. The resolved destination is passed explicitly to the `ProjectSource` provider and is used explicitly for post-download inspection.

The compatibility result still computes `cdPath` from the original directory and application path.

## TU-015: Generated `package.json` symlink and allocation boundary

After download, the TypeScript function checks `existsSync` and calls `readJsonSync` without an explicit size limit or no-follow open. A generated `package.json` can therefore point outside the project through a symlink or consume excessive memory. Parse errors are swallowed, so the unsafe read can occur even though the final result contains no scripts.

Rust preserves the observable presence result but extracts scripts only from a regular, non-symlink file no larger than 1 MiB. The read itself is capped at one MiB plus one byte. Unix opens use `O_NOFOLLOW` and `O_CLOEXEC` to close the check/open symlink race for the final component.

**Residual risk:** the non-Unix open still has a check/open race until a platform-native no-follow handle implementation is added.

**Regressions:** `symlinked_package_json_is_not_read` and `oversized_package_json_is_not_parsed`.

## TU-016: Destination replacement and extraction TOCTOU

The TypeScript flow checks writability and emptiness before downloading, but another process can replace the target between those checks and extraction. The Rust coordinator rejects a symlink or non-directory target and immediate parent, then revalidates the target before and after every provider attempt.

This narrows the race but does not close it. The production archive provider must perform descriptor-relative, no-follow creation or extract into a private staging directory and atomically promote the completed tree. That provider is intentionally not implemented in this tranche.

**Regressions:** `conflicting_target_is_rejected_before_download` and `symlinked_project_root_is_never_followed`.

## TU-017: Retry over partially written destinations

The TypeScript `async-retry({ retries: 3 })` contract performs up to four attempts in the same destination. A failed attempt may leave partial files that influence a later attempt, and the successful return does not prove the tree came from one coherent archive operation.

Rust preserves the four-attempt behavior in the coordinator for parity. The production provider remains blocked until each attempt stages into an isolated directory, applies archive limits and link/path checks, and promotes only a complete result. The provider must also clean failed staging directories deterministically.

**Regressions:** `retries_three_times_and_succeeds_on_the_fourth_attempt` and `stops_after_four_failed_download_attempts`.

## TU-018: Repository extraction paths have different explicit safeguards

`streamingExtract` explicitly calls `isPathSafe`, rejects symbolic and hard links, tracks writes, and enforces a download timeout. `downloadAndExtractRepo` uses `tar.extract` with a prefix filter but does not apply those same explicit entry checks in its own code path.

The underlying tar library may reject some unsafe paths by default, so this review does not claim a confirmed traversal exploit. It records that the two acquisition paths do not expose one proven extraction contract. The Rust production provider must use one shared extractor with entry-count, per-file, total-size, path-depth, link, device-node, permission, and timeout limits, plus tests for every rejected archive type.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and its package/advisory index.
- GitHub Advisory Database and official global-advisory API documentation.
- Official upstream repository/security information for existing direct dependencies used by `turbo-utils-rs`.

The project-creation coordinator adds **no new Rust dependency**. It uses the standard library plus the crate's existing `serde_json`, `thiserror`, and Unix `libc` dependencies.

The lookup confirmed that the resolved `unsafe-libyaml` version is `0.2.11`, which is above the patched floor for `RUSTSEC-2023-0075`; the existing maintenance concern recorded as `RUSTSEC-2025-0068` still needs policy disposition before production cutover.

The lookup also found `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33` for `webbrowser`: affected functions include `webbrowser::open` through version `1.2.1`, with `1.2.2` or later patched. The workspace currently declares `webbrowser = "0.8.7"`. The observed call in `turborepo-query` opens the constant HTTP URL `http://localhost:8000`, so the reviewed call site does not pass attacker-controlled non-HTTP(S) text described by the advisory. The dependency must still be upgraded or otherwise removed because future call sites and environment behavior can change. This repository-level finding is indexed in `docs/rust-migration-security-findings.md`.

A manual package-name search is not equivalent to auditing the complete resolved graph. `cargo audit` or an equivalent RustSec/OSV lockfile scan remains a required CI and review gate. Any finding must be recorded with the affected path, reachability, disposition, and remediation before production cutover.
