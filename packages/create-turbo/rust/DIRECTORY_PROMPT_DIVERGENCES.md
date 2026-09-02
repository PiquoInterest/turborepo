# Project-directory prompt parity and divergence ledger

## Scope

TypeScript oracles:

- `packages/create-turbo/src/commands/create/prompts.ts`
- the `prompts.directory` caller in `packages/create-turbo/src/commands/create/index.ts`
- `packages/create-turbo/src/cli.ts`
- `packages/turbo-utils/src/validate-directory.ts`
- `packages/turbo-utils/src/is-folder-empty.ts`

Rust target: `packages/create-turbo/rust/src/directory_prompt.rs`.

The Rust tranche owns argument-versus-prompt selection, exact prompt metadata, display-only transformation metadata, raw-answer preservation, input limits, terminal-active-text rejection, and typed validator propagation. It deliberately does not claim that the production terminal or filesystem providers are complete.

## Confirmed TypeScript defect

For a truthy CLI directory argument, the original prompt function returned `validateDirectory(dir)` even when the returned object had `valid: false`. The create command immediately destructured `root` and `projectName` without checking `valid`. A malformed, non-directory, or conflicting direct path could therefore continue into project acquisition after the validator had already rejected it.

The TypeScript RED tests require rejection. The repaired TypeScript path throws `InvalidDirectoryError`, emits only a trusted generic validation message, and the root CLI maps that known user-input error to the existing nonzero update-notifier exit path. Rust represents the validator boundary as `Result`, so rejected validation cannot be represented as success.

## Preserved behavior

- A non-empty direct argument is JavaScript-truthy, bypasses prompting, and is not trimmed.
- An absent or empty direct argument invokes the prompt.
- The prompt message remains `Where would you like to create your Turborepo?`.
- The default remains `./my-turborepo`.
- Inquirer's `transformer` trims ECMAScript whitespace for display only. The accepted answer remains raw and is validated unchanged.
- Prompt and validation providers run at most once and their failures are propagated without retry or an invented fallback.
- A successful validator output is returned without reinterpretation.

## Representation and intentional security divergences

| Boundary | TypeScript before repair | Rust and repaired TypeScript | Classification and reason |
| --- | --- | --- | --- |
| Invalid direct argument | Returned `{ valid: false }`; caller ignored `valid` | Typed rejection before acquisition | Security fix: prevents known-invalid filesystem state from continuing. |
| Validation diagnostics | Included project names, resolved paths, and conflict counts | Direct/prompted failure exposes a fixed trusted message | Security divergence: avoids path disclosure and terminal injection from hostile names. |
| Input size | No explicit bound before path and filesystem work | 4,096 UTF-8 byte limit | Security divergence: bounds allocation, encoding, path conversion, and provider work. |
| Terminal-active Unicode | Partial incidental rejection | C0/C1 controls, bidi/invisible controls, line/paragraph separators, annotation controls, soft hyphen, BOM, and related format controls are rejected before providers | Security divergence: prevents output spoofing and invisible path ambiguity. |
| Ill-formed JavaScript strings | Lone UTF-16 surrogates could reach path conversion, where encoding may substitute replacement text | TypeScript rejects unpaired surrogates; Rust accepts only valid UTF-8 `&str` | Type-boundary hardening: distinct ill-formed inputs cannot alias after replacement encoding. |
| Optional CLI argument | JavaScript truthiness | `Option<&str>` plus an exact non-empty check | Representation-only conversion preserving empty-string falsehood. |
| Prompt transformer | Display is trimmed, returned answer is raw | `DirectoryDisplayTransform` is provider metadata; core validates the raw answer | Representation-only conversion verified against the pinned Inquirer source. |
| Prompt and filesystem effects | Direct module calls | `DirectoryPrompter` and `DirectoryValidator` traits | Security boundary: terminal and filesystem authority stays outside the decision core. |
| Rust public input error | N/A | Never contains the rejected value; provider causes remain structured | Security hardening against control-text reflection. |

## Remaining provider and cutover requirements

A production `DirectoryPrompter` must enforce the advertised byte limit while reading rather than only after submission, apply the transformer for display without mutating the answer, sanitize rendering, and preserve cancellation, EOF, signal, and non-TTY behavior.

A production `DirectoryValidator` must prove lexical and platform path behavior, every-component no-follow or handle-relative inspection, stable directory identity, regular-directory requirements, bounded enumeration and conflict rendering, concurrent replacement behavior, permissions, Windows reparse points, and Linux/macOS/Windows differential fixtures. The current TypeScript validator uses separate existence, metadata, and enumeration operations, so its time-of-check/time-of-use behavior is not a safe provider design to copy.

The JavaScript/native host must reject ill-formed UTF-16 before converting to Rust strings and must map typed directory failures to the established user-input exit path without rendering raw rejected values.

## TDD evidence

- Initial RED contract: `11131d1fc01536c151bdda04ba39fdc4aec5779a`.
- Non-reflecting diagnostic RED extension: `6bbef5a95cbc0e6cf908e62083e852db473b2456`.
- Initial fail-closed implementation: `7e0f30b7f450c799930aa337184d5b7be70ba252`.
- Unicode and malformed-string RED extension: `beb38232b84dca526a5b86accf40b5dca80a734c`.
- Final GREEN implementation: `e0d5663e51c084f4f25051270ed9bb494df1b21a`.
- TypeScript security regressions: `packages/create-turbo/__tests__/directory-security.test.ts`.
- Rust parity regressions: `packages/create-turbo/rust/tests/directory_prompt_parity.rs`.
- Rust security regressions: `packages/create-turbo/rust/tests/directory_prompt_security.rs`.

This tranche adds no dependency, subprocess, network access, filesystem implementation, logger, `unsafe` code, or mutable global state.
