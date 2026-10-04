//! A Dancing Links (DLX) implementation of Knuth's Algorithm X.
//!
//! We use it as an *exact cover* solver with solution counting: sudoku maps
//! to exact cover (see [`crate::sudoku`]), and counting solutions with a
//! limit of 2 is how Polyku proves a puzzle has *exactly one* solution.
//!
//! The toroidal quadruply-linked-list trick: removing a node from a circular
//! list is reversible if you remember its neighbors — `cover`/`uncover`
//! exactly undo each other, so after a full search the structure is intact
//! and the solver can be reused.

const ROOT: usize = 0;

struct Node {
    left: usize,
    right: usize,
    up: usize,
    down: usize,
    /// Column header this node belongs to (headers point to themselves).
    col: usize,
    /// Identifies the caller's row (headers store `u32::MAX`).
    row_id: u32,
}

/// An exact-cover problem: `columns` primary columns, rows added incrementally.
pub struct ExactCover {
    nodes: Vec<Node>,
    /// Number of active rows per column.
    col_size: Vec<usize>,
    partial: Vec<u32>,
    found: u32,
    limit: u32,
    first: Option<Vec<u32>>,
}

impl ExactCover {
    /// Creates a problem with `columns` primary columns (all must be covered).
    /// Callers address columns 0-based; internally the root occupies index 0.
    pub fn new(columns: usize) -> Self {
        // Index 0 is the root; 1..=columns are the column headers.
        let mut nodes = Vec::with_capacity(columns + 1);
        for i in 0..=columns {
            nodes.push(Node {
                left: i.saturating_sub(1),
                right: i + 1,
                up: i,
                down: i,
                col: i,
                row_id: u32::MAX,
            });
        }
        nodes[ROOT].left = columns;
        nodes[columns].right = ROOT;
        ExactCover {
            nodes,
            col_size: vec![0; columns + 1],
            partial: Vec::new(),
            found: 0,
            limit: 0,
            first: None,
        }
    }

    /// Adds a row that covers exactly the given columns (0-based).
    /// `id` is handed back when the row is part of the first found solution.
    pub fn add_row(&mut self, id: u32, columns: &[usize]) {
        let first = self.nodes.len();
        for (k, &c0) in columns.iter().enumerate() {
            // Callers use 0-based columns; headers live at 1..=columns.
            let c = c0 + 1;
            // Horizontally: circular row list. Vertically: bottom of column.
            let prev = if k == 0 { first + columns.len() - 1 } else { self.nodes.len() - 1 };
            let next = if k + 1 == columns.len() { first } else { self.nodes.len() + 1 };
            let up = self.nodes[c].up;
            self.nodes.push(Node { left: prev, right: next, up, down: c, col: c, row_id: id });
            self.nodes[up].down = self.nodes.len() - 1;
            self.nodes[c].up = self.nodes.len() - 1;
            self.col_size[c] += 1;
        }
    }

    fn cover(&mut self, c: usize) {
        // Unlink the column header itself from the header list.
        let l = self.nodes[c].left;
        let r = self.nodes[c].right;
        self.nodes[l].right = r;
        self.nodes[r].left = l;
        // For every row crossing this column, unlink all its other nodes.
        let mut i = self.nodes[c].down;
        while i != c {
            let mut j = self.nodes[i].right;
            while j != i {
                let u = self.nodes[j].up;
                let d = self.nodes[j].down;
                self.nodes[u].down = d;
                self.nodes[d].up = u;
                self.col_size[self.nodes[j].col] -= 1;
                j = self.nodes[j].right;
            }
            i = self.nodes[i].down;
        }
    }

    fn uncover(&mut self, c: usize) {
        let mut i = self.nodes[c].up;
        while i != c {
            let mut j = self.nodes[i].left;
            while j != i {
                let u = self.nodes[j].up;
                let d = self.nodes[j].down;
                self.nodes[u].down = j;
                self.nodes[d].up = j;
                self.col_size[self.nodes[j].col] += 1;
                j = self.nodes[j].left;
            }
            i = self.nodes[i].up;
        }
        let l = self.nodes[c].left;
        let r = self.nodes[c].right;
        self.nodes[l].right = c;
        self.nodes[r].left = c;
    }

