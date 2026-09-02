# Workspace package parsing parity and security ledger

## Scope

TypeScript oracle: `packages/turbo-workspaces/src/utils.ts::parseWorkspacePackages`.

Focused oracle: `packages/turbo-workspaces/__tests__/workspace-packages.test.ts`.

Rust target: `packages/turbo-workspaces/rust/src/workspace_packages.rs`.

## TDD chain

- TypeScript oracle commit: `9c8f77deee15c01baba73fdd510960e899756f0e`.
- Compiling behavioral Rust RED: `089112a3f85bc2cbaaf864991eb5b6129602ff30`.
- Rust GREEN implementation: `8b4aea45459aa09237aef7d8dd35ccf06503ae28`.
- Rust parity tests: 7.
- Rust security tests: 6.

The RED implementation exposed the final Rust input and error types but returned an empty vector for every input. The translated value tests and security tests therefore compile and fail for missing behavior rather than missing APIs.

## Preserved valid-input behavior

- an absent `workspaces` field becomes an empty list;
- an array is returned in source order;
- an object with `packages` returns that array in source order;
- an object without `packages` becomes an empty list;
- duplicates and empty strings are preserved;
- general glob syntax, including negation, braces, brackets, and recursive globs, is not restricted to Bun's smaller supported subset.

## Representation differences

TypeScript returns the original array object. Rust returns a bounded `Vec<&str>` containing borrowed string values. This preserves ordering and values while removing mutable array aliasing from the Rust API. The strings are not copied, and callers cannot mutate them through the returned shared references.

## Intentional security divergences

The TypeScript helper accepts an unbounded array and passes through terminal-active or invisible Unicode text. The TypeScript oracle records the current behavior and uses expected-failure tests for the stricter policy, keeping the TypeScript suite green without pretending the legacy implementation is fixed.

Rust rejects:

- more than 256 workspace globs;
- any single glob larger than 4096 UTF-8 bytes;
- more than 65536 aggregate UTF-8 bytes;
- C0/C1 controls, ESC, NUL, bidi controls, zero-width controls, and related format characters.

Public errors identify the failing category and index but never echo attacker-controlled glob text. Limits are checked before the result vector is copied. Aggregate accounting uses checked arithmetic.

## Security findings

### TW-RS-011: Unbounded workspace arrays can amplify later filesystem work

The TypeScript parser performs no count or byte-volume checks. A caller-controlled package manifest can therefore create large allocations and feed a large number of patterns into later glob expansion. Rust bounds the extracted value set before publication.

### TW-RS-012: Terminal-active and invisible text survives the TypeScript parser

The TypeScript helper preserves control and format characters verbatim. Those values can later reach logs, diagnostics, path display, or glob processing. Rust rejects the reviewed unsafe character classes and does not include the offending value in errors.

### TW-RS-013: Mutable JavaScript array aliasing is removed

The TypeScript return value aliases the package object array. Rust publishes a new bounded vector of immutable borrowed strings, preventing callers from mutating the source array through the result.

## Remaining production work

This tranche is a pure parsing core. It does not approve package.json reading, JSON parsing, workspace expansion, symlink handling, filesystem traversal, or host bindings. Production closure still requires bounded no-follow manifest reads, stable filesystem identity, parser limits, deterministic error conversion, Linux/macOS/Windows differential fixtures, downstream cutover, and artifact proof that TypeScript is no longer loaded or shipped.
