# create-turbo TypeScript-to-Rust parity matrix

Status values are `implemented`, `intentional-deviation`, `partial`, `blocked`, and `not-implemented`. A row marked `implemented` means the listed safe-input behavior has translated tests. It does not imply that the package is the production entry point.

## `update-commands-in-readme` tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| `src/transforms/update-commands-in-readme.ts`: supported managers `pnpm`, `npm`, `yarn`, `bun` | `PackageManager` | implemented | All four managers are covered by translated tests. |
| Ordered `<pm> run` replacement | `replace_run_commands` | implemented | Preserves the literal single-space TypeScript pattern and JavaScript-style ASCII word boundaries. |
| Bare `<pm>` replacement with `(?!\s+run)` | `replace_bare_commands` | implemented | Preserves subcommands and the JavaScript whitespace exclusion before `run`. |
| Inline code pattern `` `[^`]+` `` | `next_code_region` | implemented | Multiple independent inline regions and prose isolation are tested. |
| Triple-backtick fenced pattern | `next_code_region` | implemented | Language identifiers, mixed commands, and multiple regions are tested. |
| Fenced pattern precedence over inline pattern | `next_code_region` | implemented | The scanner evaluates triple fences before inline spans at each possible start, matching the TypeScript alternation order. |
| Text outside code regions | pure transformer output | implemented | Prose package-manager names remain unchanged. |
| `npx` text | pure transformer output | implemented | `npx` is outside the manager list and remains unchanged. |
| Identity replacement | pure transformer output | implemented | Selecting the existing manager does not corrupt the region. |
| Missing package manager | `transform_readme` | implemented | Returns `not-applicable` before filesystem access. |
| Missing `README.md` | `transform_readme` | implemented | Returns `not-applicable`. |
| Existing README read, transform, write | `transform_readme` | implemented | Exact content and response metadata are covered. |
| TypeScript unbounded `readFile` and whole-document regex | 4 MiB bounded read and linear scanner | intentional-deviation | Oversized input is rejected without modifying the file. See `SECURITY.md` CT-RS-001. |
| Node UTF-8 replacement decoding of malformed bytes | strict UTF-8 validation | intentional-deviation | Invalid UTF-8 is rejected without modifying the file. See CT-RS-002. |
| TypeScript follows symlinked root/README paths | no-follow checks and Unix identity checks | intentional-deviation | Symlink regression tests prove outside targets are not modified. See CT-RS-003. |
| TypeScript truncates and rewrites the original file in place | same-directory temporary file and replacement | intentional-deviation | Avoids ordinary partial-write corruption. Mode bits are preserved on Unix. See CT-RS-004. |
| TypeScript generic `TransformError` presentation | internal typed Rust errors | partial | A future JS/native binding must map all internal failures to the established public error class and message. |
| Windows file replacement metadata and ACL behavior | standard-library remove-then-rename fallback | blocked | Requires a reviewed atomic Windows replacement implementation before production cutover. |

## Existing TypeScript test mapping

| TypeScript test group | Rust test coverage | Status |
| --- | --- | --- |
| compound `<pm> run` replacements | `replaces_compound_run_commands_for_every_supported_manager` | implemented |
| bare manager replacements | `replaces_bare_manager_without_inserting_run` | implemented |
| subcommand preservation | `preserves_package_manager_subcommands` | implemented |
| prose isolation | `leaves_prose_outside_code_regions_unchanged`, `replaces_only_inside_backtick_regions` | implemented |
| fenced code blocks | `handles_fenced_blocks_and_language_identifiers` | implemented |
| multiple code regions | `handles_multiple_inline_and_fenced_regions` | implemented |
| identity replacement | `identity_replacement_does_not_corrupt_content` | implemented |
| realistic README and `npx` behavior | `leaves_npx_untouched_in_realistic_readme_content` | implemented |
| missing manager and README | `transform_is_not_applicable_without_package_manager`, `transform_is_not_applicable_without_readme` | implemented |
| read-transform-write and response metadata | `transform_reads_updates_and_writes_readme` | implemented |

## Remaining `create-turbo` surfaces

| Surface | Status | Required closure |
| --- | --- | --- |
| CLI argument parsing and help/version output | not-implemented | Translate CLI fixtures and process-level output/exit tests. |
| interactive prompts | not-implemented | Preserve defaults, cancellation, validation, non-TTY behavior, and ordering. |
| example resolution and download | not-implemented | Reuse the reviewed `turbo-utils-rs` network/archive provider after redirect, proxy, size, extraction, and atomic-promotion contracts are closed. |
| project creation orchestration | partial | A coordinator exists in `turbo-utils-rs`; `create-turbo` integration and differential tests remain. |
| Git initialization and commit | not-implemented | Port with bounded explicit executable resolution, argument vectors, timeouts, output limits, and platform tests. |
| `git-ignore` transform | not-implemented | Translate fixtures and define symlink/atomic-write behavior. |
| `official-starter` transform | not-implemented | Translate package and workspace mutations with deterministic JSON ordering. |
| package-manager transform | not-implemented | Preserve lockfile and package-manager metadata behavior. |
| telemetry integration | blocked | Land the reviewed package telemetry Rust core and bind it without retaining business logic in TypeScript. |
| npm/native packaging | blocked | Build, sign, publish, select, and roll back Rust binaries on every supported platform. |
| TypeScript removal | blocked | Migrate every downstream caller and add artifact/removal tests proving the old runtime is neither loaded nor shipped. |
