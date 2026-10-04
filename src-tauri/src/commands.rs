//! The Tauri IPC commands. Thin wrappers: the real logic lives in
//! `game.rs` (testable without Tauri) and `storage.rs` (injectable dirs).

use crate::dto::{
    parse_difficulty, parse_rules, ActiveGame, GameRecord, HintDto, PuzzleDto, SaveGame,
    Stats,
};
use crate::game::{compute_candidates, compute_conflicts, compute_hint, GameStore};
use crate::storage;
use std::sync::Mutex;
use tauri::State;

/// Starts a new game: generates (possibly for a few seconds on the higher
/// tiers — hence the blocking thread) and stores the solution host-side.
#[tauri::command]
async fn new_game(
    store: State<'_, GameStore>,
    difficulty: String,
    rules: Vec<String>,
) -> Result<PuzzleDto, String> {
    let diff = parse_difficulty(&difficulty)?;
    let kinds = parse_rules(&rules)?;
    let puzzle =
        tauri::async_runtime::spawn_blocking(move || {
            polyku_engine::generator::generate_with_rules(&mut rand::thread_rng(), &kinds, diff)
        })
        .await
        .map_err(|e| e.to_string())?;

    let overlays = {
        let mut set = polyku_engine::rules::RuleSet::new();
        for data in &puzzle.rules {
            set.push(data.build());
        }
        set.overlays()
    };
    let dto = PuzzleDto {
        givens: puzzle.givens.to_vec(),
        rules: puzzle.rules.clone(),
        difficulty: difficulty.clone(),
        grade: puzzle.grade,
        clue_count: puzzle.clue_count,
        overlays,
    };

    let mut guard = store.0.lock().map_err(|_| "game store poisoned")?;
    *guard = Some(ActiveGame {
        givens: puzzle.givens.to_vec(),
        solution: puzzle.solution.to_vec(),
        rules: puzzle.rules,
        difficulty,
        grade: dto.grade,
    });
    Ok(dto)
}

/// Cells currently breaking any active rule — the red-highlight set.
#[tauri::command]
fn validate_board(
    store: State<'_, GameStore>,
    cells: Vec<u8>,
) -> Result<Vec<usize>, String> {
    let guard = store.0.lock().map_err(|_| "game store poisoned")?;
    let active = guard.as_ref().ok_or("no active game")?;
    let cells: [u8; polyku_engine::grid::CELLS] = cells.try_into().map_err(|_| "need 81 cells")?;
    Ok(compute_conflicts(&cells, &active.rules))
}

/// The next logical step, as a teachable hint.
#[tauri::command]
fn get_hint(store: State<'_, GameStore>, cells: Vec<u8>) -> Result<HintDto, String> {
    let guard = store.0.lock().map_err(|_| "game store poisoned")?;
    let active = guard.as_ref().ok_or("no active game")?;
    let cells: [u8; polyku_engine::grid::CELLS] = cells.try_into().map_err(|_| "need 81 cells")?;
    Ok(compute_hint(active, &cells))
}

/// Candidate lists for every cell under the full active ruleset.
#[tauri::command]
fn fill_candidates(store: State<'_, GameStore>, cells: Vec<u8>) -> Result<Vec<Vec<u8>>, String> {
    let guard = store.0.lock().map_err(|_| "game store poisoned")?;
    let active = guard.as_ref().ok_or("no active game")?;
    let cells: [u8; polyku_engine::grid::CELLS] = cells.try_into().map_err(|_| "need 81 cells")?;
    Ok(compute_candidates(&cells, &active.rules))
}

/// Persists the running game (merged with the stored solution) to AppData.
#[tauri::command]
fn save_game(
    store: State<'_, GameStore>,
    cells: Vec<u8>,
    notes: Vec<Vec<u8>>,
    marks: Vec<u8>,
    elapsed_secs: u64,
) -> Result<(), String> {
    let guard = store.0.lock().map_err(|_| "game store poisoned")?;
    let active = guard.as_ref().ok_or("no active game")?;
    let cells: [u8; polyku_engine::grid::CELLS] = cells.try_into().map_err(|_| "need 81 cells")?;
    if notes.len() != polyku_engine::grid::CELLS {
        return Err("need 81 note lists".into());
    }
    let save = SaveGame {
        givens: active.givens.clone(),
        solution: active.solution.clone(),
        rules: active.rules.clone(),
        difficulty: active.difficulty.clone(),
        grade: active.grade,
        cells: cells.to_vec(),
        notes,
        marks,
        elapsed_secs,
        saved_at: now_unix(),
    };
    storage::save_game_in(&storage::data_dir(), &save)
}

/// The doc's assist rule: once a *correct* move is placed, every wrong entry
/// anywhere on the board is removed. A wrong move changes nothing (the red
/// conflicts tell the story instead).
#[tauri::command]
fn clean_entries(
    store: State<'_, GameStore>,
    cells: Vec<u8>,
    targets: Vec<usize>,
) -> Result<Vec<u8>, String> {
    let guard = store.0.lock().map_err(|_| "game store poisoned")?;
    let active = guard.as_ref().ok_or("no active game")?;
    if cells.len() != polyku_engine::grid::CELLS {
        return Err("need 81 cells".into());
    }
    let placed_correctly = targets.iter().all(|&i| {
        i < polyku_engine::grid::CELLS
            && (active.givens[i] != 0 || cells[i] == active.solution[i])
    });
    if !placed_correctly {
        return Ok(cells);
    }
    let mut out = cells;
    for i in 0..polyku_engine::grid::CELLS {
        if active.givens[i] == 0 && out[i] != 0 && out[i] != active.solution[i] {
            out[i] = 0;
        }
    }
    Ok(out)
}

/// Restores a saved game (if any) after a restart.
#[tauri::command]
fn load_game(store: State<'_, GameStore>) -> Result<Option<SaveGame>, String> {
    let save = storage::load_game_in(&storage::data_dir())?;
    if let Some(save) = &save {
        let mut guard = store.0.lock().map_err(|_| "game store poisoned")?;
        *guard = Some(ActiveGame {
            givens: save.givens.clone(),
            solution: save.solution.clone(),
            rules: save.rules.clone(),
            difficulty: save.difficulty.clone(),
            grade: save.grade,
        });
    }
    Ok(save.map(|s| SaveGame { solution: vec![], ..s }))
}

/// Appends a finished (or abandoned) game to the statistics file.
#[tauri::command]
fn record_result(
    difficulty: String,
    rules: Vec<String>,
    completed: bool,
    seconds: u64,
) -> Result<(), String> {
    let dir = storage::data_dir();
    let mut stats = storage::load_stats_in(&dir)?;
    stats.games.push(GameRecord {
        difficulty,
        rules,
        completed,
        seconds,
        finished_at: now_unix(),
    });
    storage::save_stats_in(&dir, &stats)
}

/// The full statistics list; the frontend aggregates for display.
#[tauri::command]
fn get_statistics() -> Result<Stats, String> {
    storage::load_stats_in(&storage::data_dir())
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Reports the engine crate's version — proves the workspace wiring
/// from the UI all the way down into `crates/polyku-engine`.
#[tauri::command]
fn engine_version() -> String {
    polyku_engine::version().to_string()
}

pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(GameStore(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            engine_version,
            new_game,
            validate_board,
            get_hint,
            fill_candidates,
            clean_entries,
            save_game,
            load_game,
            record_result,
            get_statistics,
        ])
}
