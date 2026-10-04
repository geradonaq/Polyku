//! Polyku's sudoku engine — generation, solving, and variant rules.
//!
//! - [`grid`]: the classic 9×9 board with bitmask bookkeeping
//! - [`dlx`]: Dancing Links exact-cover solver (classic specialist)
//! - [`sudoku`]: sudoku ↔ exact-cover mapping (classic uniqueness proofs)
//! - [`rules`]: the `VariantRule` trait — additional rules on top of classic
//! - [`solver`]: ruleset-aware backtracking solver for any rule combination
//! - [`generator`]: randomized puzzle generation, unique by construction
//!
//! M3 will add the human-logic deduction engine (technique-based grading
//! and hints).

pub mod deduction;
pub mod dlx;
pub mod generator;
pub mod grid;
pub mod rules;
pub mod solver;
pub mod sudoku;

/// Returns the engine's package version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_reported() {
        assert!(!super::version().is_empty());
    }
}
