import execa from "execa";
import { describe, expect, it, jest } from "@jest/globals";
import { install } from "../src";

jest.mock("execa", () => jest.fn());

describe("package-manager install execution security evidence", () => {
  it.failing(
    "does not use a shell or prefer a project-local package-manager executable",
    async () => {
      const originalPlatform = process.platform;
      Object.defineProperty(process, "platform", {
        value: "win32"
      });
      jest.clearAllMocks();
      jest.mocked(execa).mockResolvedValue({
        stdout: "",
        stderr: "",
        exitCode: 0,
        command: "",
        failed: false,
        timedOut: false,
        isCanceled: false,
        killed: false
      } as any);

      try {
        await install({
          project: {
            name: "security-evidence",
            packageManager: "bun",
            paths: {
              root: "C:\\safe-project",
              packageJson: "C:\\safe-project\\package.json",
              lockfile: "C:\\safe-project\\bun.lockb",
              nodeModules: "C:\\safe-project\\node_modules"
            },
            workspaceData: {
              globs: [],
              workspaces: []
            }
          },
          to: {
            name: "bun",
            version: "1.0.1"
          },
          options: {
            dry: false,
            interactive: false
          }
        });

        expect(execa).toHaveBeenCalledWith(
          "bun",
          ["install"],
          expect.objectContaining({
            preferLocal: false,
            shell: false,
            stdin: "ignore"
          })
        );
      } finally {
        Object.defineProperty(process, "platform", {
          value: originalPlatform
        });
      }
    }
  );
});
