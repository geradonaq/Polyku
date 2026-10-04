// Floating teaching panel for the current hint.

import type { HintDto } from "../types";

interface HintPanelProps {
  hint: HintDto;
  onApply: () => void;
  onDismiss: () => void;
}

const kindStyles: Record<HintDto["kind"], string> = {
  technique: "border-emerald-500/60 dark:border-emerald-400/60",
  error: "border-red-500/60",
  done: "border-sky-500/60",
  stuck: "border-amber-500/60",
};

export function HintPanel({ hint, onApply, onDismiss }: HintPanelProps) {
  return (
    <div
      className={`fixed bottom-5 left-1/2 z-40 w-[min(92vw,640px)] -translate-x-1/2 rounded-xl border-l-4 bg-white p-4 shadow-2xl dark:bg-zinc-800 ${kindStyles[hint.kind]}`}
    >
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
            {hint.kind === "technique" ? `${hint.technique} · tier ${hint.tier}` : hint.technique}
          </p>
          <p className="mt-1 text-sm text-zinc-700 dark:text-zinc-200">{hint.explanation}</p>
        </div>
        <div className="flex shrink-0 gap-2">
          {hint.kind === "technique" &&
            (hint.placements.length > 0 || hint.eliminations.length > 0) && (
              <button
                onClick={onApply}
                className="rounded-lg bg-emerald-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-emerald-500"
              >
                Apply
              </button>
            )}
          <button
            onClick={onDismiss}
            className="rounded-lg bg-zinc-200 px-3 py-1.5 text-sm font-medium text-zinc-700 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600"
          >
            ✕
          </button>
        </div>
      </div>
    </div>
  );
}
