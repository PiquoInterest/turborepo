#!/usr/bin/env python3
"""Patch the one-shot Node-semver generator after a strict-input RED failure."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / "tools/migration/apply_create_node_semver_matcher.py"


def replace_once(old: str, new: str) -> None:
    text = GENERATOR.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            "expected exactly one reviewed Node-semver generator anchor, "
            f"found {count}: {old[:160]!r}"
        )
    GENERATOR.write_text(text.replace(old, new, 1), encoding="utf-8")


def main() -> None:
    replace_once(
        '''        ''' + "'''" + '''        let range = requirement
            .parse::<node_semver::Range>()
            .map_err(|_error| NodeSemverMatcherError::InvalidRange)?;
        let Ok(version) = version.parse::<node_semver::Version>() else {
            return Ok(false);
        };

        Ok(version.satisfies(&range))
''' + "'''" + ''',
''',
        '''        ''' + "'''" + '''        // npm semver 7.5.2 accepts leading and trailing ASCII whitespace.
        // The Rust migration intentionally rejects non-ASCII, whitespace, and
        // control bytes before parsing so untrusted input cannot be normalized
        // into a trusted package-manager profile.
        if !version.is_ascii()
            || version
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        {
            return Ok(false);
        }

        let range = requirement
            .parse::<node_semver::Range>()
            .map_err(|_error| NodeSemverMatcherError::InvalidRange)?;
        let Ok(version) = version.parse::<node_semver::Version>() else {
            return Ok(false);
        };

        Ok(version.satisfies(&range))
''' + "'''" + ''',
''',
    )

    replace_once(
        "- ordinary malformed versions are unsupported non-matches, matching JavaScript `semver.satisfies`;\n- malformed static profile ranges are typed configuration errors;",
        "- ordinary malformed versions are unsupported non-matches, matching JavaScript `semver.satisfies`;\n- npm `semver` 7.5.2 accepts leading and trailing ASCII whitespace, while Rust intentionally rejects non-ASCII, whitespace-bearing, and control-bearing version text before parsing;\n- malformed static profile ranges are typed configuration errors;",
    )

    replace_once(
        "`NodeSemverMatcher` now uses the exact locked `node-semver` `2.2.0` package and rejects version or range text over 256 UTF-8 bytes before parsing. Malformed versions are non-matches, while malformed repository-owned profile ranges fail as typed configuration errors. Tests cover every profile boundary, malformed and oversized text, build metadata, prerelease exclusion, Unicode confusables, controls, and large numeric components.",
        "`NodeSemverMatcher` now uses the exact locked `node-semver` `2.2.0` package and rejects version or range text over 256 UTF-8 bytes. Unlike the TypeScript npm `semver` 7.5.2 path, which accepts leading and trailing ASCII whitespace, Rust rejects non-ASCII, whitespace-bearing, and control-bearing version text before parsing. This is intentional hardening, recorded by passing TypeScript parity cases plus `it.failing` security evidence. Malformed versions are non-matches, while malformed repository-owned profile ranges fail as typed configuration errors. Tests cover every profile boundary, malformed and oversized text, build metadata, prerelease exclusion, Unicode confusables, controls, and large numeric components.",
    )

    replace_once(
        "- bounds both version and range text to 256 UTF-8 bytes before parsing;\n- treats malformed versions as unsupported non-matches and malformed static ranges as typed configuration errors;",
        "- bounds both version and range text to 256 UTF-8 bytes before parsing;\n- intentionally rejects non-ASCII, whitespace-bearing, and control-bearing version text instead of preserving npm `semver` edge-whitespace normalization;\n- treats malformed versions as unsupported non-matches and malformed static ranges as typed configuration errors;",
    )

    replace_once(
        "| unbounded or permissively normalized version/range text | 256-byte pre-parse limits and strict parsing | intentional-hardening | Oversized input is typed failure; malformed versions are non-matches and malformed static ranges are configuration errors. |",
        "| unbounded or permissively normalized version/range text | 256-byte pre-parse limits and strict parsing | intentional-hardening | TypeScript npm `semver` accepts leading and trailing ASCII whitespace; Rust rejects non-ASCII, whitespace-bearing, control-bearing, or oversized text before parsing. Malformed versions are non-matches and malformed static ranges are configuration errors. |",
    )

    replace_once(
        "Rust now uses the locked `node-semver` 2.2.0 implementation. Version and range inputs are rejected above 256 UTF-8 bytes before parsing; malformed versions are unsupported non-matches; malformed repository-owned profile ranges are typed configuration errors. Regression coverage includes every source profile boundary, build metadata, prerelease exclusion, Unicode confusables, terminal controls, oversized text, and unsafe numeric components.",
        "Rust now uses the locked `node-semver` 2.2.0 implementation behind a stricter trust boundary. TypeScript npm `semver` 7.5.2 accepts leading and trailing ASCII whitespace; Rust intentionally rejects non-ASCII, whitespace-bearing, or control-bearing version text before parsing. Version and range inputs are also rejected above 256 UTF-8 bytes; malformed versions are unsupported non-matches; malformed repository-owned profile ranges are typed configuration errors. The TypeScript oracle records both the actual normalization behavior and `it.failing` expectations for the stricter policy. Regression coverage includes every source profile boundary, build metadata, prerelease exclusion, Unicode confusables, terminal controls, oversized text, and unsafe numeric components.",
    )

    replace_once(
        "- Package-manager version and range matching is limited to 256 UTF-8 bytes, performs no trimming or Unicode normalization, and cannot influence program or argument construction.",
        "- Package-manager version and range matching is limited to 256 UTF-8 bytes; unlike TypeScript npm `semver` normalization, version text containing non-ASCII bytes, whitespace, or controls is rejected rather than trimmed or normalized, and cannot influence program or argument construction.",
    )

    print("patched strict Node-semver policy and explicit TypeScript divergence evidence")


if __name__ == "__main__":
    main()
