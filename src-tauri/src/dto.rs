//! Data transfer objects crossing the Tauri IPC boundary.
//!
//! The frontend never receives the puzzle *solution* — hints and
//! validation are computed here in the host so the webview can't peek.

use polyku_engine::rules::{Overlay, RuleData};
use serde::{Deserialize, Serialize};

/// A freshly generated puzzle, as the frontend sees it.
#[derive(Clone, Serialize, Deserialize)]
pub struct PuzzleDto {
    /// The 81 starting cells (0 = empty).
    pub givens: Vec<u8>,
    pub rules: Vec<RuleData>,
    /// "beginner" | "easy" | "medium" | "hard" | "expert" | "master"
    pub difficulty: String,
    /// Graded hardest technique tier (1–6) actually required.
    pub grade: u8,
    pub clue_count: usize,
    /// Declarative drawing primitives (cages, thermos, stripes).
    pub overlays: Vec<Overlay>,
}

/// A full save file. The solution is included so hints and validation
/// survive an app restart; it stays in AppData, out of the webview.
#[derive(Clone, Serialize, Deserialize)]
pub struct SaveGame {
    pub givens: Vec<u8>,
    pub solution: Vec<u8>,
    pub rules: Vec<RuleData>,
    pub difficulty: String,
    pub grade: u8,
    /// Player entries (0 = empty).
    pub cells: Vec<u8>,
    /// Pencil marks: for every cell, the digits noted.
    pub notes: Vec<Vec<u8>>,
    /// Cell marking colors: 0 = none, 1–9 = palette index.
    #[serde(default)]
    pub marks: Vec<u8>,
    pub elapsed_secs: u64,
    /// Unix timestamp of the save.
    pub saved_at: u64,
}

/// One finished (or abandoned) game for the statistics list.
#[derive(Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub difficulty: String,
    pub rules: Vec<String>,
    pub completed: bool,
    pub seconds: u64,
    pub finished_at: u64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub games: Vec<GameRecord>,
}

/// A teaching hint. `kind` distinguishes engine hints from error callouts:
/// - `"technique"` — the next logical step.
/// - `"error"` — the player has a wrong entry; see `highlight_cells`.
/// - `"done"` — the board is complete and correct.
/// - `"stuck"` — no logical step found (implies earlier errors exist).
#[derive(Clone, Serialize, Deserialize)]
pub struct HintDto {
    pub kind: String,
    pub technique: String,
    pub tier: u8,
    pub placements: Vec<(usize, u8)>,
    pub eliminations: Vec<(usize, u8)>,
    pub highlight_cells: Vec<usize>,
    pub highlight_digits: Vec<u8>,
    pub explanation: String,
}

/// Everything the host must keep about the running game.
#[derive(Clone)]
pub struct ActiveGame {
    pub givens: Vec<u8>,
    pub solution: Vec<u8>,
    pub rules: Vec<RuleData>,
    pub difficulty: String,
    pub grade: u8,
}

impl ActiveGame {
    pub fn ruleset(&self) -> polyku_engine::rules::RuleSet {
        let mut set = polyku_engine::rules::RuleSet::new();
        for data in &self.rules {
            set.push(data.build());
        }
        set
    }
}

/// Maps a difficulty name from the frontend onto the engine enum.
pub fn parse_difficulty(name: &str) -> Result<polyku_engine::generator::Difficulty, String> {
    match name {
        "beginner" => Ok(polyku_engine::generator::Difficulty::Beginner),
        "easy" => Ok(polyku_engine::generator::Difficulty::Easy),
        "medium" => Ok(polyku_engine::generator::Difficulty::Medium),
        "hard" => Ok(polyku_engine::generator::Difficulty::Hard),
        "expert" => Ok(polyku_engine::generator::Difficulty::Expert),
        "master" => Ok(polyku_engine::generator::Difficulty::Master),
        other => Err(format!("unknown difficulty: {other}")),
    }
}

/// Maps rule ids from the frontend onto rule kinds ("classic" = none).
pub fn parse_rules(names: &[String]) -> Result<Vec<RuleKindName>, String> {
    let mut out = Vec::new();
    for n in names {
        match n.as_str() {
            "classic" => {}
            "diagonal" => out.push(polyku_engine::rules::RuleKind::Diagonal),
            "killer" => out.push(polyku_engine::rules::RuleKind::Killer),
            "thermo" => out.push(polyku_engine::rules::RuleKind::Thermo),
            "non_consecutive" => out.push(polyku_engine::rules::RuleKind::NonConsecutive),
            "anti_knight" => out.push(polyku_engine::rules::RuleKind::AntiKnight),
            other => return Err(format!("unknown rule: {other}")),
        }
    }
    Ok(out)
}

/// Alias kept for readable signatures above.
type RuleKindName = polyku_engine::rules::RuleKind;

/// Rule ids for storage/statistics, reverse of [`parse_rules`].
#[allow(dead_code)] // used by the frontend-facing stats from M5 onward
pub fn rule_names(rules: &[RuleData]) -> Vec<String> {
    rules
        .iter()
        .filter_map(|r| match r {
            RuleData::Diagonal => Some("diagonal".to_string()),
            RuleData::Killer { .. } => Some("killer".to_string()),
            RuleData::Thermo { .. } => Some("thermo".to_string()),
            RuleData::NonConsecutive => Some("non_consecutive".to_string()),
            RuleData::AntiKnight => Some("anti_knight".to_string()),
        })
        .collect()
}
