// Pure board logic — no React, no IPC. Everything here is unit-testable
// (see src/game/logic.test.ts) and used by the useGame hook.

export type Board = number[];
export type Notes = number[][];

export interface BoardState {
  cells: Board;
  notes: Notes;
}

export function emptyBoard(): Board {
  return Array(81).fill(0);
}

export function emptyNotes(): Notes {
  return Array.from({ length: 81 }, () => []);
}

const cloneNotes = (notes: Notes): Notes => notes.map((n) => [...n]);

/// Places a digit (or toggles a pencil note) on every free target cell.
/// Returns null when nothing would change (all targets given, etc.).
export function applyDigitLogic(
  cells: Board,
  notes: Notes,
  targets: number[],
  digit: number,
  asNote: boolean,
  givens: Board,
): BoardState | null {
  const free = targets.filter((i) => givens[i] === 0);
  if (free.length === 0) return null;
  const nextCells = [...cells];
  const nextNotes = cloneNotes(notes);
  let changed = false;
  for (const i of free) {
    if (asNote) {
      if (nextCells[i] !== 0) continue; // notes only on empty cells
      const list = nextNotes[i];
      nextNotes[i] = list.includes(digit)
        ? list.filter((d) => d !== digit)
        : [...list, digit].sort((a, b) => a - b);
      changed = true;
    } else {
      if (nextCells[i] === digit && nextNotes[i].length === 0) continue;
      nextCells[i] = digit;
      nextNotes[i] = [];
      changed = true;
    }
  }
  return changed ? { cells: nextCells, notes: nextNotes } : null;
}

/// Clears values and notes on the target cells.
export function eraseLogic(
  cells: Board,
  notes: Notes,
  targets: number[],
  givens: Board,
): BoardState | null {
  const free = targets.filter((i) => givens[i] === 0);
  if (free.length === 0) return null;
  const nextCells = [...cells];
  const nextNotes = cloneNotes(notes);
  let changed = false;
  for (const i of free) {
    if (nextCells[i] !== 0 || nextNotes[i].length > 0) changed = true;
    nextCells[i] = 0;
    nextNotes[i] = [];
  }
  return changed ? { cells: nextCells, notes: nextNotes } : null;
}

/// Overwrites notes on empty cells (the Fill-Candidates dialog).
export function writeNotesLogic(
  cells: Board,
  notes: Notes,
  perCell: Notes,
  givens: Board,
): BoardState | null {
  const nextCells = [...cells];
  const nextNotes = cloneNotes(notes);
  let changed = false;
  for (let i = 0; i < 81; i++) {
    if (cells[i] !== 0 || givens[i] !== 0) continue;
    const sorted = [...perCell[i]].sort((a, b) => a - b);
    if (JSON.stringify(sorted) !== JSON.stringify(nextNotes[i])) changed = true;
    nextNotes[i] = sorted;
  }
  return changed ? { cells: nextCells, notes: nextNotes } : null;
}

/// Cell marks (the 9-color palette): 0 = unmarked, 1–9 = palette color.
/// Toggling: if every target already carries the color, clear them.
export function markLogic(
  marks: number[],
  targets: number[],
  color: number,
  givens: Board,
): number[] | null {
  const free = targets.filter((i) => givens[i] === 0);
  if (free.length === 0) return null;
  const allSame = free.every((i) => marks[i] === color);
  const next = [...marks];
  for (const i of free) {
    next[i] = allSame ? 0 : color;
  }
  return next;
}

/// Classic-rules candidates for one cell (row/col/box deduction only) —
/// the "Classic rules" mode of the Fill-Candidates dialog.
export function classicCandidates(cells: Board, cell: number): number[] {
  const used = new Set<number>();
  const r = Math.floor(cell / 9);
  const c = cell % 9;
  for (let j = 0; j < 81; j++) {
    if (cells[j] === 0) continue;
    const rj = Math.floor(j / 9);
    const cj = j % 9;
    if (
      rj === r ||
      cj === c ||
      (Math.floor(rj / 3) === Math.floor(r / 3) && Math.floor(cj / 3) === Math.floor(c / 3))
    ) {
      used.add(cells[j]);
    }
  }
  return [1, 2, 3, 4, 5, 6, 7, 8, 9].filter((d) => !used.has(d));
}

/// A full board with zero rule conflicts under a uniqueness-proven puzzle
/// IS the solution — the win condition, verified host-side.
export function isBoardFull(cells: Board): boolean {
  return cells.every((d) => d !== 0);
}
