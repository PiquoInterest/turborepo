fn collect_matching_files(root: &Path, workspace_globs: &[String], leaf: &str) -> Vec<PathBuf> {
    let mut positive = vec![leaf.to_owned()];
    let mut negative = Vec::new();
    for workspace_glob in workspace_globs
        .iter()
        .filter(|glob| is_safe_workspace_glob(glob))
    {
        if let Some(glob) = workspace_glob.strip_prefix('!') {
            negative.push(join_glob(glob, leaf));
        } else {
            positive.push(join_glob(workspace_glob, leaf));
        }
    }

    let negative: Vec<Glob<'static>> = negative
        .into_iter()
        .filter_map(|pattern| Glob::new(&pattern).map(Glob::into_owned).ok())
        .collect();
    let mut files = BTreeSet::new();
    for pattern in positive {
        let Ok(glob) = Glob::new(&pattern) else {
            continue;
        };
        for entry in glob.walk(root) {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            if negative.iter().any(|excluded| excluded.is_match(relative)) {
                continue;
            }
            if let Some(path) = canonical_inside_root(root, path) {
                files.insert(path);
                if files.len() >= MAX_DISCOVERED_CONFIGS {
                    return files.into_iter().collect();
                }
            }
        }
    }
    files.into_iter().collect()
}

fn group_turbo_configs(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>, TurboConfigError> {
    let mut grouped: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for path in paths {
        let Some(directory) = path.parent() else {
            continue;
        };
        grouped
            .entry(directory.to_path_buf())
            .or_default()
            .push(path);
    }

    let mut result = Vec::new();
    for (directory, mut paths) in grouped {
        paths.sort();
        if paths.len() > 1 {
            return Err(TurboConfigError::DuplicateConfig {
                directory: directory.display().to_string(),
            });
        }
        if let Some(path) = paths.pop() {
            result.push(path);
        }
    }
    Ok(result)
}

fn read_turbo_config(path: &Path) -> Result<Value, Json5Error> {
    let content = read_regular_utf8_limited(path, crate::json5::MAX_JSON5_BYTES)
        .ok_or(Json5Error::InputTooLarge)?;
    parse_json5(&content)
}

fn cache_key(cwd: Option<&Path>) -> Option<PathBuf> {
    cwd.map(Path::to_path_buf)
}

