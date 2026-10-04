// Digit keypad with pencil/erase/undo/redo and the digit-first lock.

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
}

const iconBtn =
  "flex h-11 w-11 items-center justify-center rounded-lg bg-zinc-200 text-lg text-zinc-700 transition-colors hover:bg-zinc-300 disabled:opacity-30 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600";

export function Keypad({
  pencil, onTogglePencil, heldDigit, onHeldDigit, digitCounts,
  onDigit, onErase, onUndo, onRedo, canUndo, canRedo,
}: KeypadProps) {
  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-5 gap-2">
        {Array.from({ length: 9 }, (_, k) => k + 1).map((d) => {
          const remaining = 9 - (digitCounts[d - 1] ?? 0);
          const done = remaining <= 0;
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
              <span className="text-xl leading-none">{d}</span>
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
        <button onClick={onUndo} disabled={!canUndo} title="Undo (Ctrl+Z)" className={iconBtn}>
          ↶
        </button>
        <button onClick={onRedo} disabled={!canRedo} title="Redo (Ctrl+Y)" className={iconBtn}>
          ↷
        </button>
      </div>

      <p className="text-center text-xs text-zinc-500 dark:text-zinc-400">
        {heldDigit !== null
          ? `Digit-first: clicking cells places ${heldDigit}`
          : "Click a digit to lock it (digit-first)"}
      </p>
    </div>
  );
}
