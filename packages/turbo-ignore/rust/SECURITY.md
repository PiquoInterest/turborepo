# Security review and intentional compatibility differences

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

The items below are trust-boundary findings in the TypeScript implementation. They are not assigned CVE status here. Severity assumes an attacker can influence repository contents, deployment configuration, commit metadata, or executable search paths used by the deployment job.

## TI-001: Package-spec execution through `npx -y` (High)

**TypeScript behavior:** the inferred or supplied Turbo version is interpolated into `turbo@<value>` and passed to `npx -y`. npm package specs can select tags, aliases, URLs, Git repositories, tarballs, and local paths, not only fixed registry versions.

**Impact:** an attacker who can alter the dependency value or CLI option may cause the deployment job to acquire and execute code outside the intended locked Turbo package.

**Rust fix:** accept only parseable semver requirements, resolve an already-installed canonical Turbo executable, execute it directly, and verify that its reported version satisfies the requirement.

**Residual:** authenticity of the installed binary still depends on the repository install/lockfile and CI image trust chain.

## TI-002: Bare executable search-path substitution (High in hostile CI environments)

**TypeScript behavior:** `git` and `npx` are invoked by bare name.

**Impact:** a hostile or repository-modified `PATH` can select a different executable, especially in package-manager script environments that prepend repository-local binary directories.

**Rust fix:** explicit paths must be absolute; automatic resolution checks a small known-location set and canonicalizes the selected regular executable.

## TI-003: No subprocess deadline or descendant cleanup (Medium)

**TypeScript behavior:** Git and Turbo analysis can wait indefinitely. `maxBuffer` bounds captured output but does not provide a deadline.

**Impact:** a compromised or malfunctioning executable can hang a deployment worker or keep descendants alive.

**Rust fix:** every command has a bounded deadline and bounded stdout/stderr. Unix children run in a new process group and the group is terminated on timeout or output overflow.

**Residual:** Windows descendant-tree termination needs a Job Object implementation before production cutover.

## TI-004: Turbo filter/revision mini-language injection (Medium)

**TypeScript behavior:** workspace, task, and comparison values are inserted into Turbo/Git argument values. `execFile` prevents shell expansion but not semantic manipulation of Turbo filter or Git revision syntax.

**Impact:** crafted values may change which packages or revisions are analyzed and may lead to an incorrect skip decision.

**Rust fix:** validate workspace atoms, task names, and revision syntax before analysis. Leading-dash revisions and filter delimiters are rejected.

## TI-005: Git option confusion for comparison refs (Medium)

**TypeScript behavior:** comparison validation calls `git cat-file -t <ref>` without an option terminator.

**Impact:** a leading-dash value can be interpreted as an option rather than an object name.

**Rust fix:** reject leading dashes and call `git cat-file -e --end-of-options <ref>^{object}`.

## TI-006: Greedy and ambiguous commit directives (Medium)

**TypeScript behavior:** `/\[vercel only .+\]/` is greedy, so multiple `only` directives can collapse into one match. Other contradictory directives return `conflict`, after which analysis continues and may still produce a skip.

**Impact:** ambiguous deployment intent can resolve as a skipped deployment.

**Rust fix:** parse bounded bracketed directives individually. Multiple `only` directives and every contradictory directive force deployment without running Turbo.

## TI-007: Terminal control-sequence injection (Low to Medium)

**TypeScript behavior:** workspace names, refs, package names, paths, and subprocess text are written to terminal logs without a uniform control-character policy.

**Impact:** attacker-controlled text can forge lines, hide context, or emit terminal escape sequences in CI logs.

**Rust fix:** escape control characters and truncate individual log values before output.

## TI-008: Unbounded structured input and result presentation (Medium)

**TypeScript behavior:** configuration files and commit messages are synchronously read without explicit size/depth limits; dependency names may all be joined into one log message.

**Impact:** oversized or deeply nested input can consume memory/CPU or create oversized CI logs.

**Rust fix:** cap package/config/commit sizes, cap JSON5 nesting, bound command output, and display at most 20 dependency names while preserving the total count.

## TI-009: Symlinked configuration crosses the repository boundary (Medium)

**TypeScript behavior:** normal filesystem reads follow symlinks for `package.json` and `turbo.json`.

**Impact:** repository-controlled symlinks can make deployment decisions depend on files outside the checked-out tree or intended workspace.

**Rust fix:** decision-critical configuration must be a bounded regular non-symlink file.

## TI-010: Unsafe workspace text in directive construction (Medium)

**TypeScript behavior:** the workspace string is interpolated into workspace-specific commit directives before validation.

**Impact:** malformed workspace text can make directive matching ambiguous before the Turbo filter is validated.

**Rust fix:** unsafe workspace values are excluded from workspace-scoped directive matching; any workspace directive in that state forces deployment. Global directives remain independent.

## Security invariant

A result of `Skip` is allowed only for an explicit, unambiguous skip directive or a successfully parsed Turbo dry run with an empty package list. Every error and every ambiguous security-sensitive condition returns `Deploy`.

## Advisory lookup record

Lookup date: **2026-08-31**.

Sources checked:

- RustSec Advisory Database and package/advisory index.
- GitHub Advisory Database.
- Official upstream security notices and release information for the direct Rust dependencies and externally executed tools used by this migration core.

The direct dependencies observed for the locked migration build include `clap`, `libc`, `regex`, `semver`, `serde`, `serde_json`, `serde_yaml_ng`, `thiserror`, and test-only `tempfile`. The lookup did not identify a direct advisory against those resolved direct package versions. This statement is limited to the reviewed versions and is not a substitute for scanning the complete transitive graph.

The YAML chain resolves `unsafe-libyaml` `0.2.11`, which is above the `0.2.10` patched floor for `RUSTSEC-2023-0075`. Long-term maintenance and replacement policy remains open even though that historical vulnerability does not affect the observed resolved version.

The complete workspace lockfile contains the separately tracked `webbrowser` finding `RUSTSEC-2026-0257` / `GHSA-2ph8-5cr8-hr33`; affected releases include versions through `1.2.1`, while the workspace declares `0.8.7`. It must be upgraded to `1.2.2` or later, or removed, before migration merge.

Migration CI audits the complete resolved graph and temporarily ignores only that documented pre-existing `webbrowser` advisory so any additional advisory still fails the gate. The exception must be removed with the dependency remediation.
