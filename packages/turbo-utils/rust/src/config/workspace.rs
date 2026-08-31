fn looks_windows_absolute(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.starts_with("\\\\")
        || value.starts_with("//")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
}

fn is_safe_workspace_glob(workspace_glob: &str) -> bool {
    let glob = workspace_glob.strip_prefix('!').unwrap_or(workspace_glob);
    if glob.is_empty() || Path::new(glob).is_absolute() || looks_windows_absolute(glob) {
        return false;
    }
    !glob
        .split(['/', '\\'])
        .any(|component| component == "..")
}

fn strings_from_sequence(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn get_workspace_globs(root: &Path) -> Vec<String> {
    let pnpm_workspace = root.join("pnpm-workspace.yaml");
    if let Some(content) = read_regular_utf8_limited(&pnpm_workspace, MAX_WORKSPACE_YAML_BYTES) {
        if let Ok(value) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&content) {
            if let Some(packages) = value
                .as_mapping()
                .and_then(|mapping| {
                    let key = serde_yaml_ng::Value::String("packages".to_owned());
                    mapping.get(&key)
                })
                .and_then(serde_yaml_ng::Value::as_sequence)
            {
                return packages
                    .iter()
                    .filter_map(serde_yaml_ng::Value::as_str)
                    .map(str::to_owned)
                    .collect();
            }
        }
        return Vec::new();
    }

    let Some(content) = read_regular_utf8_limited(&root.join("package.json"), MAX_PACKAGE_JSON_BYTES)
    else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&content) else {
        return Vec::new();
    };
    let Some(workspaces) = value.get("workspaces") else {
        return Vec::new();
    };
    if workspaces.is_array() {
        return strings_from_sequence(workspaces);
    }
    workspaces
        .get("packages")
        .map(strings_from_sequence)
        .unwrap_or_default()
}

fn join_glob(workspace_glob: &str, leaf: &str) -> String {
    let trimmed = workspace_glob.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        leaf.to_owned()
    } else {
        format!("{trimmed}/{leaf}")
    }
}

fn canonical_inside_root(root: &Path, candidate: &Path) -> Option<PathBuf> {
    let metadata = fs::symlink_metadata(candidate).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return None;
    }
    let real_root = fs::canonicalize(root).ok()?;
    let real_candidate = fs::canonicalize(candidate).ok()?;
    real_candidate.strip_prefix(&real_root).ok()?;
    Some(candidate.to_path_buf())
}

