//! The human-logic deduction engine.
//!
//! Solves sudoku the way a person does: a **ladder of techniques** ordered
//! from simplest to hardest, always applying the simplest technique that
//! makes progress. What falls out of this:
//!
//! - **Honest difficulty** — a puzzle's grade is the hardest technique it
//!   actually requires (six tiers, Beginner → Master), not a clue count.
//! - **The no-guessing guarantee** — if the ladder can't finish a puzzle,
//!   it isn't human-solvable and the generator throws it away.
//! - **Hints that teach** — the next step comes with the technique name,
//!   the cells involved, and a plain-language explanation.
//!
//! Variant rules participate too: rules prune candidates (tier 0, "free"
//! logic like cage arithmetic), and diagonals join the unit list so
//! X-sudoku gets full technique coverage on its extra units.

use crate::grid::{DigitMask, CELLS};
use crate::rules::RuleSet;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Board geometry: units
// ---------------------------------------------------------------------------

/// What kind of unit a group of 9 cells is — used for hint text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitKind {
    Row,
    Column,
    Box,
    Diagonal,
}

/// A unit: 9 cells that must together hold all digits once.
#[derive(Clone, Debug)]
pub struct Unit {
    pub kind: UnitKind,
    pub index: usize, // 0-based row/col/box number
    pub cells: Vec<usize>,
}

impl Unit {
    fn name(&self) -> String {
        let kind = match self.kind {
            UnitKind::Row => "row",
            UnitKind::Column => "column",
            UnitKind::Box => "box",
            UnitKind::Diagonal => "diagonal",
        };
        if self.kind == UnitKind::Diagonal {
            kind.to_string()
        } else {
            format!("{kind} {}", self.index + 1)
        }
    }
}

/// All units relevant to a puzzle: the 27 classic units plus diagonals when
/// the Diagonal rule is active.
pub struct DeductionCtx {
    pub units: Vec<Unit>,
    /// Classic peers of each cell: same row, column, or box.
    pub peers: Vec<Vec<usize>>,
}

impl DeductionCtx {
    /// Builds the context for a puzzle's rule set (used via `RuleData`).
    pub fn for_rules(rules: &[crate::rules::RuleData]) -> DeductionCtx {
        let mut units = classic_units();
        if rules.iter().any(|r| matches!(r, crate::rules::RuleData::Diagonal)) {
            units.push(Unit {
                kind: UnitKind::Diagonal,
                index: 0,
                cells: (0..9).map(|k| k * 9 + k).collect(),
            });
            units.push(Unit {
                kind: UnitKind::Diagonal,
                index: 1,
                cells: (0..9).map(|k| k * 9 + (8 - k)).collect(),
            });
        }
        DeductionCtx { units, peers: classic_peers() }
    }

    /// Row / column / box a cell index belongs to.
    pub fn unit_of_kind(&self, cell: usize, kind: UnitKind) -> usize {
        match kind {
            UnitKind::Row => cell / 9,
            UnitKind::Column => cell % 9,
            UnitKind::Box => (cell / 27) * 3 + (cell % 9) / 3,
            UnitKind::Diagonal => if cell % 10 == 0 { 0 } else { 1 }, // (r,r) main
        }
    }
}

fn classic_units() -> Vec<Unit> {
    let mut units = Vec::with_capacity(27);
    for r in 0..9 {
        units.push(Unit { kind: UnitKind::Row, index: r, cells: (0..9).map(|c| r * 9 + c).collect() });
    }
    for c in 0..9 {
        units.push(Unit {
            kind: UnitKind::Column,
            index: c,
            cells: (0..9).map(|r| r * 9 + c).collect(),
        });
    }
    for b in 0..9 {
        let (r0, c0) = ((b / 3) * 3, (b % 3) * 3);
        units.push(Unit {
            kind: UnitKind::Box,
            index: b,
            cells: (0..9)
                .map(|k| (r0 + k / 3) * 9 + c0 + k % 3)
                .collect(),
        });
    }
    units
}

fn classic_peers() -> Vec<Vec<usize>> {
    let mut peers = vec![Vec::new(); CELLS];
    for i in 0..CELLS {
        let (r, c) = (i / 9, i % 9);
        let (br, bc) = (r / 3 * 3, c / 3 * 3);
        for j in 0..CELLS {
            if j == i {
                continue;
            }
            let (rj, cj) = (j / 9, j % 9);
            if rj == r || cj == c || (rj / 3 == br / 3 && cj / 3 == bc / 3) {
                peers[i].push(j);
            }
        }
    }
    peers
}

// ---------------------------------------------------------------------------
// Techniques
// ---------------------------------------------------------------------------

