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

TypeScript normalizes backslashes and resolves below the destination, but checks `relativePath.startsWith("..")`. That string-prefix test rejects safe names such as `..cache`, has no explicit entry-name bound, and does not define one platform-independent rule for Windows drive, UNC, or alternate-data-stream syntax.

Rust processes components instead. It permits `..` only while lexical depth remains above the root, accepts safe `..`-prefixed normal names, and rejects NULs, absolute paths, UNC paths, drive prefixes, colons/alternate streams, paths over 4,096 scalar values, and paths over 256 non-empty components on every platform. Symbolic and hard-link classification retains TypeScript behavior.

**Residual:** the production extractor must combine this pure policy with private staging or descriptor-relative no-follow writes and reject links/device nodes at the parser boundary.

**Regressions:** all tests in `archive_parity.rs` and `archive_security.rs`.

## TU-022: GitHub bearer credentials can cross an insecure transport boundary

**TypeScript behavior:** `getGitHubAuthHeaders` parses the URL and checks only `hostname` against `api.github.com` and `codeload.github.com`. Because `URL.hostname` omits scheme, credentials, and port, the helper can attach a bearer token to plaintext `http://api.github.com/...` or to an explicitly redirected/custom port on an otherwise matching hostname.

**Impact:** a token can be disclosed to an insecure transport or to a service reached through a nonstandard authority. The effect depends on network and redirect behavior, but the credential attachment decision itself is too broad.

**Rust fix:** emit a bearer value only for syntactically valid, credential-free HTTPS URLs whose complete authority is exactly `api.github.com` or `codeload.github.com`, with no explicit port. Malformed URLs, look-alike domains, whitespace, controls, userinfo, HTTP, and custom ports receive no credential.

**Intentional incompatibility:** hostname-equivalent URLs that are not exact HTTPS authorities no longer receive tokens.

**Regressions:** `authorization_is_limited_to_exact_github_api_hosts`, `github_authorization_requires_https_without_credentials_or_ports`, and `malformed_and_control_bearing_urls_never_receive_credentials`.

## TU-023: Token validation, fallback, and diagnostic boundaries

**TypeScript behavior:** the selected token is trimmed and rejected only when it contains CR, LF, or NUL. There is no explicit size or printable-character bound. `GITHUB_TOKEN` takes precedence through JavaScript `||`; once a non-empty primary token is selected, an invalid value is ignored rather than falling back to `GH_TOKEN`.

**Rust fix:** preserve that selection precedence, including no fallback from an invalid non-empty primary token. The selected token must be non-empty after trimming, at most 4,096 characters, and composed only of ASCII graphic bytes. `NetworkEnvironment` intentionally does not implement `Debug`, reducing accidental secret exposure in generic diagnostics.

**Regressions:** `github_token_takes_precedence_over_gh_token`, `invalid_primary_token_does_not_fall_back_to_secondary_credentials`, and `tokens_are_ascii_graphic_and_size_bounded`.

## TU-024: Invalid proxy configuration can create a policy bypass

**TypeScript behavior:** proxy precedence is defined, but validation is deferred to `ProxyAgent` construction. A future caller that catches that error and retries without the dispatcher could silently bypass an administrator-selected proxy. The helper also has no explicit URL-length or allowed-scheme policy and does not model `NO_PROXY`.

**Rust fix:** preserve the existing lower/uppercase precedence, but return a typed error when the winning non-empty value is malformed or is not a bounded absolute HTTP(S) URL. Lower-precedence proxies are not consulted after a value wins, and direct connection is not treated as a fallback.

**Residual:** production request execution must define and test `NO_PROXY`/`no_proxy`, proxy authentication redaction, DNS behavior, certificate trust, redirects, and whether all GitHub endpoints are required to use the selected proxy.

**Regressions:** `https_proxy_precedence_matches_the_typescript_helper`, `invalid_selected_proxy_is_an_error_instead_of_direct_connection_fallback`, `proxy_urls_are_bounded_and_restricted_to_http_or_https`, and `malformed_request_url_is_an_error_before_proxy_selection`.

## TU-025: Option-like project names bypass the TypeScript check

**Severity:** Medium

**TypeScript behavior:** `validateDirectory` resolves the supplied text to an absolute path and then checks `root.startsWith("-")`. A relative input such as `-danger` therefore becomes an absolute path beginning with `/` or a drive prefix, so the intended option-confusion check is ineffective.

**Impact:** the project basename can retain a leading hyphen and later cross display, package, or subprocess boundaries that may interpret it as an option if they do not independently use a `--` terminator or typed argument contract.

**Rust fix:** validate the resolved basename itself and reject names beginning with `-` before any later provider is invoked.

**Intentional incompatibility:** a relative project directory whose basename begins with `-` is rejected even though current TypeScript accepts it.

