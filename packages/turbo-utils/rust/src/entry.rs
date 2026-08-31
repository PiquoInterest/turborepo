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
    MANAGER_COMMAND_TIMEOUT, MAX_MANAGER_CONFIG_BYTES, MAX_MANAGER_OUTPUT_BYTES, ManagerCommand,
    ManagerCommandRunner, ManagerDetectionOptions, PackageManagers, SystemManagerCommandRunner,
    get_available_package_managers, get_available_package_managers_with,
    get_package_managers_bin_paths, get_package_managers_bin_paths_with, parse_manager_version,
    resolve_executable_in_path,
};
pub use root::{TurboRootOptions, clear_turbo_root_cache, get_turbo_root};
