# Create-command error policy parity and divergence ledger

## Scope

This tranche translates the error-classification and terminal-display decision boundary from `packages/create-turbo/src/commands/create/index.ts` into `packages/create-turbo/rust/src/create_error_policy.rs`.

The Rust module is a pure policy core. It does not log, terminate the process, emit telemetry, access the filesystem or network, or construct JavaScript error objects. Those effects remain explicit host-binding work.

## TDD and integration evidence

- corrected RED contract: `ae46b703826d866d21b5acd64fd681c0d9313e10`
- GREEN implementation: `de9be3378d3eba70ffd105bdc9692f60c6b9cc48`
- committed rustfmt proof: `13b34c6ddebbd938f0985c9201363934a2c5385a`
- integration merge: `4684079075b30125bfd1f5e6310dbb37f9319d68`
- TypeScript expected-failure evidence: `packages/create-turbo/__tests__/create-error-security.test.ts`

## Preserved behavior

- Every caught value requests create-command error telemetry.
- A nonfatal transform failure produces one labeled line and continues.
- A fatal transform failure produces one labeled line and requests exit code `1`.
- A known conversion failure produces one unlabeled line and requests exit code `1`.
- An unknown conversion failure is rethrown without display.
- A download failure produces the established heading, then the provider message, and requests exit code `1`.
- An unknown error is rethrown without display.
- Safe printable labels and messages are preserved exactly.

## Intentional security and reliability divergences

| Boundary | TypeScript behavior | Rust behavior | Reason |
| --- | --- | --- | --- |
| fatal termination | calls `process.exit(1)` inside the handler | returns `CreateCommandErrorAction::Exit(1)` | allows cleanup and telemetry flush before termination |
| classification | runtime classes and mutable fields | closed enums and typed fatality | message text cannot alter error class or fatality |
| terminal controls | provider text reaches terminal formatting directly | C0/C1, ESC, BEL, CR/LF/TAB, bidi, zero-width, and related format controls are escaped | prevents forged lines, cursor changes, OSC hyperlinks, and directionality spoofing |
| output size | unbounded provider text | messages are capped at 4096 UTF-8 bytes and labels at 256 UTF-8 bytes | bounds memory, terminal flooding, and log amplification |
| truncation | no explicit policy | complete emitted fragments are retained and `[truncated]` is appended without splitting UTF-8 | deterministic valid output |
| unknown values | may be rethrown after broad runtime handling | returned as typed `Rethrow` with zero display lines | preserves identity and minimizes accidental disclosure |

## Security invariants

- Unknown errors are never rendered by this layer.
- Raw terminal-active characters never leave the display-policy boundary.
- Every displayed field has an explicit byte limit.
- Truncation never splits a Unicode scalar or an escaped fragment.
- Error text cannot change telemetry intent, fatality, classification, or exit code.
- The core introduces no dependency, `unsafe` code, parser, process, filesystem, network, credential, or mutable-global capability.

## Production-binding blockers

The production binding must prove exact JavaScript-to-Rust error conversion, exactly-once telemetry, ordered emission of sanitized fields only, cleanup and telemetry flush before applying exit code `1`, unknown-error identity and stack behavior, Linux/macOS/Windows differential fixtures, and removal proof showing that the TypeScript handler is neither loaded nor shipped after cutover.
