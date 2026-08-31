#!/usr/bin/env python3
"""Apply the bounded GitHub authority and request-URL security tranche."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
WORKFLOW = ROOT / ".github/workflows/apply-network-authority-bounds.yml"
RED_SHA = "4c3a403017046d9c1d922d6ba6bdd1b7fb621b2c"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected one reviewed anchor in {path}, found {count}: {old[:160]!r}"
        )
    write(path, text.replace(old, new, 1))


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def replace_section(path: str, start: str, end: str, replacement: str) -> None:
    text = read(path)
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"missing section start in {path}: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"missing section end in {path}: {end!r}")
    write(path, text[:start_index] + replacement.rstrip() + "\n\n" + text[end_index:])


def apply_code() -> None:
    replace_once(
        "packages/turbo-utils/rust/src/network.rs",
        "pub const REDIRECT_MAX_HOPS: usize = 10;\n",
        "pub const REDIRECT_MAX_HOPS: usize = 10;\n"
        "/// Maximum accepted request or redirect URL length in UTF-8 bytes.\n"
        "pub const REQUEST_URL_MAX_BYTES: usize = 8 * 1_024;\n",
    )

    replace_once(
        "packages/turbo-utils/rust/src/network.rs",
        """fn parse_absolute_url(value: &str) -> Option<ParsedUrl<'_>> {
    if value.is_empty()
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }
""",
        """fn parse_absolute_url(value: &str) -> Option<ParsedUrl<'_>> {
    if value.is_empty()
        || value.len() > REQUEST_URL_MAX_BYTES
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }
""",
    )

    replace_once(
        "packages/turbo-utils/rust/src/network.rs",
        """fn is_github_api_endpoint(url: ParsedUrl<'_>) -> bool {
    url.scheme.eq_ignore_ascii_case("https")
        && (url.authority.eq_ignore_ascii_case("api.github.com")
            || url.authority.eq_ignore_ascii_case("codeload.github.com"))
}
""",
        """fn is_github_api_endpoint(url: ParsedUrl<'_>) -> bool {
    if !url.scheme.eq_ignore_ascii_case("https") || url.authority.contains('@') {
        return false;
    }

    let Some((host, port, bracketed)) = split_host_port(url.authority) else {
        return false;
    };
    if bracketed || port.is_some_and(|port| port != 443) {
        return false;
    }

    host.eq_ignore_ascii_case("api.github.com")
        || host.eq_ignore_ascii_case("codeload.github.com")
}
""",
    )

    replace_once(
        "packages/turbo-utils/rust/src/entry.rs",
        """pub use network::{
    GITHUB_TOKEN_MAX_CHARS, NO_PROXY_MAX_CHARS, NO_PROXY_MAX_ENTRIES, NetworkEnvironment,
    NetworkPolicyError, PROXY_URL_MAX_CHARS, REDIRECT_MAX_HOPS, RedirectChain,
    RedirectRequestPolicy, github_authorization_header, proxy_for_url, redirect_request_policy,
};
""",
        """pub use network::{
    GITHUB_TOKEN_MAX_CHARS, NO_PROXY_MAX_CHARS, NO_PROXY_MAX_ENTRIES, NetworkEnvironment,
    NetworkPolicyError, PROXY_URL_MAX_CHARS, REDIRECT_MAX_HOPS, REQUEST_URL_MAX_BYTES,
    RedirectChain, RedirectRequestPolicy, github_authorization_header, proxy_for_url,
    redirect_request_policy,
};
""",
    )

    replace_section(
        "packages/turbo-utils/rust/tests/network_security.rs",
        "#[test]\nfn github_authorization_requires_https_without_credentials_or_ports() {",
        "\n#[test]\nfn malformed_and_control_bearing_urls_never_receive_credentials() {",
        """#[test]
fn github_authorization_requires_https_without_userinfo_or_non_default_ports() {
    let env = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

    for url in [
        "http://api.github.com/repos/user/repo",
        "https://api.github.com:444/repos/user/repo",
        "https://user@api.github.com/repos/user/repo",
        "https://user:pass@api.github.com/repos/user/repo",
        "https://api.github.com./repos/user/repo",
    ] {
        assert_eq!(github_authorization_header(url, &env), None, "{url}");
    }
}""",
    )

    replace_once(
        "packages/turbo-utils/rust/tests/redirect_policy_security.rs",
        """#[test]
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
""",
        """#[test]
