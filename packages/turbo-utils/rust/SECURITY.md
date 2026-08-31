# Security review

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

This document records observed trust-boundary defects, hardened compatibility decisions, and remaining security blockers in the migrated `@turbo/utils` surfaces. It is not an assertion that the rest of the repository has been exhaustively audited.

## TU-001: Ancestor-search target traversal

The TypeScript `searchUp` joins an arbitrary target below each ancestor. Parent components can escape the ancestor being searched. Rust accepts only a non-empty relative target without parent, root, or platform-prefix components.

## TU-002: Unbounded content predicates

TypeScript reads each candidate fully before invoking `contentCheck`. Rust inspects only regular files up to 4 MiB and treats larger or unreadable candidates as non-matches.

## TU-003: Symlinked project roots

The current TypeScript `lstat` behavior rejects a symlink because it is not reported as a directory. Rust preserves and tests that result explicitly so later refactors cannot silently begin following project-root symlinks.

## TU-004: Metadata and read uncertainty

TypeScript can throw for permission and I/O failures even though callers generally expect a validation result. Rust treats uncertain metadata or enumeration as invalid and does not continue creation against a path it could not inspect.

## TU-005: Cross-operation TOCTOU remains

Validation and later project creation are separate operations. Another process can replace entries between them. The production implementation must use descriptor-relative no-follow operations where supported, or an isolated private staging directory followed by atomic promotion.

## TU-006: Platform writability differences

Unix uses `access(W_OK)`, which closely matches Node's effective access check. The non-Unix fallback uses metadata and the readonly flag and is not complete ACL parity. Windows cutover remains blocked pending native access checks and dedicated tests.

## TU-007: Package-manager executable substitution

**Boundary:** `PATH` and executables named `yarnpkg`, `yarn`, `npm`, `pnpm`, `bun`, `nub`, or `aube`.

TypeScript executes bare names and invokes a separate bare `which` process for Nub and Aube. A repository or deployment environment that prepends a directory to `PATH` can substitute code during detection.

Rust accepts one normal executable name, scans only absolute `PATH` entries, canonicalizes the selected regular file, rejects files resolving inside the inspected project root, and invokes an argument vector without a shell. Nub and Aube are resolved directly.

**Residual risk:** a writable absolute directory already trusted by the host remains a substitution boundary. Production hosts should supply provisioned or allow-listed executable paths.

**Regression:** `resolver_skips_relative_and_project_local_path_entries`.

## TU-008: Unbounded package-manager output

TypeScript sets a five-second timeout but no explicit stdout/stderr limit for these probes. Rust limits each stream to 1 MiB and treats overflow as an unavailable manager. Both streams are drained concurrently.

**Regression:** `command_output_is_bounded`.

## TU-009: Process-tree cleanup

Rust creates a Unix process group and terminates the group before killing and waiting for the direct child on timeout or output overflow.

**Residual risk:** Windows needs a kill-on-close Job Object and integration tests before cutover.

**Regression:** `command_execution_has_a_deadline`.

## TU-010: Project metadata symlinks and resource exhaustion

TypeScript reads `package.json` and `.yarnrc.yml` without an explicit limit and follows symlinks. Rust accepts only non-symlink regular files no larger than 1 MiB. Malformed, oversized, missing, or unsafe metadata is treated as unavailable. Custom Yarn paths are never executed.

**Regressions:** `symlinked_package_metadata_is_not_followed`, `oversized_package_metadata_is_not_parsed`, and `custom_yarn_path_is_never_executed`.

## TU-011: Windows command-shim boundary

Windows package managers commonly use `.cmd` shims. Safely invoking such scripts requires an explicit reviewed adapter rather than implicit `cmd.exe` parsing. The hardened runner resolves `.exe` and `.com` only, so `.cmd`/`.bat` parity is intentionally blocked.

## TU-012: Hostname-only repository URL policy

TypeScript accepts a parsed URL whenever `hostname === "github.com"`, including non-HTTPS schemes, credentials, and explicit ports. Rust accepts credential-free HTTPS URLs whose authority is exactly `github.com`, case-insensitively, with no explicit port or control/whitespace characters.

**Regression:** `github_url_validation_rejects_scheme_host_credential_and_port_confusion`.

## TU-013: Unvalidated examples and repository subpaths

TypeScript does not define one strict grammar for named examples and explicit repository subpaths. Rust restricts named examples to ASCII letters, digits, hyphens, and underscores. Repository subpaths reject backslashes, percent signs, URL delimiters, controls, empty segments, and `.` or `..` components before any provider call.

**Regressions:** `unsafe_named_example_is_rejected_before_any_source_operation` and `unsafe_repository_subpath_is_rejected_before_network_resolution`.

## TU-014: Process-wide current-directory mutation

