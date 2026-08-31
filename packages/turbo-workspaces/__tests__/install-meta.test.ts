import { describe, expect, it } from "@jest/globals";
import type { PackageManager } from "../src/types";
import { getPackageManagerMeta } from "../src/install";

describe("package-manager install semver oracle", () => {
  it.each<{
    name: PackageManager;
    version: string;
    expectedProfile: string | undefined;
  }>([
    { name: "npm", version: "10.9.0", expectedProfile: "npm" },
    { name: "pnpm", version: "6.35.1", expectedProfile: "pnpm6" },
    { name: "pnpm", version: "7.0.0", expectedProfile: "pnpm" },
    { name: "yarn", version: "1.22.22", expectedProfile: "yarn" },
    { name: "yarn", version: "2.0.0", expectedProfile: "berry" },
    { name: "bun", version: "1.0.0", expectedProfile: undefined },
    { name: "bun", version: "1.0.1", expectedProfile: "bun" },
    { name: "bun", version: "1.99.0", expectedProfile: "bun" },
    { name: "bun", version: "2.0.0", expectedProfile: undefined },
    { name: "nub", version: "0.1.0", expectedProfile: "nub" },
    { name: "aube", version: "0.1.0", expectedProfile: "aube" }
  ])(
    "selects $expectedProfile for $name@$version",
    ({ name, version, expectedProfile }) => {
      expect(getPackageManagerMeta({ name, version })?.name).toBe(
        expectedProfile
      );
    }
  );

  it("preserves Node-semver build and prerelease behavior", () => {
    expect(
      getPackageManagerMeta({ name: "pnpm", version: "7.0.0+build.5" })?.name
    ).toBe("pnpm");
    expect(
      getPackageManagerMeta({ name: "pnpm", version: "7.0.0-rc.1" })
    ).toBeUndefined();
    expect(
      getPackageManagerMeta({ name: "bun", version: "1.0.1-beta.1" })
    ).toBeUndefined();
  });

  it.each([
    "not-a-version",
    "1.2",
    "1.2.3.4",
    "999999999999999999999999999999.0.0",
    " 1.2.3",
    "1.2.3 ",
    "1.2.3\n",
    "１.２.３",
    "1.2.3\u202e",
    "1.2.3\u0000"
  ])("treats malformed version %j as unsupported", (version) => {
    expect(
      getPackageManagerMeta({
        name: "npm",
        version
      })
    ).toBeUndefined();
  });
});
