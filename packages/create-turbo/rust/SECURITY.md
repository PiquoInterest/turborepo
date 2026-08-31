# create-turbo Rust migration security review

This review covers the current Rust ports of:

- `packages/create-turbo/src/transforms/update-commands-in-readme.ts`
- `packages/create-turbo/src/transforms/git-ignore.ts`
- `packages/create-turbo/src/transforms/package-manager.ts`
- the shared `DEFAULT_IGNORE` constant in `src/utils/git.ts`
- the dependency-injected orchestration core for `tryGitInit` in `src/utils/git.ts`
- `packages/create-turbo/src/utils/is-default-example.ts`

It is a tranche review, not a claim that the full `create-turbo` package has been audited or migrated.

## Trust boundaries

Repository-controlled or attacker-influenced inputs include the selected package manager, package-manager version text, project-root path, example name, `README.md`, `.gitignore`, Markdown bytes, package and lockfile contents, file types, links, permissions, concurrent path replacement, and pre-created temporary filenames.

The example name determines whether acquisition is classified as a built-in default path. That routing decision must use exact membership so prefixes, paths, Unicode lookalikes, whitespace, controls, or normalization do not silently broaden trust.

The package-manager transform decides whether broad workspace mutation is invoked and which manager adapter receives control. The reviewed Rust core carries only a closed manager enum, a borrowed root path, and `skip_install: true`. Actual package metadata, lockfile, configuration, and process effects remain behind `PackageManagerConverter` and are not yet production-approved.

The Git initialization tranche adds decision boundaries for the project-root path, Git and Mercurial executable selection, process working directory, arguments, inherited environment and VCS configuration, template directories, hooks, timeouts, output, child-process cleanup, `.git` ownership, and recursive deletion.

The current Git core performs no subprocess execution or filesystem deletion itself. Those effects remain behind `VcsRunner` and `GitDirectoryCleaner`, so the production providers cannot be mistaken for reviewed merely because the command sequence is implemented.

## Findings and fixes

### CT-RS-001: Unbounded README processing

**Severity:** Medium

The TypeScript transform reads the complete README and applies whole-document regular expressions without an explicit limit. A generated repository can therefore cause excessive memory and CPU use.

The Rust implementation limits both in-memory and filesystem input to 4 MiB and uses a linear scanner.

Regression tests: `rejects_oversized_in_memory_markdown`, `rejects_oversized_readme_without_modifying_it`, and `unmatched_fence_is_bounded_and_left_unchanged`.

### CT-RS-002: Malformed UTF-8 can be silently rewritten

**Severity:** Low

Node string decoding can replace malformed byte sequences before rewriting the file. Rust rejects malformed UTF-8 and leaves the original bytes unchanged.

Regression test: `rejects_invalid_utf8_without_modifying_it`.

### CT-RS-003: README symlink following can modify an external file

**Severity:** High

The TypeScript read/write path follows a symlinked root or README. Rust requires a real root directory and regular README, and Unix builds compare device/inode identity after opening and before replacement.

Regression tests: `rejects_non_regular_readme_paths`, `rejects_symlinked_readme_without_touching_target`, and `rejects_symlinked_project_root`.

Residual risk: portable path APIs do not close every malicious concurrent-replacement race. Descriptor-relative operations and Windows identity handling remain cutover blockers.

### CT-RS-004: In-place README writes can leave a partial file

**Severity:** Medium

The TypeScript transform truncates and writes the original path. Rust writes a newly created sibling file, synchronizes it, applies the original permissions, revalidates the root and README, and then replaces the original. Ordinary failures remove the temporary file.

Regression tests: `preserves_existing_readme_permissions`, `successful_write_leaves_no_temporary_files`, and all rejected-input unchanged tests.

Residual risk: portable replacement does not preserve every ownership, ACL, extended-attribute, or hard-link property. The Windows fallback is not atomic.

### CT-RS-005: README temporary-file substitution and collisions

**Severity:** Low

The Rust safety strategy introduces temporary files. They use process/monotonic suffixes, `create_new`, and a 32-attempt bound. Existing names are never followed or overwritten.

Regression test: `successful_write_leaves_no_temporary_files`.

### CT-RS-006: Command rewriting can broaden into prose or identifiers

**Severity:** Low

A faulty port could rewrite prose, `npx`, or embedded identifiers. The Rust scanner preserves the TypeScript region precedence, ordered replacements, JavaScript ASCII word-boundary behavior, and whitespace-plus-`run` exclusion.

