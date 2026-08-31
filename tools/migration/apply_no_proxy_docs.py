#!/usr/bin/env python3
"""Record the reviewed bounded NO_PROXY migration tranche."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
WORKFLOW = ROOT / ".github/workflows/apply-no-proxy-docs.yml"

RED_SHA = "1ffcd4010de0e5505c21b64caa51af66ef44b8b6"
GREEN_SHA = "e8bdab4094be133fcbba7fd5ffda12a288deee19"
FORMAT_SHA = "bf9e7c5b5653fed8fbbfb49e384f92d2fbc477c8"
FIXTURE_SHA = "94c14b4b530db457923ede6dfee906ef45cb07d9"


def target(path: str) -> Path:
    return ROOT / path


def replace_once(path: str, old: str, new: str) -> None:
    file = target(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected exactly one reviewed anchor in {path}, found {count}: {old[:160]!r}"
        )
    file.write_text(text.replace(old, new, 1), encoding="utf-8")


def replace_section(path: str, start: str, end: str, replacement: str) -> None:
    file = target(path)
    text = file.read_text(encoding="utf-8")
    start_count = text.count(start)
    end_count = text.count(end)
    if start_count != 1 or end_count != 1:
        raise SystemExit(
            f"reviewed section anchors changed in {path}: "
            f"start={start_count}, end={end_count}"
        )
    start_index = text.index(start)
    end_index = text.index(end, start_index + len(start))
    file.write_text(
        text[:start_index] + replacement.rstrip() + "\n\n" + text[end_index:],
        encoding="utf-8",
    )


def insert_before(path: str, anchor: str, addition: str) -> None:
    replace_once(path, anchor, f"{addition.rstrip()}\n\n{anchor}")


def update_readme() -> None:
    path = "packages/turbo-utils/rust/README.md"
    replace_once(
        path,
        "- GitHub token allow-listing and HTTP/HTTPS proxy precedence policy.",
        "- GitHub token allow-listing, HTTP/HTTPS proxy precedence, and bounded `NO_PROXY`/`no_proxy` bypass policy.",
    )
    replace_once(
        path,
        "`src/network.rs` snapshots GitHub token and proxy environment policy without performing network I/O. It preserves TypeScript precedence while ensuring credentials can be attached only to exact credential-free HTTPS GitHub API/codeload authorities. Invalid selected proxies are errors rather than silent direct-connection fallbacks.",
        "`src/network.rs` snapshots GitHub token, proxy, and `NO_PROXY`/`no_proxy` environment policy without performing network I/O. It preserves the established lower/uppercase proxy precedence while ensuring credentials can be attached only to exact credential-free HTTPS GitHub API/codeload authorities. Invalid selected proxies and invalid winning bypass values are typed errors rather than silent direct-connection fallbacks or overbroad bypasses.",
    )

    network_and_tdd = f"""The network policy:

- preserves `GITHUB_TOKEN` over `GH_TOKEN` precedence;
- rejects empty, control-bearing, non-ASCII, whitespace-containing, or oversized selected tokens;
- emits bearer credentials only for HTTPS `api.github.com` and `codeload.github.com` with no userinfo or explicit port;
- rejects look-alike hosts and malformed URLs;
- preserves lower/uppercase HTTPS/HTTP proxy precedence;
- accepts only bounded HTTP(S) proxy URLs;
- returns an error for an invalid winning proxy value instead of connecting directly;
- preserves lowercase `no_proxy` precedence over uppercase `NO_PROXY`;
- accepts a deliberately narrow, bounded rule language: `*`, exact domains, explicit leading-dot domain suffixes, exact IPv4, bracketed IPv6, and optional ports matched against the effective request port;
- requires DNS-label boundaries and exact address-family/port matches, so suffix text cannot match `notexample.com` or a neighboring IP;
- rejects partial wildcards, CIDR notation, Unicode/confusable host rules, userinfo ambiguity, controls, oversized values, and more than 256 non-empty entries;
- treats an invalid winning bypass value as a typed error rather than falling back to uppercase policy or silently choosing direct/proxied transport.

## NO_PROXY policy TDD record

- Behaviorally failing contract: `{RED_SHA}`.
- GREEN bounded policy implementation: `{GREEN_SHA}`.
- Committed formatting proof: `{FORMAT_SHA}`.
- Protocol-specific oracle fixture correction: `{FIXTURE_SHA}`.

