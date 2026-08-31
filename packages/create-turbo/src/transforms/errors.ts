import { sanitizeTerminalText } from "./terminal";

export interface TransformErrorOptions {
  transform?: string;
  fatal?: boolean;
}

export class TransformError extends Error {
  public transform: string;
  public fatal: boolean;
  public readonly rawMessage: string;
  public readonly rawTransform: string;

  constructor(message: string, opts?: TransformErrorOptions) {
    const rawTransform = opts?.transform ?? "unknown";
    super(sanitizeTerminalText(message));
    this.name = "TransformError";
    this.rawMessage = message;
    this.rawTransform = rawTransform;
    this.transform = sanitizeTerminalText(rawTransform);
    this.fatal = opts?.fatal ?? true;
    Error.captureStackTrace(this, TransformError);
  }
}
