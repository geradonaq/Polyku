//! Polyku's sudoku engine — generation, solving, and variant rules.
//!
//! Milestone M1 fills this crate with the bitmask `Grid`, the DLX
//! (Dancing Links) exact-cover solver, and the puzzle generator.

/// Returns the engine's package version.
///
/// Temporary smoke-test API: it lets the Tauri app prove that the
/// engine crate is wired into the build end-to-end.
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
