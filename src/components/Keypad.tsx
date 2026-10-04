// Digit keypad with pencil/erase/undo/redo, the digit-first lock, the
// 9-color marking palette, digit highlights, and the candidate view toggle.

import { MARK_COLORS, markText } from "../theme";

interface KeypadProps {
  pencil: boolean;
  onTogglePencil: () => void;
  heldDigit: number | null;
  onHeldDigit: (d: number | null) => void;
  digitCounts: number[]; // index 0 → digit 1
  onDigit: (d: number) => void;
  onErase: () => void;
  onUndo: () => void;
  onRedo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  onMark: (color: number) => void; // color 1–9
  digitHl: Set<number>;
  onToggleDigitHl: (digit: number) => void;
  notesView: "grid" | "badges";
  onToggleNotesView: () => void;
}

const iconBtn =
  "flex h-11 w-11 items-center justify-center rounded-lg bg-zinc-200 text-lg text-zinc-700 transition-colors hover:bg-zinc-300 disabled:opacity-30 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600";

export function Keypad({
  pencil, onTogglePencil, heldDigit, onHeldDigit, digitCounts,
  onDigit, onErase, onUndo, onRedo, canUndo, canRedo,
  onMark, digitHl, onToggleDigitHl, notesView, onToggleNotesView,
}: KeypadProps) {
  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-5 gap-2">
        {Array.from({ length: 9 }, (_, k) => k + 1).map((d) => {
          const remaining = 9 - (digitCounts[d - 1] ?? 0);
          const done = remaining <= 0;
          const hl = digitHl.has(d) ? markText(d) : undefined;
          return (
            <button
              key={d}
              onClick={() => {
                onDigit(d);
                onHeldDigit(heldDigit === d ? null : d);
              }}
              className={[
                "flex h-12 flex-col items-center justify-center rounded-lg font-semibold transition-colors",
                heldDigit === d
                  ? "bg-emerald-600 text-white"
                  : "bg-zinc-200 text-zinc-700 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-100 dark:hover:bg-zinc-600",
                done && "opacity-35",
              ].join(" ")}
            >
              <span className="text-xl leading-none" style={hl ? { color: hl } : undefined}>
                {d}
              </span>
              <span className="text-[10px] leading-none opacity-70">{Math.max(remaining, 0)}</span>
            </button>
          );
        })}
        <button onClick={onErase} title="Erase (Del)" className={iconBtn}>
          ⌫
        </button>
      </div>

      <div className="flex items-center justify-between gap-2">
        <button
          onClick={onTogglePencil}
          title="Pencil marks (Space)"
          className={[
            "flex-1 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
            pencil
              ? "bg-emerald-600 text-white"
              : "bg-zinc-200 text-zinc-700 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600",
          ].join(" ")}
        >
          ✏ Notes {pencil ? "on" : "off"}
        </button>
        <button
          onClick={onToggleNotesView}
          title="Candidate layout"
          className="flex-1 rounded-lg bg-zinc-200 px-3 py-2 text-sm font-medium text-zinc-700 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600"
        >
          {notesView === "grid" ? "3×3" : "123"}
        </button>
        <button onClick={onUndo} disabled={!canUndo} title="Undo (Ctrl+Z)" className={iconBtn}>
          ↶
        </button>
        <button onClick={onRedo} disabled={!canRedo} title="Redo (Ctrl+Y)" className={iconBtn}>
          ↷
        </button>
      </div>

      <div>
        <p className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          Mark cells
        </p>
        <div className="flex gap-1.5">
          {MARK_COLORS.map((color, k) => (
            <button
              key={color}
              onClick={() => onMark(k + 1)}
              title={`Mark with color ${k + 1}`}
              className="h-7 w-7 rounded-md ring-1 ring-black/10 transition-transform hover:scale-110 dark:ring-white/20"
              style={{ backgroundColor: color }}
            />
          ))}
          <button
            onClick={() => onMark(0)}
            title="Clear marks on selection"
            className="h-7 w-7 rounded-md bg-zinc-200 text-xs text-zinc-600 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-300 dark:hover:bg-zinc-600"
          >
            ⌫
          </button>
        </div>
      </div>

      <div>
        <p className="mb-1 text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          Highlight digits
        </p>
        <div className="flex gap-1.5">
          {MARK_COLORS.map((color, k) => (
            <button
              key={color}
              onClick={() => onToggleDigitHl(k + 1)}
              title={`Highlight digit ${k + 1}`}
              className={[
                "h-7 w-7 rounded-md text-sm font-bold ring-1 transition-transform hover:scale-110",
                digitHl.has(k + 1) ? "ring-2 ring-zinc-500 dark:ring-zinc-300" : "ring-black/10 dark:ring-white/20",
              ].join(" ")}
              style={{ color, backgroundColor: `${color}22` }}
            >
              {k + 1}
            </button>
          ))}
        </div>
      </div>

      <p className="text-center text-xs text-zinc-500 dark:text-zinc-400">
        {heldDigit !== null
          ? `Digit-first: clicking cells places ${heldDigit}`
          : "Click a digit to lock it (digit-first)"}
      </p>
    </div>
  );
}
