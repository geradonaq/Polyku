// The 9×9 board: cells + the SVG overlay layer (cages, thermos, stripes).

import type { Overlay, PuzzleDto, RuleData } from "../types";
import { BOARD, cageEdges, cageLabelSpot, cellCenter, overlaysOfRules, thermoLine } from "../overlay";

interface BoardProps {
  puzzle: PuzzleDto;
  cells: number[];
  notes: number[][];
  selected: number[];
  conflicts: number[];
  heldDigit: number | null;
  onCellDown: (index: number, additive: boolean) => void;
  onCellEnter: (index: number) => void;
}

function Overlays({ rules }: { rules: RuleData[] }) {
  const overlays: Overlay[] = overlaysOfRules(rules);
  return (
    <svg
      viewBox={`0 0 ${BOARD} ${BOARD}`}
      className="pointer-events-none absolute inset-0 h-full w-full"
    >
      {overlays.map((o, oi) => {
        if ("DiagonalStripe" in o) {
          const { main } = o.DiagonalStripe;
          return (
            <line
              key={oi}
              x1={main ? 0 : BOARD}
              y1={0}
              x2={main ? BOARD : 0}
              y2={BOARD}
              stroke="currentColor"
              strokeWidth={110}
              className="text-zinc-400 opacity-[0.07] dark:text-zinc-200 dark:opacity-[0.05]"
            />
          );
        }
        if ("Cage" in o) {
          const cage = o.Cage;
          const spot = cageLabelSpot(cage.cells);
          return (
            <g key={oi} className="text-zinc-500 dark:text-zinc-400">
              {cageEdges(cage.cells).map((e, ei) => (
                <line
                  key={ei}
                  x1={e.x1 + 6}
                  y1={e.y1 + 6}
                  x2={e.x2 - 6}
                  y2={e.y2 - 6}
                  stroke="currentColor"
                  strokeWidth={3}
                  strokeDasharray="7 5"
                  opacity={0.85}
                />
              ))}
              {cage.sum !== null && (
                <text
                  x={spot.col * 100 + 10}
                  y={spot.row * 100 + 30}
                  fontSize={30}
                  fontWeight={700}
                  fill="currentColor"
                >
                  {cage.sum}
                </text>
              )}
            </g>
          );
        }
        if ("Path" in o) {
          const path = o.Path.cells;
          const start = cellCenter(path[0]);
          const end = cellCenter(path[path.length - 1]);
          return (
            <g key={oi} className="text-slate-400/60">
              <path
                d={thermoLine(path)}
                fill="none"
                stroke="currentColor"
                strokeWidth={64}
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <circle cx={start.x} cy={start.y} r={40} fill="currentColor" />
              <circle cx={end.x} cy={end.y} r={24} fill="none" stroke="currentColor" strokeWidth={8} />
            </g>
          );
        }
        return null;
      })}
    </svg>
  );
}

export function Board({
  puzzle, cells, notes, selected, conflicts, heldDigit, onCellDown, onCellEnter,
}: BoardProps) {
  const selSet = new Set(selected);
  const primary = selected[selected.length - 1];
  const primaryDigit = primary !== undefined ? cells[primary] : 0;

  return (
    <div className="relative aspect-square w-full max-w-[min(88vh,760px)] select-none rounded-xl bg-zinc-100 p-1 shadow-2xl ring-1 ring-zinc-300 dark:bg-zinc-800 dark:ring-zinc-700">
      <div className="relative h-full w-full">
        <Overlays rules={puzzle.rules} />
        <div className="grid h-full w-full grid-cols-9 grid-rows-9">
          {cells.map((value, i) => {
            const row = Math.floor(i / 9);
            const col = i % 9;
            const isGiven = puzzle.givens[i] !== 0;
            const isSelected = selSet.has(i);
            const isConflict = conflicts.includes(i);
            const isPeer =
              primary !== undefined &&
              !isSelected &&
              (Math.floor(primary / 9) === row ||
                primary % 9 === col ||
                (Math.floor(primary / 27) === Math.floor(row / 3) &&
                  Math.floor((primary % 9) / 3) === Math.floor(col / 3)));
            const sameDigit =
              value !== 0 && primaryDigit === value && !isSelected;
            const thickRight = col % 3 === 2 && col !== 8;
            const thickBottom = row % 3 === 2 && row !== 8;

            const bg = isSelected
              ? "bg-emerald-200 dark:bg-emerald-700/50"
              : isConflict
                ? "bg-red-200 dark:bg-red-900/60"
                : sameDigit
                  ? "bg-teal-100 dark:bg-teal-800/40"
                  : isPeer
                    ? "bg-zinc-200 dark:bg-zinc-700/70"
                    : "bg-white dark:bg-zinc-800";

            const text = isConflict
              ? "text-red-600 dark:text-red-300"
              : isGiven
                ? "text-zinc-800 dark:text-zinc-100"
                : "text-sky-600 dark:text-sky-300";

            return (
              <button
                key={i}
                onMouseDown={(e) => onCellDown(i, e.shiftKey || e.ctrlKey || e.metaKey)}
                onMouseEnter={() => onCellEnter(i)}
                className={[
                  "flex items-center justify-center border-[0.5px] border-zinc-300 transition-colors dark:border-zinc-600/70",
                  thickRight && "border-r-[3px] border-r-zinc-500 dark:border-r-zinc-400",
                  thickBottom && "border-b-[3px] border-b-zinc-500 dark:border-b-zinc-400",
                  bg,
                  heldDigit !== null && !isGiven ? "cursor-crosshair" : "cursor-pointer",
                ].join(" ")}
              >
                {value !== 0 ? (
                  <span className={`text-3xl font-semibold leading-none ${text} ${isGiven ? "" : ""}`}>
                    {value}
                  </span>
                ) : notes[i].length > 0 ? (
                  <span className="grid h-full w-full grid-cols-3 grid-rows-3 p-[6%] text-[clamp(6px,1.4vw,14px)] leading-none text-zinc-400 dark:text-zinc-500">
                    {Array.from({ length: 9 }, (_, k) => (
                      <span key={k} className="flex items-center justify-center">
                        {notes[i].includes(k + 1) ? k + 1 : ""}
                      </span>
                    ))}
                  </span>
                ) : null}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
