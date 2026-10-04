import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function App() {
  const [engineVersion, setEngineVersion] = useState<string | null>(null);

  async function checkEngine() {
    setEngineVersion(await invoke<string>("engine_version"));
  }

  return (
    <main className="flex h-screen flex-col items-center justify-center gap-6 bg-zinc-950 text-zinc-100">
      <h1 className="text-6xl font-bold tracking-tight">Polyku</h1>
      <p className="text-zinc-400">Offline sudoku, freshly generated every time.</p>
      <button
        onClick={checkEngine}
        className="rounded-lg bg-emerald-600 px-5 py-2.5 font-medium transition-colors hover:bg-emerald-500"
      >
        Check engine
      </button>
      {engineVersion && (
        <p className="text-sm text-zinc-500">engine version: {engineVersion}</p>
      )}
    </main>
  );
}
