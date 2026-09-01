#!/usr/bin/env python3
"""Stage a compiling behavioral RED for turbo-workspaces workspace detection."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRATE = ROOT / "packages/turbo-workspaces/rust"
PROGRAM = ROOT / "docs/typescript-deprecation.md"
INVENTORY = ROOT / "docs/rust-migration-test-inventory.md"
REPOSITORY_SECURITY = ROOT / "docs/rust-migration-security-findings.md"
ROOT_MANIFEST = ROOT / "Cargo.toml"

ORACLE_SHA = "4e7fb108a32798fc2a9f8c2f3b9caa3ae18c78ff"
RED_MARKER = "RED commit: recorded after the behavioral proof."


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:160]!r}"
        )
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def insert_before(path: Path, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def write_new(relative: str, content: str) -> None:
    path = ROOT / relative
    if path.exists():
        raise SystemExit(f"refusing to overwrite existing migration file: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def stage_manifest() -> None:
    replace_once(
        ROOT_MANIFEST,
        '  "packages/create-turbo/rust",\n  "packages/turbo-ignore/rust",',
        '  "packages/create-turbo/rust",\n  "packages/turbo-workspaces/rust",\n  "packages/turbo-ignore/rust",',
    )
    write_new(
        "packages/turbo-workspaces/rust/Cargo.toml",
        """[package]
name = "turbo-workspaces-rs"
version = "0.1.0"
edition = { workspace = true }
license = "MIT"
publish = false
description = "Rust migration of turbo-workspaces behavior"

[lib]
path = "src/lib.rs"

[lints]
workspace = true
""",
    )


def stage_source() -> None:
    write_new(
        "packages/turbo-workspaces/rust/src/lib.rs",
        """use std::{
    fmt,
    path::{Path, PathBuf},
};

pub const MANAGER_DETECTION_ORDER: [WorkspaceManager; 6] = [
    WorkspaceManager::Aube,
    WorkspaceManager::Nub,
    WorkspaceManager::Pnpm,
    WorkspaceManager::Yarn,
    WorkspaceManager::Npm,
    WorkspaceManager::Bun,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceManager {
    Aube,
    Nub,
    Pnpm,
    Yarn,
    Npm,
    Bun,
}

impl WorkspaceManager {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aube => "aube",
            Self::Nub => "nub",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Npm => "npm",
            Self::Bun => "bun",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDirectoryInfo {
    pub absolute: PathBuf,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDetailsKnownError {
    InvalidDirectory { absolute: PathBuf },
    UnableToDetect,
}

impl WorkspaceDetailsKnownError {
    #[must_use]
    pub const fn error_type(&self) -> &'static str {
        match self {
            Self::InvalidDirectory { .. } => "invalid_directory",
            Self::UnableToDetect => "package_manager-unable_to_detect",
        }
    }

    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::InvalidDirectory { absolute } => format!(
                "Could not find directory at {}. Ensure the directory exists.",
                absolute.display()
            ),
            Self::UnableToDetect =>
                "Could not determine package manager. Add `devEngines.packageManager` or legacy `packageManager` to `package.json`, or ensure a lockfile is present."
                    .to_owned(),
        }
    }
}

impl fmt::Display for WorkspaceDetailsKnownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDetailsError<E> {
    Known(WorkspaceDetailsKnownError),
    Provider(E),
}

impl<E: fmt::Display> fmt::Display for WorkspaceDetailsError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Known(error) => error.fmt(formatter),
            Self::Provider(error) => error.fmt(formatter),
        }
    }
}

pub trait WorkspaceDetailsProvider {
    type Project;
    type Error;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error>;
    fn detect(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<bool, Self::Error>;
    fn read(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<Self::Project, Self::Error>;
}

pub fn get_workspace_details<P>(
    root: &Path,
    provider: &mut P,
) -> Result<P::Project, WorkspaceDetailsError<P::Error>>
where
    P: WorkspaceDetailsProvider,
{
    let directory = provider
        .directory_info(root)
        .map_err(WorkspaceDetailsError::Provider)?;

    if !directory.exists {
        return Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::InvalidDirectory {
                absolute: directory.absolute,
            },
        ));
    }