fn non_default_port_redirect_target_does_not_receive_github_authorization() {
    let environment = NetworkEnvironment {
        github_token: Some("token".into()),
        ..Default::default()
    };

    assert_eq!(
        redirect_request_policy(
            "https://api.github.com/repos/user/repo",
            "https://api.github.com:444/repositories/1",
            1,
            &environment,
        ),
        Ok(RedirectRequestPolicy {
            authorization_header: None,
            proxy_url: None,
        })
    );
}
""",
    )


def apply_docs(green_sha: str) -> None:
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "- GitHub token allow-listing, HTTP/HTTPS proxy precedence, and bounded `NO_PROXY`/`no_proxy` bypass policy.",
        "- GitHub token allow-listing with default-port normalization, an 8 KiB request/redirect URL bound, HTTP/HTTPS proxy precedence, and bounded `NO_PROXY`/`no_proxy` bypass policy.",
    )
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "`src/network.rs` snapshots GitHub token, proxy, and `NO_PROXY`/`no_proxy` environment policy without performing network I/O. It preserves the established lower/uppercase proxy precedence while ensuring credentials can be attached only to exact credential-free HTTPS GitHub API/codeload authorities. Invalid selected proxies and invalid winning bypass values are typed errors rather than silent direct-connection fallbacks or overbroad bypasses.",
        "`src/network.rs` snapshots GitHub token, proxy, redirect, and `NO_PROXY`/`no_proxy` environment policy without performing network I/O. It preserves the established lower/uppercase proxy precedence, treats implicit HTTPS port 443 and explicit `:443` as the same safe GitHub authority, rejects non-default ports and userinfo, and rejects request or redirect URLs above 8,192 UTF-8 bytes before policy evaluation. Invalid selected proxies and invalid winning bypass values are typed errors rather than silent direct-connection fallbacks or overbroad bypasses.",
    )
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "- emits bearer credentials only for HTTPS `api.github.com` and `codeload.github.com` with no userinfo or explicit port;",
        "- emits bearer credentials only for HTTPS `api.github.com` and `codeload.github.com` with no userinfo and with either the implicit or explicit default port 443;\n"
        "- rejects request and redirect URLs above 8,192 UTF-8 bytes before credential, proxy, or redirect decisions;",
    )
    insert_before(
        "packages/turbo-utils/rust/README.md",
        "## NO_PROXY policy TDD record",
        f"""## Network authority and URL-bound TDD record

- Behaviorally failing contract: `{RED_SHA}`.
- GREEN implementation: `{green_sha}`.

