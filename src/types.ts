// Mirror of the Rust DTOs (src-tauri/src/dto.rs). Field names arrive in
// snake_case from serde, so they are kept here verbatim.

export interface Coord {
  row: number;
  col: number;
}

export interface Cage {
  cells: Coord[];
  sum: number;
}

export type ThermoPath = Coord[];

/// Serde externally-tagged enums: unit variants arrive as plain strings,
/// struct variants as single-key objects.
export type RuleData =
  | "Diagonal"
  | "NonConsecutive"
  | "AntiKnight"
  | { Killer: { cages: Cage[] } }
  | { Thermo: { paths: ThermoPath[] } };

export type Overlay =
  | { Cage: { cells: Coord[]; sum: OptionU16 } }
  | { Path: { cells: Coord[] } }
  | { DiagonalStripe: { main: boolean } };

/// Rust Option<u16> arrives as null or number.
export type OptionU16 = number | null;

export interface PuzzleDto {
  givens: number[];
  rules: RuleData[];
  difficulty: string;
  grade: number;
  clue_count: number;
  overlays: Overlay[];
}

export interface SaveGame {
  givens: number[];
  rules: RuleData[];
  difficulty: string;
  grade: number;
  cells: number[];
  notes: number[][];
  marks: number[];
  elapsed_secs: number;
  saved_at: number;
}

export interface GameRecord {
  difficulty: string;
  rules: string[];
  completed: boolean;
  seconds: number;
  finished_at: number;
}

export interface Stats {
  games: GameRecord[];
}

export interface HintDto {
  kind: "technique" | "error" | "done" | "stuck";
  technique: string;
  tier: number;
  placements: [number, number][];
  eliminations: [number, number][];
  highlight_cells: number[];
  highlight_digits: number[];
  explanation: string;
}

/// Rule ids the frontend sends back to the host.
export const RULE_IDS = ["diagonal", "killer", "thermo", "non_consecutive", "anti_knight"] as const;
export type RuleId = (typeof RULE_IDS)[number];

export const RULE_LABELS: Record<RuleId, string> = {
  diagonal: "Diagonal (X)",
  killer: "Killer",
  thermo: "Thermo",
  non_consecutive: "Non-Consecutive",
  anti_knight: "Anti-Knight",
};

export const DIFFICULTIES = [
  "beginner",
  "easy",
  "medium",
  "hard",
  "expert",
  "master",
] as const;
export type Difficulty = (typeof DIFFICULTIES)[number];

/// Human label for a graded tier (1–6).
export function tierName(grade: number): string {
  const names = ["Beginner", "Easy", "Medium", "Hard", "Expert", "Master"];
  return names[grade - 1] ?? `Tier ${grade}`;
}

/// Short badge names for the active rules.
export function ruleShortName(rule: RuleData): string {
  if (rule === "Diagonal") return "X";
  if (rule === "NonConsecutive") return "NC";
  if (rule === "AntiKnight") return "Anti-N";
  if ("Killer" in rule) return "Killer";
  return "Thermo";
}
