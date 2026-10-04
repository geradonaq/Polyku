//! The generalized backtracking solver: classic rules + any [`RuleSet`].
//!
//! DLX (see [`crate::dlx`]) remains the fast specialist for *pure classic*
//! boards. Killer cages and thermo paths are not plain "exactly-one"
//! constraints, so variants use this solver instead: depth-first search with
//! MRV cell choice and per-node candidate pruning by the active rules.
//! Solution counting with a limit of 2 is the uniqueness proof, same as M1.

use crate::grid::{Coord, Grid, DigitMask, CELLS};
use crate::rules::RuleSet;
use rand::seq::SliceRandom;
use rand::Rng;

/// Counts solutions of `cells` (0 = empty) under the classic rules plus
/// `rules`, capped at `limit`.
pub fn count_solutions(cells: &[u8; CELLS], rules: &RuleSet, limit: u32) -> u32 {
    let mut state = SearchState {
        grid: build_grid(cells),
        rules,
        limit,
        found: 0,
        first: None,
    };
    let mut cells = *cells;
    state.search(&mut cells);
    state.found
}

/// Solves `cells` if the position has exactly one solution.
pub fn solve_unique(cells: &[u8; CELLS], rules: &RuleSet) -> Option<[u8; CELLS]> {
    let mut state = SearchState {
        grid: build_grid(cells),
        rules,
        limit: 2,
        found: 0,
        first: None,
    };
    let mut cells = *cells;
    state.search(&mut cells);
    if state.found == 1 {
        state.first
    } else {
        None
    }
}

fn build_grid(cells: &[u8; CELLS]) -> Grid {
    let mut grid = Grid::new();
    for (i, &d) in cells.iter().enumerate() {
        if d != 0 {
            grid.place(Coord::from_index(i), d)
                .expect("search input must respect classic rules");
        }
    }
    grid
}

/// Fills an empty grid to a complete random solution under the classic
/// rules plus `rules`, using MRV + propagation with randomized choices.
///
/// Fixed-order filling (the classic-generation workhorse) collapses
/// catastrophically under extra constraints like the diagonal — search
/// with the full machinery is both fast and unbiased enough for our
/// "fresh random board every time" guarantee.
pub fn random_solution(rules: &RuleSet, rng: &mut impl Rng) -> [u8; CELLS] {
    let mut cells = [0u8; CELLS];
    let mut grid = Grid::new();
    let ok = fill_backtrack(&mut cells, &mut grid, rules, rng);
    assert!(ok, "no solution exists for these rules");
    cells
}

fn fill_backtrack(
    cells: &mut [u8; CELLS],
    grid: &mut Grid,
    rules: &RuleSet,
    rng: &mut impl Rng,
) -> bool {
    let mut assigned: Vec<usize> = Vec::new();
    let mut complete = false;
    let mut ok = true;

    loop {
        let mut cands = [0 as DigitMask; CELLS];
        let mut any_empty = false;
        for i in 0..CELLS {
            if cells[i] == 0 {
                cands[i] = grid.candidates(Coord::from_index(i));
                any_empty = true;
            }
        }
        if !any_empty {
            complete = true;
            break;
        }
        rules.prune(cells, &mut cands);

        let mut forced: Option<usize> = None;
        let mut dead = false;
        let mut min_count = u32::MAX;
        let mut min_cells: Vec<usize> = Vec::new();
        for i in 0..CELLS {
            if cells[i] != 0 {
                continue;
            }
            match cands[i].count_ones() {
                0 => {
                    dead = true;
                    break;
                }
                1 => {
                    forced = Some(i);
                    break;
                }
                n => {
                    if n < min_count {
                        min_count = n;
                        min_cells.clear();
                    }
                    if n == min_count {
                        min_cells.push(i);
                    }
                }
            }
        }
        if dead {
            ok = false;
            break;
        }
        match forced {
            Some(i) => {
                let d = cands[i].trailing_zeros() as u8 + 1;
                let c = Coord::from_index(i);
                grid.place(c, d).expect("forced candidate is classic-legal");
                cells[i] = d;
                assigned.push(i);
            }
            None => {
                // Random branch point among the most-constrained cells,
                // trying digits in random order — this is where the
                // "absolutely random" flavor of the solution comes from.
                let i = min_cells[rng.gen_range(0..min_cells.len())];
                let mask = cands[i];
                let mut digits: Vec<u8> =
                    (1..=9).filter(|&d| mask & (1 << (d - 1)) != 0).collect();
                digits.shuffle(rng);
                let c = Coord::from_index(i);
                for d in digits {
                    grid.place(c, d).expect("candidate came from the grid itself");
                    cells[i] = d;
                    if fill_backtrack(cells, grid, rules, rng) {
                        return true; // keep this level's assignments — part of the solution
                    }
                    cells[i] = 0;
                    grid.remove(c);
                }
                ok = false; // every option failed
                break;
            }
        }
    }

    if complete && !rules.is_consistent(cells) {
        ok = false; // safety net; propagation should never allow this
    }
    if !(ok && complete) {
        for &i in assigned.iter().rev() {
            let c = Coord::from_index(i);
            grid.remove(c);
            cells[i] = 0;
        }
        return false;
    }
    true
}

