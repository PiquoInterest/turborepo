# turbo-workspaces workspace-details tranche

## Status

- Integration branch: `rust/typescript-deprecation`
- Integration PR: #1
- TypeScript oracle: `4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff`
- TypeScript focused suite: 5 of 5 green in workflow run `33551576871`, job `100002027683`
- Rust RED: `2d4cc22e6a821c88882a87d604746dabbaa95fe2`
- Rust GREEN: `263ddc22d5b5f544768f4e089c92892339b0dce8`
- Rust tests added: 6 parity, 5 security
- Production cutover: blocked
- TypeScript removal: not started

## Implemented contract

The Rust core reproduces the safe-input orchestration of `getWorkspaceDetails`: directory resolution first, exact six-manager order, serial first-success detection, one selected read, exact known errors, and immediate provider-error propagation.

The security hardening closes manager identity to a fixed enum/array and ensures detectors/readers receive only the provider-returned absolute path. It grants no filesystem or process capability directly.

## Remaining package test debt

Eight executable TypeScript suites are tracked. `workspace-details.test.ts` is now mapped at the core level. `install-meta.test.ts` and `install-security.test.ts` have partial/shared Rust evidence. `index.test.ts`, `managers.test.ts`, `utils.test.ts`, `nub.test.ts`, and `aube.test.ts` remain wholly or substantially unported.

## Validation blocker

The Rust code and test chain are committed sequentially, but hosted Rust execution is pending because GitHub Actions has queued jobs without an active runner. This is not counted as a pass. A follow-up integration workflow must validate the RED commit, validate the GREEN commit, add the crate to the root workspace with an exact lockfile delta, and update the canonical repository ledgers.
