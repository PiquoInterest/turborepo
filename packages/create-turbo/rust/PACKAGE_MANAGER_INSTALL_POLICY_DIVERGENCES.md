# Package-manager installation profile and execution-policy ledger

## Scope

This tranche translates the profile-selection and process-invocation policy used by `packages/turbo-workspaces/src/install.ts` into a dependency-free Rust core consumed by the `create-turbo` migration.

Rust target:

- `packages/create-turbo/rust/src/package_manager_install_policy.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_parity.rs`
- `packages/create-turbo/rust/tests/package_manager_install_policy_security.rs`

TypeScript oracle and security evidence:

- `packages/turbo-workspaces/__tests__/install-meta.test.ts`
- `packages/turbo-workspaces/__tests__/install-security.test.ts`

## TDD evidence

- profile and invocation RED: `b858e98565eb0415c6ab85bb120220529b9a981b`
- profile and invocation GREEN: `a200c283e0cfb17bec0cb3422b44cdfaa3f7c60c`
- concrete matcher RED: `816216a20b5620ab381842e26ed322d9409b3cec`
- concrete matcher GREEN: `a47192630977ffec2a4208f67d01fbd948a8aa97`
- exact Rustfmt output: `149f43f4662d8ab3f44b35a2b21e4e3bfd8c3c31`

The concrete matcher RED commit exported the final callable API and bounds but deliberately returned `Ok(false)`. The translated source-profile, build-metadata, prerelease, malformed-input, and malformed-range tests therefore compiled and failed for missing matching behavior rather than missing symbols.

The TypeScript oracle remains executable and green. It records current npm `semver` behavior for all repository profiles, malformed input, build metadata, prereleases, and edge whitespace. The stricter whitespace policy is represented by `it.failing` cases so the source suite remains green while retaining evidence of the intentional Rust hardening.

GitHub Actions run `33547336164` compiled the committed migration crates, passed all migration parity and security tests, and passed Clippy with warnings denied for the exact formatted implementation. The lockfile-wide advisory audit remains an independent repository gate and is not treated as implementation failure or suppressed.

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
- build metadata does not change range satisfaction;
- prereleases are excluded because none of the six repository-owned ranges opts into a prerelease tuple;
- one leading `v` or `=` compatibility marker is accepted without trimming any other input;
- malformed versions are unsupported non-matches;
- malformed or unknown repository range literals are typed configuration errors;
- unsupported versions return no profile;
- matcher errors propagate immediately without retry or default fallback;
- the selected command, arguments, project root, and ignored-stdin policy remain typed data;
- version text never becomes an executable name or argument.

The matcher intentionally supports only the six range literals committed in the profile table: `*`, `6.x`, `>=7`, `<2`, `>=2`, and `^1.0.1`. This closed grammar is sufficient for the current source contract and prevents a future repository edit from silently expanding the accepted range language without tests and review.

## Test inventory

| Evidence set | Ported coverage | Status |
| --- | ---: | --- |
| Rust profile and matcher parity tests | 12 test functions | GREEN |
| Rust install-policy security tests | 8 test functions | GREEN |
| TypeScript semver oracle cases | 25 generated or direct cases | GREEN, including expected-failure security evidence |
| TypeScript install execution security evidence | 1 expected-failure case | GREEN as evidence of the unresolved source behavior |

This inventory counts executable test cases for this bounded tranche. It is not a repository completion percentage. Production process execution, host binding, platform differentials, packaging, caller cutover, and TypeScript removal remain open.

## Intentional security divergences

### CT-RS-029: Project-local executable substitution during installation

**Severity:** High until TypeScript cutover

TypeScript calls `execa` with `preferLocal: true`. A generated or attacker-influenced project can therefore place a package-manager-named executable in its local binary path and cause that program to run during installation.

The Rust invocation policy sets `prefer_local: false` for every manager and platform. Programs are represented by `WorkspacePackageManager`, a closed six-variant enum. Install arguments come only from static profile tables.

