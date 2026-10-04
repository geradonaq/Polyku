//! **Thermo Sudoku** — along each drawn path, digits must strictly increase
//! from the bulb (`cells[0]`) to the tip.
//!
//! Pruning combines two bounds: positional (cell at position `p` of an
//! `n`-length path needs at least `p+1` above it and `9-(n-1-p)` below, so
//! its digit is trapped in a band) and pairwise (an assigned neighbor forces
//! everything after it up / before it down).

use crate::grid::{DigitMask, CELLS};
use crate::rules::{range_mask, Overlay, ThermoPath, VariantRule};

pub struct Thermo {
    pub paths: Vec<ThermoPath>,
}

impl VariantRule for Thermo {
    fn id(&self) -> &'static str {
        "thermo"
    }

    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        for path in &self.paths {
            let n = path.len();
            for (pos, c) in path.iter().enumerate() {
                let d = cells[c.index()];
                if d == 0 {
                    continue;
                }
                let lo = (pos + 1) as u8;
                let hi = (9 - (n - 1 - pos)) as u8;
                if d < lo || d > hi {
                    return false;
                }
                if pos + 1 < n {
                    let next = cells[path[pos + 1].index()];
                    if next != 0 && next <= d {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for path in &self.paths {
            let n = path.len();
            // Positional band: position p of n must hold digit in [p+1, 9-(n-1-p)].
            for (pos, c) in path.iter().enumerate() {
                let lo = (pos + 1) as u8;
                let hi = (9 - (n - 1 - pos)) as u8;
                let idx = c.index();
                if cells[idx] == 0 {
                    candidates[idx] &= range_mask(lo, hi);
                }
            }
            // Pairwise propagation from assigned cells.
            for pos in 0..n.saturating_sub(1) {
                let a = path[pos].index();
                let b = path[pos + 1].index();
                let (da, db) = (cells[a], cells[b]);
                if da != 0 {
                    candidates[b] &= !range_mask(1, da); // b must be > da
                }
                if db != 0 {
                    candidates[a] &= !range_mask(db, 9); // a must be < db
                }
            }
        }
    }

    fn overlays(&self) -> Vec<Overlay> {
        self.paths
            .iter()
            .map(|p| Overlay::Path { cells: p.clone() })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Coord;

    fn path(cells: &[(u8, u8)]) -> ThermoPath {
        cells.iter().map(|&(r, c)| Coord::new(r, c)).collect()
    }

    #[test]
    fn positional_band_traps_ends() {
        // 4-cell thermo: position p needs p digits below and 3-p above, so
        // head ∈ [1,6], middle bands narrow in, tip ∈ [4,9].
        let t = Thermo {
            paths: vec![path(&[(0, 0), (0, 1), (0, 2), (0, 3)])],
        };
        let cells = [0u8; CELLS];
        let mut cands = [0x1FF; CELLS];
        Thermo::prune(&t, &cells, &mut cands);
        assert_eq!(cands[Coord::new(0, 0).index()], range_mask(1, 6));
        assert_eq!(cands[Coord::new(0, 1).index()], range_mask(2, 7));
        assert_eq!(cands[Coord::new(0, 3).index()], range_mask(4, 9));
    }

    #[test]
    fn assigned_head_pushes_rest_up() {
        let t = Thermo {
            paths: vec![path(&[(0, 0), (0, 1)])],
        };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 5;
        let mut cands = [0x1FF; CELLS];
        Thermo::prune(&t, &cells, &mut cands);
        // Tip must be > 5 and ≥ 2 by band: [6,9].
        assert_eq!(cands[Coord::new(0, 1).index()], range_mask(6, 9));
    }

    #[test]
    fn decreasing_pair_is_inconsistent() {
        let t = Thermo {
            paths: vec![path(&[(0, 0), (0, 1)])],
        };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 5;
        cells[Coord::new(0, 1).index()] = 5; // not strictly increasing
        assert!(!Thermo::is_consistent(&t, &cells));
    }
}