The tranche preserves safe explicit `:443` GitHub URLs, rejects non-default ports and userinfo, treats implicit and explicit default ports as one origin during redirects, and fails closed above an 8,192-byte URL ceiling without mutating redirect-chain state. It adds no dependency, network call, parser crate, subprocess, credential source, or `unsafe` block.""",
    )
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "This Rust migration core now has 73 parity tests and 47 security regression tests.",
        "This Rust migration core now has 76 parity tests and 49 security regression tests.",
    )
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "The network-policy surface now contributes 10 parity and 13 security tests, including the bounded `NO_PROXY` tranche.",
        "The network-policy surface now contributes 13 parity and 15 security tests, including bounded `NO_PROXY`, authority normalization, redirect provenance, and request-URL limits.",
    )
    replace_once(
        "packages/turbo-utils/rust/README.md",
        "a production request executor that applies the reviewed proxy/`NO_PROXY` decision exactly once, redirect/DNS/TLS/proxy-auth redaction and response bounds,",
        "a production request executor that applies the reviewed authority/proxy/`NO_PROXY` decision exactly once, redirect/DNS/TLS/proxy-auth redaction and response bounds,",
    )

    replace_once(
        "packages/turbo-utils/rust/PARITY_MATRIX.md",
        "| Plaintext HTTP, explicit ports, userinfo | Intentional deviation | TypeScript checks hostname only and can attach credentials. Rust emits no credentials unless the complete authority is exact, credential-free HTTPS with no port. |",
        "| Plaintext HTTP, non-default ports, userinfo | Intentional deviation | TypeScript checks hostname only and can attach credentials. Rust accepts the safe implicit or explicit HTTPS default port 443, but rejects plaintext, userinfo, trailing-dot aliases, and every non-default port. |",
    )
    replace_once(
        "packages/turbo-utils/rust/PARITY_MATRIX.md",
        "| Look-alike/malformed GitHub URLs | Security parity | No credentials are returned for suffix/prefix look-alikes, malformed URLs, whitespace, or controls. |",
        "| Look-alike/malformed GitHub URLs | Security parity | No credentials are returned for suffix/prefix look-alikes, trailing-dot aliases, malformed URLs, whitespace, or controls. |\n"
        "| Request and redirect URL resource bound | Intentional hardening | Rust accepts URLs through 8,192 UTF-8 bytes and rejects larger values before credential, proxy, or redirect evaluation. Rejected redirect targets cannot mutate chain state. |",
    )

    replace_section(
        "packages/turbo-utils/rust/SECURITY.md",
        "## TU-022: GitHub bearer credentials can cross an insecure transport boundary",
        "## TU-023: Token validation, fallback, and diagnostic boundaries",
        """## TU-022: GitHub bearer credentials can cross an insecure or ambiguous authority boundary

**TypeScript behavior:** `getGitHubAuthHeaders` parses the URL and checks only `hostname` against `api.github.com` and `codeload.github.com`. Because `URL.hostname` omits scheme, credentials, and port, the helper can attach a bearer token to plaintext `http://api.github.com/...`, userinfo-bearing URLs, or a service reached through a non-default port. A safe explicit default HTTPS `:443` authority is equivalent to the implicit port and should retain normal behavior.

**Impact:** a token can be disclosed to an insecure transport or to an unexpected service while an over-strict port check can also break a valid GitHub URL. Redirect comparison must use effective origins rather than raw authority text.

**Rust fix:** emit a bearer value only for syntactically valid, credential-free HTTPS URLs whose host is exactly `api.github.com` or `codeload.github.com` and whose port is omitted or numerically equal to 443. Plaintext, userinfo, trailing-dot aliases, malformed values, look-alikes, and non-default ports receive no credential. Redirect origin comparison uses the effective scheme, host, and port, so implicit 443 and explicit `:443` remain the same origin.

**Intentional incompatibility:** hostname-equivalent URLs on insecure transports, ambiguous authorities, or non-default ports no longer receive tokens. Safe explicit `:443` URLs remain compatible.

**Regressions:** `authorization_is_limited_to_exact_github_api_hosts`, `github_authorization_requires_https_without_userinfo_or_non_default_ports`, `explicit_default_https_port_receives_github_authorization`, `implicit_and_explicit_default_ports_are_the_same_authorized_origin`, and `malformed_and_control_bearing_urls_never_receive_credentials`.""",
    )
    insert_before(
        "packages/turbo-utils/rust/SECURITY.md",
        "## Directory-provider TDD and validation record",
        f"""## TU-031: Unbounded request and redirect URLs can amplify parsing and retained state

**Severity:** Medium

The TypeScript fetch helpers and the first Rust network-policy draft accepted URL strings without a byte ceiling. Very large attacker-controlled URLs can amplify repeated parsing, allocation, proxy-rule evaluation, redirect bookkeeping, logs, and retained redirect-chain state. A rejected redirect must also leave the previous authorization provenance, proxy decision, URL, and hop count untouched.

**Rust fix:** every public request-policy path now rejects URL values above 8,192 UTF-8 bytes before credential, proxy, or redirect evaluation. The exact boundary remains accepted. `RedirectChain::follow` completes all validation before mutating state.

**Intentional incompatibility:** oversized URLs are typed `InvalidRequestUrl` failures instead of being parsed or retained.