    // Compiling behavioral RED: the final API and known directory error are
    // present, but manager detection and reading are intentionally absent.
    Err(WorkspaceDetailsError::Known(
        WorkspaceDetailsKnownError::UnableToDetect,
    ))
}
""",
    )


def parity_tests() -> str:
    return """use std::path::{Path, PathBuf};

use turbo_workspaces_rs::{
    MANAGER_DETECTION_ORDER, WorkspaceDetailsError, WorkspaceDetailsKnownError,
    WorkspaceDetailsProvider, WorkspaceDirectoryInfo, WorkspaceManager, get_workspace_details,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeProject {
    manager: WorkspaceManager,
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderError {
    Directory,
    Detect(WorkspaceManager),
    Read(WorkspaceManager),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Directory(PathBuf),
    Detect(WorkspaceManager, PathBuf),
    Read(WorkspaceManager, PathBuf),
}

struct FakeProvider {
    directory: Result<WorkspaceDirectoryInfo, ProviderError>,
    detected: Option<WorkspaceManager>,
    detect_error: Option<WorkspaceManager>,
    read_error: Option<WorkspaceManager>,
    calls: Vec<Call>,
}

impl FakeProvider {
    fn existing(absolute: &Path) -> Self {
        Self {
            directory: Ok(WorkspaceDirectoryInfo {
                absolute: absolute.to_path_buf(),
                exists: true,
            }),
            detected: None,
            detect_error: None,
            read_error: None,
            calls: Vec::new(),
        }
    }
}

impl WorkspaceDetailsProvider for FakeProvider {
    type Project = FakeProject;
    type Error = ProviderError;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error> {
        self.calls.push(Call::Directory(root.to_path_buf()));
        self.directory.clone()
    }

    fn detect(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<bool, Self::Error> {
        self.calls
            .push(Call::Detect(manager, workspace_root.to_path_buf()));
        if self.detect_error == Some(manager) {
            return Err(ProviderError::Detect(manager));
        }
        Ok(self.detected == Some(manager))
    }

    fn read(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<Self::Project, Self::Error> {
        self.calls
            .push(Call::Read(manager, workspace_root.to_path_buf()));
        if self.read_error == Some(manager) {
            return Err(ProviderError::Read(manager));
        }
        Ok(FakeProject {
            manager,
            root: workspace_root.to_path_buf(),
        })
    }
}

#[test]
fn manager_order_matches_the_typescript_registry() {
    assert_eq!(
        MANAGER_DETECTION_ORDER.map(WorkspaceManager::as_str),
        ["aube", "nub", "pnpm", "yarn", "npm", "bun"]
    );
}

#[test]
fn missing_directory_returns_the_exact_known_error_before_detection() {
    let raw = Path::new("relative-input");
    let absolute = PathBuf::from("/safe/absolute/missing");
    let mut provider = FakeProvider {
        directory: Ok(WorkspaceDirectoryInfo {
            absolute: absolute.clone(),
            exists: false,
        }),
        detected: Some(WorkspaceManager::Aube),
        detect_error: None,
        read_error: None,
        calls: Vec::new(),
    };

    let result = get_workspace_details(raw, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::InvalidDirectory {
                absolute: absolute.clone(),
            }
        ))
    );
    let WorkspaceDetailsError::Known(error) = result.err().unwrap_or_else(|| {
        panic!("the missing directory must return a known error");
    }) else {
        panic!("the missing directory must not return a provider error");
    };
    assert_eq!(error.error_type(), "invalid_directory");
    assert_eq!(
        error.message(),
        "Could not find directory at /safe/absolute/missing. Ensure the directory exists."
    );
    assert_eq!(provider.calls, [Call::Directory(raw.to_path_buf())]);
}

#[test]
fn first_detected_manager_is_read_and_later_managers_are_not_consulted() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);
    provider.detected = Some(WorkspaceManager::Pnpm);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Ok(FakeProject {
            manager: WorkspaceManager::Pnpm,
            root: root.to_path_buf(),
        })
    );
    assert_eq!(
        provider.calls,
        [
            Call::Directory(root.to_path_buf()),
            Call::Detect(WorkspaceManager::Aube, root.to_path_buf()),
            Call::Detect(WorkspaceManager::Nub, root.to_path_buf()),
            Call::Detect(WorkspaceManager::Pnpm, root.to_path_buf()),
            Call::Read(WorkspaceManager::Pnpm, root.to_path_buf()),
        ]
    );
}

