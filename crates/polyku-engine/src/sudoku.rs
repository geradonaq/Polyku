//! Maps sudoku onto exact cover so the DLX solver can count solutions.
//!
//! The classic encoding uses 324 constraint columns:
//! - **cell** (81): every cell holds exactly one digit — column `r*9 + c`
//! - **row-digit** (81): every row contains digit `d` once — `81 + r*9 + (d-1)`
//! - **col-digit** (81): every column contains digit `d` once — `162 + c*9 + (d-1)`
//! - **box-digit** (81): every box contains digit `d` once — `243 + b*9 + (d-1)`
//!
//! A placement `(r, c, d)` covers exactly 4 columns (one of each kind).
//! Solving = choosing placements so that every column is covered exactly once,
//! which is sudoku by definition.

use crate::dlx::ExactCover;
use crate::grid::{Coord, Grid, CELLS};

const CONSTRAINTS: usize = 4 * CELLS;

/// Flat row id for a placement — encodes (row, col, digit) in 9 bits each.
pub fn placement_id(r: u8, c: u8, d: u8) -> u32 {
    ((r as u32 * 9 + c as u32) * 9 + (d as u32 - 1))
}

pub fn decode_placement(id: u32) -> (Coord, u8) {
    let d = (id % 9) as u8 + 1;
    let cell = (id / 9) as usize;
    (Coord::from_index(cell), d)
}

fn constraint_columns(r: u8, c: u8, d: u8) -> [usize; 4] {
    let d = (d - 1) as usize;
    let (r, c, b) = (r as usize, c as usize, Coord::new(r, c).box_index());
    [
        r * 9 + c,       // cell
        81 + r * 9 + d,  // row-digit
        162 + c * 9 + d, // col-digit
        243 + b * 9 + d, // box-digit
    ]
}

/// Builds the exact-cover instance for `puzzle` (0 = empty cell).
///
/// Returns `None` if the givens themselves break classic rules — such a
/// board has no solutions at all, and encoding it naively could mask the
/// contradiction. Only *legal* placements are added: a given cell
/// contributes just its own digit; empty cells contribute digits not
/// blocked in their row/col/box.
fn build(puzzle: &Grid) -> Option<ExactCover> {
    // Validate the givens against each other.
    let mut check = Grid::new();
    for i in 0..CELLS {
        let c = Coord::from_index(i);
        let d = puzzle.get(c);
        if d != 0 {
            check.place(c, d).ok()?;
        }
    }

    let mut p = ExactCover::new(CONSTRAINTS);
    for i in 0..CELLS {
        let c = Coord::from_index(i);
        let given = puzzle.get(c);
        let digits: Vec<u8> = if given != 0 {
            vec![given]
        } else {
            (1..=9).filter(|&d| puzzle.candidates(c) & (1 << (d - 1)) != 0).collect()
        };
        for d in digits {
            p.add_row(placement_id(c.row, c.col, d), &constraint_columns(c.row, c.col, d));
        }
    }
    Some(p)
}

/// Counts solutions of `puzzle`, capped at `limit` (use 2 for uniqueness).
pub fn count_solutions(puzzle: &Grid, limit: u32) -> u32 {
    match build(puzzle) {
        Some(mut p) => p.count_solutions(limit),
        None => 0,
    }
}

/// Solves `puzzle` if it has exactly one solution.
pub fn solve_unique(puzzle: &Grid) -> Option<Grid> {
    let mut p = build(puzzle)?;
    if p.count_solutions(2) != 1 {
        return None;
    }
    let mut solved = puzzle.clone();
    for &id in p.first_solution()? {
        let (c, d) = decode_placement(id);
        // Givens are already in place; `place` would reject re-placing them.
        if solved.is_empty(c) {
            solved.place(c, d).expect("DLX solution must be legal");
        }
    }
    Some(solved)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOLUTION: &str = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn solves_known_puzzle_to_known_solution() {
        let puzzle = Grid::parse(EASY);
        let solved = solve_unique(&puzzle).expect("must be unique");
        assert_eq!(solved.cells(), Grid::parse(EASY_SOLUTION).cells());
    }

    #[test]
    fn unique_puzzle_counts_one() {
        assert_eq!(count_solutions(&Grid::parse(EASY), 2), 1);
    }

    #[test]
    fn empty_grid_has_many_solutions() {
        assert_eq!(count_solutions(&Grid::new(), 2), 2);
    }

    #[test]
    fn underconstrained_puzzle_counts_multiple() {
        // Only four givens from the solution — far too few to pin it down,
        // so at least two solutions must exist.
        let sol = Grid::parse(EASY_SOLUTION);
        let mut g = Grid::new();
        for i in [0usize, 10, 40, 80] {
            let c = Coord::from_index(i);
            g.place(c, sol.get(c)).unwrap();
        }
        assert_eq!(count_solutions(&g, 2), 2);
    }

    #[test]
    fn unsolvable_puzzle_counts_zero() {
        // Two 5s in the same box among the givens — a contradiction before
        // solving even starts. Built via the test-only unchecked constructor
        // because Grid's normal API cannot represent illegal boards.
        let text = "5300700005..195000098000060800060003400803001700020006060000280000419005000080079";
        assert_eq!(count_solutions(&Grid::parse_unchecked(text), 2), 0);
    }
}