Evidence: all README parity tests.

The shared TypeScript package-manager type also admits `nub` and `aube`, while the source regex scans only `pnpm`, `npm`, `yarn`, and `bun`. The Rust README core currently models the four scanned spellings. This is recorded as an unresolved differential target-type contract, not silently treated as full shared-type parity.

### CT-RS-007: `.gitignore` check/write race can overwrite a concurrent path

**Severity:** Medium

The TypeScript transform performs `existsSync` and then `writeFileSync`. A destination can appear between those operations; the write call is overwrite-capable.

Rust writes the exact constant to a newly created sibling temporary file, synchronizes it, and publishes through `hard_link`, which fails when any destination already exists. A concurrent regular path wins and is returned as `not-applicable`; it is never overwritten.

Regression tests: `regular_existing_file_is_never_overwritten` and `successful_creation_has_only_the_expected_file`.

Residual risk: a malicious actor with write access can continuously win publication and cause denial of service. That is preferable to overwriting their path.

### CT-RS-008: Broken `.gitignore` symlink can create or overwrite an external target

**Severity:** High

`existsSync` returns false for a broken symlink. The subsequent TypeScript write follows the link and can create the target outside the generated project. An existing symlink is also treated as an ordinary already-present path, hiding an unsafe project state.

Rust uses `symlink_metadata` and rejects both broken and existing destination symlinks. It also rejects a symlinked project root.

Regression tests: `broken_symlink_is_rejected_without_creating_its_external_target`, `existing_symlink_is_rejected_without_modifying_its_target`, and `symlinked_project_root_is_rejected_without_writing_through_it`.

### CT-RS-009: `.gitignore` publication must not expose partial content

**Severity:** Low

Writing directly to the final path exposes a partially written file to concurrent readers. Rust fully writes and synchronizes the temporary inode before linking it under `.gitignore`.

Regression test: `successful_creation_has_only_the_expected_file`.

### CT-RS-010: Project-root replacement remains a descriptor-relative gap

**Severity:** Medium

The Rust implementation revalidates root identity, but a malicious concurrent actor may still exchange path components between path-based checks and filesystem operations. This cannot be completely solved with portable standard-library path APIs.

Current mitigation: reject root symlinks, compare root identity on Unix, use no-overwrite target publication, and never follow destination symlinks.

Required closure: descriptor-relative directory handles on Unix and reviewed Windows handle-based operations before the Rust transform becomes the production path in attacker-writable directories.

### CT-RS-011: The TypeScript Git path blacklist rejects harmless values but misses structural hazards

**Severity:** Medium

The TypeScript implementation rejects characters such as `$`, `#`, `;`, and `!` even though `spawnSync` receives an argument vector and does not construct a shell command. That is a compatibility failure rather than an injection defense. At the same time, the check does not reject relative roots, filesystem roots, parent components, controls, or characters such as `?` that are invalid in Windows filenames.

The Rust core validates path structure instead of shell syntax. It requires an absolute non-root path, rejects current/parent components, controls, and Windows-invalid filename characters, and permits harmless shell metacharacters because no shell is involved.

Regression tests: `rejects_relative_roots_before_any_subprocess`, `rejects_filesystem_roots_before_any_subprocess_or_cleanup`, `rejects_parent_components_before_any_subprocess`, `rejects_control_and_windows_invalid_filename_characters`, and `shell_metacharacters_are_not_treated_as_injection_without_a_shell`.

### CT-RS-012: Stringifying the project root can corrupt or reject valid Unix paths

**Severity:** Low

The first RED draft placed the project-root string directly in Mercurial arguments. That was not the TypeScript contract, which uses `--cwd .` and sets the process working directory to the root. It would also require lossy or fallible conversion for non-UTF-8 Unix paths.

The corrected Rust contract carries the root as `PathBuf` in `VcsInvocation.cwd` and retains literal `--cwd . root` arguments.

Regression tests: `returns_false_when_inside_mercurial_repository` and Unix-only `non_utf8_roots_do_not_require_lossy_argument_conversion`.

### CT-RS-013: Production VCS execution can inherit executable, environment, template, and hook behavior

**Severity:** High until the provider contract is closed

The TypeScript implementation launches `git` and `hg` by command name and inherits process environment and user/system VCS configuration. Git documents that `git init` may take templates from `GIT_TEMPLATE_DIR` or `init.templateDir`, and `git commit` may execute commit-related hooks. A hostile executable selected through `PATH`, a configured template, or a configured hook can therefore execute code during project creation.

