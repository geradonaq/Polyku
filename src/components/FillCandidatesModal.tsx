// Fill-Candidates dialog: Fill 1-9 / Classic rules / Full ruleset, on the
// selection or the whole board.

import { useState } from "react";
import { api } from "../api";

interface FillCandidatesModalProps {
  selected: number[];
  cells: number[];
  onFill: (perCell: number[][]) => void;
  onClose: () => void;
}

type Mode = "all" | "classic" | "full";

const MODE_TEXT: Record<Mode, string> = {
  all: "Fill 1–9",
  classic: "Classic rules",
  full: "Full ruleset",
};

export function FillCandidatesModal({
  selected, cells, onFill, onClose,
}: FillCandidatesModalProps) {
  const [mode, setMode] = useState<Mode>("full");
  const [selectionOnly, setSelectionOnly] = useState(true);

  const targets = selectionOnly
    ? selected.filter((i) => cells[i] === 0)
    : cells.map((_, i) => i).filter((i) => cells[i] === 0);

  async function apply() {
    if (mode === "all") {
      const perCell = Array.from({ length: 81 }, () => [1, 2, 3, 4, 5, 6, 7, 8, 9]);
      onFill(perCell);
      onClose();
      return;
    }
    if (mode === "full") {
      const perCell = await api.fillCandidates(cells);
      onFill(perCell);
      onClose();
      return;
    }
    // classic: digits 1-9 minus what the row/column/box already holds
    const perCell: number[][] = Array.from({ length: 81 }, () => []);
    for (const i of targets) {
      const used = new Set<number>();
      const r = Math.floor(i / 9), c = i % 9;
      for (let j = 0; j < 81; j++) {
        if (cells[j] === 0) continue;
        const rj = Math.floor(j / 9), cj = j % 9;
        if (rj === r || cj === c || (Math.floor(rj / 3) === Math.floor(r / 3) && Math.floor(cj / 3) === Math.floor(c / 3))) {
          used.add(cells[j]);
        }
      }
      perCell[i] = ([1, 2, 3, 4, 5, 6, 7, 8, 9] as number[]).filter((d) => !used.has(d));
    }
    onFill(perCell);
    onClose();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-sm rounded-2xl bg-white p-6 shadow-2xl dark:bg-zinc-800">
        <h2 className="text-lg font-bold text-zinc-800 dark:text-zinc-100">Fill candidates</h2>

        <div className="mt-4 flex flex-col gap-2">
          {(Object.keys(MODE_TEXT) as Mode[]).map((m) => (
            <label key={m} className="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-zinc-100 dark:hover:bg-zinc-700">
              <input
                type="radio"
                checked={mode === m}
                onChange={() => setMode(m)}
                className="h-4 w-4 accent-emerald-600"
              />
              <span className="text-sm text-zinc-700 dark:text-zinc-200">{MODE_TEXT[m]}</span>
            </label>
          ))}
        </div>

        <label className="mt-4 flex cursor-pointer items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-zinc-100 dark:hover:bg-zinc-700">
          <input
            type="checkbox"
            checked={selectionOnly}
            onChange={(e) => setSelectionOnly(e.target.checked)}
            className="h-4 w-4 accent-emerald-600"
          />
          <span className="text-sm text-zinc-700 dark:text-zinc-200">
            Selected cells only ({targets.length} empty)
          </span>
        </label>

        <div className="mt-6 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-sm font-medium text-zinc-600 hover:bg-zinc-100 dark:text-zinc-300 dark:hover:bg-zinc-700"
          >
            Cancel
          </button>
          <button
            onClick={apply}
            disabled={targets.length === 0}
            className="rounded-lg bg-emerald-600 px-5 py-2 text-sm font-semibold text-white hover:bg-emerald-500 disabled:opacity-60"
          >
            Fill
          </button>
        </div>
      </div>
    </div>
  );
}
