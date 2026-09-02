pub fn get_turbo_configs(
    cwd: Option<&Path>,
    options: ConfigOptions,
) -> Result<Vec<TurboConfig>, TurboConfigError> {
    let key = cache_key(cwd);
    if options.cache
        && let Some(key) = key.as_ref()
            && let Ok(cache) = turbo_configs_cache().read()
                && let Some(configs) = cache.get(key) {
                    return Ok(configs.clone());
                }

    let Some(root) = get_turbo_root(cwd, TurboRootOptions { cache: options.cache }) else {
        return Ok(Vec::new());
    };
    let workspace_globs = get_workspace_globs(&root);
    let paths = collect_matching_files(&root, &workspace_globs, "{turbo.json,turbo.jsonc}");
    let paths = group_turbo_configs(paths)?;
    let mut configs = Vec::new();
    for config_path in paths {
        let Ok(config) = read_turbo_config(&config_path) else {
            continue;
        };
        let Some(object) = config.as_object() else {
            continue;
        };
        let Some(workspace_path) = config_path.parent().map(Path::to_path_buf) else {
            continue;
        };
        let is_root_config = workspace_path == root;
        if (is_root_config && object.contains_key("extends"))
            || (!is_root_config && !object.contains_key("extends"))
        {
            continue;
        }
        configs.push(TurboConfig {
            config,
            turbo_config_path: config_path,
            workspace_path,
            is_root_config,
        });
    }

    if options.cache
        && let Some(key) = key
            && let Ok(mut cache) = turbo_configs_cache().write() {
                cache.insert(key, configs.clone());
            }
    Ok(configs)
}
