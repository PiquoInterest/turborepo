#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RED = "36b49a6cfad94bab8487dda62871b60c99a84115"
GREEN = "4f00ff3ebe627acb5a15ead535f27d623d8a9a2c"


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"expected one anchor in {path}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, addition.rstrip() + "\n\n" + anchor)


def write_ledger() -> None:
    path = ROOT / "packages/create-turbo/rust/PACKAGE_MANAGER_PROMPT_DIVERGENCES.md"
    if path.exists():
        raise SystemExit("package-manager prompt divergence ledger already exists")
    path.write_text(f"""# Package-manager prompt parity and divergence ledger

TypeScript oracle: `packages/create-turbo/src/commands/create/prompts.ts`.

Rust target: `packages/create-turbo/rust/src/package_manager_prompt.rs`.

## Preserved behavior

- `skipTransforms` returns no selection before package-manager discovery or prompting.
- The source choice order is `npm`, `pnpm`, `yarn`, `bun`, `nub`, `aube`.
- An exact requested manager with a truthy discovered version is returned without prompting.
- Unknown, unavailable, or empty-version manager arguments fall back to the selector.
- Installed choices sort before unavailable choices while retaining source order within both groups.
- The selected manager returns its exact discovered version.
- Selector failure is propagated and never retried.

## Divergences and type conversions

| Area | TypeScript behavior | Rust behavior | Classification and reason |
| --- | --- | --- | --- |
| Requested manager | Free-form value cast with `as PackageManager`, then object indexing | Exact six-literal parse into `WorkspacePackageManager` | Intentional type hardening. Unknown, confusable, path-like, case-changed, or whitespace-padded text cannot become a trusted manager key. |
| Availability | JavaScript object value is checked by truthiness | Borrowed `Option<&str>` with empty string treated as unavailable | Type conversion preserving the declared string/undefined contract and JavaScript empty-string behavior. |
| Choice ordering | Stable JavaScript sort by installed boolean | Stable Rust sort by `disabled` | Parity. Installed choices move first without reordering peers. |
| Prompt UI | Inquirer disables unavailable choices | Typed selector receives six choices and Rust revalidates the result | Intentional defense in depth. A compromised or faulty adapter cannot return a disabled manager. |
| All managers unavailable | Inquirer receives six disabled choices, with host-specific cancellation behavior | Selector may fail, or an unavailable result is rejected | Explicit boundary. No version is fabricated and no retry loop is introduced. |
| Large manager text | Dynamic property lookup input | Borrowed exact comparisons only | Resource hardening. Input is not normalized, copied, logged, or added to the choice list. |
| Discovery and terminal UI | Async process discovery plus interactive Inquirer prompt | Dependency-injected availability and selector traits | Representation only. Production providers and non-TTY/cancellation differentials remain blocked. |

## Security and production requirements

The core adds no crate, process execution, filesystem access, network call, terminal output, unsafe code, or mutable global state. Production binding remains blocked until it proves:

1. exact discovery results for all six managers without shell construction or project-local executable substitution;
2. bounded subprocess duration and output plus descendant cleanup;
3. exact prompt labels, disabled text, source order, cancellation, non-TTY, and signal behavior;
4. terminal-control-safe rendering;
5. no retry or fallback to an unavailable manager;
6. Linux, macOS, and Windows differential fixtures;
7. removal proof showing the TypeScript prompt logic is neither loaded nor shipped.

## TDD evidence

- RED integration commit: `{RED}`.
- GREEN integration commit: `{GREEN}`.
- Parity tests: `tests/package_manager_prompt_parity.rs`.
- Security tests: `tests/package_manager_prompt_security.rs`.
""", encoding="utf-8")


