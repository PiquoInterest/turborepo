# create-turbo TypeScript-to-Rust parity matrix

Status values are `implemented`, `intentional-deviation`, `intentional-hardening`, `partial`, `blocked`, and `not-implemented`. A row marked `implemented` means the listed safe-input behavior has translated tests. It does not imply that the package is the production entry point.

## `update-commands-in-readme` tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| supported command spellings `pnpm`, `npm`, `yarn`, `bun` | `PackageManager` | implemented | All four managers named by the source transform's replacement patterns are covered by translated tests. |
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
| shared `PackageManager` type also admits `nub` and `aube` | README transform enum currently models only four source replacement spellings | partial | The TypeScript regex intentionally scans only the four real package-manager command spellings while its target type is wider. Differential tests must decide the exact `nub`/`aube` target behavior before production binding. |
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
| inherited Git templates/config/hooks | no provider yet | blocked | Git init can copy configured templates and Git commit can execute commit-related hooks; production execution must isolate or explicitly approve this behavior. |

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

## Package-manager transform tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| transform metadata name `package-manager` | `PACKAGE_MANAGER_TRANSFORM_NAME` | implemented | Exact response name is covered in no-op and success tests. |
| package-manager union `yarn`, `npm`, `pnpm`, `bun`, `nub`, `aube` | `WorkspacePackageManager` | implemented core | All six repository variants and exact string spellings are translated. |
| no selected manager | `selection: None` | implemented | Returns `not-applicable` and never invokes the converter. |
| selected manager equals project manager | enum equality | implemented | Returns `not-applicable` and never invokes the converter, regardless of supplied version text. |
| selected manager differs | `PackageManagerConverter::convert` | implemented core | Exactly one typed conversion request is issued, and success is returned only after the provider succeeds. |
| `root: project.paths.root` | borrowed `&Path` in `PackageManagerConversion` | implemented | Root bytes are preserved, including non-UTF-8 Unix paths; the orchestration core performs no lossy conversion. |
| `to: packageManager.name` | closed `WorkspacePackageManager` enum | implemented | No free-form executable, package spec, or command text crosses the provider boundary. |
| `options: { skipInstall: true }` | `skip_install: true` | implemented | Exact source option is asserted for every manager target. |
| prompt `version` exists but source transform does not forward it | `PackageManagerSelection.version` borrowed but omitted from conversion | implemented | Large or control-containing version values are not copied, logged, or passed to the mutation provider. See CT-RS-018. |
| `convert` rejection | provider error propagation | implemented | The Rust core does not synthesize a success response after a converter error. |
| `@turbo/workspaces.convert` cleanup/create/package metadata/lockfile mutation | production `PackageManagerConverter` | blocked | Requires translated manager-specific tests, atomicity or rollback, no-follow filesystem handling, bounded process execution, and platform closure. See CT-RS-017. |
| TypeScript transform's untyped broad side effects | explicit mutation provider boundary | intentional-hardening | The reviewed core cannot execute a process or mutate a file directly. |

## Official-starter transform tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| no `example.repo` | `is_official_starter(None)` | implemented core | Preserves the source's built-in official route. |
| repository exactly `vercel/turbo` or `vercel/turborepo` | `ExampleRepository` plus exact borrowed-string matching | implemented core | Case, whitespace, prefixes, suffixes, paths, controls, and Unicode confusables do not broaden the route. |
| non-official repository returns before filesystem access | early `NotApplicable` response | implemented core | An exploding provider test proves no store method can run. |
| `existsSync(package.json)` before metadata handling | `package_json_exists` before `read_meta_json` | implemented core | Provider-call order is translated exactly. |
| `readJsonSync(meta.json)` followed by best-effort forced removal | `read_meta_json` then ignored `remove_meta_json` result | implemented core | Read failure returns no metadata and skips removal; removal failure still returns the parsed metadata. |
| missing `package.json` | source existence snapshot | implemented core | Returns success after metadata handling without package read/write. |
| package read error | `OfficialStarterError::ReadPackageJson` | implemented core | Exact public message, transform name, and nonfatal metadata are covered. |
| falsey parsed package value | `read_package_json` returns `None` | implemented core | Returns success without writing, matching the source guard. |
| `basic`/`default` package rename | `is_default_example` plus `set_name` | implemented core | Exact project name is forwarded as data. |
| truthy existing `devDependencies.turbo` | typed truthiness query plus setter | implemented core | Non-empty explicit version wins; absent or empty option becomes `^<invocation version>`. |
| truthy package object with no relevant field changes | unconditional provider write after successful read | implemented core | Preserves the source's write side effect and ordering. |
| package write error | `OfficialStarterError::WritePackageJson` | implemented core | Cannot become a false success. |
| `fs-extra` JSON parsing, ordering, deletion, and write behavior | production `OfficialStarterStore` | blocked | Requires bounded strict parsing, unknown-field/order preservation, JavaScript truthiness, no-follow paths, atomic publication, metadata policy, and supported-platform differentials. |
| public JavaScript `TransformError` instance | typed Rust error metadata | partial | Native/host binding must construct the exact public error class and stack-facing behavior. |