    /// Chooses the active column with the fewest rows — the branching
    /// heuristic that makes Algorithm X fast in practice.
    fn choose_column(&self) -> usize {
        let mut best = self.nodes[ROOT].right;
        let mut size = self.col_size[best];
        let mut c = self.nodes[best].right;
        while c != ROOT {
            if self.col_size[c] < size {
                best = c;
                size = self.col_size[c];
                if size <= 1 {
                    break; // can't do better (0 is a dead end, 1 is forced)
                }
            }
            c = self.nodes[c].right;
        }
        best
    }

    fn search(&mut self) {
        if self.found >= self.limit {
            return;
        }
        if self.nodes[ROOT].right == ROOT {
            // All columns covered — `partial` is one complete solution.
            self.found += 1;
            if self.first.is_none() {
                self.first = Some(self.partial.clone());
            }
            return;
        }
        let c = self.choose_column();
        if self.col_size[c] == 0 {
            return; // dead end: an uncoverable column
        }
        self.cover(c);
        let mut i = self.nodes[c].down;
        while i != c {
            let row_id = self.nodes[i].row_id;
            self.partial.push(row_id);
            // Cover every other column of this row.
            let mut j = self.nodes[i].right;
            while j != i {
                self.cover(self.nodes[j].col);
                j = self.nodes[j].right;
            }
            self.search();
            // Uncover in exact reverse order.
            let mut j = self.nodes[i].left;
            while j != i {
                self.uncover(self.nodes[j].col);
                j = self.nodes[j].left;
            }
            self.partial.pop();
            i = self.nodes[i].down;
            if self.found >= self.limit {
                break;
            }
        }
        self.uncover(c);
    }

    /// Searches for up to `limit` solutions and returns how many were found.
    /// The structure is left fully intact, so this can be called repeatedly.
    pub fn count_solutions(&mut self, limit: u32) -> u32 {
        assert!(limit >= 1);
        self.found = 0;
        self.first = None;
        self.partial.clear();
        self.limit = limit;
        self.search();
        self.found
    }

    /// The first solution found by the most recent `count_solutions` call.
    pub fn first_solution(&self) -> Option<&[u32]> {
        self.first.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knuth_example_has_one_solution() {
        // The worked example from Knuth's DLX paper: 7 columns; the unique
        // cover is rows {1,4} {3,5,6} {2,7}.
        let mut p = ExactCover::new(7);
        p.add_row(0, &[0, 3, 6]);
        p.add_row(1, &[0, 3]);
        p.add_row(2, &[3, 4, 6]);
        p.add_row(3, &[2, 4, 5]);
        p.add_row(4, &[1, 2, 5, 6]);
        p.add_row(5, &[1, 6]);
        assert_eq!(p.count_solutions(2), 1);
        let mut rows: Vec<u32> = p.first_solution().unwrap().to_vec();
        rows.sort_unstable();
        assert_eq!(rows, vec![1, 3, 5]);
    }

    #[test]
    fn counts_up_to_limit_and_is_reusable() {
        // Two rows that each cover everything: two distinct solutions.
        let mut p = ExactCover::new(2);
        p.add_row(0, &[0, 1]);
        p.add_row(1, &[0, 1]);
        assert_eq!(p.count_solutions(2), 2);
        assert_eq!(p.count_solutions(1), 1); // stops early
        assert_eq!(p.count_solutions(5), 2); // intact after reuse
    }

    #[test]
    fn unsolvable_counts_zero() {
        let mut p = ExactCover::new(3);
        p.add_row(0, &[0, 1]);
        // column 2 never covered
        assert_eq!(p.count_solutions(2), 0);
    }
}
