use turbo_workspaces_rs::{WorkspacePackages, parse_workspace_packages};

#[test]
fn missing_workspaces_return_an_empty_list() {
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Missing),
        Ok(Vec::<&str>::new())
    );
}

#[test]
fn an_empty_workspace_array_remains_empty() {
    let globs: [&str; 0] = [];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Ok(Vec::<&str>::new())
    );
}

#[test]
fn an_array_is_returned_in_source_order() {
    let globs = ["apps/*", "packages/*"];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Ok(vec!["apps/*", "packages/*"])
    );
}

#[test]
fn an_object_packages_array_is_returned_in_source_order() {
    let globs = ["apps/*", "packages/*"];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Object {
            packages: Some(&globs),
        }),
        Ok(vec!["apps/*", "packages/*"])
    );
}

#[test]
fn an_object_without_packages_returns_an_empty_list() {
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Object { packages: None }),
        Ok(Vec::<&str>::new())
    );
}

#[test]
fn ordering_duplicates_and_empty_values_are_preserved() {
    let globs = ["packages/*", "", "packages/*", "apps/*"];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Ok(vec!["packages/*", "", "packages/*", "apps/*"])
    );
}

#[test]
fn general_workspace_glob_syntax_is_not_restricted_to_the_bun_subset() {
    let globs = ["!fixtures/**", "apps/{web,docs}", "packages/[ab]"];
    assert_eq!(
        parse_workspace_packages(WorkspacePackages::Array(&globs)),
        Ok(vec!["!fixtures/**", "apps/{web,docs}", "packages/[ab]"])
    );
}
