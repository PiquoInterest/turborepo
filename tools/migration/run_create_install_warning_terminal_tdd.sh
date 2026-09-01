#!/usr/bin/env bash
set -euo pipefail

REPO_SLUG="PiquoInterest/turborepo"
BRANCH="rust/typescript-deprecation"

V1_WORKFLOW_PATH=".github/workflows/repair-create-install-warning-terminal-text.yml"
V2_WORKFLOW_PATH=".github/workflows/repair-create-install-warning-terminal-text-v2.yml"
WORKFLOW_PATH=".github/workflows/complete-create-install-warning-terminal-tdd.yml"
TRANSFORM_SCRIPT="tools/migration/repair_create_install_warning_terminal_text.py"
RUNNER_PATH="tools/migration/run_create_install_warning_terminal_tdd.sh"

TEST_PATH="packages/create-turbo/__tests__/create-install-policy.test.ts"
TEST_RELATIVE="__tests__/create-install-policy.test.ts"
SANITIZER_PATH="packages/create-turbo/src/utils/sanitize-terminal-text.ts"
RENDERER_PATH="packages/create-turbo/src/commands/create/install-warning.ts"
CREATE_PATH="packages/create-turbo/src/commands/create/index.ts"

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

assert_clean() {
  if [[ -n "$(git status --porcelain=v1 --untracked-files=all)" ]]; then
    git status --short >&2
    fail "working tree is not clean"
  fi
}

assert_exact_paths() {
  local mode="$1"
  shift
  local -a expected=("$@")
  local -a actual=()

  if [[ "$mode" == "staged" ]]; then
    mapfile -t actual < <(git diff --cached --name-only | LC_ALL=C sort)
  elif [[ "$mode" == "unstaged" ]]; then
    mapfile -t actual < <(git diff --name-only | LC_ALL=C sort)
  else
    fail "unknown path comparison mode: $mode"
  fi

  if [[ "${#actual[@]}" -ne "${#expected[@]}" ]]; then
    printf 'expected paths:\n' >&2
    printf '  %s\n' "${expected[@]}" >&2
    printf 'actual paths:\n' >&2
    printf '  %s\n' "${actual[@]}" >&2
    fail "$mode path count changed"
  fi

  local index
  for index in "${!expected[@]}"; do
    if [[ "${actual[$index]}" != "${expected[$index]}" ]]; then
      printf 'expected paths:\n' >&2
      printf '  %s\n' "${expected[@]}" >&2
      printf 'actual paths:\n' >&2
      printf '  %s\n' "${actual[@]}" >&2
      fail "$mode path set changed"
    fi
  done
}

test "$GITHUB_REPOSITORY" = "$REPO_SLUG"
test "$GITHUB_REF_NAME" = "$BRANCH"
test "$(git rev-parse HEAD)" = "$GITHUB_SHA"
assert_clean

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

printf '\n== Build the real TypeScript oracle dependency ==\n'
pnpm --filter @turbo/workspaces exec tsdown
pnpm --filter @turbo/workspaces exec tsc -p tsconfig.build.json
test -s packages/turbo-workspaces/dist/index.js
test -s packages/turbo-workspaces/dist/index.mjs
test -s packages/turbo-workspaces/dist/index.d.ts

printf '\n== Prove the committed TypeScript failing oracle is executable ==\n'
pnpm --filter create-turbo exec jest \
  --runInBand \
  --coverage=false \
  "$TEST_RELATIVE" \
  -t "does not pass raw terminal-control text from the example name to warning output"

printf '\n== Commit the TypeScript RED contract ==\n'
python3 "$TRANSFORM_SCRIPT" red
pnpm exec oxfmt "$TEST_PATH"
git diff --check
git add "$TEST_PATH"
assert_exact_paths staged "$TEST_PATH"
git commit -m "test(create-turbo): Expose install warning terminal injection"
RED_SHA="$(git rev-parse HEAD)"

printf '\n== Prove RED fails in behavior, not in the harness ==\n'
set +e
pnpm --filter create-turbo exec jest \
  --runInBand \
  --coverage=false \
  "$TEST_RELATIVE" \
  > /tmp/create-install-warning-red.log 2>&1