def update_readme() -> None:
    path = "packages/create-turbo/rust/README.md"
    replace_once(path,
        "7. the fixed-order transform-pipeline and fatal/nonfatal error-control contract.",
        "7. the fixed-order transform-pipeline and fatal/nonfatal error-control contract.\n8. the package-manager prompt resolution and installed-choice ordering contract.")
    section = """### Package-manager prompt core

- returns no selection without discovery or prompting when transforms are skipped;
- preserves the exact six-choice source order;
- resolves an exact requested and installed manager without prompting;
- treats an empty discovered version as JavaScript-falsey;
- stably moves installed choices ahead of unavailable choices;
- propagates selector failure without retry;
- revalidates the selected manager and rejects disabled results.

Discovery and terminal UI remain behind typed providers. Exact string casting, truthiness, disabled-choice validation, and remaining platform/UI differences are recorded in [`PACKAGE_MANAGER_PROMPT_DIVERGENCES.md`](./PACKAGE_MANAGER_PROMPT_DIVERGENCES.md)."""
    insert_before(path, "## Not yet implemented in Rust", section)
    replace_once(path,
        "- interactive prompts;",
        "- production package-manager discovery and interactive prompt providers, including cancellation and non-TTY behavior;")
    replace_once(path,
        "`transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow.",
        "`transform_pipeline` owns the fixed transform order and typed fatal/nonfatal control flow. `package_manager_prompt` owns exact manager parsing, discovered-version truthiness, stable choice ordering, and disabled-selection validation.")
    replace_once(path,
        "Pipeline GREEN:          7b208824412f008a942567faa5e37740948a541e",
        f"Pipeline GREEN:          7b208824412f008a942567faa5e37740948a541e\nPackage prompt RED:      {RED}\nPackage prompt GREEN:    {GREEN}")
    replace_once(path,
        "The crate contains 65 translated parity tests and 46 security regression tests, for 111 authored focused Rust tests.",
        "The crate contains 73 translated parity tests and 51 security regression tests, for 124 authored focused Rust tests.")


def update_parity() -> None:
    path = "packages/create-turbo/rust/PARITY_MATRIX.md"
    section = """## Package-manager prompt tranche

| TypeScript boundary | Rust boundary | Status | Evidence and notes |
| --- | --- | --- | --- |
| `skipTransforms` early return | `resolve_package_manager_prompt` returns `None` | implemented core | Discovery and selector providers are untouched. |
| source choices `npm`, `pnpm`, `yarn`, `bun`, `nub`, `aube` | `PACKAGE_MANAGER_PROMPT_ORDER` | implemented core | Exact order and closed variants are tested. |
| requested installed manager | exact parse plus truthy version | implemented core | Bypasses selector like the source. |
| unknown or unavailable manager | selector path | implemented core | No free-form value crosses the typed boundary. |
| stable installed-first sort | stable sort by disabled state | implemented core | Relative order inside both groups is preserved. |
| empty discovered version | unavailable | implemented core | Matches JavaScript string truthiness. |
| selector cancellation/error | propagated once | implemented core | No retry or synthesized fallback. |
| disabled selection | explicit unavailable-selection error | intentional-hardening | Defense in depth beyond Inquirer's UI disable flag. |
| process discovery and interactive Inquirer behavior | production providers | blocked | Requires secure execution, cancellation/non-TTY/signal parity, terminal-safe UI, and platform differentials. |

Detailed differences are in `PACKAGE_MANAGER_PROMPT_DIVERGENCES.md`."""
    insert_before(path, "## Existing TypeScript test mapping", section)
    replace_once(path,
        "| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |",
        "| fixed-pipeline/error-boundary regressions | seven security tests | intentional-hardening evidence |\n| package-manager prompt source contract | eight translated parity tests | implemented core |\n| manager-cast, disabled-choice, confusable, and bound regressions | five security tests | intentional-hardening evidence |")
    replace_once(path,
        "| interactive prompts | not-implemented | Preserve defaults, cancellation, validation, non-TTY behavior, and ordering. |",
        "| interactive prompts | package-manager decision core implemented, providers blocked | Add secure manager discovery, Inquirer-compatible UI, cancellation/non-TTY/signal behavior, platform differentials, binding, and removal proof. |")


