use turbo_utils_rs::{
    NetworkEnvironment, NetworkPolicyError, REDIRECT_MAX_HOPS, RedirectRequestPolicy,
    redirect_request_policy,
};

#[test]
fn https_to_http_redirect_is_rejected() {
    assert_eq!(
        redirect_request_policy(
            "https://downloads.example.com/start",
            "http://downloads.example.com/archive",
            1,
            &NetworkEnvironment::default(),
        ),
        Err(NetworkPolicyError::InsecureRedirect)
    );
}

#[test]
fn redirect_hop_must_be_within_the_closed_limit() {
    for hop in [0, REDIRECT_MAX_HOPS + 1, usize::MAX] {
        assert_eq!(
            redirect_request_policy(
                "https://downloads.example.com/start",
                "https://downloads.example.com/archive",
                hop,
                &NetworkEnvironment::default(),
            ),
            Err(NetworkPolicyError::RedirectLimitExceeded),
            "hop={hop}"
        );
    }
}

#[test]
fn redirect_target_recomputes_no_proxy_instead_of_reusing_the_source_dispatcher() {
    let environment = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some("internal.example".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://internal.example/archive",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}

#[test]
fn invalid_target_proxy_is_not_downgraded_to_a_direct_redirect() {
    let environment = NetworkEnvironment {
        https_proxy: Some("not a proxy URL".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://downloads.example.com/start",
            "https://downloads.example.com/archive",
            1,
            &environment,
        ),
        Err(NetworkPolicyError::InvalidProxyUrl)
    );
}

#[test]
fn userinfo_in_either_redirect_authority_is_rejected() {
    for (source, target) in [
        (
            "https://user@downloads.example.com/start",
            "https://downloads.example.com/archive",
        ),
        (
            "https://downloads.example.com/start",
            "https://user:pass@downloads.example.com/archive",
        ),
    ] {
        assert_eq!(
            redirect_request_policy(source, target, 1, &NetworkEnvironment::default()),
            Err(NetworkPolicyError::InvalidRequestUrl)
        );
    }
}

#[test]
fn unsupported_or_malformed_redirect_schemes_are_rejected() {
    for target in [
        "ftp://downloads.example.com/archive",
        "file:///tmp/archive",
        "javascript:alert(1)",
        "//downloads.example.com/archive",
        "https:///archive",
    ] {
        assert_eq!(
            redirect_request_policy(
                "https://downloads.example.com/start",
                target,
                1,
                &NetworkEnvironment::default(),
            ),
            Err(NetworkPolicyError::InvalidRequestUrl),
            "target={target:?}"
        );
    }
}

#[test]
fn untrusted_origin_redirecting_to_github_does_not_gain_a_token() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://attacker.example/start",
            "https://api.github.com/repos/user/repo",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}

#[test]
fn github_lookalike_redirect_target_never_receives_authorization() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://api.github.com.attacker.invalid/collect",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}

#[test]
fn explicit_port_redirect_target_does_not_receive_github_authorization() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://api.github.com:443/repositories/1",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}