TypeScript calls `process.chdir(root)` and does not restore it before returning. This mutates global process state and makes concurrent relative I/O depend on call order. Rust never changes the process-wide current directory; the resolved destination is passed explicitly to providers and inspection code.

## TU-015: Generated `package.json` boundary

TypeScript checks existence and calls `readJsonSync` without a size limit or no-follow open. Rust preserves the observable presence result but extracts scripts only from a regular non-symlink file no larger than 1 MiB. Unix opens use `O_NOFOLLOW` and `O_CLOEXEC`.

**Residual risk:** non-Unix final-component no-follow behavior still needs a native handle implementation.

**Regressions:** `symlinked_package_json_is_not_read` and `oversized_package_json_is_not_parsed`.

## TU-016: Destination replacement and extraction TOCTOU

Rust rejects symlink/non-directory targets and immediate-parent symlinks, then revalidates the destination before and after every provider attempt. This narrows but does not eliminate replacement races. The production provider must extract into an isolated directory and atomically promote a complete tree, or use descriptor-relative no-follow operations.

**Regressions:** `conflicting_target_is_rejected_before_download` and `symlinked_project_root_is_never_followed`.

## TU-017: Retry over partially written destinations

The TypeScript `async-retry({ retries: 3 })` contract can perform four attempts in one destination, allowing failed partial state to affect later attempts. Rust preserves four attempts in the coordinator for parity, but the production provider must isolate and clean every attempt before atomic promotion.

**Regressions:** `retries_three_times_and_succeeds_on_the_fourth_attempt` and `stops_after_four_failed_download_attempts`.

## TU-018: Inconsistent archive safeguards

One TypeScript extraction path explicitly rejects unsafe paths and links, while another uses a prefix filter without exposing the same complete entry contract. This review does not claim a confirmed traversal in the underlying tar library. It records that one proven policy is missing.

The Rust provider must use one extractor with entry-count, per-file, total-size, path-depth, link, device-node, permission, timeout, cleanup, and decompression-ratio limits, with tests for every rejected archive type.

## TU-019: Update-notification terminal and bidi spoofing

**Boundary:** package name, dynamic/static upgrade command, and debug error text rendered into terminal or CI logs.

TypeScript passes these values to the logger without a uniform control-character or length policy. Newlines, escape sequences, and Unicode bidirectional formatting characters can forge or visually reorder log content. Very large values can create oversized logs.

Rust escapes C0/C1 controls, escape characters, Arabic letter mark, left/right marks, embedding/override controls, isolate controls, and byte-order mark before rendering. Each untrusted field is limited to 1,024 Unicode scalar values and receives an ellipsis when truncated. Safe printable text and TypeScript message ordering remain unchanged.

**Intentional incompatibility:** unsafe control and directionality characters are rendered as visible escapes instead of verbatim text.

**Regressions:** `terminal_controls_are_escaped_in_package_name_and_command`, `unicode_directionality_controls_are_escaped_before_rendering`, `dynamic_error_controls_are_escaped_before_debug_logging`, and `rendered_untrusted_fields_are_bounded`.

## TU-020: Production update checker is not yet security-closed

`PreparedUpdateNotification` accepts an injected `UpdateChecker`; it does not yet perform registry/network I/O. This prevents an unreviewed client from becoming the production path.

The eventual provider must define and test HTTPS/TLS policy, proxy handling, redirects, DNS and destination policy, connect/read/total deadlines, response-size and JSON-depth bounds, registry authentication and redaction, cache integrity, rate limits, cancellation, and fail-silent compatibility. It must not construct executable package specifications from registry data.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and package/advisory index.
- GitHub Advisory Database.
- Official upstream security notices and release information for direct dependencies and externally executed tools changed by these tranches.

The project coordinator and update-notification core add no new Rust dependency. They use the standard library plus existing workspace-managed dependencies.

The resolved `unsafe-libyaml` version observed in the lockfile is `0.2.11`, above the `0.2.10` patched floor for `RUSTSEC-2023-0075`. Long-term maintenance and replacement policy for the YAML parsing chain remains open and must not be conflated with a direct vulnerability in the resolved version.

The lookup found `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33` for `webbrowser`: affected `webbrowser::open` versions include releases through `1.2.1`, with `1.2.2` or later patched. The workspace currently declares `webbrowser = "0.8.7"`. The observed `turborepo-query` call opens the constant HTTP URL `http://localhost:8000`, which limits reachability of the advisory's attacker-controlled non-HTTP(S) vector at that call site, but the dependency must still be upgraded or removed before migration merge.

Migration CI runs a complete lockfile audit and temporarily ignores only `RUSTSEC-2026-0257`, which is separately tracked in `docs/rust-migration-security-findings.md`. The exception exists so this documented pre-existing blocker does not hide newly introduced advisories. It must be removed in the same change that upgrades or removes `webbrowser`.
