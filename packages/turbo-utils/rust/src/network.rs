use thiserror::Error;

/// GitHub tokens are ASCII and short in normal use. This cap prevents
/// environment-controlled secrets from becoming unbounded allocation or header
/// inputs.
pub const GITHUB_TOKEN_MAX_CHARS: usize = 4_096;
/// Maximum accepted proxy URL length.
pub const PROXY_URL_MAX_CHARS: usize = 2_048;

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
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum NetworkPolicyError {
    #[error("request URL is not a valid absolute URL")]
    InvalidRequestUrl,
    #[error("selected proxy URL is invalid or unsupported")]
    InvalidProxyUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedUrl<'a> {
    scheme: &'a str,
    authority: &'a str,
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

fn valid_proxy_url(value: &str) -> bool {
    if value.chars().count() > PROXY_URL_MAX_CHARS {
        return false;
    }
    let Some(parsed) = parse_absolute_url(value) else {
        return false;
    };
    parsed.scheme.eq_ignore_ascii_case("http") || parsed.scheme.eq_ignore_ascii_case("https")
}

/// Selects proxy configuration using the same lowercase/uppercase precedence
/// as the TypeScript helper. Once a non-empty value wins precedence, an invalid
/// proxy is an error rather than permission to silently bypass the configured
/// proxy with a direct connection.
pub fn proxy_for_url(
    url: &str,
    environment: &NetworkEnvironment,
) -> Result<Option<String>, NetworkPolicyError> {
    let request = parse_absolute_url(url).ok_or(NetworkPolicyError::InvalidRequestUrl)?;
    let Some(proxy) = selected_proxy(request, environment) else {
        return Ok(None);
    };
    if !valid_proxy_url(proxy) {
        return Err(NetworkPolicyError::InvalidProxyUrl);
    }
    Ok(Some(proxy.to_owned()))
}
