//! Game logic between the engine and the IPC commands — kept free of
//! Tauri types so it can be unit-tested directly.

use crate::dto::{ActiveGame, HintDto};
use polyku_engine::grid::CELLS;
use polyku_engine::rules::{RuleData, RuleSet};

/// Shared, mutex-guarded slot for the game currently in play.
/// `None` until the first `new_game`.
#[derive(Default)]
pub struct GameStore(pub std::sync::Mutex<Option<ActiveGame>>);

/// Indices of cells that break the classic rules or any active variant rule.
/// Conflicting cells are marked in pairs (both sides of a duplicate), so the
/// frontend can simply highlight them.
pub fn compute_conflicts(cells: &[u8; CELLS], rules: &[RuleData]) -> Vec<usize> {
    let mut bad = std::collections::BTreeSet::new();

    // Classic: duplicate digits within a row, column, or box.
    for i in 0..CELLS {
        if cells[i] == 0 {
            continue;
        }
        let (r, c) = (i / 9, i % 9);
        for j in 0..CELLS {
            if j != i && cells[j] == cells[i] {
                let (rj, cj) = (j / 9, j % 9);
                let same_box = r / 3 == rj / 3 && c / 3 == cj / 3;
                if rj == r || cj == c || same_box {
                    bad.insert(i);
                    bad.insert(j);
                }
            }
        }
    }

    for rule in rules {
        match rule {
            RuleData::Diagonal => {
                for main in [true, false] {
                    for a in 0..9usize {
                        for b in (a + 1)..9usize {
                            let ia = if main { a * 9 + a } else { a * 9 + (8 - a) };
                            let ib = if main { b * 9 + b } else { b * 9 + (8 - b) };
                            if cells[ia] != 0 && cells[ia] == cells[ib] {
                                bad.insert(ia);
                                bad.insert(ib);
                            }
                        }
                    }
                }
            }
            RuleData::Killer { cages } => {
                for cage in cages {
                    let mut seen: Vec<usize> = Vec::new();
                    let mut sum = 0u16;
                    for c in &cage.cells {
                        let idx = c.index();
                        if cells[idx] != 0 {
                            if let Some(&first) = seen.iter().find(|&&s| cells[s] == cells[idx]) {
                                bad.insert(first);
                                bad.insert(idx);
                            }
                            seen.push(idx);
                            sum += cells[idx] as u16;
                        }
                    }
                    // Assigned cells already exceed the cage target.
                    if sum > cage.sum && !seen.is_empty() {
                        for idx in seen {
                            bad.insert(idx);
                        }
                    }
                }
            }
            RuleData::Thermo { paths } => {
                for path in paths {
                    for w in path.windows(2) {
                        let (a, b) = (w[0].index(), w[1].index());
                        if cells[a] != 0 && cells[b] != 0 && cells[b] <= cells[a] {
                            bad.insert(a);
                            bad.insert(b);
                        }
                    }
                }
            }
            RuleData::NonConsecutive => {
                for i in 0..CELLS {
                    if cells[i] == 0 {
                        continue;
                    }
                    let (r, c) = (i / 9, i % 9);
                    let mut neighbors = Vec::new();
                    if r > 0 {
                        neighbors.push(i - 9);
                    }
                    if r < 8 {
                        neighbors.push(i + 9);
                    }
                    if c > 0 {
                        neighbors.push(i - 1);
                    }
                    if c < 8 {
                        neighbors.push(i + 1);
                    }
                    for n in neighbors {
                        if cells[n] != 0 && (cells[n] as i8 - cells[i] as i8).abs() == 1 {
                            bad.insert(i);
                            bad.insert(n);
                        }
                    }
                }
            }
            RuleData::AntiKnight => {
                for i in 0..CELLS {
                    if cells[i] == 0 {
                        continue;
                    }
                    let (r, c) = ((i / 9) as i32, (i % 9) as i32);
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
                        if !(0..9).contains(&rr) || !(0..9).contains(&cc) {
                            continue;
                        }
                        let j = rr as usize * 9 + cc as usize;
                        if cells[j] == cells[i] {
                            bad.insert(i);
                            bad.insert(j);
                        }
                    }
                }
            }
        }
    }

    bad.into_iter().collect()
}

