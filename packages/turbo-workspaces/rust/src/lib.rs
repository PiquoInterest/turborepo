include!("workspace_core.rs");

mod workspace_packages;
pub use workspace_packages::{
    WORKSPACE_PACKAGE_GLOB_COUNT_LIMIT, WORKSPACE_PACKAGE_GLOB_INPUT_LIMIT,
    WORKSPACE_PACKAGE_GLOB_TOTAL_INPUT_LIMIT, WorkspacePackages, WorkspacePackagesError,
    parse_workspace_packages,
};
