#[path = "lib.rs"]
mod legacy;

mod config;
mod json5;
mod managers;
mod root;

pub use config::{
    ConfigOptions, TurboConfig, TurboConfigError, TurboConfigPathResolution, WorkspaceConfig,
    clear_config_caches, for_each_task_def, get_turbo_configs, get_workspace_configs,
    resolve_turbo_config_path,
};
pub use json5::{Json5Error, parse_json5};
pub use legacy::*;
pub use managers::{
    CommandRequest, PackageManager, PackageManagerCommandRunner, PackageManagerValues,
    SystemPackageManagerCommandRunner, PACKAGE_MANAGER_EXEC_TIMEOUT,
    PACKAGE_MANAGER_MAX_OUTPUT_BYTES, get_available_package_managers,
    get_available_package_managers_with, get_package_managers_bin_paths,
    get_package_managers_bin_paths_with,
};
pub use root::{TurboRootOptions, clear_turbo_root_cache, get_turbo_root};
