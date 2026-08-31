# create-turbo Rust migration security review

This review covers the current Rust ports of:

- `packages/create-turbo/src/transforms/update-commands-in-readme.ts`
- `packages/create-turbo/src/transforms/git-ignore.ts`
- the shared `DEFAULT_IGNORE` constant in `src/utils/git.ts`

It is a tranche review, not a claim that the full `create-turbo` package has been audited or migrated.

## Trust boundaries

Repository-controlled or attacker-influenced inputs include the selected package manager, project-root path, `README.md`, `.gitignore`, Markdown bytes, file types, links, permissions, concurrent path replacement, and pre-created temporary filenames.

These tranches perform no network access, package acquisition, subprocess execution, credentials, telemetry, archives, or privileged operations. Those boundaries remain outside the current implementation.

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

Evidence: all 12 README parity tests.

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

## Security invariants

- No `unsafe`, shell invocation, subprocess, network, archive, package acquisition, credential, or telemetry code is introduced by these tranches.
- No new third-party Rust dependency is introduced.
- Untrusted README size is bounded before allocation and writing.
- Rejected README inputs remain unchanged.
- Existing `.gitignore` content is never overwritten.
- Broken or existing destination symlinks are errors.
- Temporary files use `create_new`, bounded retries, and ordinary failure cleanup.
- Every intentional incompatibility is recorded here and in `PARITY_MATRIX.md` with regression coverage.

## Advisory lookup

**Lookup date: 2026-08-31**

Authoritative sources checked:

- RustSec Advisory Database: <https://rustsec.org/>
- RustSec advisory repository: <https://github.com/RustSec/advisory-db>
- GitHub Advisory Database, Rust ecosystem: <https://github.com/advisories?query=ecosystem%3Arust>
- Rust Project security policy and advisories: <https://www.rust-lang.org/policies/security> and <https://github.com/rust-lang/rust/security>

Disposition:

- These tranches add no external crate or externally executed tool, so there is no new package-specific advisory exposure.
- They rely on standard-library string and filesystem APIs and do not invoke Windows batch files, Cygwin path classification, or process execution.
- The repository-wide lockfile audit remains authoritative for transitive workspace dependencies.
- The existing `webbrowser` finding (`RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`) is unrelated to these transforms but remains an open repository blocker.

Repeat the lookup before merge when dependencies, subprocesses, network access, archives, or platform-specific filesystem APIs change.

## Production cutover blockers

- map typed failures to the existing JavaScript `TransformError` contract and fatality metadata;
- run TypeScript-versus-Rust differential host fixtures on Linux, macOS, and Windows;
- implement handle-relative publication and atomic Windows replacement with an explicit metadata/ACL policy;
- integrate both transforms into Rust orchestration;
- migrate package entry points and downstream callers;
- prove through artifact/removal tests that the TypeScript transforms are neither loaded nor shipped before deletion.
