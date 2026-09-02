# Create installation policy and warning-output divergence ledger

## Scope

This ledger covers the Rust translation of the package-install decision in `packages/create-turbo/src/commands/create/index.ts` and the security hardening of its unavailable-package-manager warning output.

Rust targets:

- `packages/create-turbo/rust/src/create_install_policy.rs`
- `packages/create-turbo/rust/tests/create_install_policy_parity.rs`
- `packages/create-turbo/rust/tests/create_install_policy_security.rs`
- `packages/create-turbo/rust/tests/create_install_warning_parity.rs`
- `packages/create-turbo/rust/tests/create_install_warning_security.rs`

TypeScript oracle and executable security evidence:

- `packages/create-turbo/__tests__/create-install-policy.test.ts`
- `packages/create-turbo/__tests__/index.test.ts`

## TDD and integration evidence

Install decision:

- RED contract: `ff359432f3b91d1f164c68ed0270d62ec8b15f42`
- GREEN implementation: `02eb3f5ba3a8733cf27c5377aaca3fae1ad09f2a`
- TypeScript oracle: `bcdf38204875bf440ee057cc442aebec02e29e0b`
- committed formatting proof: `d0ad43451ad97b2113b612e1bfbf4ac23c313e71`
- integration merge: `00062700bf4fc625de2c2c6cb38de970e7a013ec`

Warning rendering:

- RED security contract: `39a4ed083dcb021f673d51b599cf58bc7878e7a2`
- GREEN implementation: `9423b807e72883f30c3e6bbf83fa918d2d846e34`

## Preserved install-decision behavior

- The source manager is used when transforms are skipped or when no selected manager exists.
- Manager resolution occurs before the `hasPackageJson` and `skipInstall` gates.
- The unavailable-manager warning branch exists only when transforms are skipped.
- Missing or empty selected-manager versions silently skip installation.
- Successful installation is requested with `interactive: false`.
- The installer is invoked at most once and its error is propagated directly.
- All six repository variants are represented as a closed enum: `npm`, `yarn`, `pnpm`, `bun`, `nub`, and `aube`.
- Safe warning text remains byte-for-byte equivalent to the TypeScript wording.

## Intentional divergences

### Availability snapshot

TypeScript indexes the mutable availability object during manager resolution and again while deciding whether to warn. Rust snapshots the selected source-manager version once. This removes a time-of-check/time-of-use ambiguity while preserving results for stable valid input.

### Structured warning outcome

The decision core returns `CreateInstallOutcome::WarnUnavailable` rather than logging. This keeps terminal output outside the install decision and allows the host to consume only reviewed rendered strings.

### CT-RS-028: Untrusted example names reach terminal warnings

**Severity:** Medium

The TypeScript warning interpolates the example name directly into two terminal messages. A repository or CLI-controlled name can contain ESC/OSC sequences, BEL, newlines, carriage returns, tabs, bidi overrides, zero-width controls, or a very large payload. The TypeScript `it.failing` case preserves this defect as executable evidence while TypeScript remains the oracle.

The Rust renderer:

- sanitizes the example name before interpolation;
- caps the sanitized field at 2048 UTF-8 bytes;
- renders the package manager from a closed enum rather than free-form text;
- sanitizes each complete line again;
- caps each line at 4096 UTF-8 bytes;
- appends `[truncated]` without splitting Unicode scalars or escaped fragments;
- returns two strings without performing terminal I/O.

Regression tests cover exact safe text, all manager variants, ESC/OSC/BEL and line controls, Unicode directionality and format controls, a 4 MiB name, and multibyte truncation.

## Security invariants

- An unavailable source manager never reaches the installer.
- `skipInstall` and missing `package.json` never reach the installer.
- Installer failures are not retried or downgraded.
- Selected-manager input does not trigger a second availability lookup.
- No raw terminal-active character is emitted by the warning renderer.
- Warning output is bounded independently of input size.
- The renderer introduces no process, filesystem, network, credential, parser, `unsafe`, or mutable-global capability.

## Remaining production blockers

The production host must bind `CreateInstallOutcome` to the real installer and logger, emit only the rendered warning strings, prove exactly-once warning behavior, preserve installer error mapping, and pass Linux/macOS/Windows differential fixtures. Package-manager installation itself remains behind `CreateInstaller` until executable resolution, environment isolation, timeouts, bounded output, descendant cleanup, rollback or atomic promotion, and the complete six-manager behavior matrix are proven.
