pub fn for_each_task_def<F>(config: &Value, mut callback: F)
where
    F: FnMut(&str, &Value),
{
    let Some(object) = config.as_object() else {
        return;
    };
    let tasks = if object.contains_key("pipeline") {
        object.get("pipeline").and_then(Value::as_object)
    } else {
        object.get("tasks").and_then(Value::as_object)
    };
    if let Some(tasks) = tasks {
        for (name, definition) in tasks {
            callback(name, definition);
        }
    }
}

pub fn clear_config_caches() {
    if let Ok(mut cache) = turbo_configs_cache().write() {
        cache.clear();
    }
    if let Ok(mut cache) = workspace_configs_cache().write() {
        cache.clear();
    }
    clear_turbo_root_cache();
}