The final fixture correction did not weaken any bypass assertion. The HTTP address tests now configure `http_proxy` as well as `https_proxy`, matching the pre-existing TypeScript rule that HTTP requests do not consume HTTPS-only proxy configuration. The integration workflow is authoritative for formatting, compilation, parity/security tests, Clippy, and the unchanged lockfile-wide advisory gate."""
    replace_section(
        path,
        "The network policy:\n",
        "## Directory-provider TDD record",
        network_and_tdd,
    )

    replace_once(
        path,
        "This Rust migration core now has 70 parity tests and 41 security regression tests. The directory-provider tranche contributes five new security tests. The network-policy tranche contributes 7 parity and 7 security tests.",
        "This Rust migration core now has 73 parity tests and 47 security regression tests. The directory-provider tranche contributes five new security tests. The network-policy surface now contributes 10 parity and 13 security tests, including the bounded `NO_PROXY` tranche.",
    )
    replace_once(
        path,
        "Blocked. Remaining work includes stable handle-relative directory validation and mutation, request execution and response bounds, GitHub repository/default-branch resolution, the production archive provider and safe writes behind `ProjectSource`, a bounded registry update checker behind `UpdateChecker`, explicit `NO_PROXY` semantics, Windows-native process-tree/ACL/reparse-point parity, native/WASM or JavaScript bindings, npm packaging, downstream migration, supported-platform differential tests, and proof that executable TypeScript is no longer loaded or shipped.",
        "Blocked. Remaining work includes stable handle-relative directory validation and mutation, a production request executor that applies the reviewed proxy/`NO_PROXY` decision exactly once, redirect/DNS/TLS/proxy-auth redaction and response bounds, GitHub repository/default-branch resolution, the production archive provider and safe writes behind `ProjectSource`, a bounded registry update checker behind `UpdateChecker`, Windows-native process-tree/ACL/reparse-point parity, native/WASM or JavaScript bindings, npm packaging, downstream migration, supported-platform differential tests, and proof that executable TypeScript is no longer loaded or shipped.",
    )


def update_parity_matrix() -> None:
    path = "packages/turbo-utils/rust/PARITY_MATRIX.md"
    replace_once(
        path,
        "| Proxy URL policy | Intentional deviation | Rust accepts only bounded absolute HTTP(S) proxy URLs. Production `NO_PROXY` semantics remain open. |",
        "| Proxy URL policy | Intentional deviation | Rust accepts only bounded absolute HTTP(S) proxy URLs. |\n"
        "| `no_proxy` / `NO_PROXY` precedence | Intentional hardening | A non-empty lowercase value wins over uppercase. An invalid winning value is an error and cannot silently fall back. |\n"
        "| `NO_PROXY` rule language | Intentional hardening | Bounded comma-separated `*`, exact domains, explicit leading-dot suffixes, exact IPv4, bracketed IPv6, and optional effective-port rules are supported. |\n"
        "| `NO_PROXY` matching and malformed input | Intentional hardening | Domain matches require label boundaries; IP family and ports match exactly. Partial wildcards, CIDR, Unicode/confusables, controls, userinfo ambiguity, oversized values, and more than 256 rules are rejected. |",
    )
    replace_once(
        path,
        "| Network/archive acquisition and writes | Blocked | Request execution, redirect/TLS/proxy agents, GitHub lookup, Git fallback, tar streaming/writes, cleanup, and atomic promotion remain to be ported. |",
        "| Network/archive acquisition and writes | Blocked | A production executor must apply the reviewed proxy/`NO_PROXY` decision exactly once and close redirect, DNS, TLS, proxy-credential redaction, timeout, response-size, GitHub lookup, Git fallback, tar streaming/writes, cleanup, and atomic-promotion behavior. |",
    )


def update_security() -> None:
    path = "packages/turbo-utils/rust/SECURITY.md"
    tu24 = """## TU-024: Invalid proxy configuration can create a policy bypass

**TypeScript behavior:** proxy precedence is defined, but validation is deferred to `ProxyAgent` construction. A future caller that catches that error and retries without the dispatcher could silently bypass an administrator-selected proxy. The helper also has no explicit URL-length or allowed-scheme policy and does not model `NO_PROXY`.

**Rust fix:** preserve the existing lower/uppercase proxy precedence, but return a typed error when the winning non-empty value is malformed or is not a bounded absolute HTTP(S) URL. Lower-precedence proxies are not consulted after a value wins, and direct connection is not treated as a fallback. The Rust core also snapshots `no_proxy` before `NO_PROXY` and evaluates a bounded, explicit bypass rule set before returning the selected proxy.

**Residual:** production request execution must consume this decision exactly once and define proxy authentication redaction, DNS/IP pinning or rebinding behavior, certificate trust, redirects, timeouts, response bounds, and whether every GitHub endpoint is required to use the selected proxy.