#[test]
fn selected_manager_read_failure_propagates_without_parser_fallback() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);
    provider.detected = Some(WorkspaceManager::Pnpm);
    provider.read_error = Some(WorkspaceManager::Pnpm);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Provider(ProviderError::Read(
            WorkspaceManager::Pnpm
        )))
    );
    assert_eq!(
        provider.calls.last(),
        Some(&Call::Read(WorkspaceManager::Pnpm, root.to_path_buf()))
    );
    assert!(!provider.calls.iter().any(|call| matches!(
        call,
        Call::Detect(WorkspaceManager::Yarn, _)
            | Call::Detect(WorkspaceManager::Npm, _)
            | Call::Detect(WorkspaceManager::Bun, _)
    )));
}

#[test]
fn all_six_rejections_return_the_exact_unable_to_detect_error() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider::existing(root);

    let result = get_workspace_details(root, &mut provider);
    assert_eq!(
        result,
        Err(WorkspaceDetailsError::Known(
            WorkspaceDetailsKnownError::UnableToDetect
        ))
    );
    let WorkspaceDetailsError::Known(error) = result.err().unwrap_or_else(|| {
        panic!("all manager rejections must return a known error");
    }) else {
        panic!("all manager rejections must not return a provider error");
    };
    assert_eq!(error.error_type(), "package_manager-unable_to_detect");
    assert_eq!(
        error.message(),
        "Could not determine package manager. Add `devEngines.packageManager` or legacy `packageManager` to `package.json`, or ensure a lockfile is present."
    );
    assert_eq!(
        provider
            .calls
            .iter()
            .filter(|call| matches!(call, Call::Detect(_, _)))
            .count(),
        6
    );
}

#[test]
fn directory_provider_failure_propagates_before_manager_authority() {
    let root = Path::new("/workspace");
    let mut provider = FakeProvider {
        directory: Err(ProviderError::Directory),
        detected: None,
        detect_error: None,
        read_error: None,
        calls: Vec::new(),
    };

    assert_eq!(
        get_workspace_details(root, &mut provider),
        Err(WorkspaceDetailsError::Provider(ProviderError::Directory))
    );
    assert_eq!(provider.calls, [Call::Directory(root.to_path_buf())]);
}
"""


def security_tests() -> str:
    return """use std::path::{Path, PathBuf};

