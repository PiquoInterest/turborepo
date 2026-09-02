# turbo-workspaces TypeScript-to-Rust parity matrix

Status values are `implemented-core`, `intentional-hardening`, `partial`, and `blocked`. An implemented core is not a production cutover claim.

## Workspace-details orchestration

| TypeScript boundary | Rust boundary | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| `directoryInfo({ directory: root })` runs before detection | `WorkspaceDetailsProvider::directory_info` | implemented-core | Missing directories return before any detector call. |
| provider absolute path becomes `workspaceRoot` | `WorkspaceDirectoryInfo::absolute` | implemented-core | Every detector and reader receives this path, never the raw caller path. |
| `Object.values(MANAGERS)` insertion order | `MANAGER_DETECTION_ORDER` | implemented-core | Exact order is `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`. |
| serial `await detect` loop | fixed provider loop | implemented-core | First successful detector stops the loop; async binding remains blocked. |
| selected manager `read` | one provider read | implemented-core | Only the first detected manager receives read authority. |
| detector or selected reader rejects | `WorkspaceDetailsError::Provider` | implemented-core | Error propagates immediately without parser fallback. |
| missing-directory `ConvertError` | `WorkspaceDetailsKnownError::InvalidDirectory` | implemented-core | Exact type and message are translated. |
| unable-to-detect `ConvertError` | `WorkspaceDetailsKnownError::UnableToDetect` | implemented-core | Exact type and message follow six false detections. |
| mutable JavaScript registry | closed six-variant enum and fixed array | intentional-hardening | Prevents runtime registry extension or reordering at this trust boundary. |
| real directory, detector, and reader implementations | production provider | blocked | Requires bounded no-follow I/O, stable identity, parser limits, deterministic errors, and platform differentials. |

TDD chain: TypeScript oracle `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`, Rust RED `2d4cc22e6a821c88882a87d604746dabbaa95fe2`, Rust GREEN `263ddc22d5b5f544768f4e089c92892339b0dce8`.

## Bun workspace-glob compatibility

| TypeScript behavior | Rust behavior | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| accepts `apps/*`, multiple terminal wildcards, and `*` | same compatible subset | implemented-core | Ordinary valid cases retain the TypeScript result. |
| rejects recursive `**` patterns | same | implemented-core | Covered by parity tests. |
| rejects a wildcard before the final path segment | same | implemented-core | Covered by parity tests. |
| rejects negation, bracket, and brace syntax | same | implemented-core | Covered by parity tests. |
| unbounded input count and size | 256-value, 4096-byte per-value, and 65536-byte aggregate limits | intentional-hardening | Prevents allocation and later glob-work amplification. |
| control, bidi, and zero-width text is accepted as data | unsafe text is rejected | intentional-hardening | Prevents terminal/path spoofing before later consumers. |
| real workspace expansion | production provider | blocked | Requires root confinement, no-follow traversal, work limits, and platform differentials. |

Tests: 12 parity and 6 security.

## Workspace package parsing

| TypeScript behavior | Rust behavior | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| absent `workspaces` returns `[]` | `WorkspacePackages::Missing` returns an empty vector | implemented-core | Exact value behavior is mapped. |
| array form is returned | ordered borrowed vector | implemented-core | Ordering, duplicates, empty values, and syntax are preserved. |
| object `packages` form is returned | `WorkspacePackages::Object` | implemented-core | Missing `packages` becomes an empty vector. |
| negation, recursive, brace, and bracket patterns pass through | same | implemented-core | The general parser is not incorrectly restricted to Bun syntax. |
| original JavaScript array remains aliased and mutable | new vector of immutable borrowed strings | intentional-hardening | Preserves values while removing mutable result-to-input aliasing. |
| unlimited array length | maximum 256 globs | intentional-hardening | Rust fails before allocating the returned vector. |
| unlimited per-value and aggregate bytes | 4096 bytes per value and 65536 total bytes | intentional-hardening | Checked before result publication with checked arithmetic. |
| controls and invisible format characters pass through | reviewed unsafe classes are rejected | intentional-hardening | Errors never echo the offending text. |
| package.json read and dynamic JSON conversion | production binding/provider | blocked | Requires bounded no-follow reads, JSON limits, exact error mapping, and differentials. |

TDD chain: TypeScript oracle `9c8f77deee15c01baba73fdd510960e899756f0e`, Rust RED `089112a3f85bc2cbaaf864991eb5b6129602ff30`, Rust GREEN `8b4aea45459aa09237aef7d8dd35ccf06503ae28`.

Detailed divergence rationale is in `WORKSPACE_PACKAGES_DIVERGENCES.md`.

## Test mapping

| TypeScript evidence | Rust evidence | Status |
| --- | --- | --- |
| `workspace-details.test.ts` | `workspace_details_parity.rs`, `workspace_details_security.rs` | 6 parity and 5 security mapped |
| Bun compatibility cases and `bun-workspace-glob-security.test.ts` | `bun_workspace_glob_parity.rs`, `bun_workspace_glob_security.rs` | 12 parity and 6 security mapped |
| `workspace-packages.test.ts` and `parseWorkspacePackages` cases | `workspace_packages_parity.rs`, `workspace_packages_security.rs` | 7 parity and 6 security mapped |
| package-manager declaration cases from `utils.test.ts` | active declaration TDD tranche | partial and intentionally not counted here |
| remaining `utils.test.ts`, `managers.test.ts`, `index.test.ts`, `nub.test.ts`, and `aube.test.ts` behavior | none or partial provider evidence | blocked or partial |

Current crate inventory: 25 parity tests and 17 security tests, 42 total. See `TEST_INVENTORY.md` for the remaining sequence.
