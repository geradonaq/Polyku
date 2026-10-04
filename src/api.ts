// Typed wrappers around the Tauri IPC commands (src-tauri/src/commands.rs).

import { invoke } from "@tauri-apps/api/core";
import type { Difficulty, HintDto, PuzzleDto, RuleId, SaveGame, Stats } from "./types";

export const api = {
  engineVersion: (): Promise<string> => invoke("engine_version"),

  newGame: (difficulty: Difficulty, rules: RuleId[]): Promise<PuzzleDto> =>
    invoke("new_game", { difficulty, rules }),

  validateBoard: (cells: number[]): Promise<number[]> =>
    invoke("validate_board", { cells }),

  getHint: (cells: number[]): Promise<HintDto> => invoke("get_hint", { cells }),

  fillCandidates: (cells: number[]): Promise<number[][]> =>
    invoke("fill_candidates", { cells }),

  saveGame: (cells: number[], notes: number[][], marks: number[], elapsedSecs: number): Promise<void> =>
    invoke("save_game", { cells, notes, marks, elapsedSecs }),

  cleanEntries: (cells: number[], targets: number[]): Promise<number[]> =>
    invoke("clean_entries", { cells, targets }),

  loadGame: (): Promise<SaveGame | null> => invoke("load_game"),

  recordResult: (
    difficulty: string,
    rules: string[],
    completed: boolean,
    seconds: number,
  ): Promise<void> =>
    invoke("record_result", { difficulty, rules, completed, seconds }),

  getStatistics: (): Promise<Stats> => invoke("get_statistics"),
};
