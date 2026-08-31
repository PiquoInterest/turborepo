import { describe, expect, it } from "@jest/globals";
import { TransformError } from "../src/transforms/errors";
import { MAX_TERMINAL_DIAGNOSTIC_SCALARS } from "../src/transforms/terminal";

describe("TransformError terminal security", () => {
  it("neutralizes terminal controls and bidirectional format characters", () => {
    const rawMessage =
      "failed\u001b[31m\nnext\rline\tcol\u202etxt\u2066iso\u200bhidden\u009bcsi";
    const rawTransform = "../../official-starter\u0000\u2069";

    const error = new TransformError(rawMessage, {
      transform: rawTransform,
      fatal: false
    });

    expect(error.message).toBe(
      "failed\\u{1b}[31m\\nnext\\rline\\tcol\\u{202e}txt\\u{2066}iso\\u{200b}hidden\\u{9b}csi"
    );
    expect(error.transform).toBe("../../official-starter\\0\\u{2069}");
    expect(error.rawMessage).toBe(rawMessage);
    expect(error.rawTransform).toBe(rawTransform);
    expect(error.fatal).toBe(false);
  });

  it("bounds attacker-controlled terminal fields without scanning the full suffix", () => {
    const rawMessage = "x".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4096);
    const rawTransform = "y".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4096);

    const error = new TransformError(rawMessage, { transform: rawTransform });

    expect([...error.message]).toHaveLength(
      MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    expect([...error.transform]).toHaveLength(
      MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    expect(error.message.endsWith("…")).toBe(true);
    expect(error.transform.endsWith("…")).toBe(true);
    expect(error.rawMessage).toBe(rawMessage);
    expect(error.rawTransform).toBe(rawTransform);
  });

  it("preserves ordinary printable Unicode exactly", () => {
    const message = "Unable to transform café 🚀";
    const transform = "official-starter";
    const error = new TransformError(message, { transform });

    expect(error.message).toBe(message);
    expect(error.transform).toBe(transform);
  });
});
