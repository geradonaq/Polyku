// Unit tests for the pure board logic.

import { describe, expect, it } from "vitest";
import {
  applyDigitLogic, classicCandidates, emptyBoard, emptyNotes, eraseLogic,
  markLogic, writeNotesLogic,
} from "./logic";

const GIVENS = (() => {
  const g = emptyBoard();
  g[0] = 5; // r1c1 is a given
  return g;
})();

describe("applyDigitLogic", () => {
  it("places a digit and clears notes", () => {
    const cells = emptyBoard();
    const notes = emptyNotes();
    notes[10] = [1, 2, 3];
    const result = applyDigitLogic(cells, notes, [10], 7, false, GIVENS);
    expect(result).not.toBeNull();
    expect(result!.cells[10]).toBe(7);
    expect(result!.notes[10]).toEqual([]);
  });

  it("toggles a note in pencil mode", () => {
    const cells = emptyBoard();
    const notes = emptyNotes();
    const added = applyDigitLogic(cells, notes, [10], 3, true, GIVENS);
    expect(added!.notes[10]).toEqual([3]);
    const removed = applyDigitLogic(cells, added!.notes, [10], 3, true, GIVENS);
    expect(removed!.notes[10]).toEqual([]);
  });

  it("keeps notes sorted", () => {
    const result = applyDigitLogic(emptyBoard(), emptyNotes(), [10], 9, true, GIVENS);
    const again = applyDigitLogic(emptyBoard(), result!.notes, [10], 2, true, GIVENS);
    expect(again!.notes[10]).toEqual([2, 9]);
  });

  it("never writes into given cells", () => {
    const result = applyDigitLogic(emptyBoard(), emptyNotes(), [0], 7, false, GIVENS);
    expect(result).toBeNull();
  });

  it("is a no-op when nothing changes", () => {
    const cells = emptyBoard();
    cells[10] = 7;
    const result = applyDigitLogic(cells, emptyNotes(), [10], 7, false, GIVENS);
    expect(result).toBeNull();
  });
});

describe("eraseLogic", () => {
  it("clears values and notes but not givens", () => {
    const cells = emptyBoard();
    cells[0] = 5; // the given, present on the player board too
    cells[10] = 7;
    const notes = emptyNotes();
    notes[11] = [4];
    const result = eraseLogic(cells, notes, [0, 10, 11], GIVENS);
    expect(result!.cells[10]).toBe(0);
    expect(result!.cells[0]).toBe(5); // given untouched
    expect(result!.notes[11]).toEqual([]);
  });

  it("is a no-op when targets are empty", () => {
    const result = eraseLogic(emptyBoard(), emptyNotes(), [10], GIVENS);
    expect(result).toBeNull();
  });
});

describe("writeNotesLogic", () => {
  it("overwrites notes on empty cells only", () => {
    const cells = emptyBoard();
    cells[10] = 7;
    const notes = emptyNotes();
    notes[11] = [1, 2];
    const perCell = emptyNotes();
    perCell[11] = [8, 2, 5];
    perCell[10] = [1]; // filled cell — must be ignored
    const result = writeNotesLogic(cells, notes, perCell, GIVENS);
    expect(result!.notes[11]).toEqual([2, 5, 8]);
    expect(result!.notes[10]).toEqual([]);
  });
});

describe("markLogic", () => {
  it("sets and clears marks with toggle behavior", () => {
    const marks = Array(81).fill(0);
    const set = markLogic(marks, [10, 11], 3, GIVENS)!;
    expect(set[10]).toBe(3);
    expect(set[11]).toBe(3);
    const cleared = markLogic(set, [10, 11], 3, GIVENS)!;
    expect(cleared[10]).toBe(0);
    expect(cleared[11]).toBe(0);
  });

  it("mixed colors: re-marking unifies, clears only when uniform", () => {
    const marks = Array(81).fill(0);
    marks[10] = 3;
    const unified = markLogic(marks, [10, 11], 3, GIVENS)!;
    expect(unified[10]).toBe(3);
    expect(unified[11]).toBe(3);
  });

  it("is a no-op when clearing already unmarked cells", () => {
    const marks = Array(81).fill(0);
    const result = markLogic(marks, [10, 11], 0, GIVENS);
    expect(result).toBeNull();
  });
});

describe("classicCandidates", () => {
  it("excludes digits used in the row, column, and box", () => {
    const cells = emptyBoard();
    // Row 0: 1 at c0; Column 4: 2 at r0c4... build a known mini situation.
    cells[0] = 1; // r1c1
    cells[40] = 2; // r5c5 — same column as r1c5 (index 4)
    cells[80] = 3; // r9c9 — different row/col/box from cell 4 (r1c5)
    const result = classicCandidates(cells, 4); // r1c5
    expect(result).not.toContain(1); // same row
    expect(result).not.toContain(2); // same column
    expect(result).toContain(3); // unrelated
  });
});

describe("emptyBoard/emptyNotes", () => {
  it("fresh boards are 81 empty cells with empty note lists", () => {
    expect(emptyBoard()).toHaveLength(81);
    expect(emptyBoard().every((d) => d === 0)).toBe(true);
    expect(emptyNotes()).toHaveLength(81);
    expect(emptyNotes().every((n) => n.length === 0)).toBe(true);
  });
});
