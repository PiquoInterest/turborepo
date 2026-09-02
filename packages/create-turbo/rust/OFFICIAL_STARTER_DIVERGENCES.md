# `official-starter` parity and divergence ledger

## Scope and status

TypeScript oracle: `packages/create-turbo/src/transforms/official-starter.ts`.

Rust target: `packages/create-turbo/rust/src/official_starter.rs`.

The reviewed Rust tranche implements the decision and effect-ordering core. It is not yet the production transform. Filesystem access, JSON parsing and serialization, JavaScript host errors, and package publication remain behind typed providers and bindings that are deliberately absent until their safety contracts are proven.

## Preserved observable behavior

- A missing repository, `vercel/turbo`, and `vercel/turborepo` are the only official routes.
- Every other repository returns `not-applicable` before any side-effect provider is called.
- `package.json` existence is observed before the best-effort `meta.json` sequence.
- A metadata read failure is swallowed and prevents the remove attempt.
- A metadata remove failure is swallowed while the already parsed metadata is still returned.
- A missing `package.json` returns success after metadata handling.
- A falsey parsed package value is not written.
- `basic` and `default` rename the package to the requested project name.
- A truthy existing `devDependencies.turbo` receives a truthy explicit version or `^<create-turbo version>`.
- An empty explicit version is JavaScript-falsey and therefore uses the fallback.
- Any truthy parsed package object is written even when neither relevant field changes.
- Package read and write failures retain the exact public messages and `fatal: false` metadata.

## Representation and intentional-divergence ledger

| Area | TypeScript behavior | Rust representation or planned behavior | Classification | Reason |
| --- | --- | --- | --- | --- |
| Missing repository | `example.repo` is absent | `Option::None` | representation-only | Makes absence explicit without changing route selection. |
| Repository matching | Exact string equality and `includes` over two literals | Borrowed `&str` exact comparisons over a fixed array | parity plus hardening | Avoids normalization, fuzzy matching, allocation, and mutable global state. |
| Falsey package result | Runtime falsey guard around parsed JSON | `Option<PackageJson>::None` | representation-only | Prevents null/undefined-style states from leaking into later mutation. |
| Existing Turbo dependency truthiness | JavaScript runtime truthiness | `OfficialStarterPackageJson::turbo_dev_dependency_is_truthy` | type-boundary conversion | The production JSON adapter must reproduce JavaScript truthiness exactly for all supported JSON values rather than guessing from a Rust string type. |
| `create-turbo` version | Runtime `require("../../package.json").version` | Explicit borrowed `create_turbo_version` input | representation-only | Removes hidden module/global lookup and makes the oracle value injectable and testable. |
| Package mutation | Dynamic object property updates | Typed setter methods that preserve all unowned fields in the provider object | representation-only | Narrows mutation authority and makes unknown-field preservation reviewable. |
| Public error | JavaScript `TransformError` object | Typed `OfficialStarterError<E>` with exact message, transform name, and nonfatal flag | partial until binding | Rust must retain the provider cause without inventing JavaScript stack/class behavior; the host binding still has to construct the exact public error. |
| Filesystem and JSON operations | Broad synchronous `fs-extra` reads, forced removal, and in-place JSON write | No production provider yet | intentional security block | Directly cloning the source would retain unbounded parsing, link following, special-file handling, concurrent replacement, partial-write, metadata, and ordering risks. |
| Future production reads | Source follows ordinary path semantics without explicit bounds | Bounded, regular-file, root-confined, no-follow reads | intentional security divergence | Rejects oversized files, links, special files, and redirected roots instead of reading attacker-selected resources. |
| Future package publication | Source rewrites `package.json` directly | Same-directory staged and synchronized atomic replacement with identity checks | intentional security divergence | Prevents partial publication and reduces time-of-check/time-of-use replacement attacks. |
| Metadata/package transaction | Metadata can be removed before a later package failure | Source ordering preserved in the core; production provider requires an explicit rollback or journal decision | compatibility risk, unresolved | Changing this silently would alter behavior, but shipping without recovery can leave partial state. A separate RED contract is required for any transactional hardening. |

## Production-provider security requirements

The Rust binding must not become production-active until one provider proves all of the following with failure injection and Linux, macOS, and Windows differential fixtures:

1. bounded reads for `meta.json` and `package.json`, including nesting, string, collection, and total-byte limits;
2. strict JSON parsing with exact JavaScript-compatible truthiness for the existing Turbo dependency;
3. preservation of unknown fields and the source-observable insertion/serialization order, with deterministic two-space output;
4. root confinement, regular-file requirements, link/reparse-point rejection, and file identity revalidation;
5. same-directory staging, synchronization, atomic replacement, and approved mode/ACL/ownership handling;
6. defined concurrent modification behavior and cleanup of every temporary artifact;
7. an explicit transaction or rollback policy for metadata removal followed by package publication;
8. exact public `TransformError` mapping and no false-success result after package failure;
9. no shell construction, process execution, network access, credential access, or unredacted logging.

## Dependency and advisory impact

This tranche adds no crate, parser, network client, subprocess, logger, unsafe code, or mutable global state. It therefore introduces no new dependency advisory surface. The repository-wide `quick-xml`, `h2`, and `webbrowser` findings remain open and are not ignored or weakened by this change.

## TDD evidence

- RED parity/security contract: `2ca25bd457cbe216f345b5f67cf9ac32f43a2c7a`.
- GREEN orchestration core: `cd2ba74b3040e654a63c9799e42c35a12f2c4dbc`.
- Parity tests: `tests/official_starter_parity.rs`.
- Security tests: `tests/official_starter_security.rs`.
