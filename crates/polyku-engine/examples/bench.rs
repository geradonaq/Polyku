//! Manual benchmark: times puzzle generation per rule combination.
//! Run: cargo run --release -p polyku-engine --example bench

use polyku_engine::generator::{generate_with_rules, Difficulty};
use polyku_engine::rules::RuleKind;
use rand::thread_rng;
use std::time::Instant;

fn main() {
    let mut rng = thread_rng();
    let cases: Vec<(&str, Vec<RuleKind>, Difficulty)> = vec![
        ("classic Easy", vec![], Difficulty::Easy),
        ("classic Medium", vec![], Difficulty::Medium),
        ("classic Hard", vec![], Difficulty::Hard),
        ("diagonal Easy", vec![RuleKind::Diagonal], Difficulty::Easy),
        (
            "nonconsecutive Easy",
            vec![RuleKind::NonConsecutive],
            Difficulty::Easy,
        ),
        ("killer Easy", vec![RuleKind::Killer], Difficulty::Easy),
        ("thermo Easy", vec![RuleKind::Thermo], Difficulty::Easy),
        (
            "diag+killer Easy",
            vec![RuleKind::Diagonal, RuleKind::Killer],
            Difficulty::Easy,
        ),
        (
            "nonconsecutive Medium",
            vec![RuleKind::NonConsecutive],
            Difficulty::Medium,
        ),
        (
            "nonconsecutive Hard",
            vec![RuleKind::NonConsecutive],
            Difficulty::Hard,
        ),
        ("killer Medium", vec![RuleKind::Killer], Difficulty::Medium),
        ("killer Hard", vec![RuleKind::Killer], Difficulty::Hard),
    ];
    for (name, kinds, difficulty) in cases {
        let t = Instant::now();
        let p = generate_with_rules(&mut rng, &kinds, difficulty);
        println!(
            "{:<22} {:>10?} — {} clues, grade {} (asked {:?})",
            name,
            t.elapsed(),
            p.clue_count,
            p.grade,
            difficulty
        );
    }
}
