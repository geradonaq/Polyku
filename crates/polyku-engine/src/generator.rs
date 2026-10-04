//! The procedural puzzle generator.
//!
//! Two stages, both randomized:
//! 1. **Fill** the empty grid by backtracking with a shuffled digit order —
//!    produces a uniformly-shaped random *solution*.
//! 2. **Carve** clues away in random order. A clue is only removed if the
//!    puzzle still has exactly one solution afterwards (verified by the
//!    ruleset-aware solver). Every surviving puzzle is unique *by construction*.
//!
//! Variant data (killer cages, thermo paths) is derived from the generated
//! solution, then participates in the uniqueness checks — so stacked rule
//! combinations work with zero special cases.
//!
//! Difficulty in M2 is a clue-count target; milestone M3 replaces this with
//! technique-based grading and a no-guessing filter.

use crate::grid::{Coord, Grid, CELLS};
use crate::rules::{Cage, RuleData, RuleKind, RuleSet, ThermoPath};
use crate::solver;
use rand::seq::SliceRandom;
use rand::Rng;

/// Difficulty presets for M2 (clue-count based; technique-based in M3).
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

/// A finished product: what the player sees, plus the hidden solution
/// and the rule instances that shape it.
#[derive(Clone, Debug)]
pub struct Puzzle {
    /// The board with holes; 0 = empty cell.
    pub givens: [u8; CELLS],
    /// The unique solution.
    pub solution: [u8; CELLS],
    /// Serializable descriptions of the active rules (empty for classic).
    pub rules: Vec<RuleData>,
    pub difficulty: Difficulty,
    pub clue_count: usize,
}

impl Puzzle {
    pub fn grid(&self) -> Grid {
        grid_of(&self.givens)
    }

    /// Rebuilds the live rule objects from the puzzle's saved rule data.
    pub fn ruleset(&self) -> RuleSet {
        let mut set = RuleSet::new();
        for data in &self.rules {
            set.push(data.build());
        }
        set
    }
}

fn grid_of(cells: &[u8; CELLS]) -> Grid {
    let mut g = Grid::new();
    for (i, &d) in cells.iter().enumerate() {
        if d != 0 {
            g.place(Coord::from_index(i), d)
                .expect("sub-board of a valid solution must be legal");
        }
    }
    g
}

/// Generates a random solution under the classic rules plus any
/// *self-contained* rules in `rules` (diagonal, non-consecutive —
/// constraints that don't need to be derived from the solution first).
/// Killer/Thermo data is generated *from* the filled grid, so they
/// contribute nothing here.
fn fill_solution(rng: &mut impl Rng, rules: &RuleSet) -> [u8; CELLS] {
    crate::solver::random_solution(rules, rng)
}

/// Partitions all 81 cells into killer cages (2–4 cells, sometimes a
/// forced singleton) by random adjacent growth. Sums come from the solution.
fn generate_cages(rng: &mut impl Rng, solution: &[u8; CELLS]) -> Vec<Cage> {
    let mut owner = [-1i32; CELLS]; // cell -> cage id
    let mut cages: Vec<Cage> = Vec::new();

    let mut order: Vec<usize> = (0..CELLS).collect();
    order.shuffle(rng);

    for &seed in &order {
        if owner[seed] != -1 {
            continue;
        }
        let id = cages.len() as i32;
        let target = rng.gen_range(2..=4);
        let mut members = vec![seed];
        owner[seed] = id;
        // Killer rule: digits must not repeat within a cage — only grow
        // into cells whose solution digit isn't in the cage yet.
        let mut used_digits: u16 = 1 << (solution[seed] - 1);

        // Grow by random adjacent expansion.
        while members.len() < target {
            let mut frontier: Vec<usize> = Vec::new();
            for &m in &members {
                let c = Coord::from_index(m);
                if c.row > 0 {
                    frontier.push(m - 9);
                }
                if c.row < 8 {
                    frontier.push(m + 9);
                }
                if c.col > 0 {
                    frontier.push(m - 1);
                }
                if c.col < 8 {
                    frontier.push(m + 1);
                }
            }
            frontier.retain(|&f| {
                owner[f] == -1 && used_digits & (1 << (solution[f] - 1)) == 0
            });
            if frontier.is_empty() {
                break;
            }
            let pick = frontier[rng.gen_range(0..frontier.len())];
            owner[pick] = id;
            used_digits |= 1 << (solution[pick] - 1);
            members.push(pick);
        }

        cages.push(Cage {
            cells: members.iter().map(|&m| Coord::from_index(m)).collect(),
            sum: members.iter().map(|&m| solution[m] as u16).sum(),
        });
    }
    cages
}

