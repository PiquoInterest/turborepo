# Parity matrix

| Function | Status | Notes |
|---|---:|---|
| `convertCase(..., camel)` | Safe-input parity | Preserves the exact ASCII `[-_][a-z]` replacement rule. |
| Other `convertCase` modes | Parity | Remain explicit not-implemented errors. |
| `searchUp` current/parent lookup | Safe-input parity | Filesystem root remains excluded, matching the TypeScript loop. |
| `searchUp` content predicate | Hardened parity | Read errors remain non-matches; unsafe target traversal and files over 4 MiB are rejected/non-matches. |
| `isFolderEmpty` | Safe-input parity | Preserves allow-list and `.iml` handling. Conflict ordering follows filesystem enumeration, as in Node. |
| `isWriteable` | Unix parity | Uses `access(W_OK)` on Unix. Windows ACL parity is still open. |
| `validateDirectory` | Safe-input parity | Preserves normal valid/file/conflict/missing outcomes and singular/plural wording. ANSI dim styling is not represented in the Rust value. |
| Metadata errors | Intentional deviation | Rust returns invalid instead of throwing or continuing under uncertainty. |

The TypeScript package remains the production API. This crate is a tested migration core and does not remove JavaScript host bindings yet.