**Regressions:** `https_proxy_precedence_matches_the_typescript_helper`, `invalid_selected_proxy_is_an_error_instead_of_direct_connection_fallback`, `proxy_urls_are_bounded_and_restricted_to_http_or_https`, `malformed_request_url_is_an_error_before_proxy_selection`, and the `NO_PROXY` parity/security suites."""
    replace_section(path, "## TU-024:", "## TU-025:", tu24)

    tu30 = f"""## TU-030: Missing or permissive `NO_PROXY` policy can misroute sensitive traffic

**Severity:** High at the production request boundary

**TypeScript behavior:** the current helper selects `HTTP_PROXY`/`HTTPS_PROXY` values but does not model `NO_PROXY` or `no_proxy`. Internal, loopback, or explicitly exempt destinations can therefore be sent through a configured proxy. A loose port could create the opposite problem by using substring, Unicode, partial-wildcard, or CIDR matching to bypass a proxy more broadly than intended.

**Rust fix:** lowercase `no_proxy` wins over uppercase `NO_PROXY`, matching the established lowercase-first environment convention. Values are limited to 4,096 ASCII bytes and 256 non-empty comma-separated rules. The accepted language is deliberately narrow: global `*`, exact domains, explicit leading-dot domain suffixes, exact IPv4 addresses, bracketed IPv6 addresses, and optional ports compared with the explicit or scheme-default request port. Domain suffixes require a dot-label boundary. Invalid winning values fail closed with `InvalidNoProxy`; they do not fall back to uppercase policy or silently select direct/proxied transport.

**Intentional incompatibility:** partial wildcards such as `*.example.com`, CIDR blocks, Unicode/confusable names, unbracketed IPv6, controls, empty-only lists, invalid ports, userinfo-bearing request authorities, oversized values, and excessive rule counts are rejected instead of being interpreted permissively.

**Residual:** this is a pure decision core. The production HTTP provider must prove that redirects cannot bypass the decision, hostnames and resolved addresses follow an explicit DNS/rebinding policy, proxy credentials never enter diagnostics, and the selected transport is applied once with bounded time and response size.

**Regressions:** `no_proxy_exact_suffix_and_star_rules_bypass_configured_proxy`, `lowercase_no_proxy_takes_precedence_over_uppercase_no_proxy`, `no_proxy_port_rules_use_the_effective_request_port`, `no_proxy_domain_matching_uses_label_boundaries_not_substrings`, `invalid_winning_no_proxy_value_is_an_error_without_uppercase_fallback`, `no_proxy_rejects_ambiguous_wildcards_unicode_and_cidr_rules`, `no_proxy_values_are_bounded_by_length_and_entry_count`, `no_proxy_supports_exact_ipv4_and_bracketed_ipv6_without_cross_matching`, and `no_proxy_rejects_request_authority_ambiguity_before_bypass`.

TDD evidence: RED `{RED_SHA}`, GREEN `{GREEN_SHA}`, formatting `{FORMAT_SHA}`, and corrected protocol fixture `{FIXTURE_SHA}`."""
    insert_before(path, "## Directory-provider TDD and validation record", tu30)

    replace_once(
        path,
        "The project, notification, archive-policy, and network-policy tranches add no new Rust dependency. They use the standard library plus existing workspace-managed dependencies.",
        "The project, notification, archive-policy, network-policy, and bounded `NO_PROXY` tranches add no new Rust dependency. The bypass parser uses only the standard library's IP address types plus existing workspace-managed dependencies.",
    )


def update_program_ledger() -> None:
    path = "docs/typescript-deprecation.md"
    replace_once(
        path,
        "- `packages/turbo-utils/rust`: 70 translated parity tests and 41 security regression tests.",
        "- `packages/turbo-utils/rust`: 73 translated parity tests and 47 security regression tests.",
    )
    replace_once(
        path,
        "That is **373 authored Rust migration tests** on the integration branch.",
        "That is **382 authored Rust migration tests** on the integration branch.",
    )
    replace_once(
        path,
        "The official-starter tranche advances create-turbo core and test evidence without completing a new production stage, so the recalculated rounded repository score remains about **8%**.",
        "The bounded `NO_PROXY` tranche advances turbo-utils network-policy core and test evidence without completing the production request-execution, binding, packaging, caller, platform, or removal stages, so the recalculated rounded repository score remains about **8%**.",
    )
    replace_once(
        path,
        "| `packages/turbo-utils` | `packages/turbo-utils/rust` plus bindings | In progress | Stable handle-relative directory validation/mutation, production network/archive and registry providers, remaining utilities, Windows ACL/process/shim closure, bindings, callers, removal proof. |",
        "| `packages/turbo-utils` | `packages/turbo-utils/rust` plus bindings | In progress | Stable handle-relative directory validation/mutation; production network execution that applies the reviewed proxy/`NO_PROXY` decision exactly once and closes redirects, DNS, TLS, credentials, timeouts, and bounds; archive/registry providers; remaining utilities; Windows ACL/process/shim closure; bindings, callers, and removal proof. |",
    )

    network_section = f"""## Current `turbo-utils` network-policy tranche

