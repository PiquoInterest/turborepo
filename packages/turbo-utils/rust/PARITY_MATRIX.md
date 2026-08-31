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
| `isWriteable` | Unix parity | Uses `access(W_OK)` on Unix. Windows ACL parity is still open. |
| `validateDirectory` | Safe-input parity | Preserves normal valid/file/conflict/missing outcomes and singular/plural wording. ANSI dim styling is not represented in the Rust value. |
| Metadata errors | Intentional deviation | Rust returns invalid instead of throwing or continuing under uncertainty. |
| `getAvailablePackageManagers` | Safe-input parity | Preserves manager set, semver extraction, missing-command behavior, Yarn project metadata precedence, five-second command contract, and `COREPACK_ENABLE_STRICT=0`. |
| `getPackageManagersBinPaths` | Hardened parity | Preserves Yarn 1/Berry, npm, pnpm, Bun, Nub, and Aube results for safe fixtures. Rust resolves Nub/Aube directly instead of executing `which`. |
| Custom Yarn paths | Security-preserving parity | A custom or malformed `yarnPath` disables Yarn probing and is never executed. |
| Package-manager executable lookup | Intentional deviation | Rust ignores relative `PATH` entries and canonical executables inside the inspected project root to prevent repository-controlled binary substitution. |
| Package-manager metadata reads | Intentional deviation | Rust reads only non-symlink regular `package.json`/`.yarnrc.yml` files up to 1 MiB. TypeScript follows symlinks and has no explicit read bound. |
| Package-manager subprocesses | Intentional deviation | Rust bounds stdout/stderr and kills the Unix process group on timeout/overflow. Windows descendant cleanup remains a cutover blocker. |
| Windows `.cmd`/`.bat` manager shims | Blocked | The hardened system runner currently resolves only direct executable files on Windows. Native Windows package-manager resolution and process-tree tests are required before cutover. |
| `createProject` default example | Safe-input parity | Downloads the `basic` example without repository discovery and returns no repository metadata. |
| `createProject` named example | Safe-input parity | Checks the upstream example catalog, downloads the selected example, and reports `vercel/turborepo` repository metadata. |
| `createProject` GitHub repository | Safe-input parity | Resolves repository metadata, checks for a package, and delegates repository download through a testable provider boundary. |
| Project download retry count | Parity | Preserves `async-retry({ retries: 3 })` as four total attempts. Timing/backoff belongs to the production provider and remains open. |
| Generated `package.json` detection | Hardened parity | Presence is retained for malformed/unsafe metadata, but scripts are read only from regular non-symlink files no larger than 1 MiB. |
| Generated script ordering | Parity | Reproduces JavaScript `Object.keys` ordering: array-index keys first in numeric order, followed by other keys in insertion order. |
| Project current-directory mutation | Intentional deviation | Rust never calls process-wide `chdir`; the resolved root is passed explicitly to the provider. |
| GitHub URL classification | Intentional deviation | TypeScript checks only `hostname === "github.com"`. Rust also requires HTTPS, exact authority, no credentials, no explicit port, and no control/whitespace characters. |
| Named example/repository subpath validation | Intentional deviation | Rust rejects traversal, separators in named examples, backslashes, empty path segments, and `.`/`..` repository components before provider calls. |
| Project target symlinks | Intentional deviation | Rust rejects symlink targets and immediate-parent symlinks before download. Descriptor-relative TOCTOU closure remains open. |
| Network/archive acquisition | Blocked | `ProjectSource` is currently an injected boundary. The GitHub API, proxy/auth, Git fallback, tar streaming, extraction, timeout, and cleanup implementation must be ported and differentially tested before cutover. |
| `createNotifyUpdate` eager update check | Parity core | `PreparedUpdateNotification::prepare` invokes the injected checker exactly once and stores the result before later notification calls. Host/module initialization remains a binding concern. |
| No update or failed update check | Parity | Produces no notification output and preserves the requested exit code. Checker errors are swallowed, matching the TypeScript promise catch. |
| Available update announcement | Safe-input parity | Emits the same blank-line, announcement, optional command, blank-line sequence as the unstyled TypeScript message values. Terminal styling belongs to the host adapter. |
| Static upgrade command | Parity | Renders the supplied command only when it is non-empty and an update exists. |
| Dynamic upgrade command | Parity core | Resolves the provider only after update detection. `None` omits the command line. Provider failure preserves the exit code and is reported only when debug is enabled. |
| Notification exit behavior | Parity core | The Rust result retains exact success/failure exit intent. The production host adapter must call the platform exit API after flushing output and telemetry. |
| Notification terminal rendering | Intentional deviation | TypeScript logs package names, commands, and debug errors verbatim. Rust escapes terminal controls and Unicode directionality controls and truncates each untrusted field after 1,024 scalar values. |
| Production registry update checker | Blocked | `UpdateChecker` is injected. The real registry/network implementation needs timeout, response-size, redirect, proxy, TLS, cache, rate-limit, and differential tests before cutover. |

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
