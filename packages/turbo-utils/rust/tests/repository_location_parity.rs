use turbo_utils_rs::{GitHubRepositoryLocation, RepoInfo, parse_github_repository_location};

fn resolved(username: &str, name: &str, branch: &str, file_path: &str) -> GitHubRepositoryLocation {
    GitHubRepositoryLocation::Resolved(RepoInfo {
        username: username.into(),
        name: name.into(),
        branch: branch.into(),
        file_path: file_path.into(),
    })
}

#[test]
fn repository_root_requires_default_branch_resolution() {
    for url in [
        "https://github.com/vercel/turborepo",
        "https://github.com/vercel/turborepo/",
    ] {
        assert_eq!(
            parse_github_repository_location(url, None).unwrap(),
            GitHubRepositoryLocation::NeedsDefaultBranch {
                username: "vercel".into(),
                name: "turborepo".into(),
                file_path: String::new(),
            }
        );
    }
}

#[test]
fn repository_root_preserves_an_explicit_example_path() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/",
            Some("examples/basic")
        )
        .unwrap(),
        GitHubRepositoryLocation::NeedsDefaultBranch {
            username: "vercel".into(),
            name: "turborepo".into(),
            file_path: "examples/basic".into(),
        }
    );
}

#[test]
fn tree_url_uses_first_tail_segment_as_branch_without_explicit_path() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/canary/examples/kitchen-sink",
            None
        )
        .unwrap(),
        resolved("vercel", "turborepo", "canary", "examples/kitchen-sink")
    );
}

#[test]
fn explicit_path_allows_a_branch_with_slashes() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/tek/test-branch/",
            Some("examples/basic")
        )
        .unwrap(),
        resolved("vercel", "turborepo", "tek/test-branch", "examples/basic")
    );
}

#[test]
fn explicit_path_is_removed_from_the_url_tail_at_component_boundaries() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/feature/one/examples/basic",
            Some("examples/basic")
        )
        .unwrap(),
        resolved("vercel", "turborepo", "feature/one", "examples/basic")
    );
}

#[test]
fn query_and_fragment_do_not_change_repository_path_parsing() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/main/examples/basic?plain=1#readme",
            None
        )
        .unwrap(),
        resolved("vercel", "turborepo", "main", "examples/basic")
    );
}

#[test]
fn github_authority_and_scheme_are_ascii_case_insensitive() {
    assert_eq!(
        parse_github_repository_location(
            "HTTPS://GITHUB.COM/vercel/turborepo/tree/main/crates",
            None
        )
        .unwrap(),
        resolved("vercel", "turborepo", "main", "crates")
    );
}

#[test]
fn explicit_path_not_present_in_url_leaves_the_whole_tail_as_the_branch() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/release/next",
            Some("examples/basic")
        )
        .unwrap(),
        resolved("vercel", "turborepo", "release/next", "examples/basic")
    );
}
