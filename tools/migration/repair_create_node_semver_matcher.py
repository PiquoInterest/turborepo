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
        '''        ''' + "'''" + '''        // The Rust dependency intentionally accepts optional leading ASCII
        // whitespace after an optional `v` prefix. JavaScript `semver.satisfies`
        // does not normalize that input, so reject hostile or ambiguous text at
        // this boundary before invoking the compatibility parser.
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
        "`NodeSemverMatcher` now uses the exact locked `node-semver` `2.2.0` package and rejects version or range text over 256 UTF-8 bytes before parsing. Malformed versions are non-matches, while malformed repository-owned profile ranges fail as typed configuration errors. Tests cover every profile boundary, malformed and oversized text, build metadata, prerelease exclusion, Unicode confusables, controls, and large numeric components.",
        "`NodeSemverMatcher` now uses the exact locked `node-semver` `2.2.0` package, rejects version or range text over 256 UTF-8 bytes, and rejects non-ASCII, whitespace-bearing, or control-bearing version text before parsing. Malformed versions are non-matches, while malformed repository-owned profile ranges fail as typed configuration errors. Tests cover every profile boundary, malformed and oversized text, build metadata, prerelease exclusion, Unicode confusables, controls, and large numeric components.",
    )

    replace_once(
        "- bounds both version and range text to 256 UTF-8 bytes before parsing;\n- treats malformed versions as unsupported non-matches and malformed static ranges as typed configuration errors;",
        "- bounds both version and range text to 256 UTF-8 bytes before parsing;\n- rejects non-ASCII, whitespace-bearing, and control-bearing version text before invoking the compatibility parser;\n- treats malformed versions as unsupported non-matches and malformed static ranges as typed configuration errors;",
    )

    replace_once(
        "Rust now uses the locked `node-semver` 2.2.0 implementation. Version and range inputs are rejected above 256 UTF-8 bytes before parsing; malformed versions are unsupported non-matches; malformed repository-owned profile ranges are typed configuration errors. Regression coverage includes every source profile boundary, build metadata, prerelease exclusion, Unicode confusables, terminal controls, oversized text, and unsafe numeric components.",
        "Rust now uses the locked `node-semver` 2.2.0 implementation behind a stricter trust boundary. Version and range inputs are rejected above 256 UTF-8 bytes, and non-ASCII, whitespace-bearing, or control-bearing version text is rejected before parsing; malformed versions are unsupported non-matches; malformed repository-owned profile ranges are typed configuration errors. Regression coverage includes every source profile boundary, build metadata, prerelease exclusion, Unicode confusables, terminal controls, oversized text, and unsafe numeric components.",
    )

    replace_once(
        "- Package-manager version and range matching is limited to 256 UTF-8 bytes, performs no trimming or Unicode normalization, and cannot influence program or argument construction.",
        "- Package-manager version and range matching is limited to 256 UTF-8 bytes; version text containing non-ASCII bytes, whitespace, or controls is rejected rather than trimmed or normalized, and cannot influence program or argument construction.",
    )

    print("patched strict Node-semver input policy and generated documentation text")


if __name__ == "__main__":
    main()
