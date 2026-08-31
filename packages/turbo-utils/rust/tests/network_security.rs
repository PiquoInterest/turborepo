use turbo_utils_rs::{
    GITHUB_TOKEN_MAX_CHARS, NetworkEnvironment, github_authorization_header, proxy_for_url,
};

#[test]
fn github_authorization_requires_https_without_credentials_or_ports() {
    let mut env = NetworkEnvironment::default();
    env.github_token = Some("token".into());

    for url in [
        "http://api.github.com/repos/user/repo",
        "https://api.github.com:443/repos/user/repo",
        "https://api.github.com:444/repos/user/repo",
        "https://user@api.github.com/repos/user/repo",
        "https://user:pass@api.github.com/repos/user/repo",
    ] {
        assert_eq!(github_authorization_header(url, &env), None, "{url}");
    }
}

#[test]
fn malformed_and_control_bearing_urls_never_receive_credentials() {
    let mut env = NetworkEnvironment::default();
    env.github_token = Some("token".into());

    for url in [
        "api.github.com/repos/user/repo",
        "https:///repos/user/repo",
        "https://api.github.com\n.evil.example/repos/user/repo",
        "https://api.github.com/\u{001b}[31m",
        "https:// api.github.com/repos/user/repo",
    ] {
        assert_eq!(github_authorization_header(url, &env), None, "{url:?}");
    }
}

#[test]
fn invalid_primary_token_does_not_fall_back_to_secondary_credentials() {
    let mut env = NetworkEnvironment::default();
    env.github_token = Some("invalid\ntoken".into());
    env.gh_token = Some("secondary-token".into());

    assert_eq!(
        github_authorization_header("https://api.github.com/repos/user/repo", &env),
        None
    );
}

#[test]
fn tokens_are_ascii_graphic_and_size_bounded() {
    for token in [
        "has space",
        "has\ttab",
        "has\u{001b}escape",
        "ümlaut",
        "line\rreturn",
    ] {
        let mut env = NetworkEnvironment::default();
        env.github_token = Some(token.into());
        assert_eq!(
            github_authorization_header("https://api.github.com/repos/user/repo", &env),
            None,
            "{token:?}"
        );
    }

    let mut at_limit = NetworkEnvironment::default();
    at_limit.github_token = Some("a".repeat(GITHUB_TOKEN_MAX_CHARS));
    assert!(
        github_authorization_header("https://api.github.com/repos/user/repo", &at_limit).is_some()
    );

    let mut oversized = NetworkEnvironment::default();
    oversized.github_token = Some("a".repeat(GITHUB_TOKEN_MAX_CHARS + 1));
    assert_eq!(
        github_authorization_header("https://api.github.com/repos/user/repo", &oversized),
        None
    );
}

#[test]
fn invalid_selected_proxy_is_an_error_instead_of_direct_connection_fallback() {
    let mut env = NetworkEnvironment::default();
    env.https_proxy = Some("not a URL".into());
    env.http_proxy = Some("http://fallback.example:8080".into());

    assert!(proxy_for_url("https://example.com/archive", &env).is_err());
}

#[test]
fn proxy_urls_are_bounded_and_restricted_to_http_or_https() {
    for proxy in [
        "file:///tmp/proxy",
        "socks5://proxy.example:1080",
        "http://proxy.example\n.evil:8080",
        "http:///missing-authority",
    ] {
        let mut env = NetworkEnvironment::default();
        env.https_proxy = Some(proxy.into());
        assert!(
            proxy_for_url("https://example.com/archive", &env).is_err(),
            "{proxy:?}"
        );
    }
}

#[test]
fn malformed_request_url_is_an_error_before_proxy_selection() {
    let mut env = NetworkEnvironment::default();
    env.https_proxy = Some("http://proxy.example:8080".into());
    assert!(proxy_for_url("not a URL", &env).is_err());
}
