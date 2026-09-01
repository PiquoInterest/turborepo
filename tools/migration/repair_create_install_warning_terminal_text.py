#!/usr/bin/env python3
"""One-shot TDD migration for the create-turbo install warning boundary."""

from __future__ import annotations

import argparse
from pathlib import Path

WORKFLOW_PATH = Path(
    ".github/workflows/repair-create-install-warning-terminal-text.yml"
)
SCRIPT_PATH = Path("tools/migration/repair_create_install_warning_terminal_text.py")


def replace_once(path: str | Path, old: str, new: str, label: str) -> None:
    file_path = Path(path)
    text = file_path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label} anchor count changed in {file_path}: {count}")
    file_path.write_text(text.replace(old, new, 1), encoding="utf-8")


def apply_red() -> None:
    path = Path("packages/create-turbo/__tests__/create-install-policy.test.ts")
    text = path.read_text(encoding="utf-8")

    import_anchor = 'import path from "node:path";\n'
    if text.count(import_anchor) != 1:
        raise SystemExit("node:path import anchor changed")
    text = text.replace(
        import_anchor,
        'import { Buffer } from "node:buffer";\n' + import_anchor,
        1,
    )

    start_anchor = '''  it.failing(
    "does not pass raw terminal-control text from the example name to warning output",
'''
    start = text.find(start_anchor)
    if start < 0:
        raise SystemExit("install-warning failing-oracle anchor changed")
    end_anchor = "\n  );\n});"
    end = text.find(end_anchor, start)
    if end < 0:
        raise SystemExit("install-warning failing-oracle end anchor changed")
    end += len("\n  );")

    replacement = r'''  it(
    "does not pass raw terminal-control text from the example name to warning output",
    async () => {
      const { root } = useFixture({ fixture: "hostile-warning-name" });
      const hostileExample =
        "community\u001B]8;;https://attacker.invalid\u0007name\nspoof\u202Ertl\u200Bhidden";
      const arranged = arrange({
        root,
        sourcePackageManager: "aube",
        availablePackageManagers: {
          npm: "10.9.0",
          pnpm: "9.15.4",
          yarn: "1.22.22",
          bun: "1.2.3",
          nub: "0.1.0",
          aube: undefined
        }
      });
      const errorSpy = requireConsoleErrorSpy();
      errorSpy.mockClear();

      try {
        await create(root as CreateCommandArgument, {
          skipTransforms: true,
          skipInstall: false,
          example: hostileExample,
          git: false,
          telemetry: undefined
        });

        const warningMessages = errorSpy.mock.calls.map(([, message]) =>
          String(message)
        );
        const rendered = warningMessages.join(" ");
        for (const raw of [
          "\u001B",
          "\u0007",
          "\n",
          "\u202E",
          "\u200B"
        ]) {
          expect(rendered).not.toContain(raw);
        }
        for (const escaped of [
          "\\u{1b}",
          "\\u{7}",
          "\\n",
          "\\u{202e}",
          "\\u{200b}"
        ]) {
          expect(rendered).toContain(escaped);
        }
      } finally {
        arranged.restore();
      }
    }
  );

  it("bounds attacker-controlled example names in warning output", async () => {
    const { root } = useFixture({ fixture: "oversized-warning-name" });
    const oversizedExample = "x".repeat(16 * 1024);
    const arranged = arrange({
      root,
      sourcePackageManager: "aube",
      availablePackageManagers: {
        npm: "10.9.0",
        pnpm: "9.15.4",
        yarn: "1.22.22",
        bun: "1.2.3",
        nub: "0.1.0",
        aube: undefined
      }
    });
    const errorSpy = requireConsoleErrorSpy();
    errorSpy.mockClear();

    try {
      await create(root as CreateCommandArgument, {
        skipTransforms: true,
        skipInstall: false,
        example: oversizedExample,
        git: false,
        telemetry: undefined
      });

      const warningMessages = errorSpy.mock.calls.map(([, message]) =>
        String(message)
      );
      expect(warningMessages).toHaveLength(2);
      for (const message of warningMessages) {
        expect(Buffer.byteLength(message, "utf8")).toBeLessThanOrEqual(4096);
        expect(message).toContain("[truncated]");
      }
    } finally {
      arranged.restore();
    }
  });'''

    text = text[:start] + replacement + text[end:]
    if "it.failing(" in text:
        raise SystemExit("install-warning test still uses it.failing")
    path.write_text(text, encoding="utf-8")


