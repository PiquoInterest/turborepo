use std::error::Error;

use turbo_utils_rs::{
    NetworkEnvironment, NetworkPolicyError, RedirectChain, RedirectRequestPolicy,
    github_authorization_header, proxy_for_url, redirect_request_policy,
};

const EXPECTED_REQUEST_URL_MAX_BYTES: usize = 8 * 1_024;

fn github_environment() -> NetworkEnvironment {
    NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    }
}

fn github_url_with_length(length: usize) -> String {
    const PREFIX: &str = "https://api.github.com/";
    assert!(length >= PREFIX.len());
    format!("{PREFIX}{}", "a".repeat(length - PREFIX.len()))
}

#[test]
fn explicit_default_https_port_receives_github_authorization() {
    let environment = github_environment();

    for url in [
        "https://api.github.com:443/repos/user/repo",
        "https://codeload.github.com:443/user/repo/tar.gz/main",
        "https://API.GITHUB.COM:443/repos/user/repo",
    ] {
        assert_eq!(
            github_authorization_header(url, &environment),
            Some("Bearer token".into()),
            "{url}"
        );
    }
}

#[test]
fn implicit_and_explicit_default_ports_are_the_same_authorized_origin(
) -> Result<(), Box<dyn Error>> {
    let environment = github_environment();
    let mut chain = RedirectChain::new(
        "https://api.github.com/repos/user/repo",
        &environment,
    )?;

    assert_eq!(
        chain.follow("https://api.github.com:443/repos/user/repo/archive")?,
        RedirectRequestPolicy {
            authorization_header: Some("Bearer token".into()),
            proxy_url: None,
        }
    );
    Ok(())
}

#[test]
fn request_url_boundary_preserves_safe_input() -> Result<(), Box<dyn Error>> {
    let environment = github_environment();
    let url = github_url_with_length(EXPECTED_REQUEST_URL_MAX_BYTES);

    assert_eq!(url.len(), EXPECTED_REQUEST_URL_MAX_BYTES);
    assert_eq!(
        github_authorization_header(&url, &environment),
        Some("Bearer token".into())
    );
    assert_eq!(proxy_for_url(&url, &environment)?, None);
    let chain = RedirectChain::new(&url, &environment)?;
    assert_eq!(chain.current_url(), url);
    Ok(())
}

#[test]
fn oversized_request_urls_fail_closed_across_policy_entrypoints() {
    let environment = github_environment();
    let oversized = github_url_with_length(EXPECTED_REQUEST_URL_MAX_BYTES + 1);

    assert_eq!(
        github_authorization_header(&oversized, &environment),
        None
    );
    assert_eq!(
        proxy_for_url(&oversized, &environment),
        Err(NetworkPolicyError::InvalidRequestUrl)
    );
    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            &oversized,
            1,
            &environment,
        ),
        Err(NetworkPolicyError::InvalidRequestUrl)
    );
    assert!(matches!(
        RedirectChain::new(&oversized, &environment),
        Err(NetworkPolicyError::InvalidRequestUrl)
    ));
}

#[test]
fn oversized_redirect_target_does_not_mutate_chain_state() -> Result<(), Box<dyn Error>> {
    let environment = github_environment();
    let mut chain = RedirectChain::new(
        "https://api.github.com/repos/user/repo",
        &environment,
    )?;
    let oversized = github_url_with_length(EXPECTED_REQUEST_URL_MAX_BYTES + 1);
    let original_url = chain.current_url().to_owned();
    let original_policy = chain.current_policy();
    let original_hops = chain.redirect_hops();

    assert_eq!(
        chain.follow(&oversized),
        Err(NetworkPolicyError::InvalidRequestUrl)
    );
    assert_eq!(chain.current_url(), original_url);
    assert_eq!(chain.current_policy(), original_policy);
    assert_eq!(chain.redirect_hops(), original_hops);
    Ok(())
}
