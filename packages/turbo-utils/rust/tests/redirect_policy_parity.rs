use turbo_utils_rs::{NetworkEnvironment, RedirectRequestPolicy, redirect_request_policy};

#[test]
fn same_origin_github_redirect_recomputes_authorization_and_proxy() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        https_proxy: Some("http://proxy.example:8080".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://api.github.com/repositories/1",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: Some("Bearer token".into()),
            proxy_url: Some("http://proxy.example:8080".into()),
        })
    );
}

#[test]
fn cross_origin_redirect_strips_authorization_even_between_allowed_github_hosts() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        https_proxy: Some("http://proxy.example:8080".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://codeload.github.com/user/repo/tar.gz/main",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: Some("http://proxy.example:8080".into()),
        })
    );
}

#[test]
fn third_party_same_origin_redirect_never_gains_github_authorization() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        https_proxy: Some("http://proxy.example:8080".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://downloads.example.com/start",
            "https://downloads.example.com/archive",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: Some("http://proxy.example:8080".into()),
        })
    );
}

#[test]
fn http_to_https_upgrade_uses_the_target_https_proxy_policy() {
    let environment = NetworkEnvironment {
        https_proxy: Some("http://secure-proxy.example:8443".into()),
        http_proxy: Some("http://plain-proxy.example:8080".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "http://downloads.example.com/start",
            "https://downloads.example.com/archive",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: Some("http://secure-proxy.example:8443".into()),
        })
    );
}

#[test]
fn maximum_redirect_hop_is_accepted() {
    let environment = NetworkEnvironment::default();

    assert_eq!(
        redirect_request_policy(
            "https://downloads.example.com/start",
            "https://downloads.example.com/archive",
            turbo_utils_rs::REDIRECT_MAX_HOPS,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}