use turbo_workspaces_rs::{
    MANAGER_DETECTION_ORDER, WorkspaceDetailsError, WorkspaceDetailsProvider,
    WorkspaceDirectoryInfo, WorkspaceManager, get_workspace_details,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderError {
    Detect(WorkspaceManager),
    Read(WorkspaceManager),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Directory(PathBuf),
    Detect(WorkspaceManager, PathBuf),
    Read(WorkspaceManager, PathBuf),
}

struct SecurityProvider {
    absolute: PathBuf,
    detected: Option<WorkspaceManager>,
    detect_error: Option<WorkspaceManager>,
    read_error: Option<WorkspaceManager>,
    calls: Vec<Call>,
}

impl SecurityProvider {
    fn new(absolute: &Path) -> Self {
        Self {
            absolute: absolute.to_path_buf(),
            detected: None,
            detect_error: None,
            read_error: None,
            calls: Vec::new(),
        }
    }
}

impl WorkspaceDetailsProvider for SecurityProvider {
    type Project = WorkspaceManager;
    type Error = ProviderError;

    fn directory_info(&mut self, root: &Path) -> Result<WorkspaceDirectoryInfo, Self::Error> {
        self.calls.push(Call::Directory(root.to_path_buf()));
        Ok(WorkspaceDirectoryInfo {
            absolute: self.absolute.clone(),
            exists: true,
        })
    }

    fn detect(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<bool, Self::Error> {
        self.calls
            .push(Call::Detect(manager, workspace_root.to_path_buf()));
        if self.detect_error == Some(manager) {
            return Err(ProviderError::Detect(manager));
        }
        Ok(self.detected == Some(manager))
    }

    fn read(
        &mut self,
        manager: WorkspaceManager,
        workspace_root: &Path,
    ) -> Result<Self::Project, Self::Error> {
        self.calls
            .push(Call::Read(manager, workspace_root.to_path_buf()));
        if self.read_error == Some(manager) {
            return Err(ProviderError::Read(manager));
        }
        Ok(manager)
    }
}

#[test]
fn manager_identity_is_closed_ascii_data() {
    let names = MANAGER_DETECTION_ORDER.map(WorkspaceManager::as_str);
    assert_eq!(names, ["aube", "nub", "pnpm", "yarn", "npm", "bun"]);
    for name in names {
        assert!(!name.is_empty());
        assert!(name.is_ascii());
        assert!(name.bytes().all(|byte| byte.is_ascii_lowercase()));
    }
}

#[test]
fn detection_error_stops_without_trying_a_less_trusted_parser() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);
    provider.detect_error = Some(WorkspaceManager::Pnpm);
    provider.detected = Some(WorkspaceManager::Yarn);

    assert_eq!(
        get_workspace_details(root, &mut provider),
        Err(WorkspaceDetailsError::Provider(ProviderError::Detect(
            WorkspaceManager::Pnpm
        )))
    );
    assert!(!provider.calls.iter().any(|call| matches!(
        call,
        Call::Detect(WorkspaceManager::Yarn, _)
            | Call::Read(WorkspaceManager::Yarn, _)
    )));
}

#[test]
fn false_detectors_never_receive_read_authority() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);
    provider.detected = Some(WorkspaceManager::Npm);

    let _ = get_workspace_details(root, &mut provider);
    for manager in [
        WorkspaceManager::Aube,
        WorkspaceManager::Nub,
        WorkspaceManager::Pnpm,
        WorkspaceManager::Yarn,
    ] {
        assert!(!provider
            .calls
            .contains(&Call::Read(manager, root.to_path_buf())));
    }
    assert!(provider
        .calls
        .contains(&Call::Read(WorkspaceManager::Npm, root.to_path_buf())));
}

#[test]
fn the_provider_absolute_path_is_the_only_path_given_to_managers() {
    let raw = Path::new("relative-or-user-facing-input");
    let absolute = Path::new("/trusted/absolute/workspace");
    let mut provider = SecurityProvider::new(absolute);
    provider.detected = Some(WorkspaceManager::Aube);

    assert_eq!(
        get_workspace_details(raw, &mut provider),
        Ok(WorkspaceManager::Aube)
    );
    assert_eq!(provider.calls[0], Call::Directory(raw.to_path_buf()));
    assert_eq!(
        provider.calls[1],
        Call::Detect(WorkspaceManager::Aube, absolute.to_path_buf())
    );
    assert_eq!(
        provider.calls[2],
        Call::Read(WorkspaceManager::Aube, absolute.to_path_buf())
    );
}

