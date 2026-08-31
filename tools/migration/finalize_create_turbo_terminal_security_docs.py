#!/usr/bin/env python3
"""Patch the staged evidence writer for the final sink-level design, then run it."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SELF = Path(__file__).resolve()
EVIDENCE_WRITER = ROOT / "tools/migration/apply_create_turbo_terminal_security_docs.py"
SINK_SHA = "e6d9d3ce596d897c7b4239b6814ad18353fa8292"


def replace_once(text: str, old: str, new: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"expected one staged evidence anchor, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


def main() -> None:
    text = EVIDENCE_WRITER.read_text(encoding="utf-8")
    text = replace_once(
        text,
        'FORMATTED_SHA = "dc7de326654e679bfc9b595e3b1089b9cb4aa0cb"',
        'FORMATTED_SHA = "dc7de326654e679bfc9b595e3b1089b9cb4aa0cb"\n'
        f'SINK_SHA = "{SINK_SHA}"',
    )
    text = replace_once(
        text,
        'f"Terminal rustfmt:        {FORMATTED_SHA}\\n"\n'
        '        f"Terminal validation:     {VALIDATED_SHA} (Actions run {VALIDATION_RUN})",',
        'f"Terminal rustfmt:        {FORMATTED_SHA}\\n"\n'
        '        f"Terminal sink hardening: {SINK_SHA}\\n"\n'
        '        f"Terminal validation:     {VALIDATED_SHA} (Actions run {VALIDATION_RUN})",',
    )
    text = replace_once(
        text,
        "The TypeScript oracle now separates raw and terminal-facing data. `rawMessage` and `rawTransform` retain the original structured values, while `message` and `transform` are rendered by a bounded scalar scanner. The Rust `TransformFailure` retains its raw fields and applies the same renderer only through `Display` and `terminal_transform`.",
        "The TypeScript oracle preserves the standard raw `Error.message` and `transform` fields and exposes dynamically computed `terminalMessage` and `terminalTransform` getters. The create-command logger consumes only those getters, so post-construction mutation is re-sanitized at the actual terminal sink. The Rust `TransformFailure` retains its raw fields and applies the same renderer only through `Display` and `terminal_transform`.",
    )
    text = replace_once(
        text,
        "The original vulnerability is retained as genuine RED evidence in `{RED_SHA}`. The implementation is `{GREEN_SHA}`, with formatting closure in `{FORMATTED_SHA}` and validation at `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",
        "The original vulnerability is retained as genuine RED evidence in `{RED_SHA}`. The initial implementation is `{GREEN_SHA}`, formatting closure is `{FORMATTED_SHA}`, sink-level mutable-field hardening is `{SINK_SHA}`, and validation is `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",
    )
    text = replace_once(
        text,
        "| Transform-error display fields | `TransformError.message` and `.transform` previously reached terminal coloring raw; they now use `sanitizeTerminalText`, while `rawMessage` and `rawTransform` preserve the source data | `TransformFailure` keeps raw fields and applies `sanitize_terminal_text` through `Display` and `terminal_transform` | Intentional security fix. Safe printable text is preserved; hostile controls and oversized fields deliberately render differently. |",
        "| Transform-error display fields | Raw `TransformError.message` and `.transform` remain API-compatible; dynamic `terminalMessage` and `terminalTransform` getters apply `sanitizeTerminalText`, and the create-command logger consumes only those getters | `TransformFailure` keeps raw fields and applies `sanitize_terminal_text` through `Display` and `terminal_transform` | Intentional security fix. Safe printable text is preserved; hostile controls and oversized fields deliberately render differently, and post-construction mutation is re-sanitized at the sink. |",
    )
    text = replace_once(
        text,
        'f"- Formatting closure: `{FORMATTED_SHA}`.\\n"\n'
        '        f"- Validated tranche head: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.\\n"',
        'f"- Formatting closure: `{FORMATTED_SHA}`.\\n"\n'
        '        f"- Sink-level mutable-field hardening: `{SINK_SHA}`.\\n"\n'
        '        f"- Validated tranche head: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.\\n"',
    )
    text = replace_once(
        text,
        'f"- transform terminal-diagnostic formatting closure: `{FORMATTED_SHA}`.\\n"\n'
        '        f"- transform terminal-diagnostic validation: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",',
        'f"- transform terminal-diagnostic formatting closure: `{FORMATTED_SHA}`.\\n"\n'
        '        f"- transform terminal-diagnostic sink hardening: `{SINK_SHA}`.\\n"\n'
        '        f"- transform terminal-diagnostic validation: `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",',
    )
    text = replace_once(
        text,
        "RED evidence is `{RED_SHA}`. The implementation is `{GREEN_SHA}`, formatting closure is `{FORMATTED_SHA}`, and the validated tranche head is `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",
        "RED evidence is `{RED_SHA}`. The initial implementation is `{GREEN_SHA}`, formatting closure is `{FORMATTED_SHA}`, sink-level mutable-field hardening is `{SINK_SHA}`, and the validated tranche head is `{VALIDATED_SHA}` in Actions run `{VALIDATION_RUN}`.",
    )
    if "rawMessage" in text or "rawTransform" in text:
        raise SystemExit("stale constructor-only raw-field design remains in evidence writer")

    EVIDENCE_WRITER.write_text(text, encoding="utf-8")
    subprocess.run([sys.executable, str(EVIDENCE_WRITER)], cwd=ROOT, check=True)
    SELF.unlink()


if __name__ == "__main__":
    main()
