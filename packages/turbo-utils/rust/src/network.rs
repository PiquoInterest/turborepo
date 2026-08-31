use std::net::{Ipv4Addr, Ipv6Addr};

use thiserror::Error;

/// GitHub tokens are ASCII and short in normal use. This cap prevents
/// environment-controlled secrets from becoming unbounded allocation or header
/// inputs.
pub const GITHUB_TOKEN_MAX_CHARS: usize = 4_096;
/// Maximum accepted proxy URL length.
pub const PROXY_URL_MAX_CHARS: usize = 2_048;
/// Maximum accepted `NO_PROXY`/`no_proxy` value length.
pub const NO_PROXY_MAX_CHARS: usize = 4_096;
/// Maximum number of comma-separated `NO_PROXY` rules.
pub const NO_PROXY_MAX_ENTRIES: usize = 256;
/// Maximum number of redirects that can be evaluated for one request chain.
pub const REDIRECT_MAX_HOPS: usize = 10;

/// Snapshot of the environment values consumed by the TypeScript networking
/// helpers. It intentionally has no `Debug` implementation so tokens are not
/// accidentally included in diagnostics.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct NetworkEnvironment {
    pub github_token: Option<String>,
    pub gh_token: Option<String>,
    pub https_proxy: Option<String>,
    pub https_proxy_upper: Option<String>,
    pub http_proxy: Option<String>,
    pub http_proxy_upper: Option<String>,
    pub no_proxy: Option<String>,
    pub no_proxy_upper: Option<String>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum NetworkPolicyError {
    #[error("request URL is not a valid absolute URL")]
    InvalidRequestUrl,
    #[error("selected proxy URL is invalid or unsupported")]
    InvalidProxyUrl,
    #[error("selected NO_PROXY value is invalid or unsupported")]
    InvalidNoProxy,
    #[error("redirect hop is outside the allowed range")]
    RedirectLimitExceeded,
    #[error("redirect from HTTPS to HTTP is not allowed")]
    InsecureRedirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectRequestPolicy {
    pub authorization_header: Option<String>,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedUrl<'a> {
    scheme: &'a str,
    authority: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestHost<'a> {
    Domain(&'a str),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequestEndpoint<'a> {
    host: RequestHost<'a>,
    port: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoProxyHost<'a> {
    Any,
    ExactDomain(&'a str),
    DomainSuffix(&'a str),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NoProxyRule<'a> {
    host: NoProxyHost<'a>,
    port: Option<u16>,
}

fn is_valid_scheme(scheme: &str) -> bool {
    let mut characters = scheme.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

fn parse_absolute_url(value: &str) -> Option<ParsedUrl<'_>> {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }

    let (scheme, remainder) = value.split_once("://")?;
    if !is_valid_scheme(scheme) {
        return None;
    }
    let authority_end = remainder.find(['/', '?', '#']).unwrap_or(remainder.len());
    let authority = remainder.get(..authority_end)?;
    if authority.is_empty() {
        return None;
    }
    Some(ParsedUrl { scheme, authority })
}

fn selected_token(environment: &NetworkEnvironment) -> Option<&str> {
    let selected = match environment.github_token.as_deref() {
        Some("") | None => environment.gh_token.as_deref()?,
        Some(primary) => primary,
    };
    let token = selected.trim();
    if token.is_empty()
        || token.chars().count() > GITHUB_TOKEN_MAX_CHARS
        || !token.bytes().all(|byte| matches!(byte, 0x21..=0x7e))
    {
        return None;
    }
    Some(token)
}

fn is_github_api_endpoint(url: ParsedUrl<'_>) -> bool {
    url.scheme.eq_ignore_ascii_case("https")
        && (url.authority.eq_ignore_ascii_case("api.github.com")
            || url.authority.eq_ignore_ascii_case("codeload.github.com"))
}

/// Returns a bearer header only for credential-free HTTPS requests to the two
/// exact GitHub hosts used by the migration provider.
#[must_use]
pub fn github_authorization_header(url: &str, environment: &NetworkEnvironment) -> Option<String> {
    let parsed = parse_absolute_url(url)?;
    if !is_github_api_endpoint(parsed) {
        return None;
    }
    selected_token(environment).map(|token| format!("Bearer {token}"))
}

fn configured(value: Option<&str>) -> Option<&str> {
    value.filter(|candidate| !candidate.is_empty())
}

fn selected_proxy<'a>(
    request: ParsedUrl<'_>,
    environment: &'a NetworkEnvironment,
) -> Option<&'a str> {
    if request.scheme.eq_ignore_ascii_case("https") {
        configured(environment.https_proxy.as_deref())
            .or_else(|| configured(environment.https_proxy_upper.as_deref()))
            .or_else(|| configured(environment.http_proxy.as_deref()))
            .or_else(|| configured(environment.http_proxy_upper.as_deref()))
    } else {
        configured(environment.http_proxy.as_deref())
            .or_else(|| configured(environment.http_proxy_upper.as_deref()))
    }
}

fn selected_no_proxy(environment: &NetworkEnvironment) -> Option<&str> {
    configured(environment.no_proxy.as_deref())
        .or_else(|| configured(environment.no_proxy_upper.as_deref()))
}

fn valid_proxy_url(value: &str) -> bool {
    if value.chars().count() > PROXY_URL_MAX_CHARS {
        return false;
    }
    let Some(parsed) = parse_absolute_url(value) else {
        return false;
    };
    parsed.scheme.eq_ignore_ascii_case("http") || parsed.scheme.eq_ignore_ascii_case("https")
}

fn parse_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let port = value.parse::<u16>().ok()?;
    (port != 0).then_some(port)
}

fn split_host_port(value: &str) -> Option<(&str, Option<u16>, bool)> {
    if let Some(remainder) = value.strip_prefix('[') {
        let close = remainder.find(']')?;
        let host = remainder.get(..close)?;
        let suffix = remainder.get(close + 1..)?;
        if host.is_empty() || suffix.contains(']') {
            return None;
        }
        let port = if suffix.is_empty() {
            None
        } else {
            Some(parse_port(suffix.strip_prefix(':')?)?)
        };
        return Some((host, port, true));
    }

    if value.contains('[') || value.contains(']') {
        return None;
    }
    match value.bytes().filter(|byte| *byte == b':').count() {
        0 => Some((value, None, false)),
        1 => {
            let (host, port) = value.rsplit_once(':')?;
            if host.is_empty() {
                return None;
            }
            Some((host, Some(parse_port(port)?), false))
        }
        _ => None,
    }
}

fn normalized_domain(value: &str) -> Option<&str> {
    let candidate = value.strip_suffix('.').unwrap_or(value);
    if candidate.is_empty() || candidate.len() > 253 || !candidate.is_ascii() {
        return None;
    }
    for label in candidate.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return None;
        }
    }
    Some(candidate)
}

fn looks_numeric(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
}

fn parse_unbracketed_host(value: &str) -> Option<RequestHost<'_>> {
    let candidate = value.strip_suffix('.').unwrap_or(value);
    if looks_numeric(candidate) {
        return candidate.parse::<Ipv4Addr>().ok().map(RequestHost::Ipv4);
    }
    normalized_domain(candidate).map(RequestHost::Domain)
}

