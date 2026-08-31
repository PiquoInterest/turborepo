use std::error::Error;

use turbo_utils_rs::{
    NetworkEnvironment, NetworkPolicyError, REDIRECT_MAX_HOPS, RedirectChain,
    RedirectRequestPolicy,
};

#[test]
fn authorization_cannot_reappear_after_a_cross_origin_redirect() -> Result<(), Box<dyn Error>> {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };
    let mut chain = RedirectChain::new(
        "https://api.github.com/repos/user/repo",
        &environment,
    )?;

    assert_eq!(
        chain.current_policy()?,
        RedirectRequestPolicy {
            authorization_header: Some("Bearer token".into()),
            proxy_url: None,
        }
    );
    assert_eq!(
        chain.follow("https://codeload.github.com/user/repo/tar.gz/main")?,
        RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        }
    );
    assert_eq!(
        chain.follow("https://codeload.github.com/user/repo/tar.gz/next")?,
        RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        }
    );
    Ok(())
}

#[test]
fn redirect_chain_owns_the_hop_counter() -> Result<(), Box<dyn Error>> {
    let environment = NetworkEnvironment::default();
    let mut chain = RedirectChain::new("https://downloads.example.com/start", &environment)?;

    for hop in 1..=REDIRECT_MAX_HOPS {
        let target = format!("https://downloads.example.com/archive-{hop}");
        assert_eq!(
            chain.follow(&target)?,
            RedirectRequestPolicy {
                authorization_header: None,
                proxy_url: None,
            }
        );
        assert_eq!(chain.redirect_hops(), hop);
        assert_eq!(chain.current_url(), target);
    }

    let current_url = chain.current_url().to_owned();
    assert_eq!(
        chain.follow("https://downloads.example.com/too-many"),
        Err(NetworkPolicyError::RedirectLimitExceeded)
    );
    assert_eq!(chain.redirect_hops(), REDIRECT_MAX_HOPS);
    assert_eq!(chain.current_url(), current_url);
    Ok(())
}

#[test]
fn rejected_redirect_does_not_mutate_chain_state() -> Result<(), Box<dyn Error>> {
    let environment = NetworkEnvironment::default();
    let mut chain = RedirectChain::new("https://downloads.example.com/start", &environment)?;

    assert_eq!(
        chain.follow("http://downloads.example.com/insecure"),
        Err(NetworkPolicyError::InsecureRedirect)
    );
    assert_eq!(chain.redirect_hops(), 0);
    assert_eq!(chain.current_url(), "https://downloads.example.com/start");

    assert_eq!(
        chain.follow("https://downloads.example.com/archive")?,
        RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        }
    );
    assert_eq!(chain.redirect_hops(), 1);
    Ok(())
}

#[test]
fn an_untrusted_chain_cannot_gain_github_authorization() -> Result<(), Box<dyn Error>> {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };
    let mut chain = RedirectChain::new("https://attacker.example/start", &environment)?;

    assert_eq!(
        chain.follow("https://api.github.com/repos/user/repo")?,
        RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        }
    );
    Ok(())
}
