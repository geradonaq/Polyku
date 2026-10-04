//! **Diagonal (X) Sudoku** — both main diagonals must also contain all
//! digits 1–9 exactly once. The base row/column/box masks don't know about
//! diagonals, so this rule adds its own exclusion pass.

use crate::grid::{DigitMask, CELLS};
use crate::rules::{Overlay, VariantRule};

pub struct Diagonal;

fn diagonal_index(k: usize, main: bool) -> usize {
    if main {
        k * 9 + k
    } else {
        k * 9 + (8 - k)
    }
}

impl Diagonal {
    fn used_mask(&self, cells: &[u8; CELLS], main: bool) -> DigitMask {
        let mut used = 0;
        for k in 0..9 {
            let d = cells[diagonal_index(k, main)];
            if d != 0 {
                used |= 1 << (d - 1);
            }
        }
        used
    }
}

impl VariantRule for Diagonal {
    fn id(&self) -> &'static str {
        "diagonal"
    }

    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        // Any duplicate on either diagonal breaks the rule.
        for main in [true, false] {
            let mut seen = 0u16;
            for k in 0..9 {
                let d = cells[diagonal_index(k, main)];
                if d != 0 {
                    let bit = 1 << (d - 1);
                    if seen & bit != 0 {
                        return false;
                    }
                    seen |= bit;
                }
            }
        }
        true
    }

    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for main in [true, false] {
            let used = self.used_mask(cells, main);
            for k in 0..9 {
                let idx = diagonal_index(k, main);
                if cells[idx] == 0 {
                    candidates[idx] &= !used;
                }
            }
        }
    }

    fn overlays(&self) -> Vec<Overlay> {
        vec![
            Overlay::DiagonalStripe { main: true },
            Overlay::DiagonalStripe { main: false },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Coord;
    use crate::rules::range_mask;

    #[test]
    fn duplicate_on_diagonal_is_inconsistent() {
        let mut cells = [0u8; CELLS];
        cells[diagonal_index(0, true)] = 7;
        cells[diagonal_index(3, true)] = 7; // same main diagonal
        assert!(!Diagonal.is_consistent(&cells));

        cells[diagonal_index(3, true)] = 0;
        cells[diagonal_index(2, false)] = 7; // anti-diagonal is a different unit
        assert!(Diagonal.is_consistent(&cells));
    }

    #[test]
    fn prune_removes_diagonal_digits() {
        let mut cells = [0u8; CELLS];
        cells[diagonal_index(0, true)] = 4; // (0,0)
        let mut cands = [0x1FF; CELLS];
        Diagonal.prune(&cells, &mut cands);
        let on_main = Coord::new(5, 5).index();
        let off_diagonal = Coord::new(5, 6).index();
        assert_eq!(cands[on_main] & (1 << 3), 0, "digit 4 banned on main diagonal");
        assert_ne!(cands[off_diagonal] & (1 << 3), 0, "unrelated cell unaffected");
        assert_eq!(cands[off_diagonal], range_mask(1, 9));
    }
}
