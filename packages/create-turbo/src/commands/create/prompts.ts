import { Buffer } from "node:buffer";
import type { PackageManager } from "@turbo/utils";
import { getAvailablePackageManagers, validateDirectory } from "@turbo/utils";
import { input, select } from "@inquirer/prompts";
import type { CreateCommandArgument } from "./types";

export const DIRECTORY_PROMPT_MESSAGE =
  "Where would you like to create your Turborepo?";
export const DEFAULT_PROJECT_DIRECTORY = "./my-turborepo";
export const MAX_DIRECTORY_INPUT_BYTES = 4096;
export const INVALID_PROJECT_DIRECTORY_MESSAGE =
  "The project directory is invalid or contains conflicting files.";

const UNSAFE_DIRECTORY_CONTROL_PATTERN =
  /[\u0000-\u001f\u007f-\u009f\u00ad\u034f\u061c\u180e\u200b-\u200f\u2028-\u202e\u2060-\u206f\ufeff\ufff9-\ufffb]/u;

export type InvalidDirectoryReason = "unsafe-input" | "validation";

export class InvalidDirectoryError extends Error {
  public readonly reason: InvalidDirectoryReason;

  constructor(message: string, reason: InvalidDirectoryReason) {
    super(message);
    this.name = "InvalidDirectoryError";
    this.reason = reason;
    Error.captureStackTrace(this, InvalidDirectoryError);
  }
}

function hasUnpairedSurrogate(value: string): boolean {
  for (let index = 0; index < value.length; index += 1) {
    const codeUnit = value.charCodeAt(index);
    if (codeUnit >= 0xd800 && codeUnit <= 0xdbff) {
      const nextCodeUnit = value.charCodeAt(index + 1);
      if (!(nextCodeUnit >= 0xdc00 && nextCodeUnit <= 0xdfff)) {
        return true;
      }
      index += 1;
    } else if (codeUnit >= 0xdc00 && codeUnit <= 0xdfff) {
      return true;
    }
  }
  return false;
}

function unsafeDirectoryInputError(directory: string): string | undefined {
  if (hasUnpairedSurrogate(directory)) {
    return "Project directory input contains invalid Unicode text.";
  }
  if (Buffer.byteLength(directory, "utf8") > MAX_DIRECTORY_INPUT_BYTES) {
    return `Project directory input exceeds the ${MAX_DIRECTORY_INPUT_BYTES}-byte limit.`;
  }
  if (UNSAFE_DIRECTORY_CONTROL_PATTERN.test(directory)) {
    return "Project directory input contains unsafe control characters.";
  }
  return undefined;
}

function validateDirectoryOrThrow(directory: string) {
  const inputError = unsafeDirectoryInputError(directory);
  if (inputError) {
    throw new InvalidDirectoryError(inputError, "unsafe-input");
  }

  const result = validateDirectory(directory);
  if (!result.valid) {
    throw new InvalidDirectoryError(
      INVALID_PROJECT_DIRECTORY_MESSAGE,
      "validation"
    );
  }
  return result;
}

export async function directory({ dir }: { dir: CreateCommandArgument }) {
  if (dir) {
    return validateDirectoryOrThrow(dir);
  }

  const projectDirectory = await input({
    message: DIRECTORY_PROMPT_MESSAGE,
    default: DEFAULT_PROJECT_DIRECTORY,
    validate: (d: string) => {
      const inputError = unsafeDirectoryInputError(d);
      if (inputError) {
        return inputError;
      }
      return validateDirectory(d).valid
        ? true
        : INVALID_PROJECT_DIRECTORY_MESSAGE;
    },
    transformer: (d: string) => d.trim()
  });

  // Inquirer's transformer changes the display only. Preserve and validate the
  // exact accepted answer rather than silently rewriting the requested path.
  return validateDirectoryOrThrow(projectDirectory);
}

export async function packageManager({
  manager,
  skipTransforms
}: {
  manager: CreateCommandArgument;
  skipTransforms?: boolean;
}) {
  // if skip transforms is passed, we don't need to ask about the package manager (because that requires a transform)
  if (skipTransforms) {
    return undefined;
  }

  const availablePackageManagers = await getAvailablePackageManagers();

  if (manager && availablePackageManagers[manager as PackageManager]) {
    return {
      name: manager as PackageManager,
      version: availablePackageManagers[manager as PackageManager]
    };
  }

  const selectedPackageManager = await select<PackageManager>({
    message: "Which package manager do you want to use?",
    choices: [
      { pm: "npm", label: "npm" },
      { pm: "pnpm", label: "pnpm" },
      { pm: "yarn", label: "yarn" },
      { pm: "bun", label: "bun" },
      { pm: "nub", label: "nub" },
      { pm: "aube", label: "aube" }
    ]
      .sort((a, b) => {
        const aInstalled = Boolean(
          availablePackageManagers[a.pm as PackageManager]
        );
        const bInstalled = Boolean(
          availablePackageManagers[b.pm as PackageManager]
        );
        return Number(bInstalled) - Number(aInstalled);
      })
      .map(({ pm, label }) => ({
        name: label,
        value: pm as PackageManager,
        disabled: availablePackageManagers[pm as PackageManager]
          ? false
          : `not installed`
      }))
  });

  return {
    name: selectedPackageManager,
    version: availablePackageManagers[selectedPackageManager]
  };
}
