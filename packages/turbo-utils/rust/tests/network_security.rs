use turbo_utils_rs::{
    GITHUB_TOKEN_MAX_CHARS, NO_PROXY_MAX_CHARS, NO_PROXY_MAX_ENTRIES, NetworkEnvironment,
    NetworkPolicyError, github_authorization_header, proxy_for_url,
};

#[test]
fn github_authorization_requires_https_without_credentials_or_ports() {
    let env = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

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
    let env = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

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
    let env = NetworkEnvironment {
        github_token: Some("invalid\ntoken".into()),
        gh_token: Some("secondary-token".into()),
        ..Default::default()
    };

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
        let env = NetworkEnvironment {
            github_token: Some(token.into()),
            ..Default::default()
        };
        assert_eq!(
            github_authorization_header("https://api.github.com/repos/user/repo", &env),
            None,
            "{token:?}"
        );
    }

    let at_limit = NetworkEnvironment {
        github_token: Some("a".repeat(GITHUB_TOKEN_MAX_CHARS)),
        ..Default::default()
    };
    assert!(
        github_authorization_header("https://api.github.com/repos/user/repo", &at_limit).is_some()
    );

    let oversized = NetworkEnvironment {
        github_token: Some("a".repeat(GITHUB_TOKEN_MAX_CHARS + 1)),
        ..Default::default()
    };
    assert_eq!(
        github_authorization_header("https://api.github.com/repos/user/repo", &oversized),
        None
    );
}

#[test]
fn invalid_selected_proxy_is_an_error_instead_of_direct_connection_fallback() {
    let env = NetworkEnvironment {
        https_proxy: Some("not a URL".into()),
        http_proxy: Some("http://fallback.example:8080".into()),
        ..Default::default()
    };

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
        let env = NetworkEnvironment {
            https_proxy: Some(proxy.into()),
            ..Default::default()
        };
        assert!(
            proxy_for_url("https://example.com/archive", &env).is_err(),
            "{proxy:?}"
        );
    }
}

#[test]
fn malformed_request_url_is_an_error_before_proxy_selection() {
    let env = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        ..Default::default()
    };
    assert!(proxy_for_url("not a URL", &env).is_err());
}

#[test]
fn no_proxy_domain_matching_uses_label_boundaries_not_substrings() {
    let env = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some(".example.com".into()),
        ..Default::default()
    };

    for url in [
        "https://notexample.com/archive",
        "https://example.com.attacker.invalid/archive",
    ] {
        assert_eq!(
            proxy_for_url(url, &env),
            Ok(Some("http://proxy.example:8080".into())),
            "{url}"
        );
    }
}

#[test]
fn invalid_winning_no_proxy_value_is_an_error_without_uppercase_fallback() {
    let env = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some("https://example.com/path".into()),
        no_proxy_upper: Some("*".into()),
        ..Default::default()
    };

    assert_eq!(
        proxy_for_url("https://example.com/archive", &env),
        Err(NetworkPolicyError::InvalidNoProxy)
    );
}

#[test]
fn no_proxy_rejects_ambiguous_wildcards_unicode_and_cidr_rules() {
    for rule in ["*.example.com", "exаmple.com", "10.0.0.0/8"] {
        let env = NetworkEnvironment {
            https_proxy: Some("http://proxy.example:8080".into()),
            no_proxy: Some(rule.into()),
            ..Default::default()
        };
        assert_eq!(
            proxy_for_url("https://example.com/archive", &env),
            Err(NetworkPolicyError::InvalidNoProxy),
            "{rule:?}"
        );
    }
}

#[test]
fn no_proxy_values_are_bounded_by_length_and_entry_count() {
    let oversized = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some("a".repeat(NO_PROXY_MAX_CHARS + 1)),
        ..Default::default()
    };
    assert_eq!(
        proxy_for_url("https://example.com/archive", &oversized),
        Err(NetworkPolicyError::InvalidNoProxy)
    );

    let too_many_entries = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some(
            (0..=NO_PROXY_MAX_ENTRIES)
                .map(|index| format!("host{index}.example"))
                .collect::<Vec<_>>()
                .join(","),
        ),
        ..Default::default()
    };
    assert_eq!(
        proxy_for_url("https://example.com/archive", &too_many_entries),
        Err(NetworkPolicyError::InvalidNoProxy)
    );
}

#[test]
fn no_proxy_supports_exact_ipv4_and_bracketed_ipv6_without_cross_matching() {
    let mut env = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        http_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some("127.0.0.1,[::1]".into()),
        ..Default::default()
    };

    assert_eq!(proxy_for_url("http://127.0.0.1/status", &env), Ok(None));
    assert_eq!(proxy_for_url("http://[::1]/status", &env), Ok(None));
    assert_eq!(
        proxy_for_url("http://127.0.0.2/status", &env),
        Ok(Some("http://proxy.example:8080".into()))
    );

    env.no_proxy = Some("[::1]:8080".into());
    assert_eq!(proxy_for_url("http://[::1]:8080/status", &env), Ok(None));
    assert_eq!(
        proxy_for_url("http://[::1]:8081/status", &env),
        Ok(Some("http://proxy.example:8080".into()))
    );
}

#[test]
fn no_proxy_rejects_request_authority_ambiguity_before_bypass() {
    let env = NetworkEnvironment {
        https_proxy: Some("http://proxy.example:8080".into()),
        no_proxy: Some("api.github.com".into()),
        ..Default::default()
    };

    assert_eq!(
        proxy_for_url("https://user@api.github.com/repos/user/repo", &env),
        Err(NetworkPolicyError::InvalidRequestUrl)
    );
}
