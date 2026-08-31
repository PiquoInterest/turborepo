use turbo_utils_rs::{
    GIT_REFERENCE_MAX_CHARS, GITHUB_REPOSITORY_URL_MAX_CHARS, GitHubRepositoryLocation, RepoInfo,
    parse_github_repository_location,
};

fn resolved(branch: &str, file_path: &str) -> GitHubRepositoryLocation {
    GitHubRepositoryLocation::Resolved(RepoInfo {
        username: "vercel".into(),
        name: "turborepo".into(),
        branch: branch.into(),
        file_path: file_path.into(),
    })
}

#[test]
fn non_exact_github_https_urls_are_rejected() {
    for url in [
        "http://github.com/vercel/turborepo",
        "https://github.com:443/vercel/turborepo",
        "https://user@github.com/vercel/turborepo",
        "https://github.com.evil.test/vercel/turborepo",
        "https://evil-github.com/vercel/turborepo",
    ] {
        assert!(
            parse_github_repository_location(url, None).is_err(),
            "{url}"
        );
    }
}

#[test]
fn owner_and_repository_identifiers_are_validated() {
    for url in [
        "https://github.com/-owner/repo",
        "https://github.com/owner-/repo",
        "https://github.com/owner_/repo",
        "https://github.com/owner/repo~name",
        "https://github.com/owner/repo:name",
        "https://github.com/owner/repo@name",
        "https://github.com/owner/repo/extra",
    ] {
        assert!(
            parse_github_repository_location(url, None).is_err(),
            "{url}"
        );
    }
}

#[test]
fn dot_prefixed_and_lock_suffixed_repository_names_remain_valid() {
    for name in [".github", "repo..name", "repo.lock"] {
        let url = format!("https://github.com/owner/{name}");
        assert!(
            parse_github_repository_location(&url, None).is_ok(),
            "{name}"
        );
    }
}

#[test]
fn encoded_or_malformed_path_components_are_rejected() {
    for url in [
        "https://github.com/vercel/turborepo%2fother",
        "https://github.com/vercel/turborepo/tree/main/%2e%2e/secret",
        "https://github.com/vercel/turborepo/tree/main\\evil/path",
        "https://github.com/vercel/turborepo/tree/main//path",
        "https://github.com/vercel/turborepo/tree/main/./path",
        "https://github.com/vercel/turborepo/tree/main/../path",
    ] {
        assert!(
            parse_github_repository_location(url, None).is_err(),
            "{url}"
        );
    }
}

#[test]
fn invalid_git_reference_forms_are_rejected() {
    for branch in [
        "-option",
        ".hidden",
        "name.",
        "name.lock",
        "one//two",
        "one..two",
        "one@{two",
        "one~two",
        "one^two",
        "one:two",
        "one*two",
        "one[two",
        "one\\two",
        "@",
    ] {
        let url = format!("https://github.com/vercel/turborepo/tree/{branch}");
        assert!(
            parse_github_repository_location(&url, Some("examples/basic")).is_err(),
            "{branch}"
        );
    }
}

#[test]
fn explicit_example_path_is_matched_literally_not_as_a_regular_expression() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/feature/examples/[basic]",
            Some("examples/[basic]")
        )
        .unwrap(),
        resolved("feature", "examples/[basic]")
    );

    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/feature/examples/(a+)+$",
            Some("examples/(a+)+$")
        )
        .unwrap(),
        resolved("feature", "examples/(a+)+$")
    );
}

#[test]
fn explicit_path_is_removed_only_on_complete_component_suffixes() {
    assert_eq!(
        parse_github_repository_location(
            "https://github.com/vercel/turborepo/tree/feature/examples/basicity",
            Some("examples/basic")
        )
        .unwrap(),
        resolved("feature/examples/basicity", "examples/basic")
    );
}

#[test]
fn example_path_traversal_and_url_delimiters_are_rejected() {
    for path in [
        "../secret",
        "examples/../secret",
        "examples//basic",
        "examples\\basic",
        "examples/%2e%2e/basic",
        "examples/basic?raw=1",
        "examples/basic#readme",
        "examples/basic\nforged",
    ] {
        assert!(
            parse_github_repository_location(
                "https://github.com/vercel/turborepo/tree/main",
                Some(path)
            )
            .is_err(),
            "{path:?}"
        );
    }
}

#[test]
fn repository_url_and_git_reference_lengths_are_bounded() {
    let oversized_url = format!(
        "https://github.com/vercel/{}/tree/main",
        "a".repeat(GITHUB_REPOSITORY_URL_MAX_CHARS)
    );
    assert!(parse_github_repository_location(&oversized_url, None).is_err());

    let oversized_branch = "a".repeat(GIT_REFERENCE_MAX_CHARS + 1);
    let url = format!("https://github.com/vercel/turborepo/tree/{oversized_branch}");
    assert!(parse_github_repository_location(&url, Some("examples/basic")).is_err());
}

#[test]
fn unsupported_github_page_paths_are_not_repository_locations() {
    for url in [
        "https://github.com/vercel/turborepo/issues/1",
        "https://github.com/vercel/turborepo/blob/main/README.md",
        "https://github.com/vercel/turborepo/commit/deadbeef",
        "https://github.com/vercel/turborepo/tree/",
    ] {
        assert!(
            parse_github_repository_location(url, None).is_err(),
            "{url}"
        );
    }
}
