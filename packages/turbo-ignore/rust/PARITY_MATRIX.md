# Parity matrix

Base TypeScript revision: `813d54ae054923e85269979dfa98fe5e47331070`.

| Behavior | TypeScript source/test boundary | Rust status | Notes |
|---|---|---:|---|
| Exit `0` means skip, exit `1` means deploy | `src/ignore.ts`, `__tests__/ignore.test.ts` | Implemented | All errors deploy. |
| Global skip/deploy directives | `src/check-commit.ts`, `__tests__/check-commit.test.ts` | Safe-input parity | Same directive strings, precedence, scope, and reasons. |
| Workspace-specific directives | Same | Safe-input parity | Unsafe workspace text cannot participate in directive matching. |
| Single `[vercel only …]` directive | Same | Safe-input parity | Multiple `only` directives are an intentional security deviation. |
| Conflicting directives | Same | Intentional deviation | Rust immediately deploys; TypeScript continues affected-package analysis. |
| Vercel-provided commit message | Same | Implemented | Does not require Git. |
| Local Git commit message | Same | Implemented | Uses an absolute trusted Git executable and argument-vector invocation. |
| Task selection/default | `src/get-task.ts`, its Jest tests | Implemented | Default remains `build`. |
| Workspace argument/inference | `src/get-workspace.ts`, its Jest tests | Core parity | Reads a string `name` from the current package. File-size and symlink limits are intentional deviations. |
| Turbo version precedence | `src/get-turbo-version.ts`, its Jest tests | Core parity | Argument, dependency, devDependency, then `tasks`/`pipeline` shape. |
| npm selector behavior | `src/get-turbo-version.ts`, `src/ignore.ts` | Intentional deviation | Only semver requirements are accepted; remote/Git/file/alias/tag specs are rejected. |
| JSON5 `turbo.json` shape detection | `src/get-turbo-version.ts` | Conservative parity | Supports comments, quoted/unquoted ASCII keys, trailing commas, strings, arrays, objects, and common JSON5 numbers. Full Unicode-identifier JSON5 remains open. |
| Root discovery | `@turbo/utils#getTurboRoot` call in `src/ignore.ts` | Partial | Nearest non-extending Turbo config, then workspaces/lockfile root. Exact `@manypkg/find-root` edge parity still needs differential fixtures. |
| Vercel comparison selection | `src/get-comparison.ts`, its Jest tests | Implemented | Previous SHA, unreachable fallback, first-deploy behavior, branch logging. |
| Local comparison selection | Same | Implemented | Defaults to `HEAD^`; custom fallback wins. |
| Git object validation | Same | Hardened | Uses `--end-of-options` and an object suffix. |
| Turbo dry-run arguments | `src/ignore.ts`, `__tests__/ignore.test.ts` | Implemented | Calls trusted Turbo directly with `run`, task, filter, and `--dry=json`. |
| Affected/unaffected parsing | Same | Implemented | Empty packages skip; one or more deploy; missing packages means single-package deploy. |
| Known error classification | `src/errors.ts`, `__tests__/errors.test.ts` | Implemented | Preserves warning categories and `UNKNOWN_ERROR`. |
| Logging text/ANSI behavior | `src/logger.ts` | Partial | Core messages preserved; control characters are escaped and long values truncated. Exact colors/spacing are not guaranteed. |
| Telemetry | `src/cli.ts`, `src/ignore.ts` | Not implemented | Must be designed or explicitly retired before distribution cutover. |
| npm executable/package publishing | `package.json`, build/release files | Not implemented | Rust code is not yet the shipped `turbo-ignore` binary. |
| Whole-repository TypeScript removal | Repository-wide | Not implemented | Most core build logic was already Rust; JS ecosystem surfaces remain. |

## Test inventory

- 25 translated parity tests.
- 13 security regression tests.
- 38 Rust tests total in this slice.

These tests were authored against the existing Jest contracts. The validation record in the bundle states whether they were actually compiled and executed in the producing environment.
