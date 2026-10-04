//! **Killer Sudoku** — the grid is partitioned into cages; each cage's
//! digits (all distinct) must sum to the cage's target.
//!
//! The interesting work is `prune`: for a cage with `m` empty cells and
//! remaining sum `R`, an empty cell may take digit `d` only if the other
//! `m-1` cells can still make `R - d` with distinct digits. Exact matching
//! per cell would be expensive; the classic min/max bounds (smallest /
//! largest possible sums of distinct digits) prune hard enough in practice.

use crate::grid::{DigitMask, CELLS};
use crate::rules::{Cage, Overlay, VariantRule};

pub struct Killer {
    pub cages: Vec<Cage>,
}

/// Sum of the `k` smallest distinct digits: 1+2+…+k.
fn min_sum(k: u16) -> u16 {
    k * (k + 1) / 2
}

/// Sum of the `k` largest distinct digits: 9+8+…+(10-k).
fn max_sum(k: u16) -> u16 {
    k * (19 - k) / 2
}

impl Killer {
    fn cage_status(&self, cells: &[u8; CELLS], cage: &Cage) -> (DigitMask, u16, Vec<usize>) {
        let mut used_mask = 0;
        let mut used_sum = 0u16;
        let mut empties = Vec::new();
        for c in &cage.cells {
            let idx = c.index();
            let d = cells[idx];
            if d == 0 {
                empties.push(idx);
            } else {
                used_mask |= 1 << (d - 1);
                used_sum += d as u16;
            }
        }
        (used_mask, used_sum, empties)
    }
}

impl VariantRule for Killer {
    fn id(&self) -> &'static str {
        "killer"
    }

    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        for cage in &self.cages {
            let (used_mask, used_sum, empties) = self.cage_status(cells, cage);
            // Digits must not repeat inside a cage.
            if used_mask.count_ones() as usize != cage.cells.len() - empties.len() {
                return false;
            }
            if used_sum > cage.sum {
                return false;
            }
            if empties.is_empty() && used_sum != cage.sum {
                return false;
            }
        }
        true
    }

    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for cage in &self.cages {
            let (used_mask, used_sum, empties) = self.cage_status(cells, cage);
            let rest = empties.len() as u16;
            if rest == 0 {
                continue; // complete cage — leaf check handles it
            }
            // i32 arithmetic: `remaining` may legitimately go negative when
            // the partial sum already overshoots — that's a dead end, not a
            // panic.
            let remaining = cage.sum as i32 - used_sum as i32;
            // The other (rest-1) empties must make up (remaining - d), with
            // distinct digits — bounded by min/max sums.
            let lo_rest = min_sum(rest - 1) as i32;
            let hi_rest = max_sum(rest - 1) as i32;
            for &idx in &empties {
                let mut ok = 0;
                for d in 1..=9u8 {
                    let bit = 1 << (d - 1);
                    if used_mask & bit == 0 {
                        let s = remaining - d as i32;
                        if s >= lo_rest && s <= hi_rest {
                            ok |= bit;
                        }
                    }
                }
                candidates[idx] &= ok;
            }
        }
    }

    fn overlays(&self) -> Vec<Overlay> {
        self.cages
            .iter()
            .map(|c| Overlay::Cage { cells: c.cells.clone(), sum: Some(c.sum) })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Coord;

    fn cage(cells: &[(u8, u8)], sum: u16) -> Cage {
        Cage {
            cells: cells.iter().map(|&(r, c)| Coord::new(r, c)).collect(),
            sum,
        }
    }

    #[test]
    fn min_max_sum_bands_are_right() {
        assert_eq!(min_sum(0), 0);
        assert_eq!(min_sum(2), 3); // 1+2
        assert_eq!(max_sum(2), 17); // 9+8
        assert_eq!(max_sum(9), 45);
    }

    #[test]
    fn complete_cage_must_hit_its_sum() {
        let k = Killer { cages: vec![cage(&[(0, 0), (0, 1)], 7)] };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 3;
        cells[Coord::new(0, 1).index()] = 4;
        assert!(k.is_consistent(&cells));
        cells[Coord::new(0, 1).index()] = 5; // 3+5 != 7
        assert!(!k.is_consistent(&cells));
    }

    #[test]
    fn partial_cage_may_not_exceed_sum() {
        let k = Killer { cages: vec![cage(&[(0, 0), (0, 1)], 7)] };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 8;
        assert!(!k.is_consistent(&cells)); // 8 > 7 already
    }

    #[test]
    fn prune_forces_last_cage_cell() {
        // Cage {3,4} sum 7: after placing 3, the last cell is forced to 4.
        let k = Killer { cages: vec![cage(&[(0, 0), (0, 1)], 7)] };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 3;
        let mut cands = [0x1FF; CELLS];
        Killer::prune(&k, &cells, &mut cands);
        let idx = Coord::new(0, 1).index();
        assert_eq!(cands[idx], 1 << 3, "only digit 4 survives");
    }

    #[test]
    fn prune_bans_cage_duplicates_and_overshoots() {
        // Cage of two, sum 17: only pairs (8,9)/(9,8) are possible.
        let k = Killer { cages: vec![cage(&[(0, 0), (0, 1)], 17)] };
        let mut cells = [0u8; CELLS];
        cells[Coord::new(0, 0).index()] = 9;
        let mut cands = [0x1FF; CELLS];
        Killer::prune(&k, &cells, &mut cands);
        assert_eq!(cands[Coord::new(0, 1).index()], 1 << 7); // forced 8
        let mut cands2 = [0x1FF; CELLS];
        Killer::prune(&k, &cells, &mut cands2); // cell (0,1) perspective
        assert_eq!(cands2[Coord::new(1, 0).index()], 0x1FF, "cells outside cage untouched");
    }
}
