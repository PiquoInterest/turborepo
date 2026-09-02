import { describe, expect, it } from "@jest/globals";
import { parseWorkspacePackages } from "../src/utils";

describe("parseWorkspacePackages", () => {
  it.each([
    { workspaces: undefined, expected: [] },
    { workspaces: [], expected: [] },
    { workspaces: ["apps/*"], expected: ["apps/*"] },
    {
      workspaces: { packages: ["apps/*", "packages/*"] },
      expected: ["apps/*", "packages/*"]
    },
    { workspaces: { packages: undefined }, expected: [] },
    { workspaces: {}, expected: [] }
  ])("preserves the source value contract for %#", ({ workspaces, expected }) => {
    expect(parseWorkspacePackages({ workspaces })).toEqual(expected);
  });

  it("preserves ordering, duplicates, empty values, and supported glob syntax", () => {
    const globs = [
      "packages/*",
      "apps/{web,docs}",
      "!fixtures/**",
      "packages/[ab]",
      "",
      "packages/*"
    ];

    expect(parseWorkspacePackages({ workspaces: globs })).toEqual(globs);
  });

  it.failing("rejects more than 256 workspace globs", () => {
    const globs = Array.from({ length: 257 }, (_, index) => `packages/${index}`);

    expect(() => parseWorkspacePackages({ workspaces: globs })).toThrow(
      "workspace glob count exceeds 256"
    );
  });

  it.failing("rejects one workspace glob larger than 4096 UTF-8 bytes", () => {
    const globs = [`packages/${"a".repeat(4097)}`];

    expect(() => parseWorkspacePackages({ workspaces: globs })).toThrow(
      "workspace glob exceeds 4096 UTF-8 bytes"
    );
  });

  it.failing("rejects more than 65536 total workspace-glob bytes", () => {
    const globs = Array.from({ length: 17 }, (_, index) =>
      String(index).padStart(4, "0").repeat(1024)
    );

    expect(() => parseWorkspacePackages({ workspaces: globs })).toThrow(
      "workspace glob input exceeds 65536 UTF-8 bytes"
    );
  });

  it.each(["apps/\u0000*", "apps/\u001b[31m*", "apps/\u202e*", "apps/\u2066*"])(
    "documents the current acceptance of terminal-active or invisible text %j",
    (workspaceGlob) => {
      expect(parseWorkspacePackages({ workspaces: [workspaceGlob] })).toEqual([
        workspaceGlob
      ]);
    }
  );

  it.failing.each([
    "apps/\u0000*",
    "apps/\u001b[31m*",
    "apps/\u202e*",
    "apps/\u2066*"
  ])("rejects terminal-active or invisible text %j", (workspaceGlob) => {
    expect(() =>
      parseWorkspacePackages({ workspaces: [workspaceGlob] })
    ).toThrow("workspace glob contains unsafe text");
  });
});
