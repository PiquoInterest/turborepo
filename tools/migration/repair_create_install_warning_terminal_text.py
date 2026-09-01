#!/usr/bin/env python3
"""Run the reviewed install-warning transformer and normalize lint-only literals."""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from pathlib import Path

ORIGINAL_COMMIT = "4fa9ec064504884b6ec0d5055bdbfbdf672c563c"
ORIGINAL_BLOB = "37ba7035e0d3dec608f7d3cc4b3b20cee1a0affc"
SCRIPT_PATH = Path("tools/migration/repair_create_install_warning_terminal_text.py")
SANITIZER_PATH = Path(
    "packages/create-turbo/src/utils/sanitize-terminal-text.ts"
)

HEX_LITERAL_REPLACEMENTS = {
    "0x1f": "0x1F",
    "0x7f": "0x7F",
    "0x9f": "0x9F",
    "0xd800": "0xD800",
    "0xdfff": "0xDFFF",
    "0x00ad": "0x00AD",
    "0x034f": "0x034F",
    "0x061c": "0x061C",
    "0x180e": "0x180E",
    "0x200b": "0x200B",
    "0x200f": "0x200F",
    "0x202e": "0x202E",
    "0x206f": "0x206F",
    "0xfeff": "0xFEFF",
    "0xfff9": "0xFFF9",
    "0xfffb": "0xFFFB",
}


def reviewed_source() -> bytes:
    source = subprocess.check_output(
        ["git", "show", f"{ORIGINAL_COMMIT}:{SCRIPT_PATH.as_posix()}"],
    )
    blob = subprocess.check_output(
        ["git", "hash-object", "--stdin"],
        input=source,
        text=False,
    ).decode("ascii").strip()
    if blob != ORIGINAL_BLOB:
        raise SystemExit(
            f"reviewed transformer blob changed: expected {ORIGINAL_BLOB}, found {blob}"
        )
    return source


def run_reviewed_transformer(arguments: list[str]) -> None:
    source = reviewed_source()
    with tempfile.TemporaryDirectory(prefix="create-install-warning-") as directory:
        script = Path(directory, "reviewed_transformer.py")
        script.write_bytes(source)
        subprocess.run(
            [sys.executable, str(script), *arguments],
            check=True,
        )


def normalize_hex_literals() -> None:
    text = SANITIZER_PATH.read_text(encoding="utf-8")
    for old, new in HEX_LITERAL_REPLACEMENTS.items():
        count = text.count(old)
        if count != 1:
            raise SystemExit(
                f"sanitizer hexadecimal anchor changed: {old} occurred {count} times"
            )
        text = text.replace(old, new, 1)

    lowercase_literal = re.search(r"0x[0-9A-F]*[a-f][0-9A-Fa-f]*", text)
    if lowercase_literal:
        raise SystemExit(
            "lowercase hexadecimal literal remains in sanitizer: "
            + lowercase_literal.group(0)
        )

    SANITIZER_PATH.write_text(text, encoding="utf-8")


def main() -> None:
    arguments = sys.argv[1:]
    if not arguments or arguments[0] not in {"red", "green", "docs"}:
        raise SystemExit("expected phase: red, green, or docs")

    run_reviewed_transformer(arguments)
    if arguments[0] == "green":
        normalize_hex_literals()


if __name__ == "__main__":
    main()
