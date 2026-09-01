# turbo-workspaces Rust migration security review

This review covers only the read-only workspace-details orchestration core. It does not approve production filesystem, parser, process, binding, packaging, or removal behavior.

## Trust boundaries

The entry point accepts a caller-controlled root path. A directory provider resolves that input and returns the path that manager detectors and readers may inspect. Manager order controls which parser receives authority over repository metadata. Detector and parser failures may indicate malformed, conflicting, or adversarial state and must not be converted into permission for another parser.

## Findings and fixes

### TW-RS-001: Mutable manager order can broaden parser authority

**Severity:** Medium

TypeScript iterates `Object.values(MANAGERS)`, so behavior depends on registry insertion order. A loose port, mutable registry, or fallback list could let a different parser interpret the same repository.

**Rust fix:** manager identity is a closed enum and `MANAGER_DETECTION_ORDER` is a fixed six-element array in exact source order: `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`.

**Regression tests:** `manager_order_matches_the_typescript_registry` and `manager_identity_is_closed_ascii_data`.

### TW-RS-002: Detector or selected-reader errors must not trigger parser fallback

**Severity:** Medium

The TypeScript function does not catch detector or selected-reader failures. Retrying with a less appropriate manager could reinterpret malformed or conflicting files and hide the authoritative failure.

**Rust fix:** provider errors propagate immediately. Only a clean `false` permits the next detector. A successful detection grants exactly one read; a read error terminates the operation.

**Regression tests:** `selected_manager_read_failure_propagates_without_parser_fallback`, `detection_error_stops_without_trying_a_less_trusted_parser`, and `false_detectors_never_receive_read_authority`.

### TW-RS-003: Raw caller paths must not bypass the validated path boundary

**Severity:** High until provider closure

Using the original relative or user-facing string after validation could inspect a different location when the current directory changes, when aliases differ, or when path components are replaced.

**Rust fix:** after `directory_info`, every detector and reader receives only `WorkspaceDirectoryInfo::absolute`. The original path is used only for the directory-provider call.

**Regression test:** `the_provider_absolute_path_is_the_only_path_given_to_managers`.

**Residual risk:** an absolute pathname is not a stable filesystem identity. Production providers still need handle-relative or equivalent identity-preserving operations and Windows reparse-point handling.

### TW-RS-004: Detection and parser work must be bounded

**Severity:** Low

An extensible or attacker-influenced registry could cause unbounded detector work or grant read authority to arbitrary adapters.

**Rust fix:** the core can make at most six detection calls and one read. Manager names are static lowercase ASCII data and cannot become executable or parser identifiers supplied by the caller.

**Regression tests:** `unable_to_detect_work_is_bounded_to_the_fixed_registry` and `manager_identity_is_closed_ascii_data`.

### TW-RS-005: Detection and reading remain a filesystem TOCTOU boundary

**Severity:** High until provider closure

Files and path components can change between manager detection and manager-specific reading. The orchestration core intentionally cannot solve that with path strings.

**Required production fix:** detectors and readers must share stable root/file identities or a private snapshot, reject symlinks and special files where unsafe, enforce byte/count/depth limits, define concurrent modification behavior, and test Unix links plus Windows reparse points.

## TDD evidence

- TypeScript oracle: `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`
- TypeScript focused result: 5 of 5 passed in workflow run `33551576871`, job `100002027683`
- Rust RED: `2d4cc22e6a821c88882a87d604746dabbaa95fe2`
- Rust GREEN: `263ddc22d5b5f544768f4e089c92892339b0dce8`
- Rust evidence: 6 parity tests and 5 security tests

The RED commit compiled by construction around the final public API and deliberately omitted manager iteration. Hosted Rust execution is pending because GitHub currently has no active runner for the queued job; this is recorded as a validation blocker, not a pass.

## Security invariants

- No caller-controlled manager name exists in the core API.
- No detector or reader runs before the directory provider succeeds and reports existence.
- Only a clean false detector result permits the next manager.
- At most six detectors and one reader can run.
- The raw caller path is never passed to a manager after directory resolution.
- No shell, subprocess, network, parser, filesystem operation, credential source, logging sink, dependency, mutable global registry, or `unsafe` block is introduced by this core.
- Known public error type strings and messages remain deterministic.

## Advisory lookup

**Lookup date: 2026-09-01**

This tranche adds no third-party dependency. The RustSec advisory database, GitHub Advisory Database, and the existing repository dependency findings therefore introduce no new package-specific disposition for this core. The repository-wide `webbrowser`, `h2`, and `quick-xml` findings remain open and are not ignored or weakened.

## Production blockers

- root-workspace integration and authoritative Rust format/check/test/Clippy execution;
- bounded no-follow directory and metadata reads;
- stable filesystem identity between detection and parsing;
- parser byte, collection, and nesting limits for every manager;
- exact asynchronous JavaScript error and ordering bridge;
- cancellation and concurrent-modification policy;
- Linux, macOS, and Windows differential fixtures;
- native/minimal host binding, package publication, provenance, rollback, and downstream cutover;
- artifact tests proving the TypeScript implementation is neither loaded nor shipped.