red_status=$?
set -e
cat /tmp/create-install-warning-red.log

if [[ "$red_status" -eq 0 ]]; then
  fail "TypeScript RED contract unexpectedly passed"
fi
if grep -Eq \
  'Test suite failed to run|Cannot find module|SyntaxError|ReferenceError' \
  /tmp/create-install-warning-red.log; then
  fail "RED failed in the harness instead of warning behavior"
fi
grep -Fq \
  'does not pass raw terminal-control text from the example name to warning output' \
  /tmp/create-install-warning-red.log
grep -Fq \
  'bounds attacker-controlled example names in warning output' \
  /tmp/create-install-warning-red.log
grep -Eq 'Tests:[[:space:]]+2 failed' /tmp/create-install-warning-red.log

printf '\n== Prove the Rust security implementation is already GREEN ==\n'
cargo test --locked -p create-turbo-rs \
  --test create_install_warning_parity \
  --test create_install_warning_security

printf '\n== Apply and validate the TypeScript GREEN implementation ==\n'
python3 "$TRANSFORM_SCRIPT" green
pnpm exec oxfmt \
  "$SANITIZER_PATH" \
  "$RENDERER_PATH" \
  "$CREATE_PATH" \
  "$TEST_PATH"
git diff --check

pnpm --filter create-turbo exec jest \
  --runInBand \
  --coverage=false \
  "$TEST_RELATIVE"
pnpm --filter create-turbo exec tsc --noEmit --pretty false
pnpm exec oxlint \
  "$SANITIZER_PATH" \
  "$RENDERER_PATH" \
  "$CREATE_PATH" \
  "$TEST_PATH"
pnpm exec oxfmt --check \
  "$SANITIZER_PATH" \
  "$RENDERER_PATH" \
  "$CREATE_PATH" \
  "$TEST_PATH"

cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
git diff --exit-code -- Cargo.toml Cargo.lock package.json pnpm-lock.yaml

if grep -R --line-number --fixed-strings 'unsafe {' \
  packages/create-turbo/rust/src/create_install_policy.rs \
  packages/create-turbo/rust/src/create_error_policy.rs \
  packages/create-turbo/rust/tests/create_install_warning_parity.rs \
  packages/create-turbo/rust/tests/create_install_warning_security.rs; then
  fail "install-warning tranche contains unsafe Rust"
fi

printf '\n== Commit the TypeScript GREEN implementation ==\n'
git add "$SANITIZER_PATH" "$RENDERER_PATH" "$CREATE_PATH"
assert_exact_paths staged \
  "$CREATE_PATH" \
  "$RENDERER_PATH" \
  "$SANITIZER_PATH"
git diff --cached --check
git commit -m "fix(create-turbo): Escape install warning example names"
GREEN_SHA="$(git rev-parse HEAD)"

printf '\n== Record TDD, parity, and security evidence ==\n'
python3 "$TRANSFORM_SCRIPT" docs "$RED_SHA" "$GREEN_SHA"
rm -f "$V2_WORKFLOW_PATH" "$WORKFLOW_PATH" "$RUNNER_PATH"

pnpm exec oxfmt \
  packages/create-turbo/rust/README.md \
  packages/create-turbo/rust/PARITY_MATRIX.md \
  packages/create-turbo/rust/SECURITY.md \
  packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md \
  docs/typescript-deprecation.md \
  docs/rust-migration-security-findings.md
git diff --check

grep -Fq "TS warning oracle RED: $RED_SHA" \
  packages/create-turbo/rust/README.md
grep -Fq "TS warning GREEN:      $GREEN_SHA" \
  packages/create-turbo/rust/README.md
grep -Fq 'fixed for the production TypeScript warning and Rust renderer' \
  packages/create-turbo/rust/SECURITY.md
grep -Fq 'Package-install warning fixed in TypeScript and Rust' \
  docs/rust-migration-security-findings.md
grep -Fq "TypeScript install-warning boundary RED/GREEN: \`$RED_SHA\` / \`$GREEN_SHA\`" \
  docs/typescript-deprecation.md

