export const MAX_TERMINAL_DIAGNOSTIC_SCALARS = 512;

const TERMINAL_TRUNCATION_MARKER = "…";

function isUnsafeTerminalCodePoint(codePoint: number): boolean {
  return (
    codePoint <= 0x1f ||
    (codePoint >= 0x7f && codePoint <= 0x9f) ||
    codePoint === 0x061c ||
    codePoint === 0x070f ||
    codePoint === 0x180e ||
    (codePoint >= 0x200b && codePoint <= 0x200f) ||
    (codePoint >= 0x2028 && codePoint <= 0x202e) ||
    (codePoint >= 0x2060 && codePoint <= 0x206f) ||
    codePoint === 0xfeff ||
    (codePoint >= 0xfff9 && codePoint <= 0xfffb)
  );
}

function escapeTerminalCodePoint(codePoint: number): string {
  switch (codePoint) {
    case 0:
      return "\\0";
    case 9:
      return "\\t";
    case 10:
      return "\\n";
    case 13:
      return "\\r";
    default:
      return `\\u{${codePoint.toString(16)}}`;
  }
}

/**
 * Produces a single-line, bounded terminal field while retaining the raw value
 * separately on the owning error object.
 *
 * Iteration stops after at most one scalar beyond the limit, so a very large
 * attacker-controlled suffix is neither copied nor fully scanned.
 */
export function sanitizeTerminalText(input: string): string {
  let output = "";
  let processedScalars = 0;

  for (const character of input) {
    if (processedScalars === MAX_TERMINAL_DIAGNOSTIC_SCALARS) {
      output += TERMINAL_TRUNCATION_MARKER;
      break;
    }

    processedScalars += 1;
    const codePoint = character.codePointAt(0);
    if (codePoint === undefined) {
      continue;
    }

    output += isUnsafeTerminalCodePoint(codePoint)
      ? escapeTerminalCodePoint(codePoint)
      : character;
  }

  return output;
}