/// The technique ladder, simplest first. Tier 1 = Beginner … tier 6 = Master.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Technique {
    NakedSingle,
    HiddenSingle,
    PointingClaiming,
    NakedPair,
    HiddenPair,
    NakedTriple,
    HiddenTriple,
    NakedQuad,
    XWing,
    Swordfish,
    YWing,
    SimpleColoring,
    Jellyfish,
}

pub const LADDER: [Technique; 13] = [
    Technique::NakedSingle,
    Technique::HiddenSingle,
    Technique::PointingClaiming,
    Technique::NakedPair,
    Technique::HiddenPair,
    Technique::NakedTriple,
    Technique::HiddenTriple,
    Technique::NakedQuad,
    Technique::XWing,
    Technique::Swordfish,
    Technique::YWing,
    Technique::SimpleColoring,
    Technique::Jellyfish,
];

impl Technique {
    pub fn tier(self) -> u8 {
        match self {
            Technique::NakedSingle => 1,
            Technique::HiddenSingle | Technique::PointingClaiming => 2,
            Technique::NakedPair | Technique::HiddenPair | Technique::NakedTriple
            | Technique::HiddenTriple => 3,
            Technique::NakedQuad | Technique::XWing => 4,
            Technique::Swordfish | Technique::YWing | Technique::SimpleColoring => 5,
            Technique::Jellyfish => 6,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Technique::NakedSingle => "Naked Single",
            Technique::HiddenSingle => "Hidden Single",
            Technique::PointingClaiming => "Pointing / Claiming",
            Technique::NakedPair => "Naked Pair",
            Technique::HiddenPair => "Hidden Pair",
            Technique::NakedTriple => "Naked Triple",
            Technique::HiddenTriple => "Hidden Triple",
            Technique::NakedQuad => "Naked Quad",
            Technique::XWing => "X-Wing",
            Technique::Swordfish => "Swordfish",
            Technique::YWing => "Y-Wing",
            Technique::SimpleColoring => "Simple Coloring",
            Technique::Jellyfish => "Jellyfish",
        }
    }
}

/// One deduction step: what to do, why, and what to show.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hint {
    pub technique: String,
    pub tier: u8,
    /// (cell index, digit) pairs to place.
    pub placements: Vec<(usize, u8)>,
    /// (cell index, digit) candidates to remove.
    pub eliminations: Vec<(usize, u8)>,
    /// Cells involved in the pattern (hint highlighting).
    pub highlight_cells: Vec<usize>,
    /// Digits involved in the pattern.
    pub highlight_digits: Vec<u8>,
    pub explanation: String,
}

/// Mutable deduction state: placements plus a candidate matrix that can
/// shrink beyond what the classic masks imply (eliminations persist).
#[derive(Clone)]
pub struct DeductionState {
    pub cells: [u8; CELLS],
    pub cands: [DigitMask; CELLS],
}

impl DeductionState {
    /// Builds state from a (partial, classic-legal) board.
    pub fn new(cells: &[u8; CELLS], rules: &RuleSet) -> DeductionState {
        let mut grid = crate::grid::Grid::new();
        for (i, &d) in cells.iter().enumerate() {
            if d != 0 {
                grid.place(crate::grid::Coord::from_index(i), d)
                    .expect("deduction input must be classic-legal");
            }
        }
        let mut cands = [0 as DigitMask; CELLS];
        for i in 0..CELLS {
            if cells[i] == 0 {
                cands[i] = grid.candidates(crate::grid::Coord::from_index(i));
            }
        }
        rules.prune(cells, &mut cands);
        DeductionState { cells: *cells, cands }
    }

    pub fn is_solved(&self) -> bool {
        self.cells.iter().all(|&d| d != 0)
    }

    /// Places a digit: fills the cell and clears the digit from classic
    /// peers. Variant implications are re-derived by the prune sweep that
    /// runs at the top of every ladder iteration.
    fn place(&mut self, ctx: &DeductionCtx, cell: usize, d: u8) {
        self.cells[cell] = d;
        self.cands[cell] = 0;
        let bit = !(1 << (d - 1));
        for &p in &ctx.peers[cell] {
            self.cands[p] &= bit;
        }
    }

    fn apply(&mut self, ctx: &DeductionCtx, hint: &Hint) {
        for (cell, d) in &hint.placements {
            self.place(ctx, *cell, *d);
        }
        for (cell, d) in &hint.eliminations {
            self.cands[*cell] &= !(1 << (d - 1));
        }
    }
}

