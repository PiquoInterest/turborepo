import { sanitizeTerminalText } from "./terminal";

export interface TransformErrorOptions {
  transform?: string;
  fatal?: boolean;
}

export class TransformError extends Error {
  public transform: string;
  public fatal: boolean;

  constructor(message: string, opts?: TransformErrorOptions) {
    super(message);
    this.name = "TransformError";
    this.transform = opts?.transform ?? "unknown";
    this.fatal = opts?.fatal ?? true;
    Error.captureStackTrace(this, TransformError);
  }

  public get terminalMessage(): string {
    return sanitizeTerminalText(this.message);
  }

  public get terminalTransform(): string {
    return sanitizeTerminalText(this.transform);
  }
}
