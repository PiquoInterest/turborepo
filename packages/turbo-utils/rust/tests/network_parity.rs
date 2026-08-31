use turbo_utils_rs::{NetworkEnvironment, github_authorization_header, proxy_for_url};

fn environment() -> NetworkEnvironment {
    NetworkEnvironment::default()
}

#[test]
fn github_token_takes_precedence_over_gh_token() {
    let mut env = environment();
    env.github_token = Some("primary-token".into());
    env.gh_token = Some("secondary-token".into());

    assert_eq!(
        github_authorization_header("https://api.github.com/repos/user/repo", &env),
        Some("Bearer primary-token".into())
    );
}

#[test]
fn gh_token_is_used_when_github_token_is_absent() {
    let mut env = environment();
    env.gh_token = Some("fallback-token".into());

    assert_eq!(
        github_authorization_header("https://api.github.com/repos/user/repo", &env),
        Some("Bearer fallback-token".into())
    );
}

#[test]
fn no_authorization_header_is_emitted_without_a_token() {
    assert_eq!(
        github_authorization_header(
            "https://api.github.com/repos/vercel/turborepo",
            &environment()
        ),
        None
    );
}

#[test]
fn authorization_is_limited_to_exact_github_api_hosts() {
    let mut env = environment();
    env.github_token = Some("token".into());

    for url in [
        "https://api.github.com/repos/user/repo",
        "https://codeload.github.com/user/repo/tar.gz/main",
    ] {
        assert_eq!(
            github_authorization_header(url, &env),
            Some("Bearer token".into())
        );
    }

    for url in [
        "https://github.com/user/repo",
        "https://api.github.com.evil.com/repos/user/repo",
        "https://evil-api.github.com/repos/user/repo",
        "https://codeload.github.com.attacker.io/user/repo/tar.gz/main",
        "https://example.com/some-api",
    ] {
        assert_eq!(github_authorization_header(url, &env), None, "{url}");
    }
}

#[test]
fn https_proxy_precedence_matches_the_typescript_helper() {
    let mut env = environment();
    env.https_proxy = Some("http://lower-https.example:8080".into());
    env.https_proxy_upper = Some("http://upper-https.example:8080".into());
    env.http_proxy = Some("http://lower-http.example:8080".into());
    env.http_proxy_upper = Some("http://upper-http.example:8080".into());

    assert_eq!(
        proxy_for_url("https://api.github.com/repos/user/repo", &env),
        Ok(Some("http://lower-https.example:8080".into()))
    );
}

#[test]
fn https_falls_back_through_uppercase_and_http_proxy_values() {
    let mut env = environment();
    env.https_proxy_upper = Some("http://upper-https.example:8080".into());
    env.http_proxy = Some("http://lower-http.example:8080".into());
    assert_eq!(
        proxy_for_url("https://example.com/archive", &env),
        Ok(Some("http://upper-https.example:8080".into()))
    );

    env.https_proxy_upper = None;
    assert_eq!(
        proxy_for_url("https://example.com/archive", &env),
        Ok(Some("http://lower-http.example:8080".into()))
    );
}

#[test]
fn non_https_urls_use_only_http_proxy_precedence() {
    let mut env = environment();
    env.https_proxy = Some("http://ignored.example:8080".into());
    env.http_proxy_upper = Some("http://upper-http.example:8080".into());

    assert_eq!(
        proxy_for_url("http://example.com/archive", &env),
        Ok(Some("http://upper-http.example:8080".into()))
    );
}
