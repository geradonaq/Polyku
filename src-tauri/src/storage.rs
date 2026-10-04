//! JSON persistence in `%LOCALAPPDATA%\Polyku`.
//!
//! All functions take the data directory as a parameter so tests can use a
//! temp dir; the Tauri commands call the `_in` variants with the real one.

use crate::dto::{SaveGame, Stats};
use std::fs;
use std::path::PathBuf;

/// The real Polyku data directory (created on demand).
pub fn data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Polyku");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn save_game_in(dir: &PathBuf, save: &SaveGame) -> Result<(), String> {
    let path = dir.join("save.json");
    let json = serde_json::to_string_pretty(save).map_err(|e| e.to_string())?;
    // Write-then-rename so a crash mid-write can't corrupt the save.
    let tmp = dir.join("save.json.tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn load_game_in(dir: &PathBuf) -> Result<Option<SaveGame>, String> {
    let path = dir.join("save.json");
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json)
        .map(Some)
        .map_err(|e| format!("corrupt save file: {e}"))
}

pub fn save_stats_in(dir: &PathBuf, stats: &Stats) -> Result<(), String> {
    let path = dir.join("stats.json");
    let json = serde_json::to_string_pretty(stats).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn load_stats_in(dir: &PathBuf) -> Result<Stats, String> {
    let path = dir.join("stats.json");
    if !path.exists() {
        return Ok(Stats::default());
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| format!("corrupt stats file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("polyku-test-{tag}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_save() -> SaveGame {
        SaveGame {
            givens: vec![5u8; 81],
            solution: vec![3u8; 81],
            rules: vec![polyku_engine::rules::RuleData::Diagonal],
            difficulty: "easy".into(),
            grade: 2,
            cells: vec![0u8; 81],
            notes: vec![vec![]; 81],
            marks: vec![0u8; 81],
            elapsed_secs: 42,
            saved_at: 1_700_000_000,
        }
    }

    #[test]
    fn save_round_trips() {
        let dir = temp_dir("roundtrip");
        let save = sample_save();
        save_game_in(&dir, &save).unwrap();
        let loaded = load_game_in(&dir).unwrap().expect("save must exist");
        assert_eq!(loaded.elapsed_secs, 42);
        assert_eq!(loaded.rules.len(), 1);
        assert_eq!(loaded.difficulty, "easy");
    }

    #[test]
    fn missing_save_reads_as_none() {
        let dir = temp_dir("missing");
        assert!(load_game_in(&dir).unwrap().is_none());
    }

    #[test]
    fn stats_round_trip_and_default() {
        let dir = temp_dir("stats");
        assert_eq!(load_stats_in(&dir).unwrap().games.len(), 0);
        let stats = Stats {
            games: vec![crate::dto::GameRecord {
                difficulty: "hard".into(),
                rules: vec!["killer".into()],
                completed: true,
                seconds: 300,
                finished_at: 1,
            }],
        };
        save_stats_in(&dir, &stats).unwrap();
        assert_eq!(load_stats_in(&dir).unwrap().games.len(), 1);
    }
}