### CT-RS-030: Windows package-manager execution through a command shell

**Severity:** High until TypeScript cutover

TypeScript sets `shell: true` on Windows. Shell mediation expands the interpretation surface for executable resolution, metacharacters, quoting, environment expansion, file associations, and command shims.

The Rust invocation policy sets `shell: false` on every platform. A production Windows runner must resolve an approved package-manager executable or shim explicitly and execute it without constructing a shell command. If a manager cannot be launched safely without a shell, the provider must return a typed unsupported-platform error rather than weakening this policy.

### CT-RS-036: Version matching was delegated to an unbounded provider

**Severity:** Medium

The first Rust profile tranche delegated `semver.satisfies` to an injected matcher. A permissive or incompatible provider could normalize hostile text, accept an unreviewed range grammar, or select the wrong installation profile.

The committed Rust matcher now:

- limits version and range text to 256 UTF-8 bytes before parsing;
- rejects non-ASCII, whitespace-bearing, and control-bearing version text;
- accepts only the six reviewed repository range literals;
- rejects core components above JavaScript's maximum safe integer;
- enforces strict three-component versions and leading-zero rules;
- validates prerelease and build identifiers;
- excludes prereleases for the current ranges;
- performs no allocation proportional to attacker-controlled range grammar;
- introduces no dependency, subprocess, filesystem, network, credential, logging, mutable-global, or `unsafe` authority.

### CT-RS-037: npm edge-whitespace normalization is not preserved

**Severity:** Low intentional hardening

The TypeScript path uses npm `semver`, which accepts leading or trailing ASCII whitespace around an otherwise valid version. A package-manager version normally comes from executable discovery and has no reason to include hidden line or spacing characters. Normalizing that text can turn an ambiguous or terminal-derived value into a trusted installation profile.

Rust rejects any ASCII whitespace or control character before parsing. The TypeScript oracle contains normal GREEN tests documenting current normalization and `it.failing` tests documenting the desired rejection. Rust security tests require the rejection. Safe canonical versions retain the same profile selection.

## Security invariants

- No install invocation requests shell execution.
- No install invocation permits project-local executable preference.
- Program identity is a closed enum, not free-form text.
- Arguments are static reviewed slices.
- Project roots remain borrowed `Path` values, including non-UTF-8 Unix paths.
- Standard input is always ignored to prevent interactive hangs.
- Version and range text are bounded before parsing.
- Version text is ASCII and control-free before it can select a profile.
- Numeric core components cannot exceed JavaScript's maximum safe integer.
- Unknown range syntax is a typed error rather than a permissive fallback.
- Profile scans are bounded to one or two entries per manager.
- The core introduces no new dependency or direct side-effect authority.

## Advisory lookup

**Lookup date: 2026-09-01**

Sources reviewed:

- npm `node-semver` source documentation and version/range compatibility notes;
- the RustSec Advisory Database and advisory repository;
- the GitHub Advisory Database for npm and Rust ecosystems;
- the repository-wide resolved-lockfile audit.

Disposition:

- This GREEN implementation adds no dependency and therefore no new transitive advisory surface.
- The TypeScript package currently uses npm `semver` 7.6.2. The relevant historical regular-expression denial-of-service advisory is patched in versions later than 7.5.2, so the installed source oracle is outside that affected range.
- The repository-wide `webbrowser`, `h2`, and `quick-xml` findings remain open. They are not ignored, downgraded, or coupled to this matcher tranche.

## Production blockers

The production runner must still prove canonical executable resolution, explicit environment and configuration policy, no shell, no project-local substitution, a strict working-directory identity contract, bounded output, deadlines, cancellation and descendant cleanup, signal semantics, Windows shim handling, deterministic error mapping, and Linux/macOS/Windows differential fixtures.

The host binding must run the TypeScript and Rust matchers over shared fixtures on every supported platform before cutover. Removal proof must show that the TypeScript `install.ts` execution path is no longer loaded or shipped.
