# create-turbo TypeScript-to-Rust parity matrix

Status values are `implemented`, `intentional-deviation`, `partial`, `blocked`, and `not-implemented`. A row marked `implemented` means the listed safe-input behavior has translated tests. It does not imply that the package is the production entry point.

## `update-commands-in-readme` tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| supported managers `pnpm`, `npm`, `yarn`, `bun` | `PackageManager` | implemented | All four managers are covered by translated tests. |
| ordered `<pm> run` replacement | `replace_run_commands` | implemented | Preserves the literal single-space TypeScript pattern and JavaScript-style ASCII word boundaries. |
| bare `<pm>` replacement with `(?!\s+run)` | `replace_bare_commands` | implemented | Preserves subcommands and the JavaScript whitespace exclusion before `run`. |
| inline code and triple-backtick fenced regions | `next_code_region` | implemented | Region precedence, multiple regions, language identifiers, and prose isolation are tested. |
| text outside code regions and `npx` | pure transformer output | implemented | Prose and `npx` remain unchanged. |
| missing manager or README | `transform_readme` | implemented | Returns `not-applicable`. |
| existing README read, transform, write | `transform_readme` | implemented | Exact content and response metadata are covered. |
| unbounded read and whole-document regex | 4 MiB bounded read and linear scanner | intentional-deviation | Oversized input is rejected without modification. See CT-RS-001. |
| replacement decoding of malformed UTF-8 | strict UTF-8 validation | intentional-deviation | Invalid bytes are not silently rewritten. See CT-RS-002. |
| symlink following | no-follow and Unix identity checks | intentional-deviation | Outside targets are not modified. See CT-RS-003. |
| in-place truncating write | synchronized sibling temporary write and replacement | intentional-deviation | Reduces partial-write corruption. See CT-RS-004. |
| public JavaScript `TransformError` mapping | internal typed Rust errors | partial | The production binding must map errors to the established public class and fatality metadata. |
| Windows replacement/metadata contract | remove-then-rename fallback | blocked | Atomic replacement and ACL/ownership policy remain required. |

## `git-ignore` tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| `DEFAULT_IGNORE` constant | `DEFAULT_IGNORE` | implemented | Exact byte-for-byte constant and `.turbo` presence are tested. |
| missing `.gitignore` | `create_git_ignore` | implemented | Creates exact content and returns `success` with name `git-ignore`. |
| existing regular `.gitignore` | `create_git_ignore` | implemented | Returns `not-applicable` without modifying customer content. |
| existing directory at `.gitignore` | `create_git_ignore` | implemented | Preserves TypeScript's harmless `existsSync` result and returns `not-applicable`. |
| missing/unwritable project root | `GitIgnoreError::Write` | implemented | Public message remains `Unable to write .gitignore`. |
| `existsSync` followed by overwrite-capable `writeFileSync` | synchronized temporary file plus no-overwrite hard link | intentional-deviation | Removes the check/write race and never replaces a concurrent destination. See CT-RS-007. |
| broken `.gitignore` symlink is treated as missing and followed by `writeFileSync` | `symlink_metadata` rejection | intentional-deviation | External target creation is blocked and tested. See CT-RS-008. |
| existing `.gitignore` symlink is silently accepted as not-applicable | explicit rejection | intentional-deviation | Prevents an unsafe path from being normalized as a valid transform state. |
| symlinked project root | explicit rejection | intentional-deviation | Prevents writing through a redirected project root. |
| temporary publication cleanup | `create_new`, bounded retries, `hard_link`, cleanup | implemented | Success leaves only `.gitignore`; collisions cannot be overwritten. |
| malicious concurrent root replacement | path revalidation and Unix identity check | partial | Descriptor-relative publication is still required to close every malicious race. |

## Existing TypeScript test mapping

| TypeScript test group | Rust test coverage | Status |
| --- | --- | --- |
| README compound/bare replacements | README parity tests | implemented |
| README subcommand preservation, prose isolation, fenced/inline regions, identity, `npx` | README parity tests | implemented |
| README missing inputs and read-transform-write | README parity tests | implemented |
| `DEFAULT_IGNORE` contains `.turbo` | `default_ignore_matches_the_typescript_constant` | implemented and strengthened to exact bytes |
| `.gitignore` transform source branches | five translated source-contract tests | implemented |
| symlink/race/resource regressions absent from TypeScript suite | five security tests | intentional-deviation evidence |

## Remaining `create-turbo` surfaces

| Surface | Status | Required closure |
| --- | --- | --- |
| CLI argument parsing and help/version output | not-implemented | Translate CLI fixtures and process-level output/exit tests. |
| interactive prompts | not-implemented | Preserve defaults, cancellation, validation, non-TTY behavior, and ordering. |
| example resolution and download | not-implemented | Reuse the reviewed `turbo-utils-rs` provider after redirect, proxy, extraction, and atomic-promotion contracts are closed. |
| project creation orchestration | partial | A coordinator exists in `turbo-utils-rs`; `create-turbo` integration and differential tests remain. |
| Git initialization and commit | not-implemented | Port existing nine Jest cases, canonical executable resolution, configuration isolation, deadlines, cleanup, and platform behavior. |
| `git-ignore` transform | implemented core | Add native binding, differential host tests, production routing, and TypeScript removal proof. |
| `official-starter` transform | not-implemented | Translate package/workspace mutations with deterministic JSON ordering. |
| package-manager transform | not-implemented | Preserve lockfile and package-manager metadata behavior. |
| telemetry integration | blocked | Land the reviewed package telemetry Rust core and bind it without retaining business logic in TypeScript. |
| npm/native packaging | blocked | Build, sign, publish, select, and roll back Rust binaries on every supported platform. |
| TypeScript removal | blocked | Migrate every downstream caller and prove the old runtime is neither loaded nor shipped. |
