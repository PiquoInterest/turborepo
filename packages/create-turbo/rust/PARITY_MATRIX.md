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

## Git initialization tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| `git rev-parse --is-inside-work-tree` in project root | first injected `VcsInvocation` | implemented | A successful probe returns `false` without further calls or cleanup. |
| `hg --cwd . root` with process cwd set to project root | second injected `VcsInvocation` | implemented | Preserves the source argument/cwd split and does not stringify the root. |
| `git init` followed by checkout, add, and commit | `try_git_init_with` | implemented core | Exact six-call successful sequence and argument order are translated. |
| initial commit message | `INITIAL_COMMIT_MESSAGE` | implemented | Exact source value is `Initial commit from create-turbo`; the initial RED draft's different text was corrected before GREEN. |
| Git/Hg probe failures are treated as “not inside a repository” | boolean runner results | implemented | Initialization continues after either failed probe. |
| checkout/add/commit failure after successful init | injected cleanup then `false` | implemented | All three failure positions are covered. Cleanup failure remains non-fatal. |
| failed `git init` | return `false` without cleanup | implemented | Preserves the TypeScript ownership boundary and avoids deleting an ambiguous or concurrently created `.git` path. |
| TypeScript shell-metacharacter blacklist on a shell-free argv call | structural path validation | intentional-deviation | Rust permits harmless `$`, `#`, `;`, and `!`, but rejects relative/root/parent paths, controls, and Windows-invalid filename characters. See CT-RS-011. |
| JavaScript string-only root | `Path`/`PathBuf` root and cwd | intentional-deviation | Unix non-UTF-8 paths remain lossless. See CT-RS-012. |
| actual `spawnSync("git"/"hg")` execution | `VcsRunner` trait only | blocked | Production provider must prove executable resolution, environment/config isolation, no shell, deadlines, bounded output, and descendant cleanup. See CT-RS-013. |
| recursive `rmSync(root/.git)` | `GitDirectoryCleaner` trait only | blocked | Production provider must prove no-follow, root identity, repository ownership, bounded traversal, and Windows reparse-point behavior. See CT-RS-014. |
| inherited Git templates/config/hooks | no provider yet | blocked | Git init can copy configured templates and Git commit can execute configured hooks; production execution must isolate or explicitly approve this behavior. |

## Default-example routing tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| exported `Set(["basic", "default"])` | `DEFAULT_EXAMPLES` array | implemented | Preserves the two values and source iteration order without mutable global state. |
| `Set.has(example)` | `is_default_example(&str)` | implemented | Exact `basic` and `default` membership only. |
| case-sensitive membership | borrowed literal match | implemented | `Basic`, `BASIC`, `Default`, and `DEFAULT` remain false. |
| no trimming or normalization | borrowed literal match | implemented | Whitespace, controls, composed/decomposed Unicode, joiners, and full-width/confusable forms remain false. |
| arbitrary prefix/suffix/path-like input | borrowed literal match | implemented | No substring, regex, path, or fuzzy matching can broaden the default acquisition route. |
| large untrusted example name | no allocation and two literal comparisons | intentional-hardening | Rust borrows the input and does not construct a set, normalized copy, or regex. See CT-RS-016. |
| use inside `create` acquisition orchestration | TypeScript caller remains | partial | Rust predicate is not yet bound into the production command path. |

## Existing TypeScript test mapping

| TypeScript test group | Rust test coverage | Status |
| --- | --- | --- |
| README compound/bare replacements | README parity tests | implemented |
| README subcommand preservation, prose isolation, fenced/inline regions, identity, `npx` | README parity tests | implemented |
| README missing inputs and read-transform-write | README parity tests | implemented |
| `DEFAULT_IGNORE` contains `.turbo` | `default_ignore_matches_the_typescript_constant` | implemented and strengthened to exact bytes |
| `.gitignore` transform source branches | five translated source-contract tests | implemented |
| Git/Mercurial detection and exact initialization order | nine Git initialization parity tests | implemented core |
| Git initialization root/cleanup regressions absent from the TypeScript suite | seven Git initialization security tests | intentional-deviation evidence |
| `isDefaultExample` source contract without direct Jest coverage | six translated parity tests | implemented and stronger than the source suite |
| default-route confusable/prefix/control/large-input regressions | five robustness/security tests | intentional-hardening evidence |
| symlink/race/resource regressions absent from TypeScript transform suite | transform security tests | intentional-deviation evidence |

## Remaining `create-turbo` surfaces

| Surface | Status | Required closure |
| --- | --- | --- |
| CLI argument parsing and help/version output | not-implemented | Translate CLI fixtures and process-level output/exit tests. |
| interactive prompts | not-implemented | Preserve defaults, cancellation, validation, non-TTY behavior, and ordering. |
| example resolution and download | partial | Exact default-route predicate is ported; discovery, GitHub/network/archive providers, redirects, extraction, and atomic promotion remain. |
| project creation orchestration | partial | A coordinator exists in `turbo-utils-rs`; `create-turbo` integration and differential tests remain. |
| Git initialization and commit | implemented core, providers blocked | Add secure Git/Hg runner and cleanup providers, TypeScript differential fixtures, Windows behavior, binding, and production routing. |
| `git-ignore` transform | implemented core | Add native binding, differential host tests, production routing, and TypeScript removal proof. |
| `official-starter` transform | not-implemented | Translate package/workspace mutations with deterministic JSON ordering. |
| package-manager transform | not-implemented | Preserve lockfile and package-manager metadata behavior. |
| telemetry integration | partial | The package telemetry Rust core is consolidated; bind it without retaining business logic in TypeScript. |
| npm/native packaging | blocked | Build, sign, publish, select, and roll back Rust binaries on every supported platform. |
| TypeScript removal | blocked | Migrate every downstream caller and prove the old runtime is neither loaded nor shipped. |
