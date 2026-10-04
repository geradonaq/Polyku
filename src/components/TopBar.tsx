// Top bar: title, difficulty + graded tier badge, timer, actions.

import { tierName } from "../types";

interface TopBarProps {
  difficulty: string | null;
  grade: number | null;
  elapsed: number;
  variantNames: string[];
  theme: "dark" | "light";
  onToggleTheme: () => void;
  onNewGame: () => void;
  onHint: () => void;
  onFillCandidates: () => void;
  onStats: () => void;
  disabled: boolean;
}

function fmt(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

const btn =
  "rounded-lg bg-zinc-200 px-3 py-1.5 text-sm font-medium text-zinc-700 transition-colors hover:bg-zinc-300 disabled:opacity-40 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600";

export function TopBar({
  difficulty, grade, elapsed, variantNames, theme, onToggleTheme,
  onNewGame, onHint, onFillCandidates, onStats, disabled,
}: TopBarProps) {
  return (
    <header className="flex flex-wrap items-center gap-3 border-b border-zinc-200 px-5 py-3 dark:border-zinc-700">
      <h1 className="mr-2 text-xl font-bold tracking-tight text-emerald-500">Polyku</h1>

      {difficulty && (
        <div className="flex items-center gap-2">
          <span className="rounded-full bg-emerald-100 px-3 py-1 text-xs font-semibold uppercase tracking-wide text-emerald-700 dark:bg-emerald-900/60 dark:text-emerald-300">
            {difficulty}
          </span>
          <span
            title="Graded hardest technique required"
            className="rounded-full bg-zinc-200 px-3 py-1 text-xs font-semibold text-zinc-600 dark:bg-zinc-700 dark:text-zinc-300"
          >
            grade: {tierName(grade ?? 1)}
          </span>
          {variantNames.length > 0 && (
            <span className="rounded-full bg-violet-100 px-3 py-1 text-xs font-semibold text-violet-700 dark:bg-violet-900/60 dark:text-violet-300">
              {variantNames.join(" + ")}
            </span>
          )}
        </div>
      )}

      <span className="ml-auto font-mono text-lg tabular-nums text-zinc-600 dark:text-zinc-300">
        {fmt(elapsed)}
      </span>

      <div className="flex items-center gap-2">
        <button onClick={onNewGame} className={btn}>New game</button>
        <button onClick={onHint} disabled={disabled} className={btn}>💡 Hint</button>
        <button onClick={onFillCandidates} disabled={disabled} className={btn}>Candidates</button>
        <button onClick={onStats} className={btn}>Stats</button>
        <button onClick={onToggleTheme} className={btn} title="Toggle theme">
          {theme === "dark" ? "☀" : "☾"}
        </button>
      </div>
    </header>
  );
}
