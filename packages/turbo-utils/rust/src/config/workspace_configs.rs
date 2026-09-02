pub fn get_workspace_configs(cwd: Option<&Path>, options: ConfigOptions) -> Vec<WorkspaceConfig> {
    let key = cache_key(cwd);
    if options.cache
        && let Some(key) = key.as_ref()
            && let Ok(cache) = workspace_configs_cache().read()
                && let Some(configs) = cache.get(key) {
                    return configs.clone();
                }

    let Some(root) = get_turbo_root(cwd, TurboRootOptions { cache: options.cache }) else {
        return Vec::new();
    };
    let workspace_globs = get_workspace_globs(&root);
    let mut paths = collect_matching_files(&root, &workspace_globs, "package.json");
    if let Some(root_package) = canonical_inside_root(&root, &root.join("package.json")) {
        paths.retain(|path| path != &root_package);
        paths.insert(0, root_package);
    }
    let mut configs = Vec::new();
    for package_path in paths {
        let Some(raw_package) = read_regular_utf8_limited(&package_path, MAX_PACKAGE_JSON_BYTES)
        else {
            continue;
        };
        let Ok(package) = serde_json::from_str::<Value>(&raw_package) else {
            continue;
        };
        let Some(package_object) = package.as_object() else {
            continue;
        };
        let Some(workspace_path) = package_path.parent() else {
            continue;
        };
        let is_workspace_root = workspace_path == root;
        let workspace_name = package_object
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_owned);

        let resolution = resolve_turbo_config_path(workspace_path);
        let mut invalid_shape = false;
        let turbo_config = if resolution.error.is_some() {
            None
        } else if let Some(config_path) = resolution.config_path {
            match read_turbo_config(&config_path) {
                Ok(config) => {
                    if let Some(object) = config.as_object() {
                        if (is_workspace_root && object.contains_key("extends"))
                            || (!is_workspace_root && !object.contains_key("extends"))
                        {
                            invalid_shape = true;
                            None
                        } else {
                            Some(config)
                        }
                    } else {
                        invalid_shape = true;
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };
        if invalid_shape {
            continue;
        }

        configs.push(WorkspaceConfig {
            workspace_name,
            workspace_path: workspace_path.to_path_buf(),
            is_workspace_root,
            turbo_config,
        });
    }

    if options.cache
        && let Some(key) = key
            && let Ok(mut cache) = workspace_configs_cache().write() {
                cache.insert(key, configs.clone());
            }
    configs
}

