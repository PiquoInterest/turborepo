# Package-manager prompt parity and divergence ledger

TypeScript oracle: `packages/create-turbo/src/commands/create/prompts.ts`.

Rust target: `packages/create-turbo/rust/src/package_manager_prompt.rs`.

## Preserved behavior

- `skipTransforms` returns no selection before package-manager discovery or prompting.
- The source choice order is `npm`, `pnpm`, `yarn`, `bun`, `nub`, `aube`.
- An exact requested manager with a truthy discovered version is returned without prompting.
- Unknown, unavailable, or empty-version manager arguments fall back to the selector.
- Installed choices sort before unavailable choices while retaining source order within both groups.
- The selected manager returns its exact discovered version.
- Selector failure is propagated and never retried.

## Divergences and type conversions

| Area | TypeScript behavior | Rust behavior | Classification and reason |
| --- | --- | --- | --- |
| Requested manager | Free-form value cast with `as PackageManager`, then object indexing | Exact six-literal parse into `WorkspacePackageManager` | Intentional type hardening. Unknown, confusable, path-like, case-changed, or whitespace-padded text cannot become a trusted manager key. |
| Availability | JavaScript object value is checked by truthiness | Borrowed `Option<&str>` with empty string treated as unavailable | Type conversion preserving the declared string/undefined contract and JavaScript empty-string behavior. |
| Choice ordering | Stable JavaScript sort by installed boolean | Stable Rust sort by `disabled` | Parity. Installed choices move first without reordering peers. |
| Prompt UI | Inquirer disables unavailable choices | Typed selector receives six choices and Rust revalidates the result | Intentional defense in depth. A compromised or faulty adapter cannot return a disabled manager. |
| All managers unavailable | Inquirer receives six disabled choices, with host-specific cancellation behavior | Selector may fail, or an unavailable result is rejected | Explicit boundary. No version is fabricated and no retry loop is introduced. |
| Large manager text | Dynamic property lookup input | Borrowed exact comparisons only | Resource hardening. Input is not normalized, copied, logged, or added to the choice list. |
| Discovery and terminal UI | Async process discovery plus interactive Inquirer prompt | Dependency-injected availability and selector traits | Representation only. Production providers and non-TTY/cancellation differentials remain blocked. |

## Security and production requirements

The core adds no crate, process execution, filesystem access, network call, terminal output, unsafe code, or mutable global state. Production binding remains blocked until it proves:

1. exact discovery results for all six managers without shell construction or project-local executable substitution;
2. bounded subprocess duration and output plus descendant cleanup;
3. exact prompt labels, disabled text, source order, cancellation, non-TTY, and signal behavior;
4. terminal-control-safe rendering;
5. no retry or fallback to an unavailable manager;
6. Linux, macOS, and Windows differential fixtures;
7. removal proof showing the TypeScript prompt logic is neither loaded nor shipped.

## TDD evidence

- RED integration commit: `36b49a6cfad94bab8487dda62871b60c99a84115`.
- GREEN integration commit: `4f00ff3ebe627acb5a15ead535f27d623d8a9a2c`.
- Parity tests: `tests/package_manager_prompt_parity.rs`.
- Security tests: `tests/package_manager_prompt_security.rs`.
