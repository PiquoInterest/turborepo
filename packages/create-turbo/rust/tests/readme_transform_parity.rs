use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use create_turbo_rs::{
    PackageManager, TRANSFORM_NAME, TransformStatus, replace_package_manager_references,
    transform_readme,
};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Result<Self, std::io::Error> {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "create-turbo-rs-{label}-{}-{sequence}",
            std::process::id()
        ));
        match fs::remove_dir_all(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn replaces_compound_run_commands_for_every_supported_manager()
-> Result<(), Box<dyn std::error::Error>> {
    for source in ["pnpm", "npm", "yarn", "bun"] {
        let input = format!("Use `{source} run test` to run tests.");
        assert_eq!(
            replace_package_manager_references(PackageManager::Bun, &input)?,
            "Use `bun run test` to run tests."
        );
    }
    Ok(())
}

#[test]
fn replaces_bare_manager_without_inserting_run() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        replace_package_manager_references(PackageManager::Npm, "Run `pnpm install` first.")?,
        "Run `npm install` first."
    );
    assert_eq!(
        replace_package_manager_references(
            PackageManager::Pnpm,
            "```\nyarn install\nyarn exec turbo\n```"
        )?,
        "```\npnpm install\npnpm exec turbo\n```"
    );
    Ok(())
}

#[test]
fn preserves_package_manager_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    for subcommand in ["dlx", "exec", "add", "init", "install", "create"] {
        let input = format!("`pnpm {subcommand} foo`");
        let expected = format!("`npm {subcommand} foo`");
        let result = replace_package_manager_references(PackageManager::Npm, &input)?;
        assert_eq!(result, expected);
        assert!(!result.contains("npm run"));
    }
    Ok(())
}

#[test]
fn leaves_prose_outside_code_regions_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let input = "This project uses pnpm as its package manager. Install pnpm first.";
    assert_eq!(
        replace_package_manager_references(PackageManager::Npm, input)?,
        input
    );
    Ok(())
}

#[test]
fn replaces_only_inside_backtick_regions() -> Result<(), Box<dyn std::error::Error>> {
    let input = "We recommend pnpm. Run `pnpm install` to get started with pnpm.";
    assert_eq!(
        replace_package_manager_references(PackageManager::Npm, input)?,
        "We recommend pnpm. Run `npm install` to get started with pnpm."
    );
    Ok(())
}

#[test]
fn handles_fenced_blocks_and_language_identifiers() -> Result<(), Box<dyn std::error::Error>> {
    let input = "```sh\npnpm exec turbo build\npnpm run test\n```";
    assert_eq!(
        replace_package_manager_references(PackageManager::Yarn, input)?,
        "```sh\nyarn exec turbo build\nyarn run test\n```"
    );
    Ok(())
}

#[test]
fn handles_multiple_inline_and_fenced_regions() -> Result<(), Box<dyn std::error::Error>> {
    let input = "Run `pnpm install` then:\n\n```sh\npnpm run build\n```\n\nOr use `yarn dev`.";
    assert_eq!(
        replace_package_manager_references(PackageManager::Bun, input)?,
        "Run `bun install` then:\n\n```sh\nbun run build\n```\n\nOr use `bun dev`."
    );
    Ok(())
}

#[test]
fn identity_replacement_does_not_corrupt_content() -> Result<(), Box<dyn std::error::Error>> {
    let input = "```\npnpm install\npnpm run build\n```";
    assert_eq!(
        replace_package_manager_references(PackageManager::Pnpm, input)?,
        input
    );
    Ok(())
}

#[test]
fn leaves_npx_untouched_in_realistic_readme_content() -> Result<(), Box<dyn std::error::Error>> {
    let input = [
        "```sh",
        "npx turbo build",
        "yarn dlx turbo build",
        "pnpm exec turbo build",
        "```",
    ]
    .join("\n");
    let expected = [
        "```sh",
        "npx turbo build",
        "pnpm dlx turbo build",
        "pnpm exec turbo build",
        "```",
    ]
    .join("\n");
    assert_eq!(
        replace_package_manager_references(PackageManager::Pnpm, &input)?,
        expected
    );
    Ok(())
}

#[test]
fn transform_is_not_applicable_without_package_manager() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("no-manager")?;
    fs::write(directory.path().join("README.md"), "Run `pnpm build`.")?;

    let result = transform_readme(directory.path(), None)?;
    assert_eq!(result.result, TransformStatus::NotApplicable);
    assert_eq!(result.name, TRANSFORM_NAME);
    Ok(())
}

#[test]
fn transform_is_not_applicable_without_readme() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("no-readme")?;
    let result = transform_readme(directory.path(), Some(PackageManager::Npm))?;
    assert_eq!(result.result, TransformStatus::NotApplicable);
    assert_eq!(result.name, TRANSFORM_NAME);
    Ok(())
}

#[test]
fn transform_reads_updates_and_writes_readme() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TestDirectory::new("transform")?;
    let readme = directory.path().join("README.md");
    fs::write(&readme, "Run `pnpm run build` and `pnpm install`.")?;

    let result = transform_readme(directory.path(), Some(PackageManager::Yarn))?;
    assert_eq!(result.result, TransformStatus::Success);
    assert_eq!(result.name, TRANSFORM_NAME);
    assert_eq!(
        fs::read_to_string(readme)?,
        "Run `yarn run build` and `yarn install`."
    );
    Ok(())
}
