# create-turbo Rust migration security review

This review covers the current Rust port of `packages/create-turbo/src/transforms/update-commands-in-readme.ts`. It is a tranche review, not a claim that the full `create-turbo` package has been audited or migrated.

## Trust boundaries

Attacker-influenced or repository-controlled inputs include:

- the selected package-manager value;
- the project root path;
- the existence, type, size, bytes, permissions, and replacement of `README.md`;
- Markdown code regions and all text inside them;
- concurrent filesystem changes while the transform is reading or writing;
- pre-created temporary filenames in the project directory.

The transform does not perform network access, package acquisition, subprocess execution, credential handling, telemetry, archive extraction, or privileged operations. Those package-level boundaries remain outside this tranche.

## Findings and fixes

### CT-RS-001: Unbounded README processing can exhaust memory and CPU

**Severity:** Medium

**TypeScript behavior:** the production transform reads the complete `README.md` into memory and runs whole-document regular-expression replacements without an explicit size limit.

**Impact:** a generated or attacker-controlled repository can provide a very large README and consume excessive memory and processing time during project creation.

**Rust fix:** both in-memory and filesystem inputs are limited to 4 MiB. The parser performs a bounded linear scan and does not compile or execute attacker-controlled regular expressions.

**Regression tests:**

- `rejects_oversized_in_memory_markdown`
- `rejects_oversized_readme_without_modifying_it`
- `unmatched_fence_is_bounded_and_left_unchanged`

**Residual risk:** the limit is a policy value and may need adjustment based on production fixtures. Changing it requires fixture and denial-of-service review.

### CT-RS-002: Malformed UTF-8 is silently replaced by the TypeScript path

**Severity:** Low

**TypeScript behavior:** Node's UTF-8 string decoding can replace malformed byte sequences, after which the file is written back and the original bytes are lost.

**Impact:** a malformed README can be silently corrupted even when the package-manager text is unrelated to the malformed region.

**Rust fix:** invalid UTF-8 is rejected before transformation or writing. The original file remains byte-for-byte unchanged.

**Regression test:** `rejects_invalid_utf8_without_modifying_it`

**Intentional compatibility difference:** this is stricter than the TypeScript implementation. A future binding must map the failure to the established non-fatal transform error contract rather than silently decoding replacement characters.

### CT-RS-003: Symlink following can modify a file outside the generated project

**Severity:** High

**TypeScript behavior:** `existsSync`, `readFile`, and `writeFile` follow a symlinked project root or `README.md` by default.

**Impact:** when project contents or the destination directory are attacker-controlled, the transform can overwrite an external file reachable through a symbolic link.

**Rust fix:** the provided project root must be a real directory rather than a symlink, and `README.md` must be a real regular file. Unix builds compare device and inode metadata after opening and again before replacement to detect ordinary path substitution races.

**Regression tests:**

- `rejects_non_regular_readme_paths`
- `rejects_symlinked_readme_without_touching_target`
- `rejects_symlinked_project_root`

**Residual risk:** standard path-based filesystem APIs cannot completely eliminate malicious concurrent replacement without descriptor-relative platform APIs. Windows identity checks remain incomplete and block production cutover.

### CT-RS-004: In-place truncating writes can leave a partial README

**Severity:** Medium

**TypeScript behavior:** the transform writes directly to `README.md`. A process crash, storage failure, or interrupted write after truncation can leave a partially written or empty file.

**Impact:** generated project documentation can be corrupted, and retries may no longer have the original input.

**Rust fix:** transformed content is written to a same-directory file opened with `create_new`, synchronized, assigned the original permissions, revalidated against the original root and README identity, and then moved into place. Temporary files are removed on ordinary failure paths.

**Regression tests:**

- `preserves_existing_readme_permissions`
- `successful_write_leaves_no_temporary_files`
- the oversized and invalid-UTF-8 tests verify that rejected inputs do not modify the original file.

**Residual risk:** on Unix, replacement is atomic but ownership, ACLs, extended attributes, and hard-link identity are not fully preserved by a portable standard-library rename. On Windows, the current standard-library fallback removes the target before rename and is not atomic. A reviewed platform-specific replacement implementation and metadata policy are required before production cutover.

### CT-RS-005: Temporary-file substitution and collisions

**Severity:** Low

**TypeScript behavior:** not applicable because the current implementation writes in place.

**Rust risk introduced by the safer write strategy:** a predictable temporary pathname could otherwise be pre-created or redirected.

**Rust fix:** temporary files use a process and monotonic sequence suffix, are opened with `create_new`, and are never followed when already present. Creation is bounded to 32 attempts.

**Regression test:** `successful_write_leaves_no_temporary_files`

**Residual risk:** filenames are not intended to be secret. Directory write access still permits denial of service by exhausting all attempts, which returns a deterministic error without modifying the original README.

### CT-RS-006: Regex behavior must not broaden substitutions into prose or identifiers

**Severity:** Low

**Impact:** an incorrect port could rewrite prose, package names embedded inside identifiers, `npx`, or command syntax, damaging generated documentation.

**Rust fix:** the scanner preserves the TypeScript region precedence and JavaScript-style ASCII word-boundary behavior. It applies the same ordered compound and bare-manager passes, including the JavaScript whitespace plus `run` negative lookahead.

**Regression tests:** all 12 parity tests in `readme_transform_parity.rs`, especially prose isolation, subcommand preservation, identity replacement, and the realistic `npx` fixture.

## Security invariants

- Production Rust code in this tranche contains no `unsafe`, shell invocation, subprocess execution, network access, package acquisition, or credential handling.
- No new third-party Rust dependency is introduced.
- The transformer accepts only the four enumerated package managers.
- Untrusted input size is bounded before output allocation and before filesystem writes.
- Rejected inputs do not intentionally modify the original README.
- A symlinked root or README is an error rather than a compatibility path.
- Security incompatibilities are recorded in this file and `PARITY_MATRIX.md` and have regression tests.

## Advisory lookup

**Lookup date: 2026-08-31**

Sources checked:

- RustSec Advisory Database: <https://rustsec.org/>
- RustSec advisory repository: <https://github.com/RustSec/advisory-db>
- GitHub Advisory Database, Rust ecosystem: <https://github.com/advisories?query=ecosystem%3Arust>
- Rust Project security policy and published Rust advisories: <https://www.rust-lang.org/policies/security> and <https://github.com/rust-lang/rust/security>

Disposition:

- This tranche adds no external crates and executes no external tools, so there is no new package-specific advisory exposure to resolve.
- It relies on stable standard-library filesystem, UTF-8, and string APIs. The reviewed code does not invoke Windows batch files, `remove_dir_all`, or Cygwin path classification, which are the boundaries highlighted by currently published Rust Project advisories visible during the lookup.
- The repository-wide lockfile audit remains authoritative for transitive workspace dependencies.
- The existing repository advisory for `webbrowser` (`RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`) is unrelated to this transform but remains an open repository blocker until upgraded or removed.

The advisory lookup must be repeated before merge if dependencies, externally executed tools, archive handling, network access, or platform-specific filesystem code are added.

## Production cutover blockers

- map typed Rust failures to the existing JavaScript `TransformError` contract;
- execute TypeScript-versus-Rust differential fixtures on Linux, macOS, and Windows;
- implement atomic Windows replacement and decide ACL/ownership/extended-attribute preservation;
- integrate the transform into the Rust `create-turbo` orchestration path;
- migrate downstream callers and package/release entry points;
- prove through artifact tests that this TypeScript transform is no longer loaded or shipped before deleting it.
