# Package-manager installation profile and execution-policy ledger

## Scope

This tranche translates the profile-selection and process-invocation policy used by `packages/turbo-workspaces/src/install.ts` into a dependency-free Rust core consumed by the `create-turbo` migration.

Rust target:

- `packages/create-turbo/rust/src/package_manager_install_policy.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_parity.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_security.rs`

TypeScript security evidence:

- `packages/turbo-workspaces/__tests__/install-security.test.ts`

## TDD evidence

- compiling behavioral RED: `b858e98565eb0415c6ab85bb120220529b9a981b`
- GREEN security implementation: `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`

The RED commit intentionally preserved `preferLocal: true` and Windows shell execution so the new Rust security tests failed for the same reasons demonstrated by the TypeScript `it.failing` fixture.

## Preserved behavior

The Rust constants preserve all eight source profiles and their order:

| Manager | Profile | Semver selector | Install arguments | Default |
| --- | --- | --- | --- | --- |
| npm | `npm` | `*` | `install` | yes |
| pnpm | `pnpm6` | `6.x` | `install` | no |
| pnpm | `pnpm` | `>=7` | `install --fix-lockfile` | yes |
| yarn | `yarn` | `<2` | `install` | yes |
| yarn | `berry` | `>=2` | `install --no-immutable` | no |
| bun | `bun` | `^1.0.1` | `install` | yes |
| nub | `nub` | `*` | `install` | yes |
| aube | `aube` | `*` | `install` | yes |

Additional preserved contracts:

- missing and empty versions use the first profile marked as default;
- supplied versions are tested in source order and the first match wins;
- unsupported versions return no profile;
- matcher errors are propagated immediately without retry or default fallback;
- the selected command, arguments, project root, and ignored-stdin policy are retained as typed data;
- the version string is borrowed and never becomes a command or argument.

The JavaScript `semver.satisfies` call remains behind `PackageManagerVersionMatcher`. Production binding must differentially prove Node-semver behavior, including prereleases, build metadata, coercion boundaries, malformed text, and resource limits, rather than silently substituting different Rust-semver semantics.

## Intentional security divergences

### CT-RS-029: Project-local executable substitution during installation

**Severity:** High until TypeScript cutover

TypeScript calls `execa` with `preferLocal: true`. A generated or attacker-influenced project can therefore place a package-manager-named executable in its local binary path and cause that program to run during installation.

The Rust invocation policy sets `prefer_local: false` for every manager and platform. Programs are represented by `WorkspacePackageManager`, a closed six-variant enum. Install arguments come only from static profile tables.

### CT-RS-030: Windows package-manager execution through a command shell

**Severity:** High until TypeScript cutover

TypeScript sets `shell: true` on Windows. Shell mediation expands the interpretation surface for executable resolution, metacharacters, quoting, environment expansion, file associations, and command shims.

The Rust invocation policy sets `shell: false` on every platform. A production Windows runner must resolve an approved package-manager executable or shim explicitly and execute it without constructing a shell command. If a manager cannot be launched safely without a shell, the provider must return a typed unsupported-platform error rather than weakening this policy.

## Security invariants

- No install invocation requests shell execution.
- No install invocation permits project-local executable preference.
- Program identity is a closed enum, not free-form text.
- Arguments are static reviewed slices.
- Project roots remain borrowed `Path` values, including non-UTF-8 Unix paths.
- Standard input is always ignored to prevent interactive hangs.
- Version text is only passed to the bounded matcher interface and cannot alter the command or argument vector.
- Profile scans are bounded to one or two entries per manager.
- The core introduces no dependency, process execution, filesystem mutation, network access, credential access, `unsafe` code, or mutable global state.

## Production blockers

The production runner must prove canonical executable resolution, explicit environment and configuration policy, no shell, no project-local substitution, a strict working-directory identity contract, bounded output, deadlines, cancellation and descendant cleanup, signal semantics, Windows shim handling, deterministic error mapping, and Linux/macOS/Windows differential fixtures. The host must also supply a Node-semver-compatible matcher and removal proof showing the TypeScript `install.ts` execution path is no longer loaded or shipped.
