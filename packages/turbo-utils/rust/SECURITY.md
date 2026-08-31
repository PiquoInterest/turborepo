# Security review

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

## TU-001: Ancestor-search target traversal

The TypeScript `searchUp` accepts an arbitrary target and joins it beneath every ancestor. Parent components can make each probe escape the ancestor being searched. The Rust API accepts only a non-empty relative target without parent, root, or platform-prefix components.

## TU-002: Unbounded content predicates

`searchUp` reads every matching candidate fully before running `contentCheck`. A large file can consume deployment-process memory. Rust content checks only read regular files up to 4 MiB and treat larger or unreadable candidates as non-matches.

## TU-003: Symlinked project roots

The TypeScript directory validator uses `lstat`, which currently rejects a symlink because it is not reported as a directory. Rust preserves and tests this behavior explicitly so later refactors cannot silently begin following project-root symlinks.

## TU-004: Metadata/read uncertainty

The TypeScript function can throw for permission and I/O failures even though callers generally expect a validation result. Rust treats uncertain metadata and directory enumeration as invalid, preventing creation logic from continuing against a path it could not inspect.

## TU-005: Time-of-check/time-of-use remains

Directory validation and later project creation are separate operations. Another process can replace entries after validation. The final migration must use descriptor-relative creation and no-follow semantics where supported. This tranche documents but does not eliminate that cross-operation race.

## TU-006: Platform writability differences

On Unix, Rust uses `access(W_OK)`, matching Node's effective access check closely. The non-Unix fallback uses directory metadata and the readonly flag, which is not complete ACL parity. Windows cutover remains blocked until native access checks and dedicated parity tests are added.