**Regression:** `option_like_project_name_is_rejected`.

## TU-026: Final-component-only symlink checks miss redirected ancestors

**Severity:** High until handle-relative mutation is implemented

**TypeScript behavior:** `lstatSync(root)` inspects only the final requested path. A path such as `redirect/project` can traverse an existing symlink at `redirect` while the final `project` entry itself is an ordinary directory.

**Impact:** validation and later writes can occur outside the caller's intended directory tree. A malicious actor who can replace path components can also race path-based checks.

**Rust fix:** inspect every existing component of the stable requested path and reject a symbolic-link component before directory enumeration.

**Residual risk:** this is still a portable path-based check. A component can be replaced after validation, and conservative component walking may differ on platforms that expose system paths through symlink aliases. Production writes require directory handles with no-follow semantics on Unix and reviewed Windows handle/reparse-point logic. Supported-platform differential tests remain mandatory.

**Regression:** `symlinked_ancestor_is_rejected_before_directory_enumeration`.

## TU-027: Filename-only allow-listing accepts symlink entries

**Severity:** High

**TypeScript behavior:** `isFolderEmpty` calls `readdirSync` for names and allow-lists entries such as `.git`, `LICENSE`, and any `*.iml` file without checking their type.

**Impact:** an allow-listed symlink can redirect later consumers or hide an unsafe pre-existing project state while the folder is reported empty.

**Rust fix:** inspect each entry type and treat every symlink as a conflict, even when its name would normally be allowed.

**Intentional incompatibility:** symlink entries with allow-listed names are no longer considered harmless.

**Regression:** `allowlisted_symlink_is_never_treated_as_an_empty_directory`.

## TU-028: Lossy filename conversion can alias an allow-listed name

**Severity:** Medium

A Rust port that calls `to_string_lossy()` before applying the allow-list can replace invalid bytes with Unicode replacement characters and then make a non-UTF-8 filename appear to end in an allowed suffix such as `.iml`.

**Impact:** an unrepresentable entry may be hidden from conflict reporting, and any displayed name would no longer identify the real filesystem bytes.

**Rust fix:** require exact UTF-8 for the current string-based public result and fail closed when an entry name cannot be represented. No lossy string participates in the security decision.

**Regression:** `non_utf8_iml_name_is_not_silently_allowlisted`.

## TU-029: Directory enumeration and conflict collection are unbounded

**Severity:** Medium

**TypeScript behavior:** `readdirSync` returns the complete entry list and the implementation filters it into another conflict array without an explicit count bound.

**Impact:** a generated or attacker-controlled directory with a very large number of entries can consume excessive memory and CPU before project creation starts.

**Rust fix:** stop after 256 entries and return `InvalidData`. Validation converts that uncertainty into an invalid directory rather than continuing.

**Intentional incompatibility:** directories above the inspection limit are rejected even when every filename is otherwise allow-listed.

**Regression:** `folder_scan_is_bounded_before_collecting_untrusted_entries`.

## Directory-provider TDD and validation record

- Test-first contract: `53a55eefd92b919824374eb27159ff876e008147`.
- GREEN implementation: `c77464a7e6f36813a3b52262e78caa9ee449bb72`.
- Formatting correction: `8ee51022fd84264e0abeee17014802da3afcae20`.
- Clippy lifetime correction: `e47b4994e0d97641c2f976231aa89833aa142913`.

The first RED workflow stopped at formatting, so it is retained as chronological test-first evidence rather than a clean behavioral RED execution. At merge head, formatting, compilation, and all migration parity/security tests passed before Clippy exposed the unrelated explicit lifetime consolidated from the create-directory tranche. The lifetime was then removed without changing behavior.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and package/advisory index.
- GitHub Advisory Database.
- Official upstream security notices and release information for direct dependencies and externally executed tools changed by these tranches.

The directory-provider tranche adds no dependency, network destination, subprocess, parser, credential, or unsafe block. Its advisory disposition is therefore unchanged from the resolved workspace graph.

The project, notification, archive-policy, and network-policy tranches add no new Rust dependency. They use the standard library plus existing workspace-managed dependencies.

The resolved `unsafe-libyaml` version is `0.2.11`, above the `0.2.10` patched floor for `RUSTSEC-2023-0075`. Long-term YAML maintenance/replacement policy remains open and is distinct from a vulnerability in this resolved version.

The lookup found `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33` for `webbrowser`; affected releases include versions through `1.2.1`, and `1.2.2` or later is patched. The workspace declares `0.8.7`. The observed `turborepo-query` call uses constant `http://localhost:8000`, limiting current attacker-controlled-scheme reachability, but the dependency must be upgraded or removed before migration merge.

Migration CI audits the complete lockfile and temporarily ignores only that separately tracked advisory so additional findings fail the gate. The exception must be removed with dependency remediation.