def apply_green() -> None:
    Path("packages/create-turbo/src/utils/sanitize-terminal-text.ts").write_text(
        '''import { Buffer } from "node:buffer";

const TRUNCATION_MARKER = "[truncated]";

export function sanitizeTerminalText(
  input: string,
  maxBytes: number
): string {
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 0) {
    throw new RangeError("maxBytes must be a non-negative safe integer");
  }

  const fragments: Array<{ value: string; bytes: number }> = [];
  let outputBytes = 0;
  let truncated = false;

  for (const character of input) {
    const value = terminalFragment(character);
    const bytes = Buffer.byteLength(value, "utf8");
    if (bytes > maxBytes - outputBytes) {
      truncated = true;
      break;
    }
    fragments.push({ value, bytes });
    outputBytes += bytes;
  }

  if (!truncated) {
    return fragments.map(({ value }) => value).join("");
  }

  if (maxBytes < TRUNCATION_MARKER.length) {
    return TRUNCATION_MARKER.slice(0, maxBytes);
  }

  const contentLimit = maxBytes - TRUNCATION_MARKER.length;
  while (outputBytes > contentLimit) {
    const fragment = fragments.pop();
    if (!fragment) {
      outputBytes = 0;
      break;
    }
    outputBytes -= fragment.bytes;
  }

  return fragments.map(({ value }) => value).join("") + TRUNCATION_MARKER;
}

function terminalFragment(character: string): string {
  if (character === "\\n") {
    return "\\\\n";
  }
  if (character === "\\r") {
    return "\\\\r";
  }
  if (character === "\\t") {
    return "\\\\t";
  }

  const codePoint = character.codePointAt(0);
  if (
    codePoint === undefined ||
    isControl(codePoint) ||
    isUnpairedSurrogate(codePoint) ||
    isTerminalFormatControl(codePoint)
  ) {
    return `\\\\u{${(codePoint ?? 0).toString(16)}}`;
  }
  return character;
}

function isControl(codePoint: number): boolean {
  return codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f);
}

function isUnpairedSurrogate(codePoint: number): boolean {
  return codePoint >= 0xd800 && codePoint <= 0xdfff;
}

function isTerminalFormatControl(codePoint: number): boolean {
  return (
    codePoint === 0x00ad ||
    codePoint === 0x034f ||
    codePoint === 0x061c ||
    codePoint === 0x180e ||
    (codePoint >= 0x200b && codePoint <= 0x200f) ||
    (codePoint >= 0x2028 && codePoint <= 0x202e) ||
    (codePoint >= 0x2060 && codePoint <= 0x206f) ||
    codePoint === 0xfeff ||
    (codePoint >= 0xfff9 && codePoint <= 0xfffb)
  );
}
''',
        encoding="utf-8",
    )

    Path("packages/create-turbo/src/commands/create/install-warning.ts").write_text(
        '''import type { PackageManager } from "@turbo/utils";
import { sanitizeTerminalText } from "../../utils/sanitize-terminal-text";

export const CREATE_INSTALL_WARNING_LINE_LIMIT = 4096;
export const CREATE_INSTALL_WARNING_EXAMPLE_LIMIT =
  CREATE_INSTALL_WARNING_LINE_LIMIT / 2;

export function renderUnavailablePackageManagerWarning(
  exampleName: string,
  packageManager: PackageManager
): readonly [string, string] {
  const safeExampleName = sanitizeTerminalText(
    exampleName,
    CREATE_INSTALL_WARNING_EXAMPLE_LIMIT
  );

  return [
    sanitizeTerminalText(
      `Unable to install dependencies - "${safeExampleName}" uses "${packageManager}" which could not be found.`,
      CREATE_INSTALL_WARNING_LINE_LIMIT
    ),
    sanitizeTerminalText(
      `Try running without "--skip-transforms" to convert "${safeExampleName}" to a package manager that is available on your system.`,
      CREATE_INSTALL_WARNING_LINE_LIMIT
    )
  ];
}
''',
        encoding="utf-8",
    )

    path = Path("packages/create-turbo/src/commands/create/index.ts")
    text = path.read_text(encoding="utf-8")
    import_anchor = (
        'import type { CreateCommandArgument, CreateCommandOptions } from "./types";\n'
    )
    if text.count(import_anchor) != 1:
        raise SystemExit("create command type import anchor changed")
    text = text.replace(
        import_anchor,
        'import { renderUnavailablePackageManagerWarning } from "./install-warning";\n'
        + import_anchor,
        1,
    )

    old = '''      warn(
        `Unable to install dependencies - "${exampleName}" uses "${project.packageManager}" which could not be found.`
      );
      warn(
        `Try running without "--skip-transforms" to convert "${exampleName}" to a package manager that is available on your system.`
      );
      logger.log();'''
    new = '''      for (const warning of renderUnavailablePackageManagerWarning(
        exampleName,
        project.packageManager
      )) {
        warn(warning);
      }
      logger.log();'''
    if text.count(old) != 1:
        raise SystemExit("raw install-warning interpolation anchor changed")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def apply_docs(red_sha: str, green_sha: str) -> None:
    replace_once(
        "packages/create-turbo/rust/README.md",
        "Install warning GREEN:  9423b807e72883f30c3e6bbf83fa918d2d846e34\n",
        "Install warning GREEN:  9423b807e72883f30c3e6bbf83fa918d2d846e34\n"
        f"TS warning oracle RED: {red_sha}\n"
        f"TS warning GREEN:      {green_sha}\n",
        "README TDD history",
    )

    replace_once(
        "packages/create-turbo/rust/PARITY_MATRIX.md",
        "| raw example name in warning output | bounded terminal-safe renderer | intentional-hardening | Safe wording is preserved; hostile controls and large text are escaped/truncated. |",
        "| raw example name in warning output | shared bounded terminal-safe renderer | repaired TypeScript and implemented Rust core | The production TypeScript warning and Rust renderer both escape controls and directionality and cap fields and lines; the Rust host binding remains pending. |",
        "parity warning row",
    )
    replace_once(
        "packages/create-turbo/rust/PARITY_MATRIX.md",
        "| create install decision and warning rendering | fourteen parity and thirteen security tests | implemented core and intentional-hardening evidence |",
        "| create install decision and warning rendering | fourteen parity and thirteen security tests plus passing TypeScript production regressions | Rust core implemented; TypeScript warning boundary repaired; production Rust binding remains blocked |",
        "parity test mapping",
    )

    old_security = '''### CT-RS-028: Untrusted example names reach unavailable-manager warnings

**Severity:** Medium until production cutover

TypeScript interpolates the example name directly into two warning lines. Rust returns structured warning data and renders the example through bounded terminal-safe fields. Safe wording remains exact; controls, bidi/invisible format text, and oversized input cannot become raw terminal output.

Regression coverage is in `create_install_warning_parity.rs` and `create_install_warning_security.rs`.'''
    new_security = f'''### CT-RS-028: Untrusted example names reached unavailable-manager warnings

**Severity:** Medium, fixed for the production TypeScript warning and Rust renderer

The TypeScript create command previously interpolated a CLI- or repository-controlled example name directly into two terminal warning lines. ESC/OSC/BEL sequences, line controls, bidi overrides, zero-width format characters, and oversized values could therefore forge or reorder terminal output.

RED `{red_sha}` converted the executable `it.failing` oracle into normal production assertions and added a 4,096-byte output bound. It failed because both warning lines still contained the raw hostile value. GREEN `{green_sha}` introduced a shared TypeScript `sanitizeTerminalText` encoder, caps the example field at 2,048 UTF-8 bytes, caps each complete warning line at 4,096 bytes, appends `[truncated]` without splitting encoded fragments, and routes both logger calls through the renderer. Safe wording remains unchanged.

The Rust renderer already enforced the same field and line limits and escaped control, directionality, and invisible format text. The focused TypeScript suite is now GREEN and `create_install_warning_parity.rs` plus `create_install_warning_security.rs` remain GREEN in Rust. No dependency, lockfile, process, filesystem, network, credential, parser, or `unsafe` capability was added.

Residual terminal-output work is tracked separately by CT-RS-024 and CT-RS-031; this fix closes only the unavailable-manager warning path.'''
    replace_once(
        "packages/create-turbo/rust/SECURITY.md",
        old_security,
        new_security,
        "CT-RS-028",
    )
    replace_once(
        "packages/create-turbo/rust/SECURITY.md",
        "**Lookup date: 2026-08-31**",
        "**Lookup date: 2026-09-01**",
        "security lookup date",
    )
    replace_once(
        "packages/create-turbo/rust/SECURITY.md",
        "Disposition:\n\n",
        "Disposition:\n\n"
        "- The TypeScript terminal-boundary repair adds no dependency and leaves `Cargo.lock` and `pnpm-lock.yaml` unchanged; the RustSec and GitHub advisory sources were rechecked on 2026-09-01.\n",
        "security disposition",
    )

    replace_once(
        "packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md",
        "- GREEN implementation: `9423b807e72883f30c3e6bbf83fa918d2d846e34`\n",
        "- GREEN implementation: `9423b807e72883f30c3e6bbf83fa918d2d846e34`\n"
        f"- TypeScript production RED: `{red_sha}`\n"
        f"- TypeScript production GREEN: `{green_sha}`\n",
        "install-warning divergence TDD",
    )
    old_divergence = '''The TypeScript warning interpolates the example name directly into two terminal messages. A repository or CLI-controlled name can contain ESC/OSC sequences, BEL, newlines, carriage returns, tabs, bidi overrides, zero-width controls, or a very large payload. The TypeScript `it.failing` case preserves this defect as executable evidence while TypeScript remains the oracle.

The Rust renderer:'''
    new_divergence = f'''The TypeScript warning previously interpolated the example name directly into two terminal messages. A repository or CLI-controlled name could contain ESC/OSC sequences, BEL, newlines, carriage returns, tabs, bidi overrides, zero-width controls, or a very large payload. RED `{red_sha}` made that executable oracle a normal failing test; GREEN `{green_sha}` routes both production messages through the shared bounded sanitizer, so the TypeScript issue is now closed rather than retained as `it.failing` evidence.

The TypeScript and Rust renderers:'''
    replace_once(
        "packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md",
        old_divergence,
        new_divergence,
        "divergence CT-RS-028",
    )
    replace_once(
        "packages/create-turbo/rust/CREATE_INSTALL_POLICY_DIVERGENCES.md",
        "- returns two strings without performing terminal I/O.\n",
        "- return two strings without performing terminal I/O.\n\n"
        "The TypeScript production regression and Rust security suite use the same hostile control and directionality classes. Exact escape spelling is reviewed per host, while both sides prohibit raw terminal-active output and enforce the same byte limits.\n",
        "divergence renderer evidence",
    )

    old_global = '''### RF-019: Create-command error, warning, and final output accept terminal-active untrusted text

**Status:** Fixed in Rust policy/rendering cores; TypeScript production output remains.

Rust escapes terminal, line, directionality, and invisible format controls, applies explicit UTF-8 and record-count limits, and never renders unknown errors. Production bindings must emit only these reviewed strings, apply coloring afterwards, and prove there is no second raw-output path.'''
    new_global = f'''### RF-019: Create-command error, warning, and final output accept terminal-active untrusted text

**Status:** Package-install warning fixed in TypeScript and Rust; other TypeScript output boundaries remain.

RED `{red_sha}` proved that the production TypeScript unavailable-manager warning emitted a hostile example name unchanged. GREEN `{green_sha}` added a shared terminal-text sanitizer and renderer with a 2,048-byte example bound and 4,096-byte line bound. The TypeScript regression is now a normal passing test, while the Rust install-warning parity and security suites remain GREEN.

Rust also escapes terminal, line, directionality, and invisible format controls across its other policy/rendering cores, applies explicit UTF-8 and record-count limits, and never renders unknown errors. Remaining production bindings and TypeScript output paths must emit only reviewed strings, apply coloring afterwards, and prove there is no second raw-output path.'''
    replace_once(
        "docs/rust-migration-security-findings.md",
        old_global,
        new_global,
        "RF-019",
    )

    replace_once(
        "docs/typescript-deprecation.md",
        "Security closure includes terminal-control and directionality escaping, explicit field/line/count bounds, unknown-error non-disclosure, cleanup-before-exit capability, one-shot installer invocation, and a single availability snapshot. Production host bindings must prove exactly-once telemetry and output, error identity, path/group/locale derivation, coloring after sanitization, and no raw logger bypass.\n",
        "Security closure includes terminal-control and directionality escaping, explicit field/line/count bounds, unknown-error non-disclosure, cleanup-before-exit capability, one-shot installer invocation, and a single availability snapshot. The production TypeScript unavailable-manager warning now uses the shared bounded encoder too: RED `"
        + red_sha
        + "` exposed raw control output and GREEN `"
        + green_sha
        + "` made both warning lines pass normal TypeScript regressions while the Rust warning suite stayed GREEN. Other production host bindings must still prove exactly-once telemetry and output, error identity, path/group/locale derivation, coloring after sanitization, and no raw logger bypass.\n",
        "program create-policy paragraph",
    )
    replace_once(
        "docs/typescript-deprecation.md",
        "- install-policy RED/GREEN: `ff359432f3b91d1f164c68ed0270d62ec8b15f42` / `02eb3f5ba3a8733cf27c5377aaca3fae1ad09f2a`;\n",
        "- install-policy RED/GREEN: `ff359432f3b91d1f164c68ed0270d62ec8b15f42` / `02eb3f5ba3a8733cf27c5377aaca3fae1ad09f2a`;\n"
        f"- TypeScript install-warning boundary RED/GREEN: `{red_sha}` / `{green_sha}`;\n",
        "program TDD history",
    )

    WORKFLOW_PATH.unlink()
    SCRIPT_PATH.unlink()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=("red", "green", "docs"))
    parser.add_argument("red_sha", nargs="?")
    parser.add_argument("green_sha", nargs="?")
    args = parser.parse_args()

    if args.phase == "red":
        apply_red()
        return
    if args.phase == "green":
        apply_green()
        return
    if not args.red_sha or not args.green_sha:
        raise SystemExit("docs phase requires RED and GREEN commit SHAs")
    apply_docs(args.red_sha, args.green_sha)


if __name__ == "__main__":
    main()