fn default_port(scheme: &str) -> Option<u16> {
    if scheme.eq_ignore_ascii_case("https") {
        Some(443)
    } else if scheme.eq_ignore_ascii_case("http") {
        Some(80)
    } else {
        None
    }
}

fn parse_request_endpoint(request: ParsedUrl<'_>) -> Option<RequestEndpoint<'_>> {
    if request.authority.contains('@') {
        return None;
    }
    let (host, explicit_port, bracketed) = split_host_port(request.authority)?;
    let host = if bracketed {
        RequestHost::Ipv6(host.parse::<Ipv6Addr>().ok()?)
    } else {
        parse_unbracketed_host(host)?
    };
    Some(RequestEndpoint {
        host,
        port: explicit_port.or_else(|| default_port(request.scheme)),
    })
}

fn parse_no_proxy_rule(value: &str) -> Option<NoProxyRule<'_>> {
    if value == "*" {
        return Some(NoProxyRule {
            host: NoProxyHost::Any,
            port: None,
        });
    }

    let (host, port, bracketed) = split_host_port(value)?;
    let host = if bracketed {
        NoProxyHost::Ipv6(host.parse::<Ipv6Addr>().ok()?)
    } else if let Some(suffix) = host.strip_prefix('.') {
        if looks_numeric(suffix) {
            return None;
        }
        NoProxyHost::DomainSuffix(normalized_domain(suffix)?)
    } else {
        match parse_unbracketed_host(host)? {
            RequestHost::Domain(domain) => NoProxyHost::ExactDomain(domain),
            RequestHost::Ipv4(address) => NoProxyHost::Ipv4(address),
            RequestHost::Ipv6(_) => return None,
        }
    };
    Some(NoProxyRule { host, port })
}

