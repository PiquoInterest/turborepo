import path from "node:path";
import { setupTestFixtures, spyConsole, spyExit } from "@turbo/test-utils";
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

describe("legacy create error security evidence", () => {
  const { useFixture } = setupTestFixtures({
    directory: path.join(__dirname, "../"),
    options: { emptyFixture: true }
  });
  const mockConsole = spyConsole();
  const mockExit = spyExit();

  async function captureDownloadFailure(message: string): Promise<string> {
    const { root } = useFixture({ fixture: "create-error-security" });
    mockConsole.error.mockClear();
    mockExit.exit.mockClear();

    const mockAvailablePackageManagers = jest
      .spyOn(turboUtils, "getAvailablePackageManagers")
      .mockResolvedValue({
        npm: "8.19.2",
        yarn: "1.22.10",
        pnpm: "7.22.2",
        bun: "1.0.1",
        nub: "0.1.0",
        aube: "0.1.0"
      });
    const mockCreateProject = jest
      .spyOn(turboUtils, "createProject")
      .mockRejectedValue(new turboUtils.DownloadError(message));
    const mockGetWorkspaceDetails = jest
      .spyOn(turboWorkspaces, "getWorkspaceDetails")
      .mockResolvedValue(
        getWorkspaceDetailsMockReturnValue({
          root,
          packageManager: "pnpm"
        })
      );

    try {
      await create(root as CreateCommandArgument, {
        packageManager: "pnpm",
        skipInstall: true,
        skipTransforms: true,
        example: "default",
        git: false
      });

      const calls = mockConsole.error.mock.calls;
      const lastCall = calls[calls.length - 1];
      return String(lastCall?.[1] ?? "");
    } finally {
      mockAvailablePackageManagers.mockRestore();
      mockCreateProject.mockRestore();
      mockGetWorkspaceDetails.mockRestore();
    }
  }

  it.failing(
    "does not pass terminal-control sequences from a download error to stderr",
    async () => {
      const message =
        "download failed\u001b]8;;https://attacker.invalid\u0007click\u001b]8;;\u0007\rspoofed\u202etxt";
      const rendered = await captureDownloadFailure(message);

      expect(rendered).not.toContain("\u001b]8;;");
      expect(rendered).not.toContain("\u0007");
      expect(rendered).not.toContain("\rspoofed");
      expect(rendered).not.toContain("\u202e");
    }
  );

  it.failing("bounds untrusted download-error output", async () => {
    const rendered = await captureDownloadFailure("A".repeat(16 * 1024));

    expect(Buffer.byteLength(rendered, "utf8")).toBeLessThanOrEqual(4096);
  });
});