#[test]
fn unable_to_detect_work_is_bounded_to_the_fixed_registry() {
    let root = Path::new("/workspace");
    let mut provider = SecurityProvider::new(root);

    let _ = get_workspace_details(root, &mut provider);
    assert_eq!(
        provider
            .calls
            .iter()
            .filter(|call| matches!(call, Call::Detect(_, _)))
            .count(),
        MANAGER_DETECTION_ORDER.len()
    );
    assert!(!provider
        .calls
        .iter()
        .any(|call| matches!(call, Call::Read(_, _))));
}
"""


def stage_tests() -> None:
    write_new(
        "packages/turbo-workspaces/rust/tests/workspace_details_parity.rs",
        parity_tests(),
    )
    write_new(
        "packages/turbo-workspaces/rust/tests/workspace_details_security.rs",
        security_tests(),
    )


def stage_component_docs() -> None:
    write_new(
        "packages/turbo-workspaces/rust/README.md",
        f"""# turbo-workspaces Rust migration

This crate is the Rust migration target for executable behavior in `packages/turbo-workspaces`.

## Current tranche

The initial tranche captures the read-only `getWorkspaceDetails` orchestration contract:

- inspect the requested directory before manager detection;
- use the provider-returned absolute path for all manager operations;
- detect managers serially in exact source insertion order: `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`;
- read only the first manager that detects the workspace;
- propagate detector or selected-reader failures without falling through to another parser;
- return the exact known invalid-directory and unable-to-detect error types and messages.

TypeScript oracle: `{ORACLE_SHA}`.

{RED_MARKER}

The Rust function intentionally returns the unable-to-detect error after a successful directory check. This makes the translated tests compile and fail for missing manager orchestration rather than missing APIs.

## Validation

```sh
cargo fmt --all --check
cargo check --locked -p turbo-workspaces-rs --all-targets
cargo test --locked -p turbo-workspaces-rs --all-targets
cargo clippy --locked -p turbo-workspaces-rs --all-targets -- -D warnings
pnpm --filter @turbo/workspaces exec jest --runInBand --coverage=false __tests__/workspace-details.test.ts
```

## Production status

This is a RED test contract, not a production implementation. Filesystem inspection, manager-specific parsing, bindings, packaging, platform differentials, callers, and TypeScript removal remain blocked.
""",
    )
    write_new(
        "packages/turbo-workspaces/rust/PARITY_MATRIX.md",
        f"""# turbo-workspaces TypeScript-to-Rust parity matrix

## Workspace-details orchestration

| TypeScript boundary | Rust boundary | Status | Evidence |
| --- | --- | --- | --- |
| `directoryInfo({{ directory: root }})` before detection | `WorkspaceDetailsProvider::directory_info` | RED contract | Missing-directory behavior is implemented; manager orchestration remains deliberately absent. |
| manager insertion order `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun` | `MANAGER_DETECTION_ORDER` | RED contract | Constant and TypeScript oracle are exact. |
| serial first-success detection | `WorkspaceDetailsProvider::detect` | RED contract | Translated parity test currently fails by design. |
| selected manager read | `WorkspaceDetailsProvider::read` | RED contract | Must execute once only for the first detection. |
| detector/read rejection propagation | `WorkspaceDetailsError::Provider` | RED contract | No fallback parser may receive authority after an error. |
| exact known errors | `WorkspaceDetailsKnownError` | partial | Invalid directory is implemented; unable-to-detect sequencing remains RED. |
| actual filesystem and manager parsers | production providers | blocked | Requires bounded no-follow reads, parser limits, path identity, platform differentials, bindings, and removal proof. |

TypeScript oracle: `{ORACLE_SHA}`.

{RED_MARKER}
""",
    )
    write_new(
        "packages/turbo-workspaces/rust/SECURITY.md",
        f"""# turbo-workspaces Rust migration security review

## Trust boundaries