/// Scatters random strictly-increasing thermo paths (length 3–6) that don't
/// overlap. Thermos don't need to cover the grid.
fn generate_thermos(rng: &mut impl Rng, solution: &[u8; CELLS]) -> Vec<ThermoPath> {
    let mut used = [false; CELLS];
    let mut paths: Vec<ThermoPath> = Vec::new();

    for _ in 0..10 {
        let start = rng.gen_range(0..CELLS);
        if used[start] {
            continue;
        }
        let mut path = vec![start];
        used[start] = true;
        let target_len = rng.gen_range(3..=6);

        while path.len() < target_len {
            let cur = *path.last().unwrap();
            let c = Coord::from_index(cur);
            let mut options: Vec<usize> = Vec::new();
            if c.row > 0 {
                options.push(cur - 9);
            }
            if c.row < 8 {
                options.push(cur + 9);
            }
            if c.col > 0 {
                options.push(cur - 1);
            }
            if c.col < 8 {
                options.push(cur + 1);
            }
            options.retain(|&o| !used[o] && solution[o] > solution[cur]);
            if options.is_empty() {
                break;
            }
            let pick = options[rng.gen_range(0..options.len())];
            used[pick] = true;
            path.push(pick);
        }

        if path.len() >= 3 {
            paths.push(path.iter().map(|&m| Coord::from_index(m)).collect());
        } else {
            // Too short to be interesting — release the cells again.
            for &m in &path {
                used[m] = false;
            }
        }
    }
    paths
}

fn build_rule_data(kind: RuleKind, rng: &mut impl Rng, solution: &[u8; CELLS]) -> RuleData {
    match kind {
        RuleKind::Diagonal => RuleData::Diagonal,
        RuleKind::Killer => RuleData::Killer { cages: generate_cages(rng, solution) },
        RuleKind::Thermo => RuleData::Thermo { paths: generate_thermos(rng, solution) },
        RuleKind::NonConsecutive => RuleData::NonConsecutive,
    }
}

/// Generates a classic puzzle (no additional rules).
pub fn generate(rng: &mut impl Rng, difficulty: Difficulty) -> Puzzle {
    generate_with_rules(rng, &[], difficulty)
}

