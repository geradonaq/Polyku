// New-game dialog: six difficulty tiers + variant mix-and-match.

import { useState } from "react";
import { DIFFICULTIES, RULE_IDS, RULE_LABELS, type Difficulty, type RuleId } from "../types";

interface NewGameModalProps {
  busy: boolean;
  onStart: (difficulty: Difficulty, rules: RuleId[]) => void;
  onClose: () => void;
  canClose: boolean;
}

const TIER_BLURB: Record<Difficulty, string> = {
  beginner: "Naked singles only",
  easy: "Hidden singles, pointing",
  medium: "Pairs & triples",
  hard: "X-Wings enter",
  expert: "Swordfish, Y-Wings, coloring",
  master: "Jellyfish territory",
};

export function NewGameModal({ busy, onStart, onClose, canClose }: NewGameModalProps) {
  const [difficulty, setDifficulty] = useState<Difficulty>("easy");
  const [rules, setRules] = useState<RuleId[]>([]);

  const toggleRule = (id: RuleId) =>
    setRules((cur) => (cur.includes(id) ? cur.filter((r) => r !== id) : [...cur, id]));

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl dark:bg-zinc-800">
        <h2 className="text-lg font-bold text-zinc-800 dark:text-zinc-100">New game</h2>

        <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          Difficulty
        </p>
        <div className="mt-2 grid grid-cols-3 gap-2">
          {DIFFICULTIES.map((d, i) => (
            <button
              key={d}
              onClick={() => setDifficulty(d)}
              title={TIER_BLURB[d]}
              className={[
                "flex flex-col items-center rounded-xl border-2 px-2 py-3 transition-colors",
                difficulty === d
                  ? "border-emerald-500 bg-emerald-50 dark:bg-emerald-900/40"
                  : "border-zinc-200 hover:border-zinc-300 dark:border-zinc-600 dark:hover:border-zinc-500",
              ].join(" ")}
            >
              <span
                className={[
                  "flex h-9 w-9 items-center justify-center rounded-full border-2 text-sm font-bold",
                  difficulty === d
                    ? "border-emerald-500 text-emerald-600 dark:text-emerald-300"
                    : "border-zinc-300 text-zinc-500 dark:border-zinc-500 dark:text-zinc-400",
                ].join(" ")}
              >
                {i + 1}
              </span>
              <span className="mt-1 text-xs font-medium capitalize text-zinc-700 dark:text-zinc-200">
                {d}
              </span>
            </button>
          ))}
        </div>
        <p className="mt-1 text-center text-xs text-zinc-500 dark:text-zinc-400">
          {TIER_BLURB[difficulty]}
        </p>

        <p className="mt-4 text-xs font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
          Variant rules (stackable)
        </p>
        <div className="mt-2 flex flex-wrap gap-2">
          {RULE_IDS.map((id) => (
            <button
              key={id}
              onClick={() => toggleRule(id)}
              className={[
                "rounded-full border-2 px-3 py-1.5 text-sm font-medium transition-colors",
                rules.includes(id)
                  ? "border-violet-500 bg-violet-500 text-white"
                  : "border-zinc-200 text-zinc-600 hover:border-zinc-300 dark:border-zinc-600 dark:text-zinc-300 dark:hover:border-zinc-500",
              ].join(" ")}
            >
              {RULE_LABELS[id]}
            </button>
          ))}
        </div>

        <div className="mt-6 flex justify-end gap-2">
          {canClose && (
            <button
              onClick={onClose}
              className="rounded-lg px-4 py-2 text-sm font-medium text-zinc-600 hover:bg-zinc-100 dark:text-zinc-300 dark:hover:bg-zinc-700"
            >
              Cancel
            </button>
          )}
          <button
            onClick={() => onStart(difficulty, rules)}
            disabled={busy}
            className="rounded-lg bg-emerald-600 px-5 py-2 text-sm font-semibold text-white transition-colors hover:bg-emerald-500 disabled:opacity-60"
          >
            {busy ? "Forging a puzzle…" : "Generate"}
          </button>
        </div>
      </div>
    </div>
  );
}