def update_security() -> None:
    path = "packages/create-turbo/rust/SECURITY.md"
    replace_once(path,
        "- the transform loop and `handleErrors` in `packages/create-turbo/src/commands/create/index.ts`",
        "- the transform loop and `handleErrors` in `packages/create-turbo/src/commands/create/index.ts`\n- package-manager selection in `packages/create-turbo/src/commands/create/prompts.ts`")
    trust = """Package-manager prompting receives free-form CLI text, discovered executable versions, terminal input, cancellation, and non-TTY state. The reviewed Rust core accepts only a closed enum after exact parsing and revalidates every selected manager against a non-empty discovered version. Discovery and UI effects remain provider-owned."""
    insert_before(path, "The transform pipeline decides which mutation stages run", trust)
    findings = """### CT-RS-025: Unchecked package-manager casting broadens a trusted key

**Severity:** Medium

The TypeScript prompt casts free-form input to `PackageManager` before indexing discovery results. The Rust core parses only six exact literals. Case changes, whitespace, paths, controls, Unicode confusables, and oversized unknown text cannot become direct manager selections or expand the choice set.

### CT-RS-026: Disabled prompt choices require provider-side revalidation

**Severity:** Medium

The source relies on Inquirer to prevent selection of unavailable managers. A faulty or compromised adapter could violate that UI contract. Rust re-reads the selected manager's discovered version and rejects absent or empty values. It never fabricates a version or retries automatically.

### CT-RS-027: Package-manager discovery and prompt UI remain privileged providers

**Severity:** High until provider closure

Actual discovery can execute manager binaries and interactive prompting consumes attacker-controlled terminal state. Production providers must use canonical executables, explicit environment policy, no shell, bounded duration/output, descendant cleanup, terminal-safe rendering, exact cancellation/non-TTY/signal behavior, and supported-platform differential tests.

The decision core adds no dependencies or direct side effects."""
    insert_before(path, "## Security invariants", findings)


def update_repository_docs() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(path,
        "- `packages/create-turbo/rust`: 65 translated parity tests and 46 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager and official-starter orchestration, and transform-pipeline control flow.",
        "- `packages/create-turbo/rust`: 73 translated parity tests and 51 security regression tests across README rewriting, `.gitignore` creation, Git initialization orchestration, exact default-example routing, package-manager prompt/transform and official-starter orchestration, and transform-pipeline control flow.")
    replace_once(path, "That is **271 authored Rust migration tests** on the integration branch.", "That is **284 authored Rust migration tests** on the integration branch.")
    replace_once(path, "the evidence-weighted estimate is now about **74%**", "the evidence-weighted estimate is now about **75%**")
    section = """### Package-manager prompt resolution

The Rust decision core preserves the exact six choices, skip branch, direct installed-manager branch, JavaScript empty-string truthiness, stable installed-first order, selected version, and one-shot cancellation propagation. It intentionally replaces the unchecked TypeScript cast with exact enum parsing and rejects disabled selector results.

Production discovery and terminal UI remain blocked pending secure executable resolution, bounded process handling, exact prompt/cancellation/non-TTY/signal behavior, terminal-safe rendering, supported-platform differentials, host binding, and removal proof. Exact differences are in `packages/create-turbo/rust/PACKAGE_MANAGER_PROMPT_DIVERGENCES.md`.
"""
    insert_before(path, "### Transform-pipeline orchestration", section)
    replace_once(path,
        "- transform-pipeline implementation: `7b208824412f008a942567faa5e37740948a541e`.",
        f"- transform-pipeline implementation: `7b208824412f008a942567faa5e37740948a541e`.\n- package-manager prompt RED: `{RED}`.\n- package-manager prompt implementation: `{GREEN}`.")

    path = "docs/rust-migration-security-findings.md"
    finding = """### RF-017: Package-manager prompt casting and disabled choices need a typed provider boundary

**Status:** Decision core implemented; discovery and UI providers blocked.

The TypeScript prompt casts free-form manager text before indexing discovered versions and relies on Inquirer to enforce disabled choices. The Rust core accepts only six exact literals, preserves source ordering and truthiness, and revalidates the selected manager against a non-empty discovered version. It never retries or fabricates a fallback.

Production closure requires canonical no-shell discovery with bounded process handling, exact interactive cancellation/non-TTY/signal behavior, terminal-safe rendering, supported-platform differentials, host binding, and TypeScript removal proof. Regression evidence is in the package-manager prompt parity/security tests and `PACKAGE_MANAGER_PROMPT_DIVERGENCES.md`."""
    insert_before(path, "## Required repository gates", finding)
    replace_once(path,
        "- close the transform-pipeline async binding, telemetry, terminal-safe logging, cleanup-before-exit, runtime typing, and supported-platform differential contract;",
        "- close the transform-pipeline async binding, telemetry, terminal-safe logging, cleanup-before-exit, runtime typing, and supported-platform differential contract;\n- close the package-manager discovery and prompt provider contract, including canonical execution, cancellation, non-TTY/signals, terminal-safe UI, and supported-platform differentials;")


def main() -> None:
    write_ledger()
    update_readme()
    update_parity()
    update_security()
    update_repository_docs()
    (ROOT / "tools/migration/apply_package_manager_prompt_docs.py").unlink()


if __name__ == "__main__":
    main()
