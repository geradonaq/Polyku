// Pure geometry helpers that turn engine Overlay primitives into SVG shapes.
// The board is a 900×900 viewBox: every cell is a 100×100 square.

import type { Coord, Overlay, RuleData, ThermoPath } from "./types";

export const BOARD = 900;
export const CELL = 100;

export function cellCenter(c: Coord): { x: number; y: number } {
  return { x: c.col * CELL + CELL / 2, y: c.row * CELL + CELL / 2 };
}

export function cageEdges(cells: Coord[]): { x1: number; y1: number; x2: number; y2: number }[] {
  const inside = new Set(cells.map((c) => c.row * 9 + c.col));
  const edges: { x1: number; y1: number; x2: number; y2: number }[] = [];
  for (const c of cells) {
    const x = c.col * CELL;
    const y = c.row * CELL;
    // Draw a dashed edge only where the neighbor is outside the cage —
    // that yields the cage outline without path-tracing the union.
    if (!inside.has((c.row - 1) * 9 + c.col)) edges.push({ x1: x, y1: y, x2: x + CELL, y2: y });
    if (!inside.has((c.row + 1) * 9 + c.col)) edges.push({ x1: x, y1: y + CELL, x2: x + CELL, y2: y + CELL });
    if (!inside.has(c.row * 9 + c.col - 1)) edges.push({ x1: x, y1: y, x2: x, y2: y + CELL });
    if (!inside.has(c.row * 9 + c.col + 1)) edges.push({ x1: x + CELL, y1: y, x2: x + CELL, y2: y + CELL });
  }
  return edges;
}

/// The cage's top-left-most cell — where the sum label sits.
export function cageLabelSpot(cells: Coord[]): Coord {
  return [...cells].sort((a, b) => a.row * 9 + a.col - (b.row * 9 + b.col))[0];
}

export function thermoLine(path: ThermoPath): string {
  const points = path.map(cellCenter);
  return points.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.y}`).join(" ");
}

/// Groups overlays per rule for rendering.
export function overlaysOfRules(rules: RuleData[]): Overlay[] {
  const out: Overlay[] = [];
  for (const rule of rules) {
    if (rule === "Diagonal") {
      out.push({ DiagonalStripe: { main: true } });
      out.push({ DiagonalStripe: { main: false } });
    } else if (rule === "NonConsecutive" || rule === "AntiKnight") {
      // invisible rules
    } else if ("Killer" in rule) {
      for (const cage of rule.Killer.cages) out.push({ Cage: { cells: cage.cells, sum: cage.sum } });
    } else if ("Thermo" in rule) {
      for (const path of rule.Thermo.paths) out.push({ Path: { cells: path } });
    }
  }
  return out;
}