The workspace-details entry point accepts an untrusted root path, delegates path resolution and existence checks, then chooses one manager detector and reader. Detection order controls which parser obtains filesystem authority. A fallback after a detector or selected-reader error could reinterpret ambiguous repository state under a less appropriate parser.

## Findings and required fixes

### TW-RS-001: Manager detection order is security-relevant

**Severity:** Medium

The TypeScript registry order is `aube`, `nub`, `pnpm`, `yarn`, `npm`, `bun`. Rust exposes that order as a closed six-element enum array. The GREEN implementation must iterate it serially and stop at the first detection.

### TW-RS-002: Parser errors must not trigger fallback reinterpretation

**Severity:** Medium

The TypeScript source does not catch detector or selected-reader failures. Falling through after such an error could give another manager parser authority over malformed or conflicting metadata. Rust must propagate the provider error immediately.

### TW-RS-003: Path resolution and later reads remain a TOCTOU boundary

**Severity:** High until provider closure

The orchestration core accepts the absolute path returned by its directory provider, but it does not own stable filesystem handles. Production providers must reject unsafe links and special files, bound reads and enumeration, revalidate identity, define Windows reparse-point behavior, and avoid path substitution between detection and reading.

### TW-RS-004: Detection work must remain bounded

**Severity:** Low

The manager set is fixed to six enum variants. The GREEN implementation must perform at most six detection calls and one read. It must not accept an extensible free-form registry at this trust boundary.

## TDD evidence

TypeScript oracle: `{ORACLE_SHA}`.

{RED_MARKER}

The RED source deliberately omits manager iteration after a successful directory check. Tests compile and fail for the missing behavior.

## Advisory lookup

**Lookup date: 2026-09-01**

Sources checked:

- RustSec Advisory Database and advisory repository;
- GitHub Advisory Database for the Rust and npm ecosystems;
- package manager and parser dependency records already present in the repository.

This RED crate adds no dependency. The repository-wide `webbrowser`, `h2`, and `quick-xml` findings remain open and are not ignored by this tranche.

## Residual risk and production blockers

No production filesystem provider or manager parser is implemented. Required closure includes bounded regular-file reads, strict parser depth and size limits, root confinement, link/reparse-point policy, stable identity across detect/read, deterministic errors, cancellation, Linux/macOS/Windows differential fixtures, host binding, packaging, caller cutover, and artifact-removal proof.
""",
    )
    write_new(
        "packages/turbo-workspaces/rust/security.txt",
        f"""# Rust migration security index for packages/turbo-workspaces
# Canonical narrative review: SECURITY.md
# Vulnerability disclosure policy: ../../../SECURITY.md

Component: turbo-workspaces-rs
Status: behavioral-red
Production-Cutover: blocked
TypeScript-Removal: not-started
TypeScript-Oracle: {ORACLE_SHA}
{RED_MARKER}

Security-Requirement: TW-RS-001 fixed six-manager detection order
Security-Requirement: TW-RS-002 no parser fallback after detector or reader error
Security-Requirement: TW-RS-003 stable bounded no-follow filesystem provider required
Security-Requirement: TW-RS-004 at most six detections and one read
New-Dependencies: none
Unsafe-Code: none
""",
    )


def stage_repository_docs() -> None:
    replace_once(
        PROGRAM,
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.\n- `crates/turborepo-telemetry::events::package`: 9 translated parity tests and 7 security regression tests for the package-facing telemetry contract.",
        "- `packages/create-turbo/rust`: 116 translated parity tests and 92 security regression tests across README/`.gitignore`, Git, default/official routing, transform and prompt policy, error/install/output policy, installation profiles, and project-directory selection.\n- `packages/turbo-workspaces/rust`: 6 parity and 5 security tests in an intentional behavioral RED for read-only workspace detection.\n- `crates/turborepo-telemetry::events::package`: 9 translated parity tests and 7 security regression tests for the package-facing telemetry contract.",
    )
    replace_once(
        PROGRAM,
        "That is **382 authored Rust migration tests** on the integration branch.",
        "That is **393 authored Rust migration tests** on the integration branch: 382 previously GREEN tests plus 11 intentional workspace-details RED tests. Test count remains evidence coverage, not a completion percentage.",
    )
    replace_once(
        PROGRAM,
        "| `packages/turbo-workspaces` | Rust CLI/library | Queued and partially exposed through provider boundary | Package-manager adapters, complete six-manager conversion, lock/workspace mutation semantics, rollback, process policy, and packaging. |",
        "| `packages/turbo-workspaces` | `packages/turbo-workspaces/rust` | RED-first read-only orchestration tranche | Complete workspace-details GREEN, then port manager detection/read providers, complete six-manager conversion, lock/workspace mutations, rollback, process policy, bindings, packaging, callers, and removal proof. |",
    )
    section = f"""## Current `turbo-workspaces` workspace-details RED