**Regressions:** `request_url_boundary_preserves_safe_input`, `oversized_request_urls_fail_closed_across_policy_entrypoints`, and `oversized_redirect_target_does_not_mutate_chain_state`.

TDD evidence: RED `{RED_SHA}` and GREEN `{green_sha}`. The tranche adds no dependency, network call, parser crate, subprocess, or `unsafe` code.""",
    )
    replace_once(
        "packages/turbo-utils/rust/SECURITY.md",
        "Lookup date: **2026-08-31**.",
        "Lookup date: **2026-09-01**.",
    )
    replace_once(
        "packages/turbo-utils/rust/SECURITY.md",
        "The project, notification, archive-policy, network-policy, and bounded `NO_PROXY` tranches add no new Rust dependency. The bypass parser uses only the standard library's IP address types plus existing workspace-managed dependencies.",
        "The project, notification, archive-policy, network-policy, bounded `NO_PROXY`, authority-normalization, and URL-bound tranches add no new Rust dependency. The network parser uses only standard-library string and IP-address types plus existing workspace-managed dependencies.",
    )

    replace_once(
        "docs/typescript-deprecation.md",
        "- `packages/turbo-utils/rust`: 73 translated parity tests and 47 security regression tests.",
        "- `packages/turbo-utils/rust`: 76 translated parity tests and 49 security regression tests.",
    )
    replace_once(
        "docs/typescript-deprecation.md",
        "That is **382 authored Rust migration tests** on the integration branch.",
        "That is **387 authored Rust migration tests** on the integration branch.",
    )
    insert_before(
        "docs/typescript-deprecation.md",
        "## Current `create-turbo` tranches",
        f"""## Current `turbo-utils` request-authority and URL-bound tranche

The network-policy core now treats implicit HTTPS port 443 and explicit `:443` as the same authorized GitHub origin while continuing to reject plaintext, userinfo, trailing-dot aliases, look-alike hosts, and non-default ports. Every public credential, proxy, redirect, and redirect-chain entry point rejects request URLs above 8,192 UTF-8 bytes before evaluation, and a rejected redirect cannot mutate prior chain state.

TDD evidence: RED `{RED_SHA}` and GREEN `{green_sha}`. The change adds three parity tests and two security regressions with no dependency, network, process, parser-crate, or `unsafe` expansion. Production execution remains blocked on DNS/rebinding, TLS, redirect execution, proxy-credential redaction, cancellation, response limits, supported-platform differentials, bindings, packaging, callers, and TypeScript removal.""",
    )

    insert_before(
        "docs/rust-migration-security-findings.md",
        "## Required repository gates",
        f"""### RF-026: Network policy accepted unbounded URLs and over-rejected safe explicit default ports

**Status:** Fixed in the Rust network-policy core; TypeScript production request execution remains and the Rust executor is still blocked.

The TypeScript host-only check can attach GitHub credentials across insecure or non-default authorities because it compares only `hostname`. The first Rust hardening rejected every explicit port, including the safe default HTTPS `:443`, and neither implementation established a URL-size ceiling.

Rust now authorizes only exact credential-free HTTPS GitHub API/codeload hosts with an omitted port or effective port 443, compares redirects by effective origin, and rejects request or redirect URLs above 8,192 UTF-8 bytes before credential, proxy, or state decisions. Rejected redirects leave URL, hop count, proxy selection, and authorization provenance unchanged.

TDD evidence: RED `{RED_SHA}` and GREEN `{green_sha}`. No dependency, socket, subprocess, parser crate, credential source, or `unsafe` block was added. The remaining production executor must still close DNS/rebinding, TLS, proxy credentials, cancellation, response limits, platform differentials, bindings, packaging, callers, and TypeScript removal.""",
    )

    for path in (WORKFLOW, SELF):
        if not path.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {path}")
        path.unlink()


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: apply_network_authority_bounds.py code|docs [green-sha]")
    mode = sys.argv[1]
    if mode == "code":
        apply_code()
        return
    if mode == "docs" and len(sys.argv) == 3:
        apply_docs(sys.argv[2])
        return
    raise SystemExit("usage: apply_network_authority_bounds.py code|docs [green-sha]")


if __name__ == "__main__":
    main()
