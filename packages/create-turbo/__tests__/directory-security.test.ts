import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "@jest/globals";
import {
  InvalidDirectoryError,
  MAX_DIRECTORY_INPUT_BYTES,
  directory
} from "../src/commands/create/prompts";

describe("directory prompt security", () => {
  it("rejects an invalid direct argument instead of returning an invalid result", async () => {
    await expect(directory({ dir: "invalid directory" })).rejects.toBeInstanceOf(
      InvalidDirectoryError
    );
  });

  it("rejects a conflicting direct directory before project creation can continue", async () => {
    const root = fs.mkdtempSync(
      path.join(os.tmpdir(), "create-turbo-directory-security-")
    );
    try {
      fs.writeFileSync(path.join(root, "package.json"), "{}", "utf8");
      await expect(directory({ dir: root })).rejects.toMatchObject({
        name: "InvalidDirectoryError",
        reason: "validation"
      });
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects terminal controls without reflecting them in the public error", async () => {
    const attackerInput = "project\u001b[31m";
    await expect(directory({ dir: attackerInput })).rejects.toMatchObject({
      name: "InvalidDirectoryError",
      reason: "unsafe-input",
      message: expect.not.stringContaining("\u001b")
    });
  });

  it("bounds direct directory input before path resolution or filesystem access", async () => {
    const oversized = "a".repeat(MAX_DIRECTORY_INPUT_BYTES + 1);
    await expect(directory({ dir: oversized })).rejects.toMatchObject({
      name: "InvalidDirectoryError",
      reason: "unsafe-input"
    });
  });
});
