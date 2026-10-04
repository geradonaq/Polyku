import { useCallback, useEffect, useRef, useState } from "react";
import { Board } from "./components/Board";
import { FillCandidatesModal } from "./components/FillCandidatesModal";
import { HintPanel } from "./components/HintPanel";
import { Keypad } from "./components/Keypad";
import { NewGameModal } from "./components/NewGameModal";
import { StatsModal, WinModal } from "./components/Modals";
import { TopBar } from "./components/TopBar";
import { useGame } from "./game/useGame";
import type { RuleId } from "./types";

const DIGIT_KEYS = new Set(["1", "2", "3", "4", "5", "6", "7", "8", "9"]);

export default function App() {
  const game = useGame();
  const [showNewGame, setShowNewGame] = useState(false);
  const [showFill, setShowFill] = useState(false);
  const [showStats, setShowStats] = useState(false);
  const [theme, setTheme] = useState<"dark" | "light">(
    () => (localStorage.getItem("polyku-theme") as "dark" | "light") ?? "dark",
  );
  const dragging = useRef(false);

  const { puzzle, cells, selected } = game;

  // First launch with no save → open the dialog.
  useEffect(() => {
    const id = setTimeout(() => {
      if (!puzzle) setShowNewGame(true);
    }, 400);
    return () => clearTimeout(id);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    localStorage.setItem("polyku-theme", theme);
  }, [theme]);

  // --- selection ----------------------------------------------------------

  const selectCell = useCallback(
    (i: number, additive: boolean) => {
      game.setSelected((cur) => {
        if (additive) {
          return cur.includes(i) ? cur.filter((x) => x !== i) : [...cur, i];
        }
        return [i];
      });
    },
    [game],
  );

  const onCellDown = useCallback(
    (i: number, additive: boolean) => {
      dragging.current = true;
      // Digit-first: a locked digit fills non-given cells on click.
      if (game.heldDigit !== null && !game.pencil && puzzle && puzzle.givens[i] === 0) {
        game.applyDigit([i], game.heldDigit, false);
        selectCell(i, false);
        return;
      }
      selectCell(i, additive);
    },
    [game, puzzle, selectCell],
  );

  const onCellEnter = useCallback(
    (i: number) => {
      if (!dragging.current || game.heldDigit !== null) return;
      game.setSelected((cur) => (cur.includes(i) ? cur : [...cur, i]));
    },
    [game],
  );

  useEffect(() => {
    const up = () => (dragging.current = false);
    window.addEventListener("mouseup", up);
    return () => window.removeEventListener("mouseup", up);
  }, []);

  // --- keyboard -----------------------------------------------------------

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (showNewGame || showFill || showStats) return;
      const primary = selected[selected.length - 1];
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        game.undo();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
        e.preventDefault();
        game.redo();
        return;
      }
      if (e.ctrlKey || e.metaKey) return;
      if (DIGIT_KEYS.has(e.key)) {
        e.preventDefault();
        if (primary === undefined) return;
        game.applyDigit(selected, Number(e.key), game.pencil || e.shiftKey);
        return;
      }
      if (e.key === "Backspace" || e.key === "Delete") {
        e.preventDefault();
        game.erase(selected);
        return;
      }
      if (e.key === " ") {
        e.preventDefault();
        game.setPencil((p) => !p);
        return;
      }
      if (e.key.toLowerCase() === "h") {
        game.fetchHint();
        return;
      }
      const moves: Record<string, number> = {
        ArrowUp: -9, ArrowDown: 9, ArrowLeft: -1, ArrowRight: 1,
        w: -9, s: 9, a: -1, d: 1, W: -9, S: 9, A: -1, D: 1,
      };
      const delta = moves[e.key];
      if (delta !== undefined && primary !== undefined) {
        e.preventDefault();
        const next = primary + delta;
        const row = Math.floor(primary / 9);
        const nextRow = Math.floor(next / 9);
        const wraps = Math.abs(delta) === 1 && nextRow !== row;
        if (next >= 0 && next < 81 && !wraps) {
          game.setSelected([next]);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [game, selected, showNewGame, showFill, showStats]);

  // --- derived ------------------------------------------------------------

  const digitCounts = Array.from({ length: 9 }, (_, k) =>
    cells.filter((d) => d === k + 1).length,
  );
  const variantNames = (puzzle?.rules ?? []).map((r) =>
    r === "Diagonal" ? "X" : r === "NonConsecutive" ? "NC" : "Killer" in r ? "Killer" : "Thermo",
  );

  return (
    <div className={theme === "dark" ? "dark" : ""}>
      <div className="flex h-screen min-h-[560px] flex-col bg-zinc-50 text-zinc-900 transition-colors dark:bg-zinc-900 dark:text-zinc-100">
        <TopBar
          difficulty={puzzle?.difficulty ?? null}
          grade={puzzle?.grade ?? null}
          elapsed={game.elapsed}
          variantNames={variantNames}
          theme={theme}
          onToggleTheme={() => setTheme(theme === "dark" ? "light" : "dark")}
          onNewGame={() => setShowNewGame(true)}
          onHint={game.fetchHint}
          onFillCandidates={() => setShowFill(true)}
          onStats={() => setShowStats(true)}
          disabled={!puzzle || game.solved}
        />

        {puzzle ? (
          <main className="flex flex-1 flex-col-reverse items-center justify-center gap-6 overflow-auto p-4 lg:flex-row lg:items-start lg:justify-center">
            <Board
              puzzle={puzzle}
              cells={cells}
              notes={game.notes}
              selected={selected}
              conflicts={game.conflicts}
              heldDigit={game.heldDigit}
              onCellDown={onCellDown}
              onCellEnter={onCellEnter}
            />
            <div className="flex w-72 shrink-0 flex-col gap-4 lg:pt-4">
              <Keypad
                pencil={game.pencil}
                onTogglePencil={() => game.setPencil((p) => !p)}
                heldDigit={game.heldDigit}
                onHeldDigit={game.setHeldDigit}
                digitCounts={digitCounts}
                onDigit={(d) => game.applyDigit(selected, d, game.pencil)}
                onErase={() => game.erase(selected)}
                onUndo={game.undo}
                onRedo={game.redo}
                canUndo={game.stackSizes.undo > 0}
                canRedo={game.stackSizes.redo > 0}
              />
              <p className="text-center text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                Arrows/WASD move · 1–9 place · Space notes ·
                <br />
                Del erases · Ctrl+Z undo · H hint
              </p>
            </div>
          </main>
        ) : (
          <main className="flex flex-1 items-center justify-center">
            <p className="text-zinc-500 dark:text-zinc-400">
              {game.busy ? "Forging a puzzle…" : "Start a new game to play."}
            </p>
          </main>
        )}

        {game.hint && (
          <HintPanel hint={game.hint} onApply={game.applyHint} onDismiss={game.dismissHint} />
        )}

        {showNewGame && (
          <NewGameModal
            busy={game.busy}
            canClose={puzzle !== null}
            onClose={() => setShowNewGame(false)}
            onStart={async (difficulty: string, rules: RuleId[]) => {
              const ok = await game.newGame(
                difficulty as import("./types").Difficulty,
                rules,
              );
              if (ok) setShowNewGame(false);
            }}
          />
        )}

        {showFill && puzzle && (
          <FillCandidatesModal
            selected={selected}
            cells={cells}
            onFill={game.writeNotes}
            onClose={() => setShowFill(false)}
          />
        )}

        {showStats && <StatsModal onClose={() => setShowStats(false)} />}

        {game.solved && puzzle && (
          <WinModal
            seconds={game.elapsed}
            grade={puzzle.grade}
            onNewGame={() => setShowNewGame(true)}
          />
        )}
      </div>
    </div>
  );
}
