import { describe, expect, it } from "@jest/globals";
import type { Project } from "../src/types";
import { isCompatibleWithBunWorkspaces } from "../src/utils";

function projectWithGlobs(globs: Array<string>): Project {
  return {
    workspaceData: { globs }
  } as Project;
}

describe("Bun workspace glob security oracle", () => {
  it.failing("rejects a workspace glob above 4096 UTF-8 bytes", () => {
    const workspaceGlob = `${"a".repeat(4095)}/*`;
    expect(
      isCompatibleWithBunWorkspaces({
        project: projectWithGlobs([workspaceGlob])
      })
    ).toBe(false);
  });

  it.failing("rejects more than 256 workspace globs", () => {
    const globs = Array.from({ length: 257 }, (_, index) => `workspace-${index}`);
    expect(
      isCompatibleWithBunWorkspaces({ project: projectWithGlobs(globs) })
    ).toBe(false);
  });

  it.failing("rejects more than 65536 total workspace-glob bytes", () => {
    const workspaceGlob = `${"a".repeat(4094)}/*`;
    const globs = Array.from({ length: 17 }, () => workspaceGlob);
    expect(
      isCompatibleWithBunWorkspaces({ project: projectWithGlobs(globs) })
    ).toBe(false);
  });

  it.failing("rejects terminal-active workspace glob text", () => {
    expect(
      isCompatibleWithBunWorkspaces({
        project: projectWithGlobs(["apps/\u001b*"])
      })
    ).toBe(false);
  });
});
