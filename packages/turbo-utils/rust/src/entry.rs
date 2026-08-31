#[path = "lib.rs"]
mod legacy;

mod config;
mod json5;
mod root;

pub use config::{
    ConfigOptions, TurboConfig, TurboConfigError, TurboConfigPathResolution, WorkspaceConfig,
    clear_config_caches, for_each_task_def, get_turbo_configs, get_workspace_configs,
    resolve_turbo_config_path,
};
pub use json5::{Json5Error, parse_json5};
pub use legacy::*;
pub use root::{TurboRootOptions, clear_turbo_root_cache, get_turbo_root};
