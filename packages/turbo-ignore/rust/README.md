# turbo-ignore (Rust migration slice)

This crate ports the behavior of `packages/turbo-ignore` from TypeScript to Rust. It is intentionally a migration slice, not a claim that the entire Turborepo repository has been rewritten: the repository's core engine is already Rust, while JavaScript-facing packages, fixtures, integrations, and publishing glue still use TypeScript or JavaScript.

## Decision contract

The exit-code contract is preserved:

- exit `0`: the deployment may be skipped;
- exit `1`: proceed with the deployment.

Any validation, discovery, Git, Turbo, subprocess, or JSON error returns `Deploy` and therefore exit `1`. A skip result is emitted only after a successful Turbo dry run returns an empty `packages` array or an explicit, unambiguous skip directive applies.

## Test-first structure

`tests/parity.rs` translates the existing Jest behavioral groups for commit directives, comparison selection, workspace and version inference, error classification, dry-run handling, task defaults, and exit semantics.

`tests/security.rs` adds regression coverage for the security differences documented in `SECURITY.md`, including unsafe npm package selectors, Turbo filter manipulation, revision option confusion, terminal control characters, symlinked configuration, subprocess deadlines, and output limits.

## Local validation

From the repository root:

```sh
cargo fmt --all --check
cargo test -p turbo-ignore
cargo clippy -p turbo-ignore --all-targets -- -D warnings
```

The original TypeScript suite remains the behavioral oracle during migration:

```sh
pnpm --filter turbo-ignore test
```

## Distribution status

This crate is not yet wired into `packages/turbo-ignore/package.json` as the published npm executable. A production cutover needs platform binary builds, npm package selection/loading, release-pipeline integration, and side-by-side CLI differential tests. Until that is implemented, this crate proves the Rust decision engine boundary but does not replace the shipped JavaScript CLI.
