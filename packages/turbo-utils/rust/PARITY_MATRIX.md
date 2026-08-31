# Parity matrix

| Function | Status | Notes |
| --- | ---: | --- |
| `convertCase(..., camel)` | Safe-input parity | Preserves the exact ASCII `[-_][a-z]` replacement rule. |
| Other `convertCase` modes | Parity | Remain explicit not-implemented errors. |
| `searchUp` current/parent lookup | Safe-input parity | Filesystem root remains excluded, matching TypeScript. |
| `searchUp` content predicate | Hardened parity | Read errors remain non-matches; traversal and files over 4 MiB are rejected/non-matches. |
| `getTurboRoot` | Safe-input parity | Preserves root-config precedence, package-root fallback, and cache controls. |
| Turbo/workspace config discovery | Safe-input parity | Preserves translated ordering, task walking, and explicit config-path behavior. |
| `isFolderEmpty` | Safe-input parity | Preserves allow-list and `.iml` handling. |
| `isWriteable` | Unix parity | Uses `access(W_OK)` on Unix. Windows ACL parity remains open. |
| `validateDirectory` | Safe-input parity | Preserves valid/file/conflict/missing outcomes and wording. ANSI styling is host-only. |
| Metadata errors | Intentional deviation | Rust returns invalid instead of throwing or continuing under uncertainty. |
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

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
