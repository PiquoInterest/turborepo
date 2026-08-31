use crate::PackageManagerInstallProfile;

pub const CREATE_OUTPUT_FIELD_LIMIT: usize = 1024;
pub const CREATE_OUTPUT_LINE_LIMIT: usize = 4096;
pub const CREATE_OUTPUT_WORKSPACE_LIMIT: usize = 256;
pub const CREATE_OUTPUT_SCRIPT_LIMIT: usize = 64;
pub const CREATE_OUTPUT_TRUNCATION_LINE: &str = " - [truncated]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateWorkspaceDisplay<'a> {
    pub group: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateDisplayScript {
    Build,
    Dev,
    Test,
    Lint,
}

impl CreateDisplayScript {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Dev => "dev",
            Self::Test => "test",
            Self::Lint => "lint",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Dev => "Develop",
            Self::Test => "Test",
            Self::Lint => "Lint",
        }
    }
}

#[must_use]
pub fn render_create_workspace_summary(
    workspaces: &[CreateWorkspaceDisplay<'_>],
    fallback_project_name: &str,
) -> Vec<String> {
    if workspaces.is_empty() {
        return vec!["apps".to_owned(), format!(" - {fallback_project_name}")];
    }

    let mut lines = Vec::with_capacity(workspaces.len().saturating_mul(2));
    let mut last_group: Option<&str> = None;

    for (index, workspace) in workspaces.iter().enumerate() {
        if index == 0 || last_group != Some(workspace.group) {
            lines.push(workspace.group.to_owned());
        }

        let description = workspace
            .description
            .filter(|description| !description.is_empty())
            .map(|description| format!(": {description}"))
            .unwrap_or_default();
        lines.push(format!(" - {}{description}", workspace.title));
        last_group = Some(workspace.group);
    }

    lines
}

#[must_use]
pub fn render_create_success(
    project_dir_is_current: bool,
    relative_project_dir: &str,
) -> String {
    if project_dir_is_current {
        ">>> Success! Your new Turborepo is ready.".to_owned()
    } else {
        format!(">>> Success! Created your Turborepo at {relative_project_dir}")
    }
}

#[must_use]
pub fn render_create_get_started(
    has_package_json: bool,
    project_dir_is_current: bool,
    relative_project_dir: &str,
    profile: Option<&PackageManagerInstallProfile>,
    scripts: &[CreateDisplayScript],
) -> Vec<String> {
    let Some(profile) = profile else {
        return Vec::new();
    };
    if !has_package_json {
        return Vec::new();
    }

    let mut lines = Vec::with_capacity(scripts.len().saturating_add(7));
    lines.push(String::new());
    lines.push("To get started:".to_owned());
    if !project_dir_is_current {
        lines.push(format!(
            "- Change to the directory: cd {relative_project_dir}"
        ));
    }
    lines.push(format!(
        "- Enable Remote Caching (recommended): {} turbo login",
        profile.executable
    ));
    lines.push("   - Learn more: https://turborepo.dev/remote-cache".to_owned());
    lines.push(String::new());
    lines.push("- Run commands with Turborepo:".to_owned());
    for script in scripts {
        lines.push(format!(
            "   - {} run {}: {} all apps and packages",
            profile.command.as_str(),
            script.as_str(),
            script.description()
        ));
    }
    lines.push("- Run a command twice to hit cache".to_owned());
    lines
}
