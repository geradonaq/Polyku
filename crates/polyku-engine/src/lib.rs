//! Polyku's sudoku engine — generation, solving, and variant rules.
//!
//! Current milestone (M1): the classic 9×9 core.
//! - [`grid`]: the board with bitmask bookkeeping
//! - [`dlx`]: Dancing Links exact-cover solver
//! - [`sudoku`]: sudoku ↔ exact-cover mapping (uniqueness proofs live here)
//! - [`generator`]: randomized puzzle generation, unique by construction
//!
//! M2 will add the `VariantRule` trait and the first variants;
//! M3 the human-logic deduction engine.

pub mod dlx;
pub mod generator;
pub mod grid;
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
