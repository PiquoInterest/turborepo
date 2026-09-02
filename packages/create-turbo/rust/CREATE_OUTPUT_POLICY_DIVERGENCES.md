# Create output rendering parity and security ledger

## Scope

This tranche translates the terminal-rendering policy for the final `create-turbo` workspace summary, success message, and get-started instructions from `packages/create-turbo/src/commands/create/index.ts`.

Rust target:

- `packages/create-turbo/rust/src/create_output_policy.rs`
- `packages/create-turbo/rust/tests/create_output_policy_parity.rs`
- `packages/create-turbo/rust/tests/create_output_policy_security.rs`

TypeScript executable security evidence:

- `packages/create-turbo/__tests__/create-output-security.test.ts`

## TDD evidence

- compiling behavioral RED: `68f5ddf67e95b41cf45623a8ada402f9a6a1cd57`
- GREEN bounded renderer: `f1ea07ef8404321a85fd0091cd612ba64779ef62`
- exact-optional TypeScript oracle repair: `ac4a98ff8fb751287d915be5e515d1be346bd9e2`

The RED implementation deliberately interpolated raw fields and iterated every workspace and script so the security tests failed against the intended TypeScript behavior before hardening.

## Preserved safe-input behavior

- No-workspace projects render the `apps` heading followed by ` - <project name>`.
- Workspace headings are emitted on the first entry and whenever the caller-provided group changes.
- Workspace item text remains ` - <title>` with `: <description>` only for a non-empty description.
- Current-directory success remains `>>> Success! Your new Turborepo is ready.`.
- Other success output remains `>>> Success! Created your Turborepo at <relative path>`.
- Get-started output preserves the directory-change line, remote-cache command, documentation URL, fixed script descriptions, and final cache hint.
- Missing package metadata or package-manager metadata yields no get-started block.
- Script command names are represented by the closed `CreateDisplayScript` enum.
- Package-manager command and executable text come from the reviewed static `PackageManagerInstallProfile` table.

The Rust renderer accepts already-derived, already-ordered workspace display records. JavaScript `path.relative`, platform separator behavior, first-component grouping, and `localeCompare` ordering remain host-binding work. This tranche does not invent Rust path or locale semantics and does not claim those dimensions are closed.

## Intentional security divergences

### CT-RS-031: Generated workspace metadata can inject terminal controls

**Severity:** Medium until production cutover

TypeScript sends workspace descriptions and path-derived group/title text directly through terminal coloring. Generated repository metadata can therefore contain ESC/OSC sequences, BEL, line breaks, carriage returns, tabs, bidirectional overrides, zero-width controls, and related terminal-active Unicode. The same terminal boundary also includes the relative project path and fallback project name.

The Rust renderer sanitizes every attacker-influenced field before interpolation, then sanitizes each completed line again. C0/C1 controls, ESC, BEL, CR/LF/TAB, bidi controls, zero-width controls, and related format characters are represented as visible escapes. Unknown raw values are not emitted by this layer.

### CT-RS-032: Workspace and script output is unbounded

**Severity:** Medium until production cutover

TypeScript has no explicit byte limit for a project name, relative path, workspace group, workspace title, or workspace description. It also emits every workspace record and every matching script entry. Large metadata or repeated entries can therefore consume excessive memory, flood terminals and logs, or delay callers.

The Rust policy applies the following deterministic limits:

- 1024 UTF-8 bytes per interpolated field;
- 4096 UTF-8 bytes per final line;
- 256 workspace records;
- 64 script records.

Truncation retains complete emitted fragments, does not split Unicode scalars or escape representations, and appends `[truncated]`. Entry-count truncation emits the explicit line ` - [truncated]`.

## Security invariants

- Raw terminal-active characters never leave this renderer.
- Every attacker-influenced field and every completed line has an explicit byte bound.
- Workspace and repeated-script output have explicit count bounds.
- Static headings and descriptions remain exact.
- Script names are closed enum values rather than free-form text.
- Package-manager command metadata comes only from reviewed static profiles.
- The renderer performs no logging, filesystem access, path resolution, locale sorting, process execution, network access, credential access, telemetry, `unsafe` code, or mutable global state.

## Production-binding blockers

Before replacing the TypeScript output path, the host binding must prove:

1. exact JavaScript-compatible relative-path derivation on Linux, macOS, and Windows;
2. exact first-component group mapping, including `apps` to `Application packages` and `packages` to `Library packages`;
3. exact `localeCompare` ordering or a deliberate documented ordering change with differential tests;
4. construction of `CreateWorkspaceDisplay` records without a raw-output bypass;
5. mapping of available script strings into the closed enum with source-compatible filtering and duplicate handling;
6. terminal coloring applied only after the Rust renderer returns sanitized strings;
7. no second logger path that prints raw project, workspace, description, or path fields;
8. removal proof showing the TypeScript rendering implementation is neither loaded nor shipped after cutover.