The Rust network decision core now covers both proxy selection and a deliberately bounded `NO_PROXY`/`no_proxy` bypass contract without opening sockets or adding dependency authority.

Preserved behavior:

- lowercase/uppercase HTTP and HTTPS proxy precedence remains unchanged;
- HTTP requests still consult only HTTP proxy variables;
- invalid winning proxy values remain typed errors rather than direct-connection fallback.

Security closure added in this tranche:

- lowercase `no_proxy` wins over uppercase `NO_PROXY`;
- `*`, exact domains, explicit leading-dot suffixes, exact IPv4, bracketed IPv6, and optional effective ports are the only accepted rules;
- domain matching requires label boundaries and IP/port matching is exact;
- rule text is ASCII-only and bounded to 4,096 bytes and 256 non-empty entries;
- partial wildcards, CIDR, Unicode/confusables, malformed ports, unbracketed IPv6, controls, userinfo-bearing request authorities, and empty-only lists fail closed;
- an invalid winning bypass value cannot fall back to uppercase policy or silently choose direct/proxied transport.

TDD history: RED `{RED_SHA}`, GREEN `{GREEN_SHA}`, formatting `{FORMAT_SHA}`, and the protocol-specific test-fixture correction `{FIXTURE_SHA}`. The correction added the HTTP proxy expected by existing protocol precedence; it did not weaken bypass assertions.

Production closure remains open. A request executor must apply this pure decision exactly once across redirects and define DNS/rebinding, TLS, proxy credentials and redaction, cancellation, timeout, response bounds, platform behavior, binding, packaging, callers, and TypeScript removal."""
    insert_before(path, "## Current `create-turbo` tranches", network_section)


def update_repository_findings() -> None:
    path = "docs/rust-migration-security-findings.md"
    finding = f"""### RF-025: Missing or permissive `NO_PROXY` handling can route sensitive destinations incorrectly

**Status:** Fixed in the Rust network-policy core; TypeScript production request execution remains and the Rust provider is still blocked.

The existing TypeScript helper selects HTTP/HTTPS proxy variables but does not model `NO_PROXY` or `no_proxy`. That can send loopback, internal, or explicitly exempt destinations through a proxy. A permissive port could also bypass a proxy too broadly through substring, partial-wildcard, Unicode/confusable, CIDR, or ambiguous authority matching.

The Rust core now evaluates a bounded lowercase-first bypass policy before returning a selected proxy:

- 4,096-byte and 256-entry limits;
- global `*`;
- exact domains and explicit leading-dot suffixes with DNS-label boundaries;
- exact IPv4 and bracketed IPv6;
- optional ports matched against explicit or HTTP/HTTPS default ports;
- fail-closed rejection of partial wildcards, CIDR, Unicode, controls, invalid ports, unbracketed IPv6, userinfo-bearing authorities, empty-only lists, and excessive input;
- no fallback from an invalid lowercase value to uppercase policy and no silent direct/proxied transport decision.

This tranche adds no dependency, network call, credential access, parser crate, subprocess, or unsafe code. TDD evidence is RED `{RED_SHA}`, GREEN `{GREEN_SHA}`, formatting `{FORMAT_SHA}`, and corrected protocol fixture `{FIXTURE_SHA}`.

Required production closure is a request executor that consumes the decision exactly once across redirects, applies an explicit DNS/rebinding and TLS policy, bounds time and response size, redacts proxy credentials, passes Linux/macOS/Windows differentials, and removes the TypeScript request path only after binding and packaging proof."""
    insert_before(path, "## Required repository gates", finding)


def remove_one_shot_automation() -> None:
    for path in (WORKFLOW, SELF):
        if not path.exists():
            raise SystemExit(f"expected one-shot automation file is missing: {path}")
        path.unlink()


def main() -> None:
    update_readme()
    update_parity_matrix()
    update_security()
    update_program_ledger()
    update_repository_findings()
    remove_one_shot_automation()


if __name__ == "__main__":
    main()
