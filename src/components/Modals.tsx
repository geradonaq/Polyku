// Win overlay + statistics modal.

import { useEffect, useState } from "react";
import { api } from "../api";
import { tierName, type Stats } from "../types";

export function WinModal({
  seconds, grade, onNewGame,
}: {
  seconds: number;
  grade: number;
  onNewGame: () => void;
}) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
      <div className="w-full max-w-sm rounded-2xl bg-white p-8 text-center shadow-2xl dark:bg-zinc-800">
        <div className="text-5xl">🎉</div>
        <h2 className="mt-3 text-2xl font-bold text-zinc-800 dark:text-zinc-100">Solved!</h2>
        <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">
          {tierName(grade)} · {String(m).padStart(2, "0")}:{String(s).padStart(2, "0")}
        </p>
        <button
          onClick={onNewGame}
          className="mt-6 w-full rounded-lg bg-emerald-600 px-5 py-2.5 font-semibold text-white hover:bg-emerald-500"
        >
          Play again
        </button>
      </div>
    </div>
  );
}

interface AggRow {
  key: string;
  played: number;
  won: number;
  best: number | null;
}

export function StatsModal({ onClose }: { onClose: () => void }) {
  const [stats, setStats] = useState<Stats | null>(null);

  useEffect(() => {
    api.getStatistics().then(setStats).catch(() => setStats({ games: [] }));
  }, []);

  const rows = new Map<string, AggRow>();
  for (const g of stats?.games ?? []) {
    const key = [g.difficulty, ...g.rules].join(" + ") || "classic";
    const row = rows.get(key) ?? { key, played: 0, won: 0, best: null };
    row.played += 1;
    if (g.completed) {
      row.won += 1;
      row.best = row.best === null ? g.seconds : Math.min(row.best, g.seconds);
    }
    rows.set(key, row);
  }
  const list = [...rows.values()].sort((a, b) => a.key.localeCompare(b.key));

  const fmt = (s: number) =>
    `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl dark:bg-zinc-800">
        <h2 className="text-lg font-bold text-zinc-800 dark:text-zinc-100">Statistics</h2>
        {list.length === 0 ? (
          <p className="mt-4 text-sm text-zinc-500 dark:text-zinc-400">
            No games recorded yet — finish one!
          </p>
        ) : (
          <table className="mt-4 w-full text-left text-sm">
            <thead>
              <tr className="text-xs uppercase tracking-wide text-zinc-500 dark:text-zinc-400">
                <th className="pb-2">Mode</th>
                <th className="pb-2 text-right">Played</th>
                <th className="pb-2 text-right">Won</th>
                <th className="pb-2 text-right">Best</th>
              </tr>
            </thead>
            <tbody>
              {list.map((row) => (
                <tr key={row.key} className="border-t border-zinc-200 dark:border-zinc-700">
                  <td className="py-2 pr-2 text-zinc-700 dark:text-zinc-200">{row.key}</td>
                  <td className="py-2 text-right text-zinc-600 dark:text-zinc-300">{row.played}</td>
                  <td className="py-2 text-right text-zinc-600 dark:text-zinc-300">{row.won}</td>
                  <td className="py-2 text-right font-mono text-zinc-600 dark:text-zinc-300">
                    {row.best === null ? "—" : fmt(row.best)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        <div className="mt-6 flex justify-end">
          <button
            onClick={onClose}
            className="rounded-lg bg-zinc-200 px-4 py-2 text-sm font-medium text-zinc-700 hover:bg-zinc-300 dark:bg-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-600"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
