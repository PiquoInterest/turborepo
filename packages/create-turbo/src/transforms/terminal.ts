export const MAX_TERMINAL_DIAGNOSTIC_SCALARS = 512;

/**
 * Security seam for text that will be written to an interactive terminal.
 *
 * The RED implementation deliberately preserves the current raw behavior so
 * the regression tests prove the vulnerability before the hardening change.
 */
export function sanitizeTerminalText(input: string): string {
  return input;
}