The current Rust tranche intentionally provides no production runner. Required closure includes canonical executable resolution, an explicit environment/config policy, no shell, bounded duration and output, descendant cleanup, and tests proving the accepted template/hook behavior. Simply adding `--no-verify` would not be sufficient because Git documents a `prepare-commit-msg` hook that is not suppressed by that option.

Authoritative references:

- <https://git-scm.com/docs/git-init>
- <https://git-scm.com/docs/git-commit>
- <https://git-scm.com/docs/githooks>

### CT-RS-014: Recursive `.git` cleanup needs an ownership and no-follow contract

**Severity:** High until the provider contract is closed

After a successful `git init`, later failure requires cleanup. A naive recursive path deletion can cross a symlink/reparse point, delete a replaced path, or remove a repository the operation did not create. Conversely, deleting after a failed `git init` is unsafe because ownership is ambiguous.

The orchestration core requests cleanup only after `git init` returned success and a later command failed. It never requests cleanup after init failure. The production cleaner remains blocked until it proves root identity, `.git` ownership, no-follow traversal, bounded work, ordinary failure handling, and Windows reparse-point behavior.

Regression tests: the checkout/add/commit cleanup tests, `cleanup_failure_is_swallowed_like_the_typescript_implementation`, and `init_failure_does_not_delete_an_unowned_or_ambiguous_git_directory`.

### CT-RS-015: Git initialization oracle drift could silently change generated history

**Severity:** Low

The first RED draft used a different commit message and added an unobserved `git --version` call. It also changed the Mercurial argument/cwd split. These differences were corrected while the implementation still returned `false`, preserving a genuine RED-first history against the source contract.

Regression tests: `initial_commit_message_matches_the_typescript_source`, `returns_false_when_inside_mercurial_repository`, `returns_false_when_git_init_is_unavailable_or_fails`, and `runs_the_exact_typescript_command_sequence_on_success`.

### CT-RS-016: Default-example routing must not broaden beyond exact membership

**Severity:** Medium

`isDefaultExample` controls whether the selected example is classified as one of the built-in default acquisition paths. Replacing exact `Set.has` behavior with trimming, case folding, Unicode normalization, substring matching, path matching, or a permissive regex could route an attacker-controlled name through a more trusted code path.

The Rust implementation exposes the exact source-order literals and uses `matches!(example, "basic" | "default")` over a borrowed `&str`. It performs no allocation, mutable global lookup, trimming, normalization, or pattern expansion.

Parity tests prove exact values, case sensitivity, whitespace sensitivity, and non-default rejection. Robustness/security tests reject prefixes, suffixes, path-like values, controls, NUL, Unicode confusables, normalization variants, joiners, and a 4 MiB arbitrary input.

Residual risk: production routing still calls the TypeScript helper. This fix protects only the Rust core until the caller is bound and differentially tested.

### CT-RS-017: Package-manager conversion mutates multiple workspace artifacts without a proven transaction

**Severity:** High until the provider contract is closed

The TypeScript `package-manager` transform delegates to `@turbo/workspaces.convert`. The reviewed conversion flow performs manager-specific cleanup and creation and updates package metadata and manager-owned files across multiple steps. No single atomic commit or repository-wide rollback contract is evident in that orchestration. A failure after early mutation can therefore leave a partially converted workspace unless every adapter supplies complete recovery behavior.

The Rust tranche intentionally implements only the decision and typed request core. It cannot read or write files, launch a package manager, or alter a lockfile. The mutation boundary is explicit as `PackageManagerConverter`.

A production provider must prove, with translated and failure-injection tests:

- the complete six-manager source/target matrix;
- root containment and no-follow behavior for package and lockfile paths;
- bounded reads, writes, parsing, subprocess time, and output;
- safe executable resolution without shell construction or project-local substitution;
- staged writes plus atomic promotion, or a documented rollback journal that restores every touched artifact;
- cleanup ownership and concurrent-path behavior;
- exact package metadata, lockfile, workspace, and manager configuration semantics;
- Linux, macOS, and Windows behavior;
- deterministic public error mapping and a non-success result on any partial failure.

Regression tests in the current core prove that no provider call occurs for absent or unchanged selections, exactly one typed call occurs for a change, `skip_install` is always true, all six variants are closed enum values, and converter errors cannot become success.

