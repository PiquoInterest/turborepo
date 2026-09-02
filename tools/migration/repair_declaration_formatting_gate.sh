#!/usr/bin/env bash
set -euo pipefail

expected_repository="PiquoInterest/turborepo"
expected_branch="rust/typescript-deprecation"
formatter_files=(
  docs/rust-migration-test-inventory.md
  docs/typescript-deprecation.md
  packages/create-turbo/__tests__/create-output-security.test.ts
  packages/create-turbo/__tests__/directory-security.test.ts
  packages/create-turbo/rust/CREATE_ERROR_POLICY_DIVERGENCES.md
  packages/create-turbo/rust/DIRECTORY_PROMPT_DIVERGENCES.md
  packages/create-turbo/rust/OFFICIAL_STARTER_DIVERGENCES.md
  packages/create-turbo/rust/PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md
  packages/create-turbo/rust/PACKAGE_MANAGER_PROMPT_DIVERGENCES.md
  packages/create-turbo/rust/PARITY_MATRIX.md
  packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md
  packages/turbo-utils/rust/PARITY_MATRIX.md
  packages/turbo-workspaces/__tests__/bun-workspace-glob-security.test.ts
  packages/turbo-workspaces/__tests__/workspace-details.test.ts
  packages/turbo-workspaces/rust/PARITY_MATRIX.md
  packages/turbo-workspaces/rust/TEST_INVENTORY.md
)

if [[ "$GITHUB_REPOSITORY" != "$expected_repository" ]]; then
  echo "unexpected repository: $GITHUB_REPOSITORY" >&2
  exit 1
fi
if [[ "$GITHUB_REF_NAME" != "$expected_branch" ]]; then
  echo "unexpected branch: $GITHUB_REF_NAME" >&2
  exit 1
fi
if [[ "$(git rev-parse HEAD)" != "$GITHUB_SHA" ]]; then
  echo "checkout SHA does not match event SHA" >&2
  exit 1
fi
if [[ "$(git rev-parse HEAD:.github/workflows/continue-package-manager-declaration.yml)" != \
  "9b4fdfa07643dfaf8509a8f52e12aae1b894cb3a" ]]; then
  echo "declaration workflow changed before repair" >&2
  exit 1
fi

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

hook_source="$RUNNER_TEMP/turbo-hook-source"
hook_target="$RUNNER_TEMP/turbo-hook-target"
hook_bin="$RUNNER_TEMP/turbo-hook-bin"
git worktree add --detach "$hook_source" \
  813d54ae054923e85269979dfa98fe5e47331070
(
  cd "$hook_source"
  CARGO_TARGET_DIR="$hook_target" cargo build --locked -p turbo --bin turbo
)
turbo_binary="$hook_target/debug/turbo"
test -x "$turbo_binary"
test "$("$turbo_binary" --version)" = \
  "$(node -p 'require("./packages/turbo/package.json").version')"
mkdir -p "$hook_bin"
ln -s "$turbo_binary" "$hook_bin/turbo"
export PATH="$hook_bin:$PATH"
git worktree remove --force "$hook_source"
rm -rf "$GITHUB_WORKSPACE/zig"

pnpm exec oxfmt --write "${formatter_files[@]}"
pnpm exec oxfmt --check
git diff --check

python3 <<'PY'
import subprocess

expected = {
    "docs/rust-migration-test-inventory.md",
    "docs/typescript-deprecation.md",
    "packages/create-turbo/__tests__/create-output-security.test.ts",
    "packages/create-turbo/__tests__/directory-security.test.ts",
    "packages/create-turbo/rust/CREATE_ERROR_POLICY_DIVERGENCES.md",
    "packages/create-turbo/rust/DIRECTORY_PROMPT_DIVERGENCES.md",
    "packages/create-turbo/rust/OFFICIAL_STARTER_DIVERGENCES.md",
    "packages/create-turbo/rust/PACKAGE_MANAGER_INSTALL_POLICY_DIVERGENCES.md",
    "packages/create-turbo/rust/PACKAGE_MANAGER_PROMPT_DIVERGENCES.md",
    "packages/create-turbo/rust/PARITY_MATRIX.md",
    "packages/create-turbo/rust/TRANSFORM_PIPELINE_DIVERGENCES.md",
    "packages/turbo-utils/rust/PARITY_MATRIX.md",
    "packages/turbo-workspaces/__tests__/bun-workspace-glob-security.test.ts",
    "packages/turbo-workspaces/__tests__/workspace-details.test.ts",
    "packages/turbo-workspaces/rust/PARITY_MATRIX.md",
    "packages/turbo-workspaces/rust/TEST_INVENTORY.md",
}
changed = set(
    subprocess.check_output(["git", "diff", "--name-only"], text=True).splitlines()
)
if changed != expected:
    raise SystemExit(
        f"formatter path set changed: expected={sorted(expected)}, actual={sorted(changed)}"
    )
PY

git add "${formatter_files[@]}"
git diff --cached --check
git commit -m "style: Normalize migration formatter debt"

command -v turbo
test "$(turbo --version)" = \
  "$(node -p 'require("./packages/turbo/package.json").version')"
test -z "$(git status --porcelain)"
remote_head="$(
  git ls-remote origin refs/heads/rust/typescript-deprecation |
    awk 'NR == 1 {print $1}'
)"
if [[ "$remote_head" != "$GITHUB_SHA" ]]; then
  echo "migration branch moved during formatting repair: $remote_head" >&2
  exit 1
fi
GIT_TERMINAL_PROMPT=0 git push origin HEAD:rust/typescript-deprecation
