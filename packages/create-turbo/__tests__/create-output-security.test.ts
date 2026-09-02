import path from "node:path";
import { setupTestFixtures, spyConsole } from "@turbo/test-utils";
import type { Project } from "@turbo/workspaces";
import * as turboWorkspaces from "@turbo/workspaces";
import * as turboUtils from "@turbo/utils";
import { describe, expect, it, jest } from "@jest/globals";
import type { CreateCommandArgument } from "../src/commands/create/types";
import { create } from "../src/commands/create";
import { getWorkspaceDetailsMockReturnValue } from "./test-utils";

jest.mock<typeof import("@turbo/workspaces")>("@turbo/workspaces", () => ({
  __esModule: true,
  ...jest.requireActual("@turbo/workspaces")
}));

describe("legacy create output security evidence", () => {
  const { useFixture } = setupTestFixtures({
    directory: path.join(__dirname, "../"),
    options: { emptyFixture: true }
  });
  const mockConsole = spyConsole();

  function requireConsoleLogSpy() {
    const logSpy = mockConsole.log;
    if (!logSpy) {
      throw new Error("spyConsole did not provide a log spy");
    }
    return logSpy;
  }

  async function renderWorkspaceDescription(description: string): Promise<string[]> {
    const { root } = useFixture({ fixture: "create-output-security" });
    const logSpy = requireConsoleLogSpy();
    logSpy.mockClear();

    const baseProject = getWorkspaceDetailsMockReturnValue({
      root,
      packageManager: "pnpm"
    });
    const workspaceRoot = path.join(root, "packages", "unsafe-workspace");
    const project: Project = {
      ...baseProject,
      workspaceData: {
        globs: ["packages/*"],
        workspaces: [
          {
            name: "packages/unsafe-workspace",
            description,
            paths: {
              root: workspaceRoot,
              packageJson: path.join(workspaceRoot, "package.json"),
              nodeModules: path.join(workspaceRoot, "node_modules")
            }
          }
        ]
      }
    };

    const available = jest
      .spyOn(turboUtils, "getAvailablePackageManagers")
      .mockResolvedValue({
        npm: "10.9.0",
        pnpm: "9.15.4",
        yarn: "1.22.22",
        bun: "1.2.3",
        nub: "0.1.0",
        aube: "0.1.0"
      });
    const createProject = jest
      .spyOn(turboUtils, "createProject")
      .mockResolvedValue({
        cdPath: "",
        hasPackageJson: true,
        availableScripts: []
      });
    const workspace = jest
      .spyOn(turboWorkspaces, "getWorkspaceDetails")
      .mockResolvedValue(project);

    try {
      await create(root as CreateCommandArgument, {
        skipTransforms: true,
        skipInstall: true,
        example: "default",
        git: false,
        telemetry: undefined
      });

      return logSpy.mock.calls.map((call) => call.map(String).join(" "));
    } finally {
      available.mockRestore();
      createProject.mockRestore();
      workspace.mockRestore();
    }
  }

  it.failing(
    "does not pass terminal-control text from workspace metadata to stdout",
    async () => {
      const description =
        "description\u001B]8;;https://attacker.invalid\u0007click\nspoof\rreset\u202ertl";
      const rendered = (await renderWorkspaceDescription(description)).join("\n");

      expect(rendered).not.toContain("\u001B]8;;");
      expect(rendered).not.toContain("\u0007");
      expect(rendered).not.toContain("\nspoof");
      expect(rendered).not.toContain("\rreset");
      expect(rendered).not.toContain("\u202e");
    }
  );

  it.failing("bounds workspace metadata output lines", async () => {
    const lines = await renderWorkspaceDescription("A".repeat(16 * 1024));
    const maximumLineLength = Math.max(
      ...lines.map((line) => Buffer.byteLength(line, "utf8"))
    );

    expect(maximumLineLength).toBeLessThanOrEqual(4096);
  });
});
