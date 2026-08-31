import { describe, expect, it } from "@jest/globals";
import { TransformError } from "../src/transforms/errors";
import { MAX_TERMINAL_DIAGNOSTIC_SCALARS } from "../src/transforms/terminal";

describe("TransformError terminal security", () => {
  it("neutralizes terminal controls while preserving the public raw fields", () => {
    const rawMessage =
      "failed\u001b[31m\nnext\rline\tcol\u202etxt\u2066iso\u200bhidden\u009bcsi";
    const rawTransform = "../../official-starter\u0000\u2069";

    const error = new TransformError(rawMessage, {
      transform: rawTransform,
      fatal: false
    });

    expect(error.message).toBe(rawMessage);
    expect(error.transform).toBe(rawTransform);
    expect(error.terminalMessage).toBe(
      "failed\\u{1b}[31m\\nnext\\rline\\tcol\\u{202e}txt\\u{2066}iso\\u{200b}hidden\\u{9b}csi"
    );
    expect(error.terminalTransform).toBe(
      "../../official-starter\\0\\u{2069}"
    );
    expect(error.fatal).toBe(false);
  });

  it("bounds terminal fields without truncating the structured error values", () => {
    const rawMessage = "x".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4096);
    const rawTransform = "y".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4096);

    const error = new TransformError(rawMessage, { transform: rawTransform });

    expect([...error.terminalMessage]).toHaveLength(
      MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    expect([...error.terminalTransform]).toHaveLength(
      MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    expect(error.terminalMessage.endsWith("…")).toBe(true);
    expect(error.terminalTransform.endsWith("…")).toBe(true);
    expect(error.message).toBe(rawMessage);
    expect(error.transform).toBe(rawTransform);
  });

  it("re-sanitizes fields that were mutated after construction", () => {
    const error = new TransformError("safe", { transform: "safe-transform" });
    error.message = "mutated\u001b[2J\nline";
    error.transform = "mutated\u0000\u202e";

    expect(error.terminalMessage).toBe("mutated\\u{1b}[2J\\nline");
    expect(error.terminalTransform).toBe("mutated\\0\\u{202e}");
    expect(error.message).toBe("mutated\u001b[2J\nline");
    expect(error.transform).toBe("mutated\u0000\u202e");
  });

  it("preserves ordinary printable Unicode exactly", () => {
    const message = "Unable to transform café 🚀";
    const transform = "official-starter";
    const error = new TransformError(message, { transform });

    expect(error.terminalMessage).toBe(message);
    expect(error.terminalTransform).toBe(transform);
    expect(error.message).toBe(message);
    expect(error.transform).toBe(transform);
  });
});
