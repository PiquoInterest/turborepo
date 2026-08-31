# Parity matrix

| Function | Status | Notes |
| --- | ---: | --- |
| `convertCase(..., camel)` | Safe-input parity | Preserves the exact ASCII `[-_][a-z]` replacement rule. |
| Other `convertCase` modes | Parity | Remain explicit not-implemented errors. |
| `searchUp` current/parent lookup | Safe-input parity | Filesystem root remains excluded, matching the TypeScript loop. |
| `searchUp` content predicate | Hardened parity | Read errors remain non-matches; unsafe target traversal and files over 4 MiB are rejected/non-matches. |
| `getTurboRoot` | Safe-input parity | Preserves root-config precedence, package-root fallback, and cache controls for translated fixtures. |
| Turbo/workspace config discovery | Safe-input parity | Preserves translated root/workspace config ordering, task walking, and explicit config-path behavior. |
| `isFolderEmpty` | Safe-input parity | Preserves allow-list and `.iml` handling. Conflict ordering follows filesystem enumeration, as in Node. |
| `isWriteable` | Unix parity | Uses `access(W_OK)` on Unix. Windows ACL parity remains open. |
| `validateDirectory` | Safe-input parity | Preserves valid/file/conflict/missing outcomes and singular/plural wording. ANSI dim styling is not represented in the Rust value. |
| Metadata errors | Intentional deviation | Rust returns invalid instead of throwing or continuing under uncertainty. |
| `getAvailablePackageManagers` | Safe-input parity | Preserves manager set, semver extraction, missing-command behavior, Yarn project metadata precedence, five-second command contract, and `COREPACK_ENABLE_STRICT=0`. |
| `getPackageManagersBinPaths` | Hardened parity | Preserves Yarn 1/Berry, npm, pnpm, Bun, Nub, and Aube results for safe fixtures. Rust resolves Nub/Aube directly instead of executing `which`. |
| Custom Yarn paths | Security-preserving parity | A custom or malformed `yarnPath` disables Yarn probing and is never executed. |
| Package-manager executable lookup | Intentional deviation | Rust ignores relative `PATH` entries and canonical executables inside the inspected project root. |
| Package-manager metadata reads | Intentional deviation | Rust reads only non-symlink regular `package.json`/`.yarnrc.yml` files up to 1 MiB. |
| Package-manager subprocesses | Intentional deviation | Rust bounds stdout/stderr and kills the Unix process group on timeout/overflow. Windows descendant cleanup remains blocked. |
| Windows `.cmd`/`.bat` manager shims | Blocked | A reviewed Windows adapter and process-tree tests are required before cutover. |
| `createProject` default example | Safe-input parity | Downloads the `basic` example without repository discovery and returns no repository metadata. |
| `createProject` named example | Safe-input parity | Checks the upstream example catalog, downloads the selected example, and reports `vercel/turborepo` metadata. |
| `createProject` GitHub repository | Safe-input parity | Resolves repository metadata, checks for a package, and delegates download through a testable provider. |
| Project retry count | Parity | Preserves `async-retry({ retries: 3 })` as four total attempts. Timing/backoff belongs to the provider. |
| Generated `package.json` detection | Hardened parity | Presence is retained for malformed/unsafe metadata, but scripts are parsed only from regular non-symlink files no larger than 1 MiB. |
| Generated script ordering | Parity | Reproduces JavaScript `Object.keys`: array-index keys numerically first, then other keys in insertion order. |
| Project current-directory mutation | Intentional deviation | Rust never calls process-wide `chdir`; the resolved root is passed explicitly. |
| GitHub URL classification | Intentional deviation | Rust requires HTTPS, exact authority, no credentials/port, and no control/whitespace characters. |
| Named example/repository subpath validation | Intentional deviation | Rust rejects traversal, separators in named examples, backslashes, empty segments, and `.`/`..` subpath components. |
| Project target symlinks | Intentional deviation | Rust rejects target and immediate-parent symlinks. Descriptor-relative TOCTOU closure remains open. |
| `isPathSafe` ordinary relative paths | Safe-input parity | Normal files, nested files, mixed separators, and internal `..` cancellation remain valid when the lexical result stays below the root. |
| `isPathSafe` escaping traversal | Parity | Parent traversal above the root and absolute entry paths are rejected. A pre-resolved root follows the same policy. |
| `isPathSafe` `..`-prefixed names | Corrected TypeScript logic | Rust accepts safe normal components such as `..cache`. TypeScript uses `relativePath.startsWith("..")` and incorrectly rejects them even though they are not parent components. |
| Archive cross-platform path forms | Intentional deviation | Rust rejects UNC and drive-prefixed paths plus Windows alternate-data-stream syntax on every platform. |
| Archive resource bounds | Intentional deviation | Rust rejects paths over 4,096 scalar values or 256 non-empty components. TypeScript has no explicit bound. |
| `isLinkEntry` | Parity | `SymbolicLink` and `Link` are rejected; ordinary files and directories are not classified as links. |
| Network/archive acquisition and writes | Blocked | `ProjectSource` is injected. GitHub API, proxy/auth, Git fallback, tar streaming, safe writes, timeout, cleanup, and atomic promotion remain to be ported and differentially tested. |
| `createNotifyUpdate` eager update check | Parity core | Preparation invokes the checker exactly once and stores the result. Host/module initialization remains a binding concern. |
| No update or failed update check | Parity | Produces no notification output and preserves the requested exit code. Checker errors are swallowed. |
| Available update announcement | Safe-input parity | Emits the same blank-line, announcement, optional command, blank-line sequence as unstyled TypeScript values. |
| Static upgrade command | Parity | Renders a non-empty supplied command only when an update exists. |
| Dynamic upgrade command | Parity core | Resolves only after an update. Provider failure preserves exit code and logs only under debug. |
| Notification exit behavior | Parity core | Rust retains exact success/failure intent; the host adapter must flush output/telemetry and exit. |
| Notification terminal rendering | Intentional deviation | Rust escapes terminal and Unicode directionality controls and truncates each untrusted field after 1,024 scalar values. |
| Production registry update checker | Blocked | `UpdateChecker` is injected. The real provider needs timeout, size, redirect, proxy, TLS, cache, rate-limit, and differential tests. |

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
