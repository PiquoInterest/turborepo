import path from "node:path";
import { setupTestFixtures, spyConsole } from "@turbo/test-utils";
import type { PackageManager } from "@turbo/utils";
import * as turboUtils from "@turbo/utils";
import * as turboWorkspaces from "@turbo/workspaces";
import { describe, expect, it, jest } from "@jest/globals";
import type { CreateCommandArgument } from "../src/commands/create/types";
import { create } from "../src/commands/create";
import { getWorkspaceDetailsMockReturnValue } from "./test-utils";

jest.mock<typeof import("@turbo/workspaces")>("@turbo/workspaces", () => ({
  __esModule: true,
  ...jest.requireActual("@turbo/workspaces")
}));

describe("create install policy", () => {
  const { useFixture } = setupTestFixtures({
    directory: path.join(__dirname, "../"),
    options: { emptyFixture: true }
  });
  const mockConsole = spyConsole();

  function requireConsoleErrorSpy() {
    const errorSpy = mockConsole.error;
    if (!errorSpy) {
      throw new Error("spyConsole did not provide an error spy");
    }
    return errorSpy;
  }

  function arrange({
    root,
    sourcePackageManager,
    availablePackageManagers,
    hasPackageJson = true
  }: {
    root: string;
    sourcePackageManager: PackageManager;
    availablePackageManagers: Record<PackageManager, string | undefined>;
    hasPackageJson?: boolean;
  }) {
    const project = getWorkspaceDetailsMockReturnValue({
      root,
      packageManager: sourcePackageManager
    });
    const available = jest
      .spyOn(turboUtils, "getAvailablePackageManagers")
      .mockResolvedValue(availablePackageManagers);
    const createProject = jest
      .spyOn(turboUtils, "createProject")
      .mockResolvedValue({
        cdPath: "",
        hasPackageJson,
        availableScripts: []
      });
    const workspace = jest
      .spyOn(turboWorkspaces, "getWorkspaceDetails")
      .mockResolvedValue(project);
    const install = jest
      .spyOn(turboWorkspaces, "install")
      .mockResolvedValue(undefined);

    return {
      install,
      project,
      restore() {
        available.mockRestore();
        createProject.mockRestore();
        workspace.mockRestore();
        install.mockRestore();
      }
    };
  }

  it("installs the source manager non-interactively when transforms are skipped", async () => {
    const { root } = useFixture({ fixture: "install-source-manager" });
    const arranged = arrange({
      root,
      sourcePackageManager: "pnpm",
      availablePackageManagers: {
        npm: "10.9.0",
        pnpm: "9.15.4",
        yarn: "1.22.22",
        bun: "1.2.3",
        nub: "0.1.0",
        aube: "0.1.0"
      }
    });

    try {
      await create(root as CreateCommandArgument, {
        packageManager: "npm",
        skipTransforms: true,
        skipInstall: false,
        example: "default",
        git: false,
        telemetry: undefined
      });

      expect(arranged.install).toHaveBeenCalledTimes(1);
      expect(arranged.install).toHaveBeenCalledWith({
        project: arranged.project,
        to: {
          name: "pnpm",
          version: "9.15.4"
        },
        options: {
          interactive: false
        }
      });
    } finally {
      arranged.restore();
    }
  });

  it("warns and does not install when a skipped-transform source manager is unavailable", async () => {
    const { root } = useFixture({ fixture: "unavailable-source-manager" });
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
        example: "community-example",
        git: false,
        telemetry: undefined
      });

      expect(arranged.install).not.toHaveBeenCalled();
      expect(errorSpy).toHaveBeenNthCalledWith(
        1,
        expect.any(String),
        'Unable to install dependencies - "community-example" uses "aube" which could not be found.'
      );
      expect(errorSpy).toHaveBeenNthCalledWith(
        2,
        expect.any(String),
        'Try running without "--skip-transforms" to convert "community-example" to a package manager that is available on your system.'
      );
    } finally {
      arranged.restore();
    }
  });

  it("does not install when package.json is absent or installation is skipped", async () => {
    for (const [fixture, hasPackageJson, skipInstall] of [
      ["missing-package-json", false, false],
      ["explicit-skip-install", true, true]
    ] as const) {
      const { root } = useFixture({ fixture });
      const arranged = arrange({
        root,
        sourcePackageManager: "npm",
        availablePackageManagers: {
          npm: "10.9.0",
          pnpm: "9.15.4",
          yarn: "1.22.22",
          bun: "1.2.3",
          nub: "0.1.0",
          aube: "0.1.0"
        },
        hasPackageJson
      });

      try {
        await create(root as CreateCommandArgument, {
          skipTransforms: true,
          skipInstall,
          example: "default",
          git: false,
          telemetry: undefined
        });

        expect(arranged.install).not.toHaveBeenCalled();
      } finally {
        arranged.restore();
      }
    }
  });

  it.failing(
    "does not pass raw terminal-control text from the example name to warning output",
    async () => {
      const { root } = useFixture({ fixture: "hostile-warning-name" });
      const hostileExample =
        "community\u001B]8;;https://attacker.invalid\u0007name\nspoof";
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

        const rendered = errorSpy.mock.calls.flat().map(String).join(" ");
        expect(rendered).not.toContain(hostileExample);
      } finally {
        arranged.restore();
      }
    }
  );
});