fn domain_suffix_matches(host: &str, suffix: &str) -> bool {
    if host.eq_ignore_ascii_case(suffix) {
        return true;
    }
    let Some(separator_index) = host.len().checked_sub(suffix.len() + 1) else {
        return false;
    };
    host.as_bytes().get(separator_index) == Some(&b'.')
        && host
            .get(separator_index + 1..)
            .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

fn no_proxy_rule_matches(request: RequestEndpoint<'_>, rule: NoProxyRule<'_>) -> bool {
    if rule.port.is_some() && rule.port != request.port {
        return false;
    }
    match (rule.host, request.host) {
        (NoProxyHost::Any, _) => true,
        (NoProxyHost::ExactDomain(rule_host), RequestHost::Domain(request_host)) => {
            request_host.eq_ignore_ascii_case(rule_host)
        }
        (NoProxyHost::DomainSuffix(rule_host), RequestHost::Domain(request_host)) => {
            domain_suffix_matches(request_host, rule_host)
        }
        (NoProxyHost::Ipv4(rule_host), RequestHost::Ipv4(request_host)) => {
            request_host == rule_host
        }
        (NoProxyHost::Ipv6(rule_host), RequestHost::Ipv6(request_host)) => {
            request_host == rule_host
        }
        _ => false,
    }
}

fn no_proxy_matches(request: RequestEndpoint<'_>, value: &str) -> Option<bool> {
    if value.len() > NO_PROXY_MAX_CHARS
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return None;
    }

    let mut entry_count = 0usize;
    let mut matched = false;
    for entry in value.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        entry_count += 1;
        if entry_count > NO_PROXY_MAX_ENTRIES {
            return None;
        }
        let rule = parse_no_proxy_rule(entry)?;
        matched |= no_proxy_rule_matches(request, rule);
    }
    (entry_count != 0).then_some(matched)
}

