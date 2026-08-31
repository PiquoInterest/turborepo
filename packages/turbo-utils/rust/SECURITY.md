# Security review

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

This document records observed trust-boundary defects, hardened compatibility decisions, and remaining blockers in migrated `@turbo/utils` surfaces. It is not an exhaustive repository audit.

## TU-001: Ancestor-search target traversal

TypeScript joins an arbitrary `searchUp` target under each ancestor, so parent components can escape the searched directory. Rust accepts only a non-empty relative target without parent, root, or platform-prefix components.

## TU-002: Unbounded content predicates

TypeScript reads each candidate fully before `contentCheck`. Rust inspects only regular files up to 4 MiB and treats larger or unreadable candidates as non-matches.

## TU-003: Symlinked project roots

Current TypeScript `lstat` behavior rejects project-root symlinks. Rust preserves and tests this explicitly so later refactors cannot silently follow them.

## TU-004: Metadata and read uncertainty

TypeScript can throw on permission and I/O failures. Rust treats uncertain metadata or enumeration as invalid and does not continue creation against an uninspected path.

## TU-005: Cross-operation TOCTOU remains

Validation and project creation are separate operations. Production code must use descriptor-relative no-follow operations or private staging followed by atomic promotion.

## TU-006: Platform writability differences

Unix uses `access(W_OK)`. The non-Unix fallback is not complete ACL parity. Windows cutover remains blocked pending native access checks and tests.

## TU-007: Package-manager executable substitution

TypeScript executes bare package-manager names and a separate bare `which`. Repository or environment `PATH` manipulation can substitute code during detection.

Rust scans only absolute `PATH` entries, canonicalizes regular executables, rejects paths inside the inspected project, and invokes argument vectors without a shell. Nub and Aube are resolved directly.

**Residual:** writable host-trusted `PATH` directories remain a boundary. Prefer provisioned or allow-listed executable paths.

**Regression:** `resolver_skips_relative_and_project_local_path_entries`.

## TU-008: Unbounded package-manager output

Rust limits stdout and stderr to 1 MiB and treats overflow as unavailable. Both streams are drained concurrently.

**Regression:** `command_output_is_bounded`.

## TU-009: Process-tree cleanup

Rust creates a Unix process group and terminates it on timeout or output overflow. Windows still needs a kill-on-close Job Object.

**Regression:** `command_execution_has_a_deadline`.

## TU-010: Project metadata symlinks and resource exhaustion

Rust accepts only non-symlink regular `package.json` and `.yarnrc.yml` files no larger than 1 MiB. Custom Yarn paths are configuration markers but are never executed.

**Regressions:** `symlinked_package_metadata_is_not_followed`, `oversized_package_metadata_is_not_parsed`, and `custom_yarn_path_is_never_executed`.

## TU-011: Windows command-shim boundary

`.cmd` and `.bat` execution requires an explicit reviewed Windows adapter rather than implicit `cmd.exe` parsing. Those shims remain intentionally blocked.

## TU-012: Hostname-only repository URL policy

TypeScript accepts any parsed URL with `hostname === "github.com"`. Rust requires credential-free HTTPS with exact `github.com` authority, no explicit port, and no control/whitespace characters.

**Regression:** `github_url_validation_rejects_scheme_host_credential_and_port_confusion`.

## TU-013: Unvalidated examples and repository subpaths

Rust restricts named examples to ASCII letters, digits, hyphens, and underscores. Repository subpaths reject backslashes, percent signs, URL delimiters, controls, empty segments, and `.`/`..` components before provider calls.

**Regressions:** `unsafe_named_example_is_rejected_before_any_source_operation` and `unsafe_repository_subpath_is_rejected_before_network_resolution`.

## TU-014: Process-wide current-directory mutation

TypeScript calls `process.chdir(root)` without restoring it. Rust never changes process-wide current-directory state and passes absolute destinations explicitly.

## TU-015: Generated `package.json` boundary

Rust preserves file-presence behavior but parses scripts only from a regular non-symlink file no larger than 1 MiB. Unix opens use `O_NOFOLLOW` and `O_CLOEXEC`.

**Residual:** non-Unix final-component no-follow handling still needs a native implementation.

**Regressions:** `symlinked_package_json_is_not_read` and `oversized_package_json_is_not_parsed`.

