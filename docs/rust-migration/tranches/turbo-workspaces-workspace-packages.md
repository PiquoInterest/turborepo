# turbo-workspaces workspace-package parser tranche

## Scope

This tranche ports the pure `parseWorkspacePackages` behavior from `packages/turbo-workspaces/src/utils.ts`. It does not include package.json I/O, JSON parsing, glob expansion, filesystem traversal, or mutations.

## Sequential TDD history

1. TypeScript oracle: `9c8f77deee15c01baba73fdd510960e899756f0e`.
2. Initial RED candidate: `089112a3f85bc2cbaaf864991eb5b6129602ff30`, rejected because one security assertion failed to compile with Rust E0505.
3. TypeScript formatter repair: `aaa354bf2a808039bdff461dc65dd5e7507a8aec`.
4. Corrected compiling behavioral Rust RED: `72aa20cf4e17f528b46111f9681f06d522994655`.
5. Corrected Rust GREEN: `d997c57b66b4d10710ecee8c98b8a72ff61f2eef`.
6. Documentation and focused validation commits follow this record.

The TypeScript oracle keeps ordinary behavior green and uses expected-failure tests to document unsafe legacy acceptance. The corrected RED Rust API compiles but returns an empty vector for all inputs. The GREEN implementation performs bounded extraction. The rejected candidate remains in history and is explicitly excluded from behavioral RED evidence.

## Behavior preserved

- missing input becomes an empty list;
- array and object `packages` forms are supported;
- source ordering, duplicates, and empty strings are preserved;
- negation, recursive globs, braces, and brackets remain valid general workspace syntax.

## Security divergences

Rust rejects more than 256 values, a value larger than 4096 UTF-8 bytes, aggregate input above 65536 bytes, and terminal-active or invisible control text. The TypeScript source accepts these values. Public Rust errors do not echo the offending input.

The Rust return value is a new vector of immutable borrowed strings rather than the original mutable JavaScript array object. This preserves values without mutable aliasing.

## Tests

- TypeScript focused oracle: `packages/turbo-workspaces/__tests__/workspace-packages.test.ts`;
- Rust parity: `workspace_packages_parity.rs`, 7 tests;
- Rust security: `workspace_packages_security.rs`, 6 tests.

## Validation gate

```sh
pnpm exec oxfmt --check packages/turbo-workspaces/__tests__/workspace-packages.test.ts
pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-packages.test.ts
cargo fmt --all --check
cargo check --locked -p turbo-workspaces-rs --all-targets
cargo test --locked -p turbo-workspaces-rs --test workspace_packages_parity --test workspace_packages_security
cargo clippy --locked -p turbo-workspaces-rs --all-targets -- -D warnings
```

## Remaining closure

Production use remains blocked on bounded no-follow package.json reads, strict JSON limits, exact TypeScript error mapping, root-confined and bounded glob expansion, platform differentials, bindings, downstream cutover, and artifact proof that executable TypeScript is neither loaded nor shipped.
