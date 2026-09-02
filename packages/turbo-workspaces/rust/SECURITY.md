# turbo-workspaces Rust migration security review

This review covers the committed Rust orchestration and pure parsing cores. It does not approve production filesystem, JSON/YAML parser, process, binding, packaging, mutation, rollback, or removal behavior.

## Trust boundaries

The package accepts caller-controlled roots, package metadata, package-manager declarations, workspace globs, and lockfile state. Manager order controls which parser receives authority over repository data. Workspace-glob values can later influence filesystem enumeration, diagnostics, and terminal output. The Rust cores therefore keep filesystem and parser authority behind providers and place explicit bounds on pure input extraction.

## Findings and fixes

### TW-RS-001: Mutable manager order can broaden parser authority

**Severity:** Medium

TypeScript iterates `Object.values(MANAGERS)`, so behavior depends on registry insertion order. A loose port, mutable registry, or fallback list could let a different parser interpret the same repository.

**Rust fix:** manager identity is a closed enum and `MANAGER_DETECTION_ORDER` is a fixed six-element array in exact source order: `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`.

### TW-RS-002: Detector or selected-reader errors must not trigger parser fallback

**Severity:** Medium

Retrying with another manager after a detector or selected-reader failure could reinterpret malformed or conflicting repository state and hide the authoritative failure.

**Rust fix:** provider errors propagate immediately. Only a clean `false` permits the next detector. A successful detection grants exactly one read.

### TW-RS-003: Raw caller paths must not bypass the validated path boundary

**Severity:** High until provider closure

Using the original relative or user-facing path after validation could inspect a different location when process state or path components change.

**Rust fix:** after `directory_info`, every detector and reader receives only `WorkspaceDirectoryInfo::absolute`. The original path is used only for the directory-provider call.

**Residual risk:** an absolute pathname is not a stable filesystem identity. Production providers still need handle-relative or equivalent operations and Windows reparse-point handling.

### TW-RS-004: Detection and parser work must be bounded

**Severity:** Low

An extensible or attacker-influenced registry could cause unbounded detector work or grant read authority to arbitrary adapters.

**Rust fix:** the orchestration core can make at most six detection calls and one read. Manager names are static lowercase ASCII data.

### TW-RS-005: Detection and reading remain a filesystem TOCTOU boundary

**Severity:** High until provider closure

Files and path components can change between manager detection and manager-specific reading.

**Required production fix:** detectors and readers must share stable root and file identities or a private snapshot, reject unsafe links and special files, enforce byte/count/depth limits, define concurrent modification behavior, and test Unix links plus Windows reparse points.

### TW-RS-006: Bun workspace-glob input can amplify work

**Severity:** Medium

The TypeScript compatibility predicate has no count, per-value, or aggregate byte bounds. Large package metadata could create excessive scanning and allocation before actual workspace expansion.

**Rust fix:** the Bun compatibility core rejects more than 256 values, any value above 4096 UTF-8 bytes, and aggregate input above 65536 bytes. Aggregate accounting uses checked arithmetic.

### TW-RS-007: Terminal-active and invisible Bun glob text is accepted by TypeScript

**Severity:** Medium

Control, bidi, and zero-width text can survive the TypeScript predicate and later influence diagnostics, logs, or path interpretation.

**Rust fix:** the pure Rust predicate rejects reviewed C0/C1, bidi, zero-width, and related format characters before later consumers receive the glob.

### TW-RS-011: Unbounded workspace arrays can amplify later filesystem work

**Severity:** Medium

`parseWorkspacePackages` returns the source array without count or byte-volume checks. A caller-controlled manifest can therefore create large allocations and pass a large pattern set into later glob expansion.

**Rust fix:** extraction rejects more than 256 values, any value above 4096 UTF-8 bytes, and aggregate input above 65536 bytes before copying the result vector.

**TypeScript evidence:** the focused oracle remains green by using `it.failing` for the stricter policy while separately asserting current legacy behavior.

### TW-RS-012: Terminal-active and invisible workspace values survive TypeScript parsing

**Severity:** Medium

The TypeScript helper passes control and format characters through verbatim. These values can later reach logs, diagnostics, path display, or glob processing.

**Rust fix:** extraction rejects the reviewed unsafe character classes. Typed errors expose only the category and item index; their display text never includes the attacker-controlled value.

### TW-RS-013: Mutable JavaScript workspace-array aliasing crosses the API boundary

**Severity:** Low

The TypeScript helper returns the original array object, so callers can mutate the package metadata through the result reference.

**Rust fix:** the core returns a new bounded `Vec<&str>` of immutable borrowed strings. Ordering and values are preserved without mutable array aliasing or string copying.

## Workspace-package parser TDD evidence

- TypeScript oracle commit: `9c8f77deee15c01baba73fdd510960e899756f0e`;
- compiling behavioral Rust RED: `089112a3f85bc2cbaaf864991eb5b6129602ff30`;
- Rust GREEN: `8b4aea45459aa09237aef7d8dd35ccf06503ae28`;
- Rust evidence: 7 parity tests and 6 security tests.

The RED source exported the final API and typed errors but returned an empty vector for every input. The GREEN commit adds bounded extraction only. It adds no third-party dependency, filesystem access, process execution, network access, logging sink, credential source, mutable global state, or `unsafe` code.

## Security invariants

- No caller-controlled manager name exists in the orchestration API.
- No detector or reader runs before the directory provider succeeds and reports existence.
- Only a clean false detector result permits the next manager.
- At most six detectors and one selected reader can run.
- The raw caller path is never passed to a manager after directory resolution.
- Pure glob parsers impose count, per-value, and aggregate byte limits before result publication.
- Public parser errors do not echo attacker-controlled workspace values.
- General workspace parsing preserves safe glob syntax and is not accidentally narrowed to Bun compatibility rules.
- The reviewed cores introduce no shell, subprocess, network, parser dependency, filesystem operation, credential source, mutable global registry, or `unsafe` block.

## Advisory lookup

**Lookup date: 2026-09-02**

The workspace-package tranche adds no dependency. It does not change or ignore any RustSec or GitHub Advisory Database finding. Repository-wide `webbrowser`, `h2`, and `quick-xml` findings remain separate blockers.

## Production blockers

- authoritative GitHub format, TypeScript oracle, Rust RED, GREEN test, and Clippy execution for the new parser tranche;
- bounded no-follow `package.json`, workspace configuration, and lockfile reads;
- stable filesystem identity between detection, parsing, expansion, and mutation;
- JSON/YAML parser byte, collection, alias, and nesting limits;
- root-confined workspace expansion with bounded matches and deterministic ordering;
- cancellation and concurrent-modification policy;
- staged publication and rollback after every injected failure point;
- Linux, macOS, and Windows differential fixtures;
- native or minimal host binding, package publication, provenance, downstream cutover, and artifact tests proving TypeScript is neither loaded nor shipped.