## TU-016: Destination replacement and extraction TOCTOU

Rust rejects unsafe targets and immediate-parent symlinks and revalidates before and after each provider attempt. Production extraction still requires private staging or descriptor-relative no-follow writes.

**Regressions:** `conflicting_target_is_rejected_before_download` and `symlinked_project_root_is_never_followed`.

## TU-017: Retry over partially written destinations

TypeScript can make four attempts in one destination. Rust preserves the count, but the production provider must isolate and clean every attempt before promotion.

**Regressions:** `retries_three_times_and_succeeds_on_the_fourth_attempt` and `stops_after_four_failed_download_attempts`.

## TU-018: Inconsistent archive safeguards

TypeScript extraction paths do not expose one proven entry policy. The production Rust provider must use one extractor with entry-count, per-file, total-size, decompression-ratio, path-depth, link, device-node, permission, timeout, cleanup, and staging limits.

## TU-019: Update-notification terminal and bidi spoofing

TypeScript renders package names, upgrade commands, and debug errors without a uniform control/length policy. Rust escapes C0/C1 controls, escape characters, Arabic/left/right marks, embedding/override controls, isolate controls, and byte-order mark. Each untrusted field is limited to 1,024 Unicode scalar values.

**Intentional incompatibility:** unsafe characters are visible escapes rather than verbatim terminal text.

**Regressions:** `terminal_controls_are_escaped_in_package_name_and_command`, `unicode_directionality_controls_are_escaped_before_rendering`, `dynamic_error_controls_are_escaped_before_debug_logging`, and `rendered_untrusted_fields_are_bounded`.

## TU-020: Production update checker is not security-closed

`UpdateChecker` is injected and performs no production I/O. The future provider must define HTTPS/TLS, proxy, redirects, DNS/destination policy, deadlines, response/JSON bounds, auth redaction, cache integrity, rate limits, cancellation, and fail-silent compatibility.

## TU-021: Archive path semantics and cross-platform prefixes

**Boundary:** every tar entry name after configured path stripping.

TypeScript normalizes backslashes and resolves the path below the destination, but checks `relativePath.startsWith("..")`. That string-prefix test rejects safe names such as `..cache`, even though the component is not `..`. It also has no explicit entry-name length/component bound and does not define one platform-independent rule for Windows drive, UNC, or alternate-data-stream syntax.

Rust processes path components rather than string prefixes. It permits `..` only while lexical depth remains above the root, accepts safe `..`-prefixed normal names, and rejects NULs, absolute paths, UNC paths, drive prefixes, colons/alternate streams, paths over 4,096 scalar values, and paths over 256 non-empty components on every platform. Symbolic and hard-link entry classification retains TypeScript behavior.

**Intentional incompatibilities:** safe `..cache`-style names now work; colon-containing Unix archive names are rejected to avoid cross-platform alternate-stream ambiguity; oversized/deep names fail before allocation or writes.

**Residual:** this pure policy does not itself prevent a destination directory or ancestor from being replaced. The production extractor must apply it together with private staging or descriptor-relative no-follow writes and must reject links/device nodes at the archive parser boundary.

**Regressions:** all tests in `archive_parity.rs` and `archive_security.rs`, including `dot_dot_prefixed_normal_names_are_not_parent_traversal`.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and package/advisory index.
- GitHub Advisory Database.
- Official upstream security notices and release information for direct dependencies and externally executed tools changed by these tranches.

The project, notification, and archive-policy tranches add no new Rust dependency. They use the standard library plus existing workspace-managed dependencies.

The resolved `unsafe-libyaml` version is `0.2.11`, above the `0.2.10` patched floor for `RUSTSEC-2023-0075`. Long-term YAML maintenance/replacement policy remains open and is distinct from a vulnerability in this resolved version.

The lookup found `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33` for `webbrowser`; affected releases include versions through `1.2.1`, and `1.2.2` or later is patched. The workspace declares `0.8.7`. The observed `turborepo-query` call uses constant `http://localhost:8000`, limiting current attacker-controlled-scheme reachability, but the dependency must be upgraded or removed before migration merge.

Migration CI audits the complete lockfile and temporarily ignores only that separately tracked advisory so additional findings fail the gate. The exception must be removed with the dependency remediation.
