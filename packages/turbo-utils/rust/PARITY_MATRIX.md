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

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
