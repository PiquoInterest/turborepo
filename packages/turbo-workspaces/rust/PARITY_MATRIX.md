# turbo-workspaces TypeScript-to-Rust parity matrix

Status values are `implemented-core`, `intentional-hardening`, `partial`, and `blocked`. An implemented core is not a production cutover claim.

## Workspace-details orchestration

| TypeScript boundary | Rust boundary | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| `directoryInfo({ directory: root })` runs before detection | `WorkspaceDetailsProvider::directory_info` | implemented-core | Missing directories return before any detector call. |
| provider absolute path becomes `workspaceRoot` | `WorkspaceDirectoryInfo::absolute` | implemented-core | Every detector and reader receives this path, never the raw caller path. |
| `Object.values(MANAGERS)` insertion order | `MANAGER_DETECTION_ORDER` | implemented-core | Exact order is `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`. |
| serial `await detect` loop | fixed synchronous provider loop | implemented-core | First successful detector stops the loop; async binding remains blocked. |
| selected manager `read` | one provider read | implemented-core | Only the first detected manager receives read authority. |
| detector rejection | `WorkspaceDetailsError::Provider` | implemented-core | Error propagates immediately without parser fallback. |
| selected reader rejection | `WorkspaceDetailsError::Provider` | implemented-core | Error propagates immediately; later managers are not consulted. |
| missing-directory `ConvertError` | `WorkspaceDetailsKnownError::InvalidDirectory` | implemented-core | Exact type and message are translated. |
| unable-to-detect `ConvertError` | `WorkspaceDetailsKnownError::UnableToDetect` | implemented-core | Exact type and message follow six false detections. |
| mutable JavaScript registry | closed six-variant enum and fixed array | intentional-hardening | Prevents runtime registry extension or reordering at this trust boundary. |
| real `directoryInfo`, detector, and reader implementations | `WorkspaceDetailsProvider` production implementation | blocked | Requires bounded no-follow I/O, stable identity, parser limits, deterministic errors, and platform differentials. |
| Promise/JavaScript error identity | native or minimal host binding | blocked | Must preserve async order, public error class/type, cancellation, and exactly-once provider calls. |
| production package and callers | npm/native packaging and downstream cutover | blocked | TypeScript remains loaded and shipped. |

## Test mapping

| TypeScript evidence | Rust evidence | Status |
| --- | --- | --- |
| `workspace-details.test.ts`: registry order | `manager_order_matches_the_typescript_registry` | mapped |
| missing directory before detection | `missing_directory_returns_the_exact_known_error_before_detection` | mapped |
| serial first success and selected read | `first_detected_manager_is_read_and_later_managers_are_not_consulted` | mapped |
| selected read rejection without fallback | `selected_manager_read_failure_propagates_without_parser_fallback` | mapped |
| all six managers reject | `all_six_rejections_return_the_exact_unable_to_detect_error` | mapped |
| provider-only failure boundaries | six security/provider tests across both Rust files | added security evidence |

TDD chain: TypeScript oracle `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`, Rust RED `2d4cc22e6a821c88882a87d604746dabbaa95fe2`, Rust GREEN `263ddc22d5b5f544768f4e089c92892339b0dce8`.