Detailed representation and security differences are in `OFFICIAL_STARTER_DIVERGENCES.md`.

## Transform-pipeline tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| exported transform array order | `TRANSFORM_PIPELINE` | implemented core | Exact four names and source order are translated. |
| `skipTransforms` guard | early empty report | implemented core | No executor method can run. |
| sequential `await` loop | one typed call per enum slot | implemented core | Async host bridge remains blocked. |
| optional string maintainer truthiness | non-empty `Option<String>` | implemented core | Empty is falsey; all non-empty strings are truthy. |
| nonfatal `TransformError` | recorded failure then continue | implemented core | Later steps still execute once. |
| fatal `TransformError` | typed fatal abort | implemented core | Later transforms are not invoked. |
| unknown error rethrow | typed unknown abort | implemented core | Unknown errors cannot be downgraded. |
| default and explicit error options | `TransformFailure` constructors | implemented core | `unknown`/true defaults and explicit empty/false values match nullish semantics. |
| logging, telemetry, exit and async adaptation | production host binding | blocked | Requires exact side effects, cleanup-before-exit, terminal-safe display, and platform differentials. |
| internal partial report | `TransformPipelineReport` | intentional-hardening | Bounded internal observability, not a new public API contract. |

Detailed differences are in `TRANSFORM_PIPELINE_DIVERGENCES.md`.

## Package-manager prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| `skipTransforms` early return | `resolve_package_manager_prompt` returns `None` | implemented core | Discovery and selector providers are untouched. |
| source choices `npm`, `pnpm`, `yarn`, `bun`, `nub`, `aube` | `PACKAGE_MANAGER_PROMPT_ORDER` | implemented core | Exact order and closed variants are tested. |
| requested installed manager | exact parse plus truthy version | implemented core | Bypasses selector like the source. |
| unknown or unavailable manager | selector path | implemented core | No free-form value crosses the typed boundary. |
| stable installed-first sort | stable sort by disabled state | implemented core | Relative order inside both groups is preserved. |
| empty discovered version | unavailable | implemented core | Matches JavaScript string truthiness. |
| selector cancellation/error | propagated once | implemented core | No retry or synthesized fallback. |
| disabled selection | explicit unavailable-selection error | intentional-hardening | Defense in depth beyond Inquirer's UI disable flag. |
| process discovery and interactive Inquirer behavior | production providers | blocked | Requires secure execution, cancellation/non-TTY/signal parity, terminal-safe UI, and platform differentials. |

Detailed differences are in `PACKAGE_MANAGER_PROMPT_DIVERGENCES.md`.

## Create-command error policy tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| transform, conversion, download, and unknown errors | closed typed classification | implemented core | Safe-input action and display order are translated. |
| immediate fatal `process.exit(1)` | typed `Exit(1)` | intentional-hardening | Allows cleanup and telemetry flush before termination. |
| raw unbounded terminal error text | bounded control-safe display fields | intentional-hardening | Unknown errors are never rendered. |
| logging, telemetry, stack/class identity, and termination | production host binding | blocked | Must consume sanitized fields only and prove exactly-once effects. |

Detailed differences are in `CREATE_ERROR_POLICY_DIVERGENCES.md`.

## Create installation and warning tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| source/selected manager resolution and install gates | `apply_create_install_policy` | implemented core | Preserves ordering, skip, warning, and noninteractive install behavior. |
| repeated mutable availability lookup | one borrowed snapshot | intentional-hardening | Removes a time-of-check/time-of-use ambiguity for unstable providers. |
| raw example name in warning output | bounded terminal-safe renderer | intentional-hardening | Safe wording is preserved; hostile controls and large text are escaped/truncated. |
| real installation and logger effects | typed providers/host binding | blocked | Requires secure runner, exact errors, and supported-platform differentials. |

Detailed differences are in `CREATE_INSTALL_POLICY_DIVERGENCES.md`.

## Create output rendering tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| workspace summary, success, and get-started safe text | pure line renderers | implemented core | Exact safe headings/items and fixed text are translated. |
| raw project/workspace/path/description fields | bounded terminal-safe fields | intentional-hardening | Prevents line forging, OSC/BEL, bidi, and invisible-control spoofing. |
| unbounded workspaces and scripts | 256 workspace and 64 script limits | intentional-hardening | Emits an explicit truncation record. |
| path-relative/group/locale derivation and coloring | host binding | blocked | Requires Linux/macOS/Windows differential fixtures and no raw logger bypass. |

Detailed differences are in `CREATE_OUTPUT_POLICY_DIVERGENCES.md`.