The TypeScript oracle commit `{ORACLE_SHA}` pins exact manager insertion order, missing-directory errors, serial first-success detection, selected-reader behavior, no fallback after read failure, and the final unable-to-detect error.

The Rust crate contains six parity and five security tests. Its current source intentionally returns unable-to-detect after a successful directory check so the tests compile and fail behaviorally. {RED_MARKER}

The GREEN implementation must perform at most six detector calls and one selected read, use only the provider-returned absolute path, and propagate provider failures without parser fallback. Filesystem and manager-specific providers remain blocked on bounded no-follow reads, stable path identity, parser limits, platform differentials, bindings, and removal proof.
"""
    insert_before(PROGRAM, "## Current `create-turbo` tranches", section)

    replace_once(
        INVENTORY,
        "| `__tests__/index.test.ts` | package-manager transform request core only | Partial | Full conversion orchestration, error order, dry-run behavior, manager lifecycle, transaction and rollback. |",
        "| `__tests__/index.test.ts` | package-manager transform request core plus focused `workspace-details.test.ts` oracle and `workspace_details_*` Rust tests | Partial, workspace-details RED | Complete read-only orchestration GREEN, then port conversion order, dry-run behavior, manager lifecycle, transaction, and rollback. |",
    )
    section = f"""## `turbo-workspaces` workspace-details TDD chain

- TypeScript oracle: `{ORACLE_SHA}`
- Rust parity tests: 6
- Rust security tests: 5
- {RED_MARKER}

The RED contract is deliberately limited to read-only orchestration. It does not grant production filesystem or process authority and does not count as a completed test-suite mapping until GREEN.
"""
    insert_before(INVENTORY, "## Bounded matcher TDD chain", section)

    if "### RF-028:" in REPOSITORY_SECURITY.read_text(encoding="utf-8"):
        raise SystemExit("RF-028 is already allocated")
    finding = f"""### RF-028: Workspace manager detection can broaden parser authority

**Status:** Behavioral RED contract committed; implementation pending.

`getWorkspaceDetails` uses the insertion order of the TypeScript manager registry to decide which parser reads a repository. Changing the order, retrying after a detector or selected-reader error, or accepting an extensible free-form registry could reinterpret conflicting or malformed repository state under a different manager.

The Rust RED contract closes manager identity to six enum variants and adds tests for source order, first-success behavior, exact known errors, provider error propagation, the provider-returned absolute path, and a six-detection/one-read work bound. {RED_MARKER}

Production closure additionally requires bounded no-follow filesystem reads, stable identity between detection and parsing, parser size/depth limits, Windows reparse-point behavior, supported-platform differentials, bindings, packaging, callers, and TypeScript removal proof.
"""
    insert_before(REPOSITORY_SECURITY, "## Required repository gates", finding)


def main() -> None:
    stage_manifest()
    stage_source()
    stage_tests()
    stage_component_docs()
    stage_repository_docs()
    print("staged turbo-workspaces workspace-details behavioral RED")


if __name__ == "__main__":
    main()