/// The teaching hint for the current position. Wrong entries short-circuit:
/// the engine's logic only makes sense on a legal board, so an illegal one
/// gets an "error" callout pointing at the first mistake.
pub fn compute_hint(active: &ActiveGame, cells: &[u8; CELLS]) -> HintDto {
    let ruleset = active.ruleset();

    // Error scan: any entry that contradicts the (unique) solution.
    for (i, (&cell, &sol)) in cells.iter().zip(active.solution.iter()).enumerate() {
        if cell != 0 && cell != sol {
            return HintDto {
                kind: "error".into(),
                technique: "Check entries".into(),
                tier: 0,
                placements: vec![],
                eliminations: vec![],
                highlight_cells: vec![i],
                highlight_digits: vec![cell],
                explanation: format!(
                    "Cell r{}c{} contradicts the unique solution — fix it before looking for logic.",
                    i / 9 + 1,
                    i % 9 + 1
                ),
            };
        }
    }

    if cells.iter().all(|&d| d != 0) {
        return HintDto {
            kind: "done".into(),
            technique: "Solved".into(),
            tier: 0,
            placements: vec![],
            eliminations: vec![],
            highlight_cells: vec![],
            highlight_digits: vec![],
            explanation: "The board is complete and correct. Well played!".into(),
        };
    }

    let ctx = polyku_engine::deduction::DeductionCtx::for_rules(&active.rules);
    let engine = polyku_engine::deduction::DeductionEngine {
        ctx: &ctx,
        rules: &ruleset,
        max_tier: 6,
    };
    let state = polyku_engine::deduction::DeductionState::new(cells, &ruleset);
    match engine.hint(&state) {
        Some(step) => HintDto {
            kind: "technique".into(),
            technique: step.technique.clone(),
            tier: step.tier,
            placements: step.placements.clone(),
            eliminations: step.eliminations.clone(),
            highlight_cells: step.highlight_cells.clone(),
            highlight_digits: step.highlight_digits.clone(),
            explanation: step.explanation.clone(),
        },
        None => HintDto {
            kind: "stuck".into(),
            technique: "No step found".into(),
            tier: 0,
            placements: vec![],
            eliminations: vec![],
            highlight_cells: vec![],
            highlight_digits: vec![],
            explanation: "No logical step found on this position — that should not happen; \
                          check your entries for mistakes."
                .into(),
        },
    }
}

/// Full candidate matrix for the Fill-Candidates dialog: for every cell,
/// the digits allowed by classic rules + all active rules right now.
pub fn compute_candidates(cells: &[u8; CELLS], rules: &[RuleData]) -> Vec<Vec<u8>> {
    let ruleset = ruleset_of(rules);
    let state = polyku_engine::deduction::DeductionState::new(cells, &ruleset);
    (0..CELLS)
        .map(|i| polyku_engine::deduction::digits_of(state.cands[i]))
        .collect()
}

/// Builds a live ruleset from serializable rule data.
pub fn ruleset_of(rules: &[RuleData]) -> RuleSet {
    let mut set = RuleSet::new();
    for r in rules {
        set.push(r.build());
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::parse_difficulty;
    use polyku_engine::generator;

    fn active_game(kinds: &[polyku_engine::rules::RuleKind]) -> ActiveGame {
        let mut rng = rand::thread_rng();
        let puzzle =
            generator::generate_with_rules(&mut rng, kinds, parse_difficulty("easy").unwrap());
        ActiveGame {
            givens: puzzle.givens.to_vec(),
            solution: puzzle.solution.to_vec(),
            rules: puzzle.rules.clone(),
            difficulty: "easy".into(),
            grade: puzzle.grade,
        }
    }

    #[test]
    fn solution_board_has_no_conflicts() {
        let active = active_game(&[]);
        let mut cells = active.givens.clone();
        cells.copy_from_slice(&active.solution);
        let cells: [u8; CELLS] = cells.try_into().unwrap();
        assert!(compute_conflicts(&cells, &active.rules).is_empty());
    }

    #[test]
    fn duplicate_in_row_is_flagged() {
        let mut cells = [0u8; CELLS];
        cells[0] = 5;
        cells[4] = 5; // same row
        let bad = compute_conflicts(&cells, &[]);
        assert!(bad.contains(&0) && bad.contains(&4));
        assert_eq!(bad.len(), 2);
    }

    #[test]
    fn wrong_entry_produces_error_hint() {
        let active = active_game(&[]);
        let mut cells = active.givens.clone();
        // Place a digit that differs from the solution in an empty cell.
        let empty = (0..CELLS).find(|&i| active.givens[i] == 0).unwrap();
        cells[empty] = if active.solution[empty] == 1 { 2 } else { 1 };
        let hint = compute_hint(&active, &cells.try_into().unwrap());
        assert_eq!(hint.kind, "error");
        assert!(hint.highlight_cells.contains(&empty));
    }

    #[test]
    fn correct_board_yields_technique_hint() {
        let active = active_game(&[]);
        let hint = compute_hint(&active, &active.givens.clone().try_into().unwrap());
        assert_eq!(hint.kind, "technique");
        assert!(!hint.explanation.is_empty());
    }
}