## Package-manager installation profile tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| eight source profiles and order/default selection | static profile tables | implemented core | All six managers and eight profiles are translated. |
| `semver.satisfies` | injected matcher | partial | Production binding must prove Node-semver behavior. |
| `preferLocal: true` | `prefer_local: false` | intentional-hardening | Blocks generated-project executable substitution. |
| Windows `shell: true` | `shell: false` | intentional-hardening | Requires an explicit safe Windows shim adapter or typed unsupported result. |
| process execution | typed invocation metadata only | blocked | Needs canonical resolution, environment policy, deadlines, output bounds, and tree cleanup. |

Detailed differences are in `PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md`.

## Project-directory prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| direct non-empty argument bypasses prompt without trimming | exact `Option<&str>` branch | implemented core | Preserves JavaScript truthiness for the empty-string case. |
| prompt message/default and display-only trim | typed prompt request | implemented core | Raw accepted answer is validated unchanged. |
| invalid direct result returned and ignored by caller | typed validator rejection | fixed in Rust and repaired TypeScript | Prevents acquisition against a validator-rejected path. |
| unbounded/control-bearing path input | 4096-byte and terminal-active-text rejection | intentional-hardening | Rejected values are not reflected in public core errors. |
| terminal and filesystem behavior | production `DirectoryPrompter`/`DirectoryValidator` | blocked | Needs bounded reading, cancellation, stable handles, Windows reparse-point behavior, and platform differentials. |

Detailed differences are in `DIRECTORY_PROMPT_DIVERGENCES.md`.

## Existing TypeScript test mapping

| TypeScript test or source contract | Rust test coverage | Status |
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
| package-manager transform source contract without focused direct Jest coverage | seven translated parity tests | implemented core |
| package-manager version/path/provider-boundary regressions | four security tests | intentional-hardening evidence |
| `official-starter` source contract without focused Jest coverage | sixteen translated parity tests | implemented core |
| official-route confusable/large-input/provider-boundary regressions | nine security tests | intentional-hardening evidence |
| create command transform-loop source contract | ten translated parity tests | implemented core |
| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |
| package-manager prompt source contract | eight translated parity tests | implemented core |
| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |
| create-command error routing and terminal bounds | seven parity and eight security tests | implemented core and intentional-hardening evidence |
| create install decision and warning rendering | fourteen parity and thirteen security tests | implemented core and intentional-hardening evidence |
| final create output rendering | six parity and six security tests | implemented core and intentional-hardening evidence |
| package-manager installation profiles | eight parity and five security tests | implemented core and intentional-hardening evidence |
| directory argument/prompt/validator behavior | eight parity and nine security tests plus TypeScript regressions | implemented core, repaired oracle, providers blocked |
| symlink/race/resource regressions absent from TypeScript transform suite | transform security tests | intentional-deviation evidence |

## Remaining `create-turbo` surfaces

| Surface | Status | Required closure |
| --- | --- | --- |
| CLI argument parsing and help/version output | not-implemented | Translate CLI fixtures and process-level output/exit tests. |
| interactive prompts | package-manager decision core implemented, providers blocked | Add secure manager discovery, Inquirer-compatible UI, cancellation/non-TTY/signal behavior, platform differentials, binding, and removal proof. |
| example resolution and download | partial | Exact default-route predicate is ported; discovery, GitHub/network/archive providers, redirects, extraction, and atomic promotion remain. |
| project creation orchestration | partial | A coordinator exists in `turbo-utils-rs`; `create-turbo` integration and differential tests remain. |
| Git initialization and commit | implemented core, providers blocked | Add secure Git/Hg runner and cleanup providers, TypeScript differential fixtures, Windows behavior, binding, and production routing. |
| `git-ignore` transform | implemented core | Add native binding, differential host tests, production routing, and TypeScript removal proof. |
| transform pipeline and error handling | implemented core, binding blocked | Add async host bridge, telemetry, terminal-safe logging, fatal-exit cleanup, JavaScript error mapping, platform differentials, and removal proof. |
| `official-starter` transform | implemented orchestration core, provider blocked | Add bounded no-follow JSON/filesystem provider, deterministic order-preserving serialization, atomic package publication, native binding, platform differentials, and removal proof. |
| package-manager transform | implemented orchestration core, provider blocked | Port and prove manager-specific conversion, package/lockfile mutation, rollback, process, and platform behavior. |
| README target behavior for `nub`/`aube` | partial | Preserve the source's four-spelling scan while proving the wider target type through differential fixtures. |
| telemetry integration | partial | The package telemetry Rust core is consolidated; bind it without retaining business logic in TypeScript. |
| npm/native packaging | blocked | Build, sign, publish, select, and roll back Rust binaries on every supported platform. |
| TypeScript removal | blocked | Migrate every downstream caller and prove the old runtime is neither loaded nor shipped. |
