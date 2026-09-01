import fs from "fs-extra";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { getWorkspaceDetails } from "../src/get-workspace-details";
import { MANAGERS } from "../src/managers";
import type { PackageManager, Project } from "../src/types";

const MANAGER_ORDER: Array<PackageManager> = [
  "aube",
  "nub",
  "pnpm",
  "yarn",
  "npm",
  "bun"
];

const temporaryRoots: Array<string> = [];

afterEach(() => {
  jest.restoreAllMocks();
  for (const root of temporaryRoots.splice(0)) {
    fs.removeSync(root);
  }
});

function temporaryDirectory(): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "turbo-workspace-details-"));
  temporaryRoots.push(root);
  return root;
}

function project(root: string, packageManager: PackageManager): Project {
  return {
    name: `${packageManager}-project`,
    packageManager,
    paths: {
      root,
      packageJson: path.join(root, "package.json"),
      lockfile: path.join(root, `${packageManager}.lock`),
      nodeModules: path.join(root, "node_modules")
    },
    workspaceData: {
      globs: [],
      workspaces: []
    }
  };
}

function mockDetectors(
  implementation: (manager: PackageManager) => Promise<boolean>
): void {
  for (const manager of MANAGER_ORDER) {
    jest
      .spyOn(MANAGERS[manager], "detect")
      .mockImplementation(() => implementation(manager));
  }
}

describe("getWorkspaceDetails orchestration oracle", () => {
  it("preserves the manager registry insertion order", () => {
    expect(Object.keys(MANAGERS)).toEqual(MANAGER_ORDER);
  });

  it("reports the absolute missing directory before invoking a manager", async () => {
    const parent = temporaryDirectory();
    const missing = path.join(parent, "missing");
    mockDetectors(async () => {
      throw new Error("a missing directory must not reach manager detection");
    });

    await expect(getWorkspaceDetails({ root: missing })).rejects.toMatchObject({
      message: `Could not find directory at ${missing}. Ensure the directory exists.`,
      type: "invalid_directory"
    });

    for (const manager of MANAGER_ORDER) {
      expect(MANAGERS[manager].detect).not.toHaveBeenCalled();
    }
  });

  it("detects serially and reads only the first successful manager", async () => {
    const root = temporaryDirectory();
    const calls: Array<string> = [];
    mockDetectors(async (manager) => {
      calls.push(`detect:${manager}`);
      return manager === "pnpm";
    });

    for (const manager of MANAGER_ORDER) {
      jest.spyOn(MANAGERS[manager], "read").mockImplementation(async () => {
        calls.push(`read:${manager}`);
        return project(root, manager);
      });
    }

    await expect(getWorkspaceDetails({ root })).resolves.toEqual(
      project(root, "pnpm")
    );
    expect(calls).toEqual([
      "detect:aube",
      "detect:nub",
      "detect:pnpm",
      "read:pnpm"
    ]);
    expect(MANAGERS.yarn.detect).not.toHaveBeenCalled();
    expect(MANAGERS.npm.detect).not.toHaveBeenCalled();
    expect(MANAGERS.bun.detect).not.toHaveBeenCalled();
  });

  it("propagates the selected manager read failure without parser fallback", async () => {
    const root = temporaryDirectory();
    const readFailure = new Error("selected parser rejected workspace metadata");
    mockDetectors(async (manager) => manager === "pnpm" || manager === "yarn");
    jest.spyOn(MANAGERS.pnpm, "read").mockRejectedValue(readFailure);
    jest.spyOn(MANAGERS.yarn, "read").mockResolvedValue(project(root, "yarn"));

    await expect(getWorkspaceDetails({ root })).rejects.toBe(readFailure);
    expect(MANAGERS.yarn.detect).not.toHaveBeenCalled();
    expect(MANAGERS.yarn.read).not.toHaveBeenCalled();
  });

  it("returns the exact unable-to-detect error after all six managers reject", async () => {
    const root = temporaryDirectory();
    mockDetectors(async () => false);

    await expect(getWorkspaceDetails({ root })).rejects.toMatchObject({
      message:
        "Could not determine package manager. Add `devEngines.packageManager` or legacy `packageManager` to `package.json`, or ensure a lockfile is present.",
      type: "package_manager-unable_to_detect"
    });

    for (const manager of MANAGER_ORDER) {
      expect(MANAGERS[manager].detect).toHaveBeenCalledTimes(1);
    }
  });
});