/// Generates a puzzle under classic rules *plus* the requested rule kinds.
/// Variant data is derived from the random solution and participates in the
/// uniqueness checks, so any stack combination just works.
pub fn generate_with_rules(
    rng: &mut impl Rng,
    kinds: &[RuleKind],
    difficulty: Difficulty,
) -> Puzzle {
    // Phase 1: fill a solution under the self-contained rules only
    // (Killer/Thermo placeholders contribute nothing — their data comes
    // from the solution itself in phase 2).
    let mut phase1 = RuleSet::new();
    for &k in kinds {
        phase1.push(RuleData::from(k).build());
    }
    let solution = fill_solution(rng, &phase1);

    // Phase 2: derive the data-carrying rules (cages, thermo paths), then
    // carve clues with the *full* ruleset active in the uniqueness checks.
    let rule_data: Vec<RuleData> =
        kinds.iter().map(|&k| build_rule_data(k, rng, &solution)).collect();

    let mut ruleset = RuleSet::new();
    for data in &rule_data {
        ruleset.push(data.build());
    }

    let mut givens = solution;
    let mut clue_count = CELLS;
    let mut order: Vec<usize> = (0..CELLS).collect();
    order.shuffle(rng);

    // Variant puzzles stop digging at 30 clues: the additional rules already
    // carry the difficulty, and near-empty variant boards make the uniqueness
    // proofs (and the gameplay) disproportionate.
    let target = if kinds.is_empty() {
        difficulty.target_clues()
    } else {
        difficulty.target_clues().max(30)
    };

    for &i in &order {
        if clue_count <= target {
            break;
        }
        let kept = givens[i];
        givens[i] = 0;
        // Prove the puzzle is still unique; otherwise put the clue back.
        let board = grid_of(&givens);
        if solver::count_solutions(board.cells(), &ruleset, 2) != 1 {
            givens[i] = kept;
        } else {
            clue_count -= 1;
        }
    }

    Puzzle {
        givens,
        solution,
        rules: rule_data,
        difficulty,
        clue_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    /// Verifies a generated puzzle end to end: unique solution, the stored
    /// solution actually solves it, and every active rule is satisfied by it.
    fn assert_valid(p: &Puzzle) {
        let board = p.grid();
        let ruleset = p.ruleset();
        assert_eq!(
            solver::count_solutions(board.cells(), &ruleset, 2),
            1,
            "{:?} puzzle with {} clues was not unique",
            p.difficulty,
            p.clue_count
        );
        let solved = solver::solve_unique(board.cells(), &ruleset).unwrap();
        assert_eq!(&solved, &p.solution);
        assert!(ruleset.is_consistent(&p.solution), "stored solution must satisfy its rules");
    }

    #[test]
    fn solution_is_complete_and_valid() {
        let mut rng = thread_rng();
        let cells = fill_solution(&mut rng, &RuleSet::new());
        let g = grid_of(&cells);
        assert!(g.cells().iter().all(|&d| (1..=9).contains(&d)));
        assert_eq!(crate::sudoku::count_solutions(&g, 2), 1);
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
    fn killer_cages_partition_the_grid() {
        let mut rng = thread_rng();
        let p = generate_with_rules(&mut rng, &[RuleKind::Killer], Difficulty::Medium);
        let mut covered = vec![false; CELLS];
        let mut total = 0u16;
        for data in &p.rules {
            let RuleData::Killer { cages } = data else { panic!("expected killer") };
            for cage in cages {
                for c in &cage.cells {
                    assert!(!covered[c.index()], "cell in two cages");
                    covered[c.index()] = true;
                }
                let actual: u16 = cage.cells.iter().map(|c| p.solution[c.index()] as u16).sum();
                assert_eq!(actual, cage.sum, "cage sum must match solution");
                total += cage.sum;
            }
        }
        assert!(covered.iter().all(|&c| c), "cages must cover all 81 cells");
        assert_eq!(total, 405); // 1+2+…+9 per row × 9 rows
    }

    #[test]
    fn thermo_paths_strictly_increase() {
        let mut rng = thread_rng();
        let p = generate_with_rules(&mut rng, &[RuleKind::Thermo], Difficulty::Medium);
        for data in &p.rules {
            let RuleData::Thermo { paths } = data else { panic!("expected thermo") };
            for path in paths {
                assert!(path.len() >= 3);
                for w in path.windows(2) {
                    assert!(
                        p.solution[w[1].index()] > p.solution[w[0].index()],
                        "thermo must strictly increase along the path"
                    );
                }
            }
        }
    }

    #[test]
    fn generated_puzzles_are_always_unique() {
        // Release builds (CI) run the full 100-puzzle Hard sweep; debug keeps
        // a small Medium sample so everyday `cargo test` stays fast.
        let (n, difficulty) = if cfg!(debug_assertions) {
            (2, Difficulty::Medium)
        } else {
            (100, Difficulty::Hard)
        };
        let mut rng = thread_rng();
        let kinds_pool = [
            Vec::new(),
            vec![RuleKind::Diagonal],
            vec![RuleKind::Killer],
            vec![RuleKind::Thermo],
            vec![RuleKind::NonConsecutive],
            vec![RuleKind::Diagonal, RuleKind::Killer], // stacked
        ];
        for _ in 0..n {
            let kinds = kinds_pool.choose(&mut rng).unwrap();
            let p = generate_with_rules(&mut rng, kinds, difficulty);
            assert_valid(&p);
        }
    }

    #[test]
    fn each_variant_and_stacked_combo_generates() {
        let difficulty = if cfg!(debug_assertions) {
            Difficulty::Medium
        } else {
            Difficulty::Hard
        };
        let mut rng = thread_rng();
        let cases: [&[RuleKind]; 6] = [
            &[],
            &[RuleKind::Diagonal],
            &[RuleKind::Killer],
            &[RuleKind::Thermo],
            &[RuleKind::NonConsecutive],
            &[RuleKind::Diagonal, RuleKind::Killer],
        ];
        for kinds in cases {
            assert_valid(&generate_with_rules(&mut rng, kinds, difficulty));
        }
    }
}
