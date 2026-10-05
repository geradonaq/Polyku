// The play-session state: board, notes, selection, undo/redo, timer,
// conflicts, hints, autosave. The frontend owns the play state; the host
// owns the puzzle (solution) and the services.

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { Difficulty, HintDto, PuzzleDto, RuleId } from "../types";
import {
  applyDigitLogic, emptyBoard, emptyNotes, eraseLogic, markLogic, writeNotesLogic,
} from "./logic";

const EMPTY_BOARD = emptyBoard;
const EMPTY_NOTES = emptyNotes;

interface Snapshot {
  cells: number[];
  notes: number[][];
  marks: number[];
}

function ruleIdsOf(puzzle: PuzzleDto): string[] {
  return puzzle.rules.map((r) =>
    r === "Diagonal"
      ? "diagonal"
      : r === "NonConsecutive"
        ? "non_consecutive"
        : r === "AntiKnight"
          ? "anti_knight"
          : "Killer" in r
            ? "killer"
            : "thermo",
  );
}

export function useGame() {
  const [puzzle, setPuzzle] = useState<PuzzleDto | null>(null);
  const [cells, setCells] = useState<number[]>(EMPTY_BOARD);
  const [notes, setNotes] = useState<number[][]>(EMPTY_NOTES);
  const [marks, setMarks] = useState<number[]>(EMPTY_BOARD);
  const [selected, setSelected] = useState<number[]>([]);
  const [pencil, setPencil] = useState(false);
  const [heldDigit, setHeldDigit] = useState<number | null>(null);
  const [conflicts, setConflicts] = useState<number[]>([]);
  const [hint, setHint] = useState<HintDto | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [solved, setSolved] = useState(false);
  const [busy, setBusy] = useState(false);
  const [digitHl, setDigitHl] = useState<Set<number>>(new Set());
  const [notesView, setNotesView] = useState<"grid" | "badges">("grid");

  // Refs mirror state for the autosave/close hooks and stable callbacks.
  const cellsRef = useRef(cells);
  const notesRef = useRef(notes);
  const marksRef = useRef(marks);
  const elapsedRef = useRef(elapsed);
  const puzzleRef = useRef(puzzle);
  const solvedRef = useRef(solved);
  cellsRef.current = cells;
  notesRef.current = notes;
  marksRef.current = marks;
  elapsedRef.current = elapsed;
  puzzleRef.current = puzzle;
  solvedRef.current = solved;

  const givens = puzzle?.givens ?? EMPTY_BOARD();

  // --- undo / redo -------------------------------------------------------

  const undoStack = useRef<Snapshot[]>([]);
  const redoStack = useRef<Snapshot[]>([]);
  const [stackSizes, setStackSizes] = useState({ undo: 0, redo: 0 });
  const syncStackSizes = () =>
    setStackSizes({ undo: undoStack.current.length, redo: redoStack.current.length });

  const pushUndo = useCallback((cells: number[], notes: number[][], marks: number[]) => {
    undoStack.current.push({
      cells: [...cells],
      notes: notes.map((n) => [...n]),
      marks: [...marks],
    });
    if (undoStack.current.length > 300) undoStack.current.shift();
    redoStack.current = [];
    syncStackSizes();
  }, []);

  const undo = useCallback(() => {
    const prev = undoStack.current.pop();
    if (!prev) return;
    redoStack.current.push({
      cells: [...cellsRef.current],
      notes: notesRef.current.map((n) => [...n]),
      marks: [...marksRef.current],
    });
    setCells(prev.cells);
    setNotes(prev.notes);
    setMarks(prev.marks);
    syncStackSizes();
  }, []);

  const redo = useCallback(() => {
    const next = redoStack.current.pop();
    if (!next) return;
    undoStack.current.push({
      cells: [...cellsRef.current],
      notes: notesRef.current.map((n) => [...n]),
      marks: [...marksRef.current],
    });
    setCells(next.cells);
    setNotes(next.notes);
    setMarks(next.marks);
    syncStackSizes();
  }, []);

  // --- mutations ---------------------------------------------------------

  const mutate = useCallback(
    (fn: (cells: number[], notes: number[][]) => { cells: number[]; notes: number[][] } | null) => {
      if (!puzzleRef.current || solvedRef.current) return;
      const result = fn(cellsRef.current, notesRef.current);
      if (!result) return;
      pushUndo(cellsRef.current, notesRef.current, marksRef.current);
      setCells(result.cells);
      setNotes(result.notes);
      setHint(null);
    },
    [pushUndo],
  );

  /// Places a digit (or toggles a note in pencil mode) on the given cells.
  /// After a *correct* placement the host prunes wrong entries everywhere
  /// (the document's assist rule); a wrong placement stays for the red
  /// conflicts to tell the story.
  const applyDigit = useCallback(
    async (targets: number[], digit: number, asNote: boolean) => {
      if (!puzzleRef.current || solvedRef.current) return;
      const result = applyDigitLogic(cellsRef.current, notesRef.current, targets, digit, asNote, givens);
      if (!result) return;
      pushUndo(cellsRef.current, notesRef.current, marksRef.current);
      setCells(result.cells);
      setNotes(result.notes);
      setHint(null);
      if (!asNote) {
        try {
          const cleaned = await api.cleanEntries(result.cells, targets);
          if (cleaned.some((d, i) => d !== result.cells[i])) {
            setCells((current) =>
              current.every((d, i) => d === result.cells[i]) ? cleaned : current,
            );
          }
        } catch {
          // host unavailable — the placement itself already applied
        }
      }
    },
    [givens, pushUndo],
  );

  const erase = useCallback(
    (targets: number[]) => {
      mutate((cells, notes) => eraseLogic(cells, notes, targets, givens));
    },
    [givens, mutate],
  );

  /// The Fill-Candidates dialog: writes candidate notes directly.
  const writeNotes = useCallback(
    (perCell: number[][]) => {
      mutate((cells, notes) => writeNotesLogic(cells, notes, perCell, givens));
    },
    [givens, mutate],
  );

  /// Cell marking (9-color palette). Toggling semantics live in markLogic.
  const markCells = useCallback(
    (targets: number[], color: number) => {
      if (!puzzleRef.current || solvedRef.current) return;
      const next = markLogic(marksRef.current, targets, color, givens);
      if (!next) return;
      pushUndo(cellsRef.current, notesRef.current, marksRef.current);
      setMarks(next);
    },
    [givens, pushUndo],
  );

  const toggleDigitHl = useCallback((digit: number) => {
    setDigitHl((cur) => {
      const next = new Set(cur);
      if (next.has(digit)) {
        next.delete(digit);
      } else {
        next.add(digit);
      }
      return next;
    });
  }, []);

  // --- new game / restore ------------------------------------------------

  const loadPuzzle = useCallback(
    (p: PuzzleDto, cells: number[], notes: number[][], marks: number[], elapsed: number) => {
      undoStack.current = [];
      redoStack.current = [];
      syncStackSizes();
      setPuzzle(p);
      setCells(cells);
      setNotes(notes);
      setMarks(marks);
      setElapsed(elapsed);
      setSelected([]);
      setConflicts([]);
      setHint(null);
      setSolved(false);
      setHeldDigit(null);
    },
    [],
  );

  const newGame = useCallback(
    async (difficulty: Difficulty, rules: RuleId[]) => {
      setBusy(true);
      try {
        const p = await api.newGame(difficulty, rules);
        loadPuzzle(p, [...p.givens], EMPTY_NOTES(), EMPTY_BOARD(), 0);
        return true;
      } catch (e) {
        console.error("generation failed", e);
        return false;
      } finally {
        setBusy(false);
      }
    },
    [loadPuzzle],
  );

  // --- conflicts & win detection -----------------------------------------
  // The host's validate_board is authoritative: it checks classic rules and
  // every active variant. A full board with zero conflicts under a
  // uniqueness-proven puzzle IS the solution — that's the win condition.

  useEffect(() => {
    if (!puzzle || solved) return;
    let cancelled = false;
    api
      .validateBoard(cells)
      .then((bad) => {
        if (cancelled) return;
        setConflicts(bad);
        if (cells.every((d) => d !== 0) && bad.length === 0) {
          setSolved(true);
          api
            .recordResult(puzzle.difficulty, ruleIdsOf(puzzle), true, elapsedRef.current)
            .catch(() => {});
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [cells, puzzle, solved]);

  // --- timer -------------------------------------------------------------

  useEffect(() => {
    if (!puzzle || solved || busy) return;
    const id = setInterval(() => setElapsed((s) => s + 1), 1000);
    return () => clearInterval(id);
  }, [puzzle, solved, busy]);

  // --- autosave (debounced) & save on close -------------------------------

  const saveNow = useCallback(async () => {
    if (!puzzleRef.current || solvedRef.current) return;
    try {
      await api.saveGame(cellsRef.current, notesRef.current, marksRef.current, elapsedRef.current);
    } catch {
      // offline app — a failed autosave is not fatal
    }
  }, []);

  useEffect(() => {
    const id = setTimeout(saveNow, 1500);
    return () => clearTimeout(id);
  }, [cells, notes, marks, elapsed, saveNow]);

  useEffect(() => {
    const unlisten = getCurrentWindow().onCloseRequested(async () => {
      await saveNow();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [saveNow]);

  // --- startup restore ----------------------------------------------------

  useEffect(() => {
    (async () => {
      try {
        const save = await api.loadGame();
        if (!save) return;
        const p: PuzzleDto = {
          givens: save.givens,
          rules: save.rules,
          difficulty: save.difficulty as PuzzleDto["difficulty"],
          grade: save.grade,
          clue_count: save.givens.filter((d) => d !== 0).length,
          overlays: [],
        };
        loadPuzzle(
          p,
          save.cells,
          save.notes,
          save.marks.length === 81 ? save.marks : EMPTY_BOARD(),
          save.elapsed_secs,
        );
      } catch {
        // no save or corrupt save — start fresh
      }
    })();
  }, [loadPuzzle]);

  /// Clears all player entries and notes, keeping the same puzzle. The
  /// escape hatch when the board gets into a hopeless state.
  const restart = useCallback(() => {
    const p = puzzleRef.current;
    if (!p) return;
    pushUndo(cellsRef.current, notesRef.current, marksRef.current);
    setCells([...p.givens]);
    setNotes(EMPTY_NOTES());
    setMarks(EMPTY_BOARD());
    setSelected([]);
    setHint(null);
    setSolved(false);
  }, [pushUndo]);

  // --- hints --------------------------------------------------------------

  const fetchHint = useCallback(async () => {
    if (!puzzleRef.current || solvedRef.current) return;
    try {
      setHint(await api.getHint(cellsRef.current));
    } catch {
      setHint(null);
    }
  }, []);

  const applyHint = useCallback(() => {
    const h = hint;
    if (!h || h.kind !== "technique") return;
    mutate((cells, notes) => {
      const nextCells = [...cells];
      const nextNotes = notes.map((n) => [...n]);
      for (const [i, d] of h.placements) {
        nextCells[i] = d;
        nextNotes[i] = [];
      }
      for (const [i, d] of h.eliminations) {
        nextNotes[i] = nextNotes[i].filter((n) => n !== d);
      }
      return { cells: nextCells, notes: nextNotes };
    });
    setHint(null);
  }, [hint, mutate]);

  const dismissHint = useCallback(() => setHint(null), []);

  return {
    // state
    puzzle, cells, notes, marks, selected, pencil, heldDigit, conflicts, hint,
    elapsed, solved, busy, stackSizes, givens, digitHl, notesView,
    // selection
    setSelected, setNotesView,
    // actions
    applyDigit, erase, undo, redo, setPencil, setHeldDigit,
    newGame, fetchHint, applyHint, dismissHint, writeNotes, restart,
    markCells, toggleDigitHl,
  };
}

export type Game = ReturnType<typeof useGame>;