struct SearchState<'a> {
    grid: Grid,
    rules: &'a RuleSet,
    limit: u32,
    found: u32,
    first: Option<[u8; CELLS]>,
}

impl SearchState<'_> {
    /// Depth-first search with **propagation to a fixpoint**: before ever
    /// guessing, every forced cell (exactly one candidate left after rule
    /// pruning) is assigned and the prune cascade re-runs. This collapses
    /// the vast majority of the tree on constrained boards — the difference
    /// between seconds and geological time when proving uniqueness.
    fn search(&mut self, cells: &mut [u8; CELLS]) {
        if self.found >= self.limit {
            return;
        }

        // Cells assigned during this level's propagation — undone on unwind.
        let mut assigned: Vec<usize> = Vec::new();
        let mut dead = false;

        loop {
            let mut cands = [0 as DigitMask; CELLS];
            let mut any_empty = false;
            for i in 0..CELLS {
                if cells[i] == 0 {
                    cands[i] = self.grid.candidates(Coord::from_index(i));
                    any_empty = true;
                }
            }
            if !any_empty {
                // Leaf: a complete board. Rules get the final say.
                if self.rules.is_consistent(cells) {
                    self.found += 1;
                    if self.first.is_none() {
                        self.first = Some(*cells);
                    }
                }
                break;
            }
            self.rules.prune(cells, &mut cands);

            // Scan: forced cell (1 candidate), contradiction (0), or the
            // most-constrained cell to branch on.
            let mut forced: Option<usize> = None;
            let mut branch = usize::MAX;
            let mut branch_count = u32::MAX;
            for i in 0..CELLS {
                if cells[i] != 0 {
                    continue;
                }
                match cands[i].count_ones() {
                    0 => {
                        dead = true;
                        break;
                    }
                    1 => {
                        forced = Some(i);
                        break;
                    }
                    n if n < branch_count => {
                        branch_count = n;
                        branch = i;
                    }
                    _ => {}
                }
            }
            if dead {
                break;
            }
            match forced {
                Some(i) => {
                    let d = cands[i].trailing_zeros() as u8 + 1;
                    let c = Coord::from_index(i);
                    self.grid.place(c, d).expect("forced candidate is classic-legal");
                    cells[i] = d;
                    assigned.push(i);
                    // Loop again — this assignment may cascade further.
                }
                None => {
                    // Nothing forced: branch on the most-constrained cell.
                    let c = Coord::from_index(branch);
                    let mask = cands[branch];
                    for d in 1..=9u8 {
                        if mask & (1 << (d - 1)) == 0 {
                            continue;
                        }
                        self.grid.place(c, d).expect("candidate came from the grid itself");
                        cells[branch] = d;
                        self.search(cells);
                        cells[branch] = 0;
                        self.grid.remove(c);
                        if self.found >= self.limit {
                            break;
                        }
                    }
                    break;
                }
            }
        }

        // Undo this level's propagation assignments; branch assignments were
        // already undone inside their loop.
        for &i in assigned.iter().rev() {
            let c = Coord::from_index(i);
            self.grid.remove(c);
            cells[i] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{diagonal::Diagonal, RuleData};

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn classic_boards_match_the_dlx_solver() {
        // Cross-validation: with an empty RuleSet, the generalized solver
        // must agree with the DLX exact-cover solver.
        assert_eq!(
            count_solutions(&Grid::parse(EASY).cells(), &RuleSet::new(), 2),
            crate::sudoku::count_solutions(&Grid::parse(EASY), 2)
        );
        assert_eq!(
            count_solutions(&Grid::new().cells(), &RuleSet::new(), 2),
            crate::sudoku::count_solutions(&Grid::new(), 2)
        );
    }

    #[test]
    fn solves_a_classic_board_correctly() {
        let board = Grid::parse(EASY);
        let solved = solve_unique(board.cells(), &RuleSet::new()).expect("unique");
        let expected = crate::sudoku::solve_unique(&board).unwrap();
        assert_eq!(&solved, expected.cells());
    }

    #[test]
    fn diagonal_rule_rejects_what_classic_allows() {
        // This completed grid is a valid classic solution, but its main
        // diagonal reads 1,5,9,3,9,4,9,3,2 — duplicates — so as X-sudoku
        // the leaf must be rejected even though classic counting accepts it.
        let cells = *Grid::parse(
            "123456789456789123789123456214365897365897214897214365531642978642978531978531642",
        )
        .cells();
        assert_eq!(count_solutions(&cells, &RuleSet::new(), 1), 1);
        let mut rules = RuleSet::new();
        rules.push(Box::new(Diagonal));
        assert_eq!(count_solutions(&cells, &rules, 2), 0);
    }

    #[test]
    fn rule_data_round_trips_into_a_ruleset() {
        let data = RuleData::Diagonal;
        let mut rules = RuleSet::new();
        rules.push(data.build());
        assert_eq!(rules.overlays().len(), 2);
    }
}