### CT-RS-018: Prompt package-manager version is present but intentionally not forwarded

**Severity:** Informational logic contract

The selected package manager includes optional version text, but the TypeScript transform forwards only the selected manager name to `convert`. A caller could incorrectly assume that the selected version pins conversion output. The Rust core preserves the observed source behavior rather than inventing version semantics.

`PackageManagerSelection.version` is borrowed for parity documentation but is not copied, logged, normalized, parsed, or passed to `PackageManagerConverter`. This also avoids accidentally propagating a large, control-containing, or sensitive value into process arguments or logs.

Required production closure: differential tests must prove whether version is purely prompt/display metadata or whether a later source stage owns pinning. Any decision to start enforcing a version is a deliberate compatibility change and needs its own RED contract, security review, and migration note.

Regression tests: `prompt_version_is_not_forwarded_to_the_converter`, `a_large_untrusted_version_is_borrowed_and_not_forwarded`, and `no_mutating_provider_call_occurs_when_the_selection_is_absent_or_unchanged`.

## Security invariants

- No new `unsafe` or shell command construction is introduced by these tranches.
- README, `.gitignore`, default-example, and package-manager orchestration add no network or credential behavior.
- The default-example route uses exact borrowed ASCII literals only.
- The package-manager core accepts a closed enum, preserves the root as a path, does not forward version text, and cannot mutate files or execute a process directly.
- The Git orchestration core does not execute a subprocess or delete a path directly; those effects remain behind unimplemented production providers.
- Untrusted README size is bounded before allocation and writing.
- Rejected README inputs remain unchanged.
- Existing `.gitignore` content is never overwritten.
- Broken or existing destination symlinks are errors.
- Temporary files use `create_new`, bounded retries, and ordinary failure cleanup.
- VCS roots are structurally validated before any provider invocation.
- Git cleanup is requested only after successful init and later command failure.
- Every intentional incompatibility is recorded here and in `PARITY_MATRIX.md` with regression coverage.

## Advisory lookup

**Lookup date: 2026-08-31**

Authoritative sources checked:

- RustSec Advisory Database: <https://rustsec.org/>
- RustSec advisory repository: <https://github.com/RustSec/advisory-db>
- GitHub Advisory Database, Rust ecosystem: <https://github.com/advisories?query=ecosystem%3Arust>
- Rust Project security policy and advisories: <https://www.rust-lang.org/policies/security> and <https://github.com/rust-lang/rust/security>
- Git command, initialization, and hook documentation: <https://git-scm.com/docs/git-init>, <https://git-scm.com/docs/git-commit>, and <https://git-scm.com/docs/githooks>

Disposition:

- The package-manager orchestration tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.
- A production converter cannot be approved until its manager adapters, executable versions, filesystem/process policies, transaction or rollback model, and supported platforms are reviewed.
- The default-example tranche adds no dependency, parser, network call, filesystem operation, subprocess, or mutable global state.
- The Git core adds no external Rust crate and does not yet execute an external tool.
- A production Git/Hg provider cannot be approved until its exact executable versions, resolution path, environment/config policy, and supported platforms are reviewed.
- The repository-wide lockfile audit remains authoritative for transitive workspace dependencies.
- The existing `webbrowser`, `h2`, and `quick-xml` advisories remain repository-level blockers and are not suppressed by this tranche.

Repeat the lookup before merge when dependencies, subprocess providers, network access, archives, or platform-specific filesystem APIs change.

## Production cutover blockers

- map typed failures to the existing JavaScript public contracts;
- bind `is_default_example` into acquisition orchestration and compare TypeScript/Rust routing over shared fixtures;
- implement the production `PackageManagerConverter` with complete six-manager parity, rollback or atomic promotion, and failure injection;
- prove the shared README target behavior for `nub` and `aube` without broadening the source's four scanned command spellings;
- run TypeScript-versus-Rust differential host fixtures on Linux, macOS, and Windows;
- implement handle-relative publication and atomic Windows replacement with an explicit metadata/ACL policy;
- implement and review production Git/Hg runner and `.git` cleanup providers;
- isolate or deliberately preserve Git templates, global/system configuration, hooks, signing, and credential-helper behavior;
- integrate the transforms and provider cores into Rust orchestration;
- migrate package entry points and downstream callers;
- prove through artifact/removal tests that the TypeScript implementation is neither loaded nor shipped before deletion.
