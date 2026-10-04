//! The classic 9×9 sudoku grid with bitmask bookkeeping.
//!
//! Each row, column, and 3×3 box tracks its used digits as a 9-bit mask
//! (bit `d - 1` set means digit `d` is used). This makes candidate
//! computation a few bitwise operations — the workhorse of both the
//! solver and the generator.

use serde::{Deserialize, Serialize};

/// Number of rows/columns/boxes — always 9 for classic sudoku.
pub const SIZE: usize = 9;
/// Total cell count of the grid.
pub const CELLS: usize = SIZE * SIZE;

/// A bit is set for every digit currently used in a unit.
pub type DigitMask = u16;

/// Bits 0..9 set — i.e. digits 1 through 9 all possible.
pub const ALL_DIGITS: DigitMask = 0x1FF;

/// A cell position on the grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Coord {
    pub row: u8,
    pub col: u8,
}

impl Coord {
    pub fn new(row: u8, col: u8) -> Self {
        assert!(row < 9 && col < 9, "coord out of range: ({row},{col})");
        Coord { row, col }
    }

    /// Flat index into the 81-cell array (row-major).
    pub fn index(self) -> usize {
        self.row as usize * SIZE + self.col as usize
    }

    /// Index of the 3×3 box this cell belongs to (0..9, row-major).
    pub fn box_index(self) -> usize {
        (self.row as usize / 3) * 3 + self.col as usize / 3
    }

    pub fn from_index(index: usize) -> Self {
        Coord {
            row: (index / SIZE) as u8,
            col: (index % SIZE) as u8,
        }
    }
}

/// A sudoku position with incremental row/col/box masks.
///
/// Invariant: `cells[i] == 0` means empty; a nonzero value is always
/// reflected in the three masks of its row, column, and box.
#[derive(Clone)]
pub struct Grid {
    cells: [u8; CELLS],
    row_used: [DigitMask; SIZE],
    col_used: [DigitMask; SIZE],
    box_used: [DigitMask; SIZE],
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            cells: [0; CELLS],
            row_used: [0; SIZE],
            col_used: [0; SIZE],
            box_used: [0; SIZE],
        }
    }

    /// Parses 9 lines of 9 characters; `.` or `0` means empty.
    /// Used by tests to load curated boards.
    pub fn parse(text: &str) -> Grid {
        // Keep digits AND dots — dots are positions, not noise.
        let chars: Vec<char> = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        assert_eq!(chars.len(), CELLS, "board must have exactly 81 cells");
        let mut grid = Grid::new();
        for (i, ch) in chars.iter().enumerate() {
            let d = if *ch == '.' { 0 } else { *ch as u8 - b'0' };
            if d != 0 {
                grid.place(Coord::from_index(i), d)
                    .expect("parsed board must be legal");
            }
        }
        grid
    }

    pub fn get(&self, c: Coord) -> u8 {
        self.cells[c.index()]
    }

    pub fn get_index(&self, index: usize) -> u8 {
        self.cells[index]
    }

    pub fn is_empty(&self, c: Coord) -> bool {
        self.cells[c.index()] == 0
    }

    pub fn cells(&self) -> &[u8; CELLS] {
        &self.cells
    }

    /// Builds a grid from raw digits **without legality checks** — test-only,
    /// for constructing impossible boards (e.g. two 5s in one box) that the
    /// normal API deliberately cannot represent.
    #[cfg(test)]
    pub(crate) fn parse_unchecked(text: &str) -> Grid {
        let chars: Vec<char> = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        assert_eq!(chars.len(), CELLS, "board must have exactly 81 cells");
        let mut grid = Grid::new();
        for (i, ch) in chars.iter().enumerate() {
            grid.cells[i] = if *ch == '.' { 0 } else { *ch as u8 - b'0' };
        }
        grid
    }

    /// Bitmask of digits still placeable at `c` under classic rules.
    pub fn candidates(&self, c: Coord) -> DigitMask {
        let used =
            self.row_used[c.row as usize] | self.col_used[c.col as usize] | self.box_used[c.box_index()];
        !used & ALL_DIGITS
    }

    /// Whether digit `d` may be placed at `c` right now.
    pub fn is_allowed(&self, c: Coord, d: u8) -> bool {
        self.is_empty(c) && self.candidates(c) & (1 << (d - 1)) != 0
    }

    /// Places a digit, updating the masks.
    /// Returns `Err` if the cell is occupied or the move breaks classic rules.
    pub fn place(&mut self, c: Coord, d: u8) -> Result<(), String> {
        assert!((1..=9).contains(&d), "digit out of range: {d}");
        if !self.is_empty(c) {
            return Err(format!("cell {:?} already holds {}", c, self.get(c)));
        }
        if self.candidates(c) & (1 << (d - 1)) == 0 {
            return Err(format!("digit {d} not allowed at {:?}", c));
        }
        let i = c.index();
        self.cells[i] = d;
        self.row_used[c.row as usize] |= 1 << (d - 1);
        self.col_used[c.col as usize] |= 1 << (d - 1);
        self.box_used[c.box_index()] |= 1 << (d - 1);
        Ok(())
    }

    /// Removes a digit (assumes the cell was non-empty), restoring the masks.
    pub fn remove(&mut self, c: Coord) -> u8 {
        let d = self.cells[c.index()];
        assert!(d != 0, "removing from an empty cell");
        let bit = !(1 << (d - 1));
        self.cells[c.index()] = 0;
        self.row_used[c.row as usize] &= bit;
        self.col_used[c.col as usize] &= bit;
        self.box_used[c.box_index()] &= bit;
        d
    }
}

impl Default for Grid {
    fn default() -> Self {
        Grid::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_grid_allows_everything() {
        let g = Grid::new();
        for r in 0..9u8 {
            for c in 0..9u8 {
                assert_eq!(g.candidates(Coord::new(r, c)), 0b1_1111_1111);
            }
        }
    }

    #[test]
    fn place_and_remove_roundtrip() {
        let mut g = Grid::new();
        let c = Coord::new(4, 4);
        g.place(c, 5).unwrap();
        assert!(!g.is_allowed(c, 5)); // occupied
        assert!(!g.is_allowed(Coord::new(4, 0), 5)); // same row
        assert!(!g.is_allowed(Coord::new(0, 4), 5)); // same col
        assert!(!g.is_allowed(Coord::new(3, 3), 5)); // same box
        assert!(g.is_allowed(Coord::new(0, 0), 5)); // unrelated
        assert_eq!(g.remove(c), 5);
        assert!(g.is_allowed(c, 5));
    }

    #[test]
    fn place_rejects_conflicts() {
        let mut g = Grid::new();
        g.place(Coord::new(0, 0), 1).unwrap();
        assert!(g.place(Coord::new(0, 8), 1).is_err());
        assert!(g.place(Coord::new(8, 0), 1).is_err());
        assert!(g.place(Coord::new(2, 2), 1).is_err());
        assert!(g.place(Coord::new(0, 0), 2).is_err()); // occupied
    }

    #[test]
    fn parse_reads_rows_and_columns() {
        let g = Grid::parse("53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79");
        assert_eq!(g.get(Coord::new(0, 0)), 5);
        assert_eq!(g.get(Coord::new(0, 1)), 3);
        assert_eq!(g.get(Coord::new(8, 8)), 9);
        assert!(g.is_empty(Coord::new(0, 2)));
    }
}