test ! -e "$V1_WORKFLOW_PATH"
test ! -e "$V2_WORKFLOW_PATH"
test ! -e "$WORKFLOW_PATH"
test ! -e "$TRANSFORM_SCRIPT"
test ! -e "$RUNNER_PATH"

git add -A
git diff --cached --check

python3 <<'PY'
import subprocess

allowed = {
    ".github/workflows/repair-create-install-warning-terminal-text.yml",
    ".github/workflows/repair-create-install-warning-terminal-text-v2.yml",
    ".github/workflows/complete-create-install-warning-terminal-tdd.yml",
    "tools/migration/repair_create_install_warning_terminal_text.py",
    "tools/migration/run_create_install_warning_terminal_tdd.sh",
    "packages/create-turbo/rust/README.md",
    "packages/create-turbo/rust/PARITY_MATRIX.md",
    "packages/create-turbo/rust/SECURITY.md",
    "packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md",
    "docs/typescript-deprecation.md",
    "docs/rust-migration-security-findings.md",
}
changed = set(
    subprocess.check_output(
        ["git", "diff", "--cached", "--name-only"], text=True
    ).splitlines()
)
unexpected = sorted(changed - allowed)
if unexpected:
    raise SystemExit(f"unexpected evidence paths: {unexpected}")
required = {
    ".github/workflows/repair-create-install-warning-terminal-text.yml",
    ".github/workflows/repair-create-install-warning-terminal-text-v2.yml",
    ".github/workflows/complete-create-install-warning-terminal-tdd.yml",
    "tools/migration/repair_create_install_warning_terminal_text.py",
    "tools/migration/run_create_install_warning_terminal_tdd.sh",
    "packages/create-turbo/rust/SECURITY.md",
    "docs/rust-migration-security-findings.md",
}
missing = sorted(required - changed)
if missing:
    raise SystemExit(f"required evidence paths are missing: {missing}")
PY

git commit -m "docs(create-turbo): Record install warning terminal fix"
DOCS_SHA="$(git rev-parse HEAD)"

printf '\n== Validate the exact final source tree ==\n'
assert_clean
test "$(git rev-parse "$RED_SHA^")" = "$GITHUB_SHA"
test "$(git rev-parse "$GREEN_SHA^")" = "$RED_SHA"
test "$(git rev-parse "$DOCS_SHA^")" = "$GREEN_SHA"
test "$(git rev-parse HEAD)" = "$DOCS_SHA"

pnpm --filter create-turbo exec jest --runInBand --coverage=false
pnpm --filter create-turbo exec tsc --noEmit --pretty false
pnpm exec oxlint \
  "$SANITIZER_PATH" \
  "$RENDERER_PATH" \
  "$CREATE_PATH" \
  "$TEST_PATH"
pnpm exec oxfmt --check \
  "$SANITIZER_PATH" \
  "$RENDERER_PATH" \
  "$CREATE_PATH" \
  "$TEST_PATH" \
  packages/create-turbo/rust/README.md \
  packages/create-turbo/rust/PARITY_MATRIX.md \
  packages/create-turbo/rust/SECURITY.md \
  packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md \
  docs/typescript-deprecation.md \
  docs/rust-migration-security-findings.md

cargo fmt --all --check
cargo check --locked -p create-turbo-rs --all-targets
cargo test --locked -p create-turbo-rs --all-targets
cargo clippy --locked -p create-turbo-rs --all-targets -- -D warnings
git diff --exit-code -- Cargo.toml Cargo.lock package.json pnpm-lock.yaml

if grep -Fq 'it.failing(' "$TEST_PATH"; then
  fail "TypeScript production regression still uses it.failing"
fi
grep -Fq 'renderUnavailablePackageManagerWarning(' "$CREATE_PATH"
grep -Fq 'sanitizeTerminalText(' "$RENDERER_PATH"
grep -Fq 'CREATE_INSTALL_WARNING_LINE_LIMIT = 4096' "$RENDERER_PATH"
grep -Fq 'CREATE_INSTALL_WARNING_EXAMPLE_LIMIT' "$RENDERER_PATH"
assert_clean

