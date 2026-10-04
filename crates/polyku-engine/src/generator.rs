//! The procedural puzzle generator.
//!
//! Two stages, both randomized:
//! 1. **Fill** the empty grid by backtracking with a shuffled digit order —
//!    produces a uniformly-shaped random *solution*.
//! 2. **Carve** clues away in random order. A clue is only removed if the
//!    puzzle still has exactly one solution afterwards (verified by DLX).
//!    Every surviving puzzle is therefore unique *by construction*.
//!
//! Difficulty in M1 is a clue-count target: Easy stops carving early,
//! Hard digs as far as randomness allows. Milestone M3 replaces this with
//! technique-based grading and a no-guessing filter.

use crate::grid::{Coord, Grid, CELLS};
use crate::sudoku;
use rand::seq::SliceRandom;
use rand::Rng;

/// Difficulty presets for M1 (clue-count based; technique-based in M3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Carving stops once this many clues remain.
    /// `Hard` digs to the random minimum (~22–26 clues).
    fn target_clues(self) -> usize {
        match self {
            Difficulty::Easy => 38,
            Difficulty::Medium => 32,
            Difficulty::Hard => 0,
        }
    }
}

/// A finished product: what the player sees, plus the hidden solution.
#[derive(Clone, Debug)]
pub struct Puzzle {
    /// The board with holes; 0 = empty cell.
    pub givens: [u8; CELLS],
    /// The unique solution.
    pub solution: [u8; CELLS],
    pub difficulty: Difficulty,
    pub clue_count: usize,
}

impl Puzzle {
    pub fn grid(&self) -> Grid {
        let mut g = Grid::new();
        for i in 0..CELLS {
            if self.givens[i] != 0 {
                g.place(Coord::from_index(i), self.givens[i])
                    .expect("generated givens must be legal");
            }
        }
        g
    }
}

/// Fills an empty grid completely, choosing digits in random order —
/// a randomized backtracking that always succeeds for 9×9.
fn fill_solution(rng: &mut impl Rng) -> [u8; CELLS] {
    let mut grid = Grid::new();
    let mut digits = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];

    fn backtrack(grid: &mut Grid, pos: usize, digits: &mut [u8; 9], rng: &mut impl Rng) -> bool {
        if pos == CELLS {
            return true;
        }
        let c = Coord::from_index(pos);
        if !grid.is_empty(c) {
            return backtrack(grid, pos + 1, digits, rng);
        }
        digits.shuffle(rng);
        // Index-based loop on purpose: copying each digit out ends the
        // borrow immediately, so the recursive call below can take `&mut`.
        for k in 0..digits.len() {
            let d = digits[k];
            if grid.is_allowed(c, d) {
                grid.place(c, d).expect("checked allowed");
                if backtrack(grid, pos + 1, digits, rng) {
                    return true;
                }
                grid.remove(c);
            }
        }
        false
    }

    backtrack(&mut grid, 0, &mut digits, rng);
    *grid.cells()
}

/// Generates a puzzle with a strictly unique solution.
pub fn generate(rng: &mut impl Rng, difficulty: Difficulty) -> Puzzle {
    let solution = fill_solution(rng);
    let mut givens = solution;

    let mut clue_count = CELLS;
    let mut order: Vec<usize> = (0..CELLS).collect();
    order.shuffle(rng);

    for &i in &order {
        if clue_count <= difficulty.target_clues() {
            break;
        }
        let kept = givens[i];
        givens[i] = 0;
        // Prove the puzzle is still unique; otherwise put the clue back.
        let board = board_of(&givens);
        if sudoku::count_solutions(&board, 2) != 1 {
            givens[i] = kept;
        } else {
            clue_count -= 1;
        }
    }

    Puzzle {
        givens,
        solution,
        difficulty,
        clue_count,
    }
}

fn board_of(cells: &[u8; CELLS]) -> Grid {
    let mut g = Grid::new();
    for (i, &d) in cells.iter().enumerate() {
        if d != 0 {
            g.place(Coord::from_index(i), d)
                .expect("sub-board of a valid solution must be legal");
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn solution_is_complete_and_valid() {
        let mut rng = thread_rng();
        let cells = fill_solution(&mut rng);
        let g = board_of(&cells);
        assert!(g.cells().iter().all(|&d| (1..=9).contains(&d)));
        // A complete, valid grid has exactly one solution: itself.
        assert_eq!(sudoku::count_solutions(&g, 2), 1);
    }

    #[test]
    fn easy_and_medium_hit_their_clue_targets() {
        let mut rng = thread_rng();
        let easy = generate(&mut rng, Difficulty::Easy);
        assert_eq!(easy.clue_count, 38);
        let medium = generate(&mut rng, Difficulty::Medium);
        assert_eq!(medium.clue_count, 32);
    }

    #[test]
    fn hard_digs_deep() {
        let mut rng = thread_rng();
        let hard = generate(&mut rng, Difficulty::Hard);
        assert!(hard.clue_count <= 30, "hard should dig to ~22-26, got {}", hard.clue_count);
    }

    #[test]
    fn generated_puzzles_are_always_unique() {
        // Full 100-puzzle sweep in release builds (CI); a quick sample in debug.
        let n = if cfg!(debug_assertions) { 5 } else { 100 };
        let mut rng = thread_rng();
        for _ in 0..n {
            let d = *[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
                .choose(&mut rng)
                .unwrap();
            let p = generate(&mut rng, d);
            assert_eq!(
                sudoku::count_solutions(&p.grid(), 2),
                1,
                "{d:?} puzzle with {} clues was not unique",
                p.clue_count
            );
            // And the stored solution must actually solve it.
            let solved = sudoku::solve_unique(&p.grid()).unwrap();
            assert_eq!(solved.cells(), &p.solution);
        }
    }
}
