# Parity matrix

| Function | Status | Notes |
| --- | ---: | --- |
| `convertCase(..., camel)` | Safe-input parity | Preserves the exact ASCII `[-_][a-z]` replacement rule. |
| Other `convertCase` modes | Parity | Remain explicit not-implemented errors. |
| `searchUp` current/parent lookup | Safe-input parity | Filesystem root remains excluded, matching TypeScript. |
| `searchUp` content predicate | Hardened parity | Read errors remain non-matches; traversal and files over 4 MiB are rejected/non-matches. |
| `getTurboRoot` | Safe-input parity | Preserves root-config precedence, package-root fallback, and cache controls. |
| Turbo/workspace config discovery | Safe-input parity | Preserves translated ordering, task walking, and explicit config-path behavior. |
| `isFolderEmpty` ordinary allow-list and `.iml` handling | Safe-input parity | Regular UTF-8 entries preserve the TypeScript allow-list and IntelliJ suffix behavior. |
| `isFolderEmpty` entry type | Intentional hardening | TypeScript classifies by filename only. Rust treats every symlink entry as a conflict, including `.git`, `LICENSE`, and `*.iml`. |
| `isFolderEmpty` filename encoding | Intentional hardening | Rust rejects non-UTF-8 entry names instead of applying a lossy string conversion that could alias an allow-listed name. |
| `isFolderEmpty` entry count | Intentional hardening | Rust fails closed after 256 entries instead of collecting an unbounded attacker-controlled directory listing. |
| `isWriteable` | Unix parity | Uses `access(W_OK)` on Unix. Windows ACL parity remains open. |
| `validateDirectory` ordinary valid/file/conflict/missing outcomes | Safe-input parity | Preserves safe-input outcomes and wording. ANSI styling is host-only. |
| Option-like project basename | Intentional hardening | The TypeScript check runs after `path.resolve`, so a relative `-danger` input becomes an absolute path and bypasses `root.startsWith("-")`. Rust validates the basename itself. |
| Existing symlinked path components | Intentional hardening | Rust rejects an existing symlink in the requested path before enumeration; TypeScript checks only the final component with `lstatSync`. |
| Metadata and enumeration errors | Intentional deviation | Rust returns invalid instead of throwing or continuing under uncertainty. |
| Directory validation versus later creation | Partial | Portable component checks do not close malicious concurrent replacement. Production mutation requires stable directory handles or private staging and atomic promotion. |
| `getAvailablePackageManagers` | Safe-input parity | Preserves manager set, semver extraction, missing-command behavior, Yarn precedence, timeout, and Corepack environment. |
| `getPackageManagersBinPaths` | Hardened parity | Preserves safe Yarn/npm/pnpm/Bun/Nub/Aube results. Nub/Aube are resolved directly. |
| Custom Yarn paths | Security-preserving parity | A custom or malformed `yarnPath` disables Yarn probing and is never executed. |
| Package-manager executable lookup | Intentional deviation | Rust ignores relative `PATH` entries and canonical executables inside the inspected project. |
| Package-manager metadata reads | Intentional deviation | Rust reads only non-symlink regular metadata up to 1 MiB. |
| Package-manager subprocesses | Intentional deviation | Rust bounds output and kills Unix process groups on timeout/overflow. Windows tree cleanup remains blocked. |
| Windows `.cmd`/`.bat` shims | Blocked | A reviewed Windows adapter and process-tree tests are required. |
| `createProject` default example | Safe-input parity | Uses the `basic` example without repository discovery and returns no repository metadata. |
| `createProject` named example | Safe-input parity | Checks the upstream example and reports `vercel/turborepo` metadata. |
| `createProject` GitHub repository | Safe-input parity | Resolves metadata and delegates acquisition through a testable provider. |
| Project retry count | Parity | Preserves four total attempts. Timing/backoff belongs to the provider. |
| Generated `package.json` detection | Hardened parity | Presence is retained; scripts are parsed only from bounded regular non-symlink files. |
| Generated script ordering | Parity | Reproduces JavaScript `Object.keys` ordering. |
| Project current-directory mutation | Intentional deviation | Rust never calls process-wide `chdir`. |
| GitHub repository URL classification | Intentional deviation | Rust requires HTTPS, exact authority, no credentials/port, and no controls/whitespace. |
| Named example/repository subpaths | Intentional deviation | Rust rejects traversal, separators in names, backslashes, empty segments, and `.`/`..`. |
| Project target symlinks | Intentional deviation | Rust rejects target/immediate-parent symlinks. Descriptor-relative closure remains open. |
| `isPathSafe` ordinary paths | Safe-input parity | Normal/nested paths, mixed separators, and internal parent cancellation remain valid below the root. |
| `isPathSafe` escaping traversal | Parity | Parent traversal above root and absolute paths are rejected. |
| `isPathSafe` `..`-prefixed names | Corrected TypeScript logic | Rust accepts `..cache`; TypeScript incorrectly rejects it with `relativePath.startsWith("..")`. |
| Archive cross-platform forms | Intentional deviation | Rust rejects UNC/drive paths and Windows alternate-data-stream syntax everywhere. |
| Archive resource bounds | Intentional deviation | Rust rejects paths over 4,096 scalar values or 256 components. |
| `isLinkEntry` | Parity | `SymbolicLink` and `Link` are rejected; files/directories are not links. |
| `GITHUB_TOKEN` / `GH_TOKEN` precedence | Parity | Non-empty `GITHUB_TOKEN` wins. An invalid selected primary is rejected rather than falling back, matching TypeScript selection order. |
| GitHub Authorization host allow-list | Hardened parity | Safe HTTPS calls to exact `api.github.com` and `codeload.github.com` receive `Bearer <token>`. |
| Plaintext HTTP, explicit ports, userinfo | Intentional deviation | TypeScript checks hostname only and can attach credentials. Rust emits no credentials unless the complete authority is exact, credential-free HTTPS with no port. |
| Look-alike/malformed GitHub URLs | Security parity | No credentials are returned for suffix/prefix look-alikes, malformed URLs, whitespace, or controls. |
| Token character/size policy | Intentional deviation | Rust requires trimmed ASCII graphic data up to 4,096 characters. TypeScript rejects only CR/LF/NUL and has no explicit limit. |
| HTTPS proxy precedence | Parity | lowercase `https_proxy`, uppercase `HTTPS_PROXY`, lowercase `http_proxy`, uppercase `HTTP_PROXY`. |
| Non-HTTPS proxy precedence | Parity | lowercase then uppercase HTTP proxy only. |
| Invalid selected proxy | Security-preserving behavior | Rust returns an error rather than bypassing the configured proxy with a direct connection. |
| Proxy URL policy | Intentional deviation | Rust accepts only bounded absolute HTTP(S) proxy URLs. Production `NO_PROXY` semantics remain open. |
| Network/archive acquisition and writes | Blocked | Request execution, redirect/TLS/proxy agents, GitHub lookup, Git fallback, tar streaming/writes, cleanup, and atomic promotion remain to be ported. |
| `createNotifyUpdate` eager check | Parity core | Preparation invokes the checker exactly once and stores the result. |
| No update or failed check | Parity | Produces no output and preserves exit code. Checker errors are swallowed. |
| Available update announcement | Safe-input parity | Preserves unstyled message ordering and optional command. |
| Static/dynamic upgrade commands | Parity core | Dynamic resolution occurs only after an update; failures log only under debug. |
| Notification exit behavior | Parity core | Rust retains exact success/failure intent for the host adapter. |
| Notification terminal rendering | Intentional deviation | Rust escapes terminal/directionality controls and bounds untrusted fields. |
| Production registry update checker | Blocked | Needs TLS, proxy, redirect, timeout, size, cache, rate-limit, and differential tests. |

## Directory-provider regression mapping

| TypeScript or provider boundary | Rust regression | Status |
| --- | --- | --- |
| resolved absolute path makes `root.startsWith("-")` ineffective for `-danger` | `option_like_project_name_is_rejected` | fixed in Rust |
| final-component-only `lstatSync` can miss a symlinked ancestor | `symlinked_ancestor_is_rejected_before_directory_enumeration` | fixed for stable existing paths; concurrent replacement remains blocked |
| allow-list checks names without entry type | `allowlisted_symlink_is_never_treated_as_an_empty_directory` | fixed in Rust |
| lossy conversion can make a non-UTF-8 name appear allow-listed | `non_utf8_iml_name_is_not_silently_allowlisted` | fixed in Rust |
| `readdirSync` and conflict collection have no entry-count bound | `folder_scan_is_bounded_before_collecting_untrusted_entries` | fixed in Rust with a 256-entry fail-closed limit |

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