/// Runs one rule-prune sweep over the candidate matrix.
/// Returns true if anything changed (a "tier 0" variant-logic step).
fn prune_sweep(state: &mut DeductionState, rules: &RuleSet) -> bool {
    let before = state.cands;
    rules.prune(&state.cells, &mut state.cands);
    before != state.cands
}

// ---------------------------------------------------------------------------
// The engine
// ---------------------------------------------------------------------------

/// Result of solving a puzzle through the ladder.
#[derive(Clone, Debug)]
pub struct Grade {
    pub solved: bool,
    /// Hardest technique tier that was actually needed (0 if already solved).
    pub max_tier: u8,
    pub steps: usize,
}

pub struct DeductionEngine<'a> {
    pub ctx: &'a DeductionCtx,
    pub rules: &'a RuleSet,
    /// Cap on technique tiers the ladder may use (the no-guess dial).
    pub max_tier: u8,
}

impl DeductionEngine<'_> {
    /// Solves `state` as far as the ladder reaches within `max_tier`.
    /// When `solved` is true, the puzzle is human-deducible without guessing.
    pub fn solve(&self, state: &mut DeductionState) -> Grade {
        let mut max_tier = 0u8;
        let mut steps = 0usize;
        loop {
            if state.is_solved() {
                return Grade { solved: true, max_tier, steps };
            }
            if prune_sweep(state, self.rules) {
                continue; // variant logic made progress — free, tier 0
            }
            match self.next_step(state) {
                Some(hint) => {
                    max_tier = max_tier.max(hint.tier);
                    steps += 1;
                    state.apply(self.ctx, &hint);
                }
                None => {
                    return Grade { solved: false, max_tier, steps };
                }
            }
        }
    }

    /// The next step the ladder would take, as a teachable hint.
    pub fn hint(&self, state: &DeductionState) -> Option<Hint> {
        self.next_step(state)
    }

    /// First technique (simplest first, capped at `max_tier`) that fires.
    fn next_step(&self, state: &DeductionState) -> Option<Hint> {
        for tech in LADDER {
            if tech.tier() > self.max_tier {
                break;
            }
            let step = match tech {
                Technique::NakedSingle => naked_single(state),
                Technique::HiddenSingle => hidden_single(state, self.ctx),
                Technique::PointingClaiming => pointing_claiming(state, self.ctx),
                Technique::NakedPair => naked_group(state, self.ctx, 2),
                Technique::HiddenPair => hidden_group(state, self.ctx, 2),
                Technique::NakedTriple => naked_group(state, self.ctx, 3),
                Technique::HiddenTriple => hidden_group(state, self.ctx, 3),
                Technique::NakedQuad => naked_group(state, self.ctx, 4),
                Technique::XWing => fish(state, 2),
                Technique::Swordfish => fish(state, 3),
                Technique::YWing => y_wing(state, self.ctx),
                Technique::SimpleColoring => simple_coloring(state, self.ctx),
                Technique::Jellyfish => fish(state, 4),
            };
            if let Some(h) = step {
                return Some(h);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Technique implementations
// ---------------------------------------------------------------------------

/// Digit mask helper: iterate set digits.
pub fn digits_of(mask: DigitMask) -> Vec<u8> {
    (1..=9).filter(|&d| mask & (1 << (d - 1)) != 0).collect()
}

/// All k-combinations of `items` (sizes here are ≤ 9, so brute materializing
/// is trivially cheap).
fn combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    fn rec<T: Clone>(
        items: &[T],
        k: usize,
        start: usize,
        current: &mut Vec<T>,
        out: &mut Vec<Vec<T>>,
    ) {
        if current.len() == k {
            out.push(current.clone());
            return;
        }
        for i in start..items.len() {
            current.push(items[i].clone());
            rec(items, k, i + 1, current, out);
            current.pop();
        }
    }
    rec(items, k, 0, &mut current, &mut out);
    out
}

fn cell_name(cell: usize) -> String {
    format!("r{}c{}", cell / 9 + 1, cell % 9 + 1)
}

fn digits_text(digits: &[u8]) -> String {
    digits.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")
}

fn naked_single(state: &DeductionState) -> Option<Hint> {
    for i in 0..CELLS {
        if state.cells[i] == 0 && state.cands[i].count_ones() == 1 {
            let d = state.cands[i].trailing_zeros() as u8 + 1;
            return Some(Hint {
                technique: Technique::NakedSingle.name().into(),
                tier: 1,
                placements: vec![(i, d)],
                eliminations: vec![],
                highlight_cells: vec![i],
                highlight_digits: vec![d],
                explanation: format!(
                    "Cell {} has only one candidate left: {}.",
                    cell_name_from_index(i),
                    d
                ),
            });
        }
    }
    None
}

fn cell_name_from_index(i: usize) -> String {
    format!("r{}c{}", i / 9 + 1, i % 9 + 1)
}

fn hidden_single(state: &DeductionState, ctx: &DeductionCtx) -> Option<Hint> {
    for unit in &ctx.units {
        for d in 1..=9u8 {
            let bit = 1 << (d - 1);
            let spots: Vec<usize> = unit
                .cells
                .iter()
                .filter(|&&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
                .copied()
                .collect();
            if spots.len() == 1 {
                let i = spots[0];
                return Some(Hint {
                    technique: Technique::HiddenSingle.name().into(),
                    tier: 2,
                    placements: vec![(i, d)],
                    eliminations: vec![],
                    highlight_cells: unit.cells.clone(),
                    highlight_digits: vec![d],
                    explanation: format!(
                        "In {}, {} is the only cell that can hold {}.",
                        unit.name(),
                        cell_name(i),
                        d
                    ),
                });
            }
        }
    }
    None
}

/// Naked pairs / triples / quads: k cells in a unit sharing exactly k
/// candidates — those digits belong to them, so they leave the rest.
fn naked_group(state: &DeductionState, ctx: &DeductionCtx, k: usize) -> Option<Hint> {
    let tech = match k {
        2 => Technique::NakedPair,
        3 => Technique::NakedTriple,
        _ => Technique::NakedQuad,
    };
    for unit in &ctx.units {
        let free: Vec<usize> = unit
            .cells
            .iter()
            .filter(|&&i| state.cells[i] == 0 && state.cands[i].count_ones() >= 2)
            .copied()
            .collect();
        if free.len() <= k {
            continue;
        }
        for combo in combinations(&free, k) {
            let mut union = 0 as DigitMask;
            for &i in &combo {
                union |= state.cands[i];
            }
            if union.count_ones() as usize != k {
                continue;
            }
            let mut eliminations = Vec::new();
            for &i in &free {
                if combo.contains(&i) {
                    continue;
                }
                for d in digits_of(state.cands[i] & union) {
                    eliminations.push((i, d));
                }
            }
            if eliminations.is_empty() {
                continue;
            }
            let digits = digits_of(union);
            return Some(Hint {
                technique: tech.name().into(),
                tier: tech.tier(),
                placements: vec![],
                eliminations,
                highlight_cells: combo.iter().copied().collect(),
                highlight_digits: digits.clone(),
                explanation: format!(
                    "{} in {} holds {{{}}}; those digits belong to it and can leave every other cell there.",
                    tech.name(),
                    unit.name(),
                    digits_text(&digits)
                ),
            });
        }
    }
    None
}

/// Hidden pairs / triples: k digits confined to the same k cells in a unit —
/// every other candidate in those cells must go.
fn hidden_group(state: &DeductionState, ctx: &DeductionCtx, k: usize) -> Option<Hint> {
    let tech = match k {
        2 => Technique::HiddenPair,
        _ => Technique::HiddenTriple,
    };
    for unit in &ctx.units {
        let mut digit_spots: Vec<(u8, Vec<usize>)> = Vec::new();
        for d in 1..=9u8 {
            let bit = 1 << (d - 1);
            let spots: Vec<usize> = unit
                .cells
                .iter()
                .filter(|&&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
                .copied()
                .collect();
            if spots.len() >= 2 {
                digit_spots.push((d, spots));
            }
        }
        if digit_spots.len() < k {
            continue;
        }
        let digit_list: Vec<u8> = digit_spots.iter().map(|&(d, _)| d).collect();
        for combo in combinations(&digit_list, k) {
            let mut cells_set: Vec<usize> = Vec::new();
            let mut keep = 0 as DigitMask;
            for &d in &combo {
                keep |= 1 << (d - 1);
                for &i in &digit_spots.iter().find(|&&(dd, _)| dd == d).unwrap().1 {
                    if !cells_set.contains(&i) {
                        cells_set.push(i);
                    }
                }
            }
            if cells_set.len() != k {
                continue;
            }
            let mut eliminations = Vec::new();
            for &i in &cells_set {
                for d in digits_of(state.cands[i] & !keep) {
                    eliminations.push((i, d));
                }
            }
            if eliminations.is_empty() {
                continue;
            }
            return Some(Hint {
                technique: tech.name().into(),
                tier: tech.tier(),
                placements: vec![],
                eliminations,
                highlight_cells: cells_set,
                highlight_digits: combo.clone(),
                explanation: format!(
                    "In {}, digits {{{}}} fit only in {} cells — everything else in those cells must go.",
                    unit.name(),
                    digits_text(&combo),
                    k
                ),
            });
        }
    }
    None
}

/// Pointing & claiming: a digit confined to one line inside a box (or one
/// box inside a line) leaves everywhere else on that line / box.
fn pointing_claiming(state: &DeductionState, _ctx: &DeductionCtx) -> Option<Hint> {
    for d in 1..=9u8 {
        let bit = 1 << (d - 1);
        // Pointing: within each box, digit spots all on one row or column.
        for b in 0..9usize {
            let (r0, c0) = ((b / 3) * 3, (b % 3) * 3);
            let box_cells: Vec<usize> =
                (0..9).map(|k| (r0 + k / 3) * 9 + c0 + k % 3).collect();
            let spots: Vec<usize> = box_cells
                .iter()
                .filter(|&&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
                .copied()
                .collect();
            if spots.len() < 2 {
                continue;
            }
            let rows: Vec<usize> = spots.iter().map(|&i| i / 9).collect();
            let cols: Vec<usize> = spots.iter().map(|&i| i % 9).collect();
            let mut eliminations = Vec::new();
            if rows.iter().all(|&r| r == rows[0]) {
                let r = rows[0];
                for c in 0..9usize {
                    let i = r * 9 + c;
                    if state.cells[i] == 0 && !spots.contains(&i) && state.cands[i] & bit != 0 {
                        eliminations.push((i, d));
                    }
                }
                if !eliminations.is_empty() {
                    return Some(Hint {
                        technique: Technique::PointingClaiming.name().into(),
                        tier: 2,
                        placements: vec![],
                        eliminations,
                        highlight_cells: spots.clone(),
                        highlight_digits: vec![d],
                        explanation: format!(
                            "In box {}, {} can only appear in row {} — it leaves the rest of that row.",
                            b + 1,
                            d,
                            r + 1
                        ),
                    });
                }
            }
            if cols.iter().all(|&c| c == cols[0]) {
                let c = cols[0];
                for r in 0..9usize {
                    let i = r * 9 + c;
                    if state.cells[i] == 0 && !spots.contains(&i) && state.cands[i] & bit != 0 {
                        eliminations.push((i, d));
                    }
                }
                if !eliminations.is_empty() {
                    return Some(Hint {
                        technique: Technique::PointingClaiming.name().into(),
                        tier: 2,
                        placements: vec![],
                        eliminations,
                        highlight_cells: spots.clone(),
                        highlight_digits: vec![d],
                        explanation: format!(
                            "In box {}, {} can only appear in column {} — it leaves the rest of that column.",
                            b + 1,
                            d,
                            c + 1
                        ),
                    });
                }
            }
        }
        // Claiming: within each row/col, digit spots all in one box.
        for line in 0..18usize {
            let line_cells: Vec<usize> = (0..9)
                .map(|k| if line < 9 { line * 9 + k } else { k * 9 + (line - 9) })
                .collect();
            let spots: Vec<usize> = line_cells
                .iter()
                .filter(|&&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
                .copied()
                .collect();
            if spots.len() < 2 {
                continue;
            }
            let box_of = |i: usize| (i / 27) * 3 + (i % 9) / 3;
            let b = box_of(spots[0]);
            if spots.iter().all(|&i| box_of(i) == b) {
                let (r0, c0) = ((b / 3) * 3, (b % 3) * 3);
                let mut eliminations = Vec::new();
                for k in 0..9usize {
                    let i = (r0 + k / 3) * 9 + c0 + k % 3;
                    if state.cells[i] == 0 && !spots.contains(&i) && state.cands[i] & bit != 0 {
                        eliminations.push((i, d));
                    }
                }
                if !eliminations.is_empty() {
                    let line_name = if line < 9 {
                        format!("row {}", line + 1)
                    } else {
                        format!("column {}", line - 8)
                    };
                    return Some(Hint {
                        technique: Technique::PointingClaiming.name().into(),
                        tier: 2,
                        placements: vec![],
                        eliminations,
                        highlight_cells: spots.clone(),
                        highlight_digits: vec![d],
                        explanation: format!(
                            "In {}, {} can only appear in box {} — it leaves the rest of that box.",
                            line_name, d, b + 1
                        ),
                    });
                }
            }
        }
    }
    None
}

/// X-Wing / Swordfish / Jellyfish: for one digit, k lines whose candidate
/// positions union to exactly k crossing positions — the digit leaves those
/// crossings in every other line.
fn fish(state: &DeductionState, k: usize) -> Option<Hint> {
    let tech = match k {
        2 => Technique::XWing,
        3 => Technique::Swordfish,
        _ => Technique::Jellyfish,
    };
    let tier = tech.tier();
    for d in 1..=9u8 {
        let bit = 1 << (d - 1);
        for transpose in [false, true] {
            let idx = |line: usize, pos: usize| {
                if transpose {
                    pos * 9 + line
                } else {
                    line * 9 + pos
                }
            };
            // Candidate positions per line (2..=k kept — 0/1 is singles work).
            let mut line_spots: Vec<Vec<usize>> = Vec::new();
            for line in 0..9usize {
                let spots: Vec<usize> = (0..9)
                    .filter(|&p| {
                        let i = idx(line, p);
                        state.cells[i] == 0 && state.cands[i] & bit != 0
                    })
                    .collect();
                line_spots.push(spots);
            }
            let active: Vec<usize> = (0..9)
                .filter(|&l| (2..=k).contains(&line_spots[l].len()))
                .collect();
            for lines in combinations(&active, k) {
                let mut union: Vec<usize> = Vec::new();
                for &l in &lines {
                    for &p in &line_spots[l] {
                        if !union.contains(&p) {
                            union.push(p);
                        }
                    }
                }
                if union.len() != k {
                    continue;
                }
                let mut eliminations = Vec::new();
                for line in 0..9usize {
                    if lines.contains(&line) {
                        continue;
                    }
                    for &p in &union {
                        let i = idx(line, p);
                        if state.cells[i] == 0 && state.cands[i] & bit != 0 {
                            eliminations.push((i, d));
                        }
                    }
                }
                if eliminations.is_empty() {
                    continue;
                }
                let mut highlight: Vec<usize> = Vec::new();
                for &l in &lines {
                    for &p in &line_spots[l] {
                        highlight.push(idx(l, p));
                    }
                }
                let line_word = if transpose { "columns" } else { "rows" };
                let pos_word = if transpose { "rows" } else { "columns" };
                return Some(Hint {
                    technique: tech.name().into(),
                    tier,
                    placements: vec![],
                    eliminations,
                    highlight_cells: highlight,
                    highlight_digits: vec![d],
                    explanation: format!(
                        "{} on {}: {} {} confine {} to exactly {} {} — it leaves those crossings in every other {}.",
                        tech.name(),
                        d,
                        lines.len(),
                        line_word,
                        d,
                        k,
                        pos_word,
                        line_word.trim_end_matches('s')
                    ),
                });
            }
        }
    }
    None
}

/// Y-Wing: pivot {a,b}; pincers {a,c} and {b,c} both seeing the pivot —
/// c leaves every cell seeing both pincers.
fn y_wing(state: &DeductionState, ctx: &DeductionCtx) -> Option<Hint> {
    let bicells: Vec<usize> = (0..CELLS)
        .filter(|&i| state.cells[i] == 0 && state.cands[i].count_ones() == 2)
        .collect();
    for &pivot in &bicells {
        let p_mask = state.cands[pivot];
        let pa = p_mask.trailing_zeros() as u8 + 1;
        let pb = 16 - p_mask.leading_zeros() as u8; // highest set digit
        for &p1 in &ctx.peers[pivot] {
            // Pincer 1: exactly {pa, c} with c ∉ {pa, pb}.
            if state.cells[p1] != 0 || state.cands[p1].count_ones() != 2 {
                continue;
            }
            if state.cands[p1] & (1 << (pa - 1)) == 0 || state.cands[p1] == p_mask {
                continue;
            }
            let c = (state.cands[p1] & !(1 << (pa - 1))).trailing_zeros() as u8 + 1;
            if c == pb {
                continue;
            }
            let p2_mask = (1 << (pb - 1)) | (1 << (c - 1));
            for &p2 in &ctx.peers[pivot] {
                if p2 == p1 || state.cells[p2] != 0 || state.cands[p2] != p2_mask {
                    continue;
                }
                // Victims: empty cells seeing both pincers that hold c.
                let mut eliminations = Vec::new();
                for &v in &ctx.peers[p1] {
                    if ctx.peers[p2].contains(&v)
                        && state.cells[v] == 0
                        && state.cands[v] & (1 << (c - 1)) != 0
                    {
                        eliminations.push((v, c));
                    }
                }
                if eliminations.is_empty() {
                    continue;
                }
                return Some(Hint {
                    technique: Technique::YWing.name().into(),
                    tier: 5,
                    placements: vec![],
                    eliminations,
                    highlight_cells: vec![pivot, p1, p2],
                    highlight_digits: vec![pa, pb, c],
                    explanation: format!(
                        "Pivot {} {{{},{}}}, pincers {} {{{},{}}} and {} {{{},{}}}: {} leaves every cell seeing both pincers.",
                        cell_name(pivot), pa, pb,
                        cell_name(p1), pa, c,
                        cell_name(p2), pb, c,
                        c
                    ),
                });
            }
        }
    }
    None
}

/// Simple coloring on one digit: chains of conjugate pairs are 2-colored;
/// a color contradicted in a unit is false along its whole chain (rule 2),
/// and cells seeing both colors of one chain lose the digit (rule 4).
/// Both rules are per-chain — colors of different chains are independent.
fn simple_coloring(state: &DeductionState, ctx: &DeductionCtx) -> Option<Hint> {
    for d in 1..=9u8 {
        let bit = 1 << (d - 1);
        let nodes: Vec<usize> = (0..CELLS)
            .filter(|&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
            .collect();
        // Conjugate edges: a unit where digit d has exactly two spots.
        let mut adjacency: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for unit in &ctx.units {
            let spots: Vec<usize> = unit
                .cells
                .iter()
                .filter(|&&i| state.cells[i] == 0 && state.cands[i] & bit != 0)
                .copied()
                .collect();
            if spots.len() == 2 {
                adjacency.entry(spots[0]).or_default().push(spots[1]);
                adjacency.entry(spots[1]).or_default().push(spots[0]);
            }
        }
        // 2-color each connected component; track which component it is.
        let mut color: std::collections::HashMap<usize, bool> = std::collections::HashMap::new();
        let mut comp: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut component_count = 0usize;
        for &n in &nodes {
            if color.contains_key(&n) {
                continue;
            }
            let mut queue = std::collections::VecDeque::new();
            color.insert(n, false);
            comp.insert(n, component_count);
            queue.push_back(n);
            while let Some(x) = queue.pop_front() {
                if let Some(neigh) = adjacency.get(&x) {
                    for &y in neigh {
                        if !color.contains_key(&y) {
                            color.insert(y, !color[&x]);
                            comp.insert(y, component_count);
                            queue.push_back(y);
                        }
                    }
                }
            }
            component_count += 1;
        }
        if color.is_empty() {
            continue;
        }
        // Rule 2 (wrap): two same-colored cells of ONE chain in a unit →
        // that color is false along the whole chain.
        for unit in &ctx.units {
            let colored: Vec<usize> = unit
                .cells
                .iter()
                .filter(|&&i| color.contains_key(&i))
                .copied()
                .collect();
            for target in [true, false] {
                let same: Vec<usize> = colored
                    .iter()
                    .filter(|&&i| color[&i] == target)
                    .copied()
                    .collect();
                if same.len() < 2 {
                    continue;
                }
                let chain = comp[&same[0]];
                if !same.iter().all(|&i| comp[&i] == chain) {
                    continue; // different chains — no contradiction
                }
                let eliminations: Vec<(usize, u8)> = color
                    .iter()
                    .filter(|(&i, &c)| c == target && comp[&i] == chain)
                    .map(|(&i, _)| (i, d))
                    .collect();
                if eliminations.is_empty() {
                    continue;
                }
                return Some(Hint {
                    technique: Technique::SimpleColoring.name().into(),
                    tier: 5,
                    placements: vec![],
                    eliminations,
                    highlight_cells: same.clone(),
                    highlight_digits: vec![d],
                    explanation: format!(
                        "Coloring {}: two {}-colored cells of one chain share {} — that color is false along the whole chain.",
                        d,
                        if target { "A" } else { "B" },
                        unit.name()
                    ),
                });
            }
        }
        // Rule 4 (trap): an uncolored d-cell seeing both colors of one
        // chain loses d.
        for &x in &nodes {
            if color.contains_key(&x) {
                continue;
            }
            let mut chains_seen: std::collections::HashMap<usize, (bool, bool)> =
                std::collections::HashMap::new();
            for (&i, &col) in &color {
                if ctx.peers[x].contains(&i) {
                    let entry = chains_seen.entry(comp[&i]).or_insert((false, false));
                    if col {
                        entry.1 = true;
                    } else {
                        entry.0 = true;
                    }
                }
            }
            if let Some((&chain, _)) =
                chains_seen.iter().find(|(_, &(a, b))| a && b)
            {
                let highlight: Vec<usize> = color
                    .iter()
                    .filter(|(&i, _)| comp[&i] == chain)
                    .map(|(&i, _)| i)
                    .collect();
                return Some(Hint {
                    technique: Technique::SimpleColoring.name().into(),
                    tier: 5,
                    placements: vec![],
                    eliminations: vec![(x, d)],
                    highlight_cells: highlight,
                    highlight_digits: vec![d],
                    explanation: format!(
                        "Cell {} sees both colors of one {}-chain — it can't be {}.",
                        cell_name(x),
                        d,
                        d
                    ),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Grid, ALL_DIGITS};

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    fn classic_ctx() -> DeductionCtx {
        DeductionCtx::for_rules(&[])
    }

    #[test]
    fn easy_puzzle_solves_with_singles_only() {
        let ctx = classic_ctx();
        let rules = RuleSet::new();
        let e = DeductionEngine { ctx: &ctx, rules: &rules, max_tier: 2 };
        let mut state = DeductionState::new(&Grid::parse(EASY).cells(), &rules);
        let grade = e.solve(&mut state);
        assert!(grade.solved);
        assert!(grade.max_tier <= 2, "newspaper-easy should not need past tier 2");
        assert_eq!(state.cells, *Grid::parse(EASY_SOLUTION).cells());
    }

    #[test]
    fn beginner_cap_stalls_on_hidden_single() {
        // A state with no naked single but a hidden one: row 0 holds
        // 5,6,7,8,9,1 with free cells {2,3},{2,3},{2,3,4} — digit 4 fits
        // only in r1c9. Tier 1 must stall; tier 2 places it.
        let ctx = classic_ctx();
        let rules = RuleSet::new();
        let mut cells = [0u8; CELLS];
        cells[0] = 5;
        cells[1] = 6;
        cells[2] = 7;
        cells[3] = 8;
        cells[4] = 9;
        cells[5] = 1;
        let mut state = DeductionState::new(&cells, &rules);
        state.cands[6] = 0b110; // {2,3}
        state.cands[7] = 0b110; // {2,3}
        state.cands[8] = 0b1110; // {2,3,4}
        let t1 = DeductionEngine { ctx: &ctx, rules: &rules, max_tier: 1 };
        assert!(!t1.solve(&mut state).solved, "no naked single exists — tier 1 stalls");
        let t2 = DeductionEngine { ctx: &ctx, rules: &rules, max_tier: 2 };
        let hint = t2.hint(&state).expect("tier 2 must find the hidden single");
        assert_eq!(hint.technique, "Hidden Single");
        assert_eq!(hint.placements, vec![(8, 4)]);
    }

    #[test]
    fn hint_applies_and_progresses() {
        let ctx = classic_ctx();
        let rules = RuleSet::new();
        let e = DeductionEngine { ctx: &ctx, rules: &rules, max_tier: 6 };
        let mut state = DeductionState::new(&Grid::parse(EASY).cells(), &rules);
        let before = state.cells.iter().filter(|&&d| d != 0).count();
        let hint = e.hint(&state).expect("must have a next step");
        state.apply(&ctx, &hint);
        let after = state.cells.iter().filter(|&&d| d != 0).count();
        assert!(after > before || !hint.eliminations.is_empty());
        assert!(!hint.explanation.is_empty());
    }

    #[test]
    fn naked_pair_is_found() {
        // Direct technique test: row 0 has 1,4,5,6,7,8 placed; cells 3,4
        // hold exactly {2,3} — a naked pair — and cell 5 must lose 2 and 3.
        let ctx = classic_ctx();
        let mut cells = [0u8; CELLS];
        cells[0] = 1;
        cells[1] = 4;
        cells[2] = 5;
        cells[6] = 6;
        cells[7] = 7;
        cells[8] = 8;
        let mut state = DeductionState::new(&cells, &RuleSet::new());
        state.cands[3] = 0b110; // {2,3}
        state.cands[4] = 0b110; // {2,3}
        state.cands[5] = 0b100000111; // {2,3,9}
        let hint = naked_group(&state, &ctx, 2).expect("naked pair must fire");
        assert_eq!(hint.technique, "Naked Pair");
        assert!(hint.eliminations.contains(&(5, 2)));
        assert!(hint.eliminations.contains(&(5, 3)));
    }

    #[test]
    fn x_wing_is_found() {
        // Direct technique test: digit 9 is possible only at the four
        // corners of rows 0/8 × cols 0/8 (an X-Wing) plus one victim at
        // r2c0, which must lose its 9. (Rows 0 and 8 are the only active
        // lines — the victim's row has a single 9-spot.)
        let ctx = classic_ctx();
        let mut state = DeductionState::new(&[0u8; CELLS], &RuleSet::new());
        state.cands = [ALL_DIGITS & !(1 << 8); CELLS];
        for &i in &[0usize, 8, 72, 80, 9] {
            state.cands[i] |= 1 << 8;
        }
        let hint = fish(&state, 2).expect("X-Wing must fire");
        assert_eq!(hint.technique, "X-Wing");
        assert!(hint.eliminations.contains(&(9, 9)), "r2c0 must lose 9");
    }
}
