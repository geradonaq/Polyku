//! The variant rule system.
//!
//! Classic row/column/box constraints are the *base* and always active
//! (baked into [`crate::grid::Grid`]). A `VariantRule` is any *additional*
//! constraint layered on top — "additional rules on classic sudoku", exactly
//! what Polyku is about. A puzzle's active rules form a [`RuleSet`], and any
//! combination is handled uniformly by the solver and generator.
//!
//! Adding a variant = one struct implementing `VariantRule` + one entry in
//! [`RuleData`] — no solver or UI changes. That's the open-source hook.

pub mod anti_knight;
pub mod diagonal;
pub mod killer;
pub mod non_consecutive;
pub mod thermo;

use crate::grid::{Coord, DigitMask, CELLS};
use serde::{Deserialize, Serialize};

/// Declarative drawing primitives the UI renders for a rule (M5 consumes
/// these). Rules describe *what* to draw; the frontend decides *how*.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Overlay {
    /// Killer-style cage with its target sum.
    Cage { cells: Vec<Coord>, sum: Option<u16> },
    /// Thermo-style path; `cells[0]` is the bulb end.
    Path { cells: Vec<Coord> },
    /// Highlight one of the two main diagonals.
    DiagonalStripe { main: bool },
}

/// One additional rule layered on top of classic sudoku.
pub trait VariantRule {
    /// Stable identifier, e.g. `"diagonal"`, `"killer"`.
    fn id(&self) -> &'static str;

    /// Whether the current (partial or complete) assignment breaks this rule.
    /// Called at solution leaves as a safety net; `prune` does the heavy work.
    fn is_consistent(&self, cells: &[u8; CELLS]) -> bool;

    /// Removes impossible digits from per-cell candidate masks given the
    /// current partial assignment. Must only ever *clear* bits, never set.
    fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]);

    /// What the UI should draw for this rule.
    fn overlays(&self) -> Vec<Overlay>;
}

/// A composed pile of active additional rules, in any combination.
#[derive(Default)]
pub struct RuleSet {
    rules: Vec<Box<dyn VariantRule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        RuleSet { rules: Vec::new() }
    }

    pub fn push(&mut self, rule: Box<dyn VariantRule>) {
        self.rules.push(rule);
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn is_consistent(&self, cells: &[u8; CELLS]) -> bool {
        self.rules.iter().all(|r| r.is_consistent(cells))
    }

    pub fn prune(&self, cells: &[u8; CELLS], candidates: &mut [DigitMask; CELLS]) {
        for r in &self.rules {
            r.prune(cells, candidates);
        }
    }

    pub fn overlays(&self) -> Vec<Overlay> {
        self.rules.iter().flat_map(|r| r.overlays()).collect()
    }
}

/// A killer cage: disjoint cells whose digits (all distinct) sum to `sum`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cage {
    pub cells: Vec<Coord>,
    pub sum: u16,
}

/// A thermo path: strictly increasing digits from head to tail.
pub type ThermoPath = Vec<Coord>;

/// Serializable description of one rule *instance* — the data a puzzle
/// carries (cage layouts, thermo paths, …). Rebuilds live rule objects
/// via [`RuleData::build`]; this is what M4 will store in save files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleData {
    Diagonal,
    Killer { cages: Vec<Cage> },
    Thermo { paths: Vec<ThermoPath> },
    NonConsecutive,
    AntiKnight,
}

impl RuleData {
    pub fn build(&self) -> Box<dyn VariantRule> {
        match self {
            RuleData::Diagonal => Box::new(diagonal::Diagonal),
            RuleData::Killer { cages } => Box::new(killer::Killer { cages: cages.clone() }),
            RuleData::Thermo { paths } => Box::new(thermo::Thermo { paths: paths.clone() }),
            RuleData::NonConsecutive => Box::new(non_consecutive::NonConsecutive),
            RuleData::AntiKnight => Box::new(anti_knight::AntiKnight),
        }
    }
}

/// The *kind* of rule a player selects (no generated data yet) — generator
/// input. Data is derived from the generated solution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    Diagonal,
    Killer,
    Thermo,
    NonConsecutive,
    AntiKnight,
}

impl From<RuleKind> for RuleData {
    fn from(kind: RuleKind) -> Self {
        match kind {
            RuleKind::Diagonal => RuleData::Diagonal,
            RuleKind::Killer => RuleData::Killer { cages: Vec::new() },
            RuleKind::Thermo => RuleData::Thermo { paths: Vec::new() },
            RuleKind::NonConsecutive => RuleData::NonConsecutive,
            RuleKind::AntiKnight => RuleData::AntiKnight,
        }
    }
}

/// Mask covering digits `lo..=hi` (inclusive), bits `lo-1 .. hi-1`.
pub(crate) fn range_mask(lo: u8, hi: u8) -> DigitMask {
    if lo > hi {
        return 0;
    }
    ((1 << hi) - 1) & !((1 << (lo - 1)) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_mask_covers_inclusive_bounds() {
        assert_eq!(range_mask(1, 9), 0x1FF);
        assert_eq!(range_mask(3, 5), 0b111 << 2);
        assert_eq!(range_mask(4, 4), 1 << 3);
        assert_eq!(range_mask(5, 2), 0);
    }

    #[test]
    fn ruleset_combines_overlays_and_checks() {
        let mut set = RuleSet::new();
        set.push(Box::new(diagonal::Diagonal));
        set.push(Box::new(non_consecutive::NonConsecutive));
        assert_eq!(set.overlays().len(), 2); // two diagonal stripes, none for nonconsecutive
        let cells = [0u8; CELLS];
        assert!(set.is_consistent(&cells));
    }
}