/// Selects proxy configuration using the same lowercase/uppercase precedence
/// as the TypeScript helper. Once a non-empty value wins precedence, an invalid
/// proxy is an error rather than permission to silently bypass the configured
/// proxy with a direct connection. A bounded, valid `no_proxy`/`NO_PROXY` rule
/// may explicitly bypass that selected proxy.
pub fn proxy_for_url(
    url: &str,
    environment: &NetworkEnvironment,
) -> Result<Option<String>, NetworkPolicyError> {
    let request = parse_absolute_url(url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
    let Some(proxy) = selected_proxy(request, environment) else {
        return Ok(None);
    };

    if let Some(no_proxy) = selected_no_proxy(environment) {
        let endpoint =
            parse_request_endpoint(request).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
        let bypass =
            no_proxy_matches(endpoint, no_proxy).ok_or(NetworkPolicyError::InvalidNoProxy)?;
        if bypass {
            return Ok(None);
        }
    }

    if !valid_proxy_url(proxy) {
        return Err(NetworkPolicyError::InvalidProxyUrl);
    }
    Ok(Some(proxy.to_owned()))
}

fn is_http_request_scheme(scheme: &str) -> bool {
    scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
}

fn request_hosts_equal(left: RequestHost<'_>, right: RequestHost<'_>) -> bool {
    match (left, right) {
        (RequestHost::Domain(left), RequestHost::Domain(right)) => left.eq_ignore_ascii_case(right),
        (RequestHost::Ipv4(left), RequestHost::Ipv4(right)) => left == right,
        (RequestHost::Ipv6(left), RequestHost::Ipv6(right)) => left == right,
        _ => false,
    }
}

fn same_request_origin(
    source: ParsedUrl<'_>,
    source_endpoint: RequestEndpoint<'_>,
    target: ParsedUrl<'_>,
    target_endpoint: RequestEndpoint<'_>,
) -> bool {
    source.scheme.eq_ignore_ascii_case(target.scheme)
        && source_endpoint.port == target_endpoint.port
        && request_hosts_equal(source_endpoint.host, target_endpoint.host)
}

/// Returns the authorization and proxy policy for a redirect destination.
/// `redirect_hop` is one-based and must not exceed `REDIRECT_MAX_HOPS`.
pub fn redirect_request_policy(
    source_url: &str,
    target_url: &str,
    redirect_hop: usize,
    environment: &NetworkEnvironment,
) -> Result<RedirectRequestPolicy, NetworkPolicyError> {
    if redirect_hop == 0 || redirect_hop > REDIRECT_MAX_HOPS {
        return Err(NetworkPolicyError::RedirectLimitExceeded);
    }

    let source = parse_absolute_url(source_url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
    let target = parse_absolute_url(target_url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
    if !is_http_request_scheme(source.scheme) || !is_http_request_scheme(target.scheme) {
        return Err(NetworkPolicyError::InvalidRequestUrl);
    }

    let source_endpoint =
        parse_request_endpoint(source).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
    let target_endpoint =
        parse_request_endpoint(target).ok_or(NetworkPolicyError::InvalidRequestUrl)?;

    if source.scheme.eq_ignore_ascii_case("https") && target.scheme.eq_ignore_ascii_case("http") {
        return Err(NetworkPolicyError::InsecureRedirect);
    }

    let source_was_authorized = github_authorization_header(source_url, environment).is_some();
    let authorization_header = if source_was_authorized
        && same_request_origin(source, source_endpoint, target, target_endpoint)
    {
        github_authorization_header(target_url, environment)
    } else {
        None
    };
    let proxy_url = proxy_for_url(target_url, environment)?;

    Ok(RedirectRequestPolicy {
        authorization_header,
        proxy_url,
    })
}
/// Stateful redirect policy for one request chain.
///
/// The chain owns the redirect count and the authorization provenance of the
/// request that actually reached the current URL. Once authorization is
/// stripped by a cross-origin redirect, later hops cannot reconstruct it from
/// the destination hostname alone.
pub struct RedirectChain<'a> {
    environment: &'a NetworkEnvironment,
    current_url: String,
    authorization_header: Option<String>,
    proxy_url: Option<String>,
    redirect_hops: usize,
}

impl<'a> RedirectChain<'a> {
    pub fn new(
        initial_url: &str,
        environment: &'a NetworkEnvironment,
    ) -> Result<Self, NetworkPolicyError> {
        let initial =
            parse_absolute_url(initial_url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
        if !is_http_request_scheme(initial.scheme) || parse_request_endpoint(initial).is_none() {
            return Err(NetworkPolicyError::InvalidRequestUrl);
        }

        Ok(Self {
            environment,
            current_url: initial_url.to_owned(),
            authorization_header: github_authorization_header(initial_url, environment),
            proxy_url: proxy_for_url(initial_url, environment)?,
            redirect_hops: 0,
        })
    }

    #[must_use]
    pub fn current_url(&self) -> &str {
        &self.current_url
    }

    #[must_use]
    pub const fn redirect_hops(&self) -> usize {
        self.redirect_hops
    }

    #[must_use]
    pub fn current_policy(&self) -> RedirectRequestPolicy {
        RedirectRequestPolicy {
            authorization_header: self.authorization_header.clone(),
            proxy_url: self.proxy_url.clone(),
        }
    }

    pub fn follow(
        &mut self,
        target_url: &str,
    ) -> Result<RedirectRequestPolicy, NetworkPolicyError> {
        let next_hop = self
            .redirect_hops
            .checked_add(1)
            .filter(|hop| *hop <= REDIRECT_MAX_HOPS)
            .ok_or(NetworkPolicyError::RedirectLimitExceeded)?;

        let source =
            parse_absolute_url(&self.current_url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
        let target = parse_absolute_url(target_url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
        if !is_http_request_scheme(source.scheme) || !is_http_request_scheme(target.scheme) {
            return Err(NetworkPolicyError::InvalidRequestUrl);
        }

        let source_endpoint =
            parse_request_endpoint(source).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
        let target_endpoint =
            parse_request_endpoint(target).ok_or(NetworkPolicyError::InvalidRequestUrl)?;

        if source.scheme.eq_ignore_ascii_case("https") && target.scheme.eq_ignore_ascii_case("http")
        {
            return Err(NetworkPolicyError::InsecureRedirect);
        }

        let authorization_header = if self.authorization_header.is_some()
            && same_request_origin(source, source_endpoint, target, target_endpoint)
        {
            github_authorization_header(target_url, self.environment)
        } else {
            None
        };
        let proxy_url = proxy_for_url(target_url, self.environment)?;
        let policy = RedirectRequestPolicy {
            authorization_header: authorization_header.clone(),
            proxy_url: proxy_url.clone(),
        };

        self.current_url.clear();
        self.current_url.push_str(target_url);
        self.authorization_header = authorization_header;
        self.proxy_url = proxy_url;
        self.redirect_hops = next_hop;

        Ok(policy)
    }
}
