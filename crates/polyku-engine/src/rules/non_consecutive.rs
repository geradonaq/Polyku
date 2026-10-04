//! **Non-Consecutive Sudoku** — orthogonally adjacent cells may never hold
//! consecutive digits (|a − b| = 1 is banned). Diagonal neighbors are free.

use crate::grid::{DigitMask, CELLS};
use crate::rules::{Overlay, VariantRule};

pub struct NonConsecutive;

/// Bits of digits forbidden next to assigned digit `d`: d-1, d, d+1.
/// (d itself is already banned by classic rules for neighbors in the same
/// row/col — but adjacent cells *are* in the same row/col, so re-banning
/// is harmless and keeps this rule self-contained.)
fn banned_around(d: u8) -> DigitMask {
    let mut m = 1 << (d - 1); // d
    if d > 1 {
        m |= 1 << (d - 2); // d-1
    }
    if d < 9 {
        m |= 1 << d; // d+1
    }
    m
}

fn neighbor_indices(idx: usize) -> [Option<usize>; 4] {
    let r = idx / 9;
    let c = idx % 9;
    [
        (r > 0).then(|| idx - 9),
        (r < 8).then(|| idx + 9),
        (c > 0).then(|| idx - 1),
        (c < 8).then(|| idx + 1),
    ]
}

impl VariantRule for NonConsecutive {
    fn id(&self) -> &'static str {
        "non_consecutive"
    }

    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        for idx in 0..CELLS {
            let d = cells[idx];
            if d == 0 {
                continue;
            }
            for n in neighbor_indices(idx).into_iter().flatten() {
                let dn = cells[n];
                if dn != 0 && (dn as i8 - d as i8).abs() == 1 {
                    return false;
                }
            }
        }
        true
    }

    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for idx in 0..CELLS {
            let d = cells[idx];
            if d == 0 {
                continue;
            }
            let banned = banned_around(d);
            for n in neighbor_indices(idx).into_iter().flatten() {
                if cells[n] == 0 {
                    candidates[n] &= !banned;
                }
            }
        }
    }

    fn overlays(&self) -> Vec<Overlay> {
        Vec::new() // invisible rule — the classic grid communicates it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Coord;
    use crate::rules::range_mask;

    #[test]
    fn consecutive_neighbors_are_inconsistent() {
        let mut cells = [0u8; CELLS];
        let a = Coord::new(4, 4).index();
        cells[a] = 5;
        cells[a + 1] = 6; // right neighbor, consecutive
        assert!(!NonConsecutive.is_consistent(&cells));

        cells[a + 1] = 7; // difference 2 — fine
        assert!(NonConsecutive.is_consistent(&cells));

        cells[a + 9] = 4; // below neighbor, consecutive
        assert!(!NonConsecutive.is_consistent(&cells));

        cells[a + 9] = 3; // below neighbor, difference 2 — fine now
        cells[a + 10] = 5; // (5,5): diagonal from (4,4) — exempt; also diff 2 from (5,4)
        assert!(NonConsecutive.is_consistent(&cells));
    }

    #[test]
    fn prune_bans_consecutive_digits_around_assignment() {
        let mut cells = [0u8; CELLS];
        let a = Coord::new(4, 4).index();
        cells[a] = 5;
        let mut cands = [0x1FF; CELLS];
        NonConsecutive.prune(&cells, &mut cands);
        // Right neighbor bans 4,5,6.
        assert_eq!(cands[a + 1] & (range_mask(4, 6)), 0);
        assert_ne!(cands[a + 1] & (1 << 2), 0, "digit 3 still allowed");
        // Far-away cell unaffected.
        assert_eq!(cands[0], 0x1FF);
    }
}
