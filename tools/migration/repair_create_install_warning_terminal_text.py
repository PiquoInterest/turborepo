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

NUMERIC_LITERAL_REPLACEMENTS = {
    "0x1f": "31",
    "0x7f": "127",
    "0x9f": "159",
    "0xd800": "55296",
    "0xdfff": "57343",
    "0x00ad": "173",
    "0x034f": "847",
    "0x061c": "1564",
    "0x180e": "6158",
    "0x200b": "8203",
    "0x200f": "8207",
    "0x2028": "8232",
    "0x202e": "8238",
    "0x2060": "8288",
    "0x206f": "8303",
    "0xfeff": "65279",
    "0xfff9": "65529",
    "0xfffb": "65531",
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


def normalize_numeric_literals() -> None:
    text = SANITIZER_PATH.read_text(encoding="utf-8")
    for old, new in NUMERIC_LITERAL_REPLACEMENTS.items():
        count = text.count(old)
        if count != 1:
            raise SystemExit(
                f"sanitizer numeric anchor changed: {old} occurred {count} times"
            )
        text = text.replace(old, new, 1)

    hexadecimal_literal = re.search(r"0x[0-9A-Fa-f]+", text)
    if hexadecimal_literal:
        raise SystemExit(
            "hexadecimal literal remains in sanitizer: "
            + hexadecimal_literal.group(0)
        )

    SANITIZER_PATH.write_text(text, encoding="utf-8")


def main() -> None:
    arguments = sys.argv[1:]
    if not arguments or arguments[0] not in {"red", "green", "docs"}:
        raise SystemExit("expected phase: red, green, or docs")

    run_reviewed_transformer(arguments)
    if arguments[0] == "green":
        normalize_numeric_literals()


if __name__ == "__main__":
    main()