printf '\n== Build the Turbo binary required by the unchanged pre-push hook ==\n'
cp Cargo.toml /tmp/install-warning-Cargo.toml.original
cp Cargo.lock /tmp/install-warning-Cargo.lock.original

python3 <<'PY'
from pathlib import Path

path = Path("Cargo.toml")
text = path.read_text(encoding="utf-8")
replacements = {
    'biome_diagnostics = { version = "0.5.7" }': 'biome_diagnostics = { version = "=0.5.7" }',
    'biome_formatter = { version = "0.5.7" }': 'biome_formatter = { version = "=0.5.7" }',
    'biome_json_parser = { version = "0.5.7" }': 'biome_json_parser = { version = "=0.5.7" }',
    'biome_json_formatter = { version = "0.5.7" }': 'biome_json_formatter = { version = "=0.5.7" }',
    'biome_json_syntax = { version = "0.5.7" }': 'biome_json_syntax = { version = "=0.5.7" }',
    'unrs_resolver = "1.11.1"': 'unrs_resolver = "=1.11.1"',
}
for old, new in replacements.items():
    if text.count(old) != 1:
        raise SystemExit(f"temporary dependency anchor changed: {old}")
    text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8")
PY

package_is_locked() {
  python3 - "$1" "$2" <<'PY'
import sys
import tomllib
from pathlib import Path

name, version = sys.argv[1:]
lock = tomllib.loads(Path("Cargo.lock").read_text(encoding="utf-8"))
present = any(
    package.get("name") == name and package.get("version") == version
    for package in lock.get("package", [])
)
raise SystemExit(0 if present else 1)
PY
}

downgrade_if_locked() {
  local name="$1"
  local from_version="$2"
  local to_version="$3"
  if package_is_locked "$name" "$from_version"; then
    cargo update -p "$name@$from_version" --precise "$to_version"
  fi
}

downgrade_if_locked biome_console 0.5.8 0.5.7
downgrade_if_locked biome_diagnostics 0.5.8 0.5.7
downgrade_if_locked biome_parser 0.5.8 0.5.7
downgrade_if_locked biome_rowan 0.5.8 0.5.7
downgrade_if_locked biome_string_case 0.5.8 0.5.7
downgrade_if_locked biome_text_edit 0.5.8 0.5.7
downgrade_if_locked biome_text_size 0.5.8 0.5.7
downgrade_if_locked biome_unicode_table 0.5.9 0.5.7
downgrade_if_locked unrs_resolver 1.12.2 1.11.1

cargo build --locked -p turbo --bin turbo
test -x "$GITHUB_WORKSPACE/target/debug/turbo"

cp /tmp/install-warning-Cargo.toml.original Cargo.toml
cp /tmp/install-warning-Cargo.lock.original Cargo.lock
git diff --exit-code -- Cargo.toml Cargo.lock
assert_clean

export PATH="$GITHUB_WORKSPACE/target/debug:$PATH"
"$GITHUB_WORKSPACE/target/debug/turbo" --version

printf '\n== Push the ordered RED, GREEN, and security evidence commits ==\n'
remote_head="$(
  git ls-remote origin "refs/heads/$BRANCH" |
    awk 'NR == 1 { print $1 }'
)"
if [[ "$remote_head" != "$GITHUB_SHA" ]]; then
  fail "integration branch moved during the tranche: $remote_head"
fi

test "$(git rev-parse "$RED_SHA^")" = "$GITHUB_SHA"
test "$(git rev-parse "$GREEN_SHA^")" = "$RED_SHA"
test "$(git rev-parse "$DOCS_SHA^")" = "$GREEN_SHA"
test "$(git rev-parse HEAD)" = "$DOCS_SHA"
assert_clean

git push origin "HEAD:refs/heads/$BRANCH"

printf '\nRED_SHA=%s\nGREEN_SHA=%s\nDOCS_SHA=%s\n' \
  "$RED_SHA" "$GREEN_SHA" "$DOCS_SHA"
