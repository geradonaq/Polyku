//! **Anti-Knight Sudoku** — cells a chess knight's move apart never hold
//! the same digit. A popular constraint that pairs well with everything.

use crate::grid::{DigitMask, CELLS};
use crate::rules::{Overlay, VariantRule};

pub struct AntiKnight;

fn knight_targets(idx: usize) -> Vec<usize> {
    let (r, c) = ((idx / 9) as i32, (idx % 9) as i32);
    let mut out = Vec::new();
    for (dr, dc) in [
        (1, 2),
        (2, 1),
        (-1, 2),
        (-2, 1),
        (1, -2),
        (2, -1),
        (-1, -2),
        (-2, -1),
    ] {
        let (rr, cc) = (r + dr, c + dc);
        if (0..9).contains(&rr) && (0..9).contains(&cc) {
            out.push(rr as usize * 9 + cc as usize);
        }
    }
    out
}

impl VariantRule for AntiKnight {
    fn id(&self) -> &'static str {
        "anti_knight"
    }

    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        for i in 0..CELLS {
            if cells[i] == 0 {
                continue;
            }
            for j in knight_targets(i) {
                if cells[j] == cells[i] {
                    return false;
                }
            }
        }
        true
    }

    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for i in 0..CELLS {
            let d = cells[i];
            if d == 0 {
                continue;
            }
            let bit = !(1 << (d - 1));
            for j in knight_targets(i) {
                if cells[j] == 0 {
                    candidates[j] &= bit;
                }
            }
        }
    }

    fn overlays(&self) -> Vec<Overlay> {
        Vec::new() // invisible rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Coord;

    #[test]
    fn knight_duplicate_is_inconsistent() {
        let mut cells = [0u8; CELLS];
        let a = Coord::new(4, 4).index();
        cells[a] = 8;
        cells[a + 10 - 9] = 0; // (3,5) is a knight move away — set below
        let knight_of_a = Coord::new(2, 5).index(); // (4,4) + (-2,+1)
        cells[knight_of_a] = 8;
        assert!(!AntiKnight.is_consistent(&cells));

        cells[knight_of_a] = 7;
        assert!(AntiKnight.is_consistent(&cells));
    }

    #[test]
    fn prune_bans_digit_on_knight_moves() {
        let mut cells = [0u8; CELLS];
        let a = Coord::new(4, 4).index();
        cells[a] = 8;
        let mut cands = [0x1FF; CELLS];
        AntiKnight.prune(&cells, &mut cands);
        let knight_cell = Coord::new(2, 5).index();
        assert_eq!(
            cands[knight_cell] & (1 << 7),
            0,
            "digit 8 banned at knight move"
        );
        let far_cell = Coord::new(0, 0).index();
        assert_eq!(cands[far_cell], 0x1FF, "unrelated cell untouched");
    }
}
