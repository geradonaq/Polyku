// The 9-color marking palette (cell marks and digit highlights share it).

export const MARK_COLORS = [
  "#10b981", // emerald
  "#0ea5e9", // sky
  "#8b5cf6", // violet
  "#f59e0b", // amber
  "#f43f5e", // rose
  "#84cc16", // lime
  "#f97316", // orange
  "#06b6d4", // cyan
  "#ec4899", // pink
] as const;

/// Translucent background for a marked cell.
export function markBg(colorIndex: number): string | undefined {
  if (colorIndex < 1 || colorIndex > MARK_COLORS.length) return undefined;
  return `${MARK_COLORS[colorIndex - 1]}2e`; // ~18% alpha
}

export function markText(colorIndex: number): string | undefined {
  if (colorIndex < 1 || colorIndex > MARK_COLORS.length) return undefined;
  return MARK_COLORS[colorIndex - 1];
}
