mod coords;
mod matrix;
mod board;

use std::fmt::{Display, Write, Formatter, Debug};
use std::str::FromStr;

use coords::*;
use matrix::*;
pub use board::Board;
pub use matrix::Matrix;

pub trait Solve {
    fn solve(&mut self, board: &mut Board) -> Result<(), ()>;
}

// 729 possibilities, aka rows in the exact cover matrix. The number comes from
// 9 * 9 cells on the board, each of which can have one of 9 numbers.
const POSSIBILITIES: usize = 9 * 9 * 9;
// 324 constraints, aka columns in the exact cover matrix. There are 9 cell
// constraints, 9 row constraints, 9 column constraints, and 9 box constraints,
// each of which consist of 9 numbers.
const CONSTRAINTS: usize = 9 * 9 + 9 * 9 + 9 * 9 + 9 * 9;
// Each possibility contributes only 4 ones in the exact cover matrix. This fact,
// combined with the regular nature of the sudoku exact cover matrix, allows us to
// represent the sparse exact cover matrix in a space-efficient dense representation.
const NONZERO_CONSTRAINTS_PER_POSSIBILITY: usize = 4;

pub struct DLXSolver {
    // See http://www.stolaf.edu/people/hansonr/sudoku/exactcovermatrix.htm for the full
    // expanded exact cover matrix.
    matrix: [[Link; NONZERO_CONSTRAINTS_PER_POSSIBILITY]; POSSIBILITIES],
    headers: [Header; CONSTRAINTS]
}

#[derive(Debug, Default, Clone, Copy)]
struct Link {
    // Going up or down involves a change in row.
    up: DenseRow,
    // Going up or down involves a change in row.
    down: DenseRow,
    // Going left or right involves a change in column.
    left: DenseColumn,
    // Going left or right involves a change in column.
    right: DenseColumn
}

#[derive(Debug, Default, Clone, Copy)]
struct Header {
    // The number of remaining 1 entries in this column in the exact cover matrix. Only used for column headers.
    column_size: usize,
    // Which column this link is a part of.
    column_index: SparseColumn,
    // Going up or down involves a change in row.
    first_row: DenseRow,
    // Going left or right involves a change in column.
    left: SparseColumn,
    // Going left or right involves a change in column.
    right: SparseColumn
}

impl DLXSolver {
    pub fn new() -> DLXSolver {
        // Allocate memory for matrix and headers.
        let mut matrix = [[Link::default(); NONZERO_CONSTRAINTS_PER_POSSIBILITY]; POSSIBILITIES];
        let mut headers = [Header::default(); CONSTRAINTS];

        // Initialize a cleared DLXSolver.
        let mut solver = DLXSolver {
            matrix,
            headers
        };
        solver.clear();

        solver
    }

    fn clear(&mut self) {
        // Set up header links.
        for column_index in 0..CONSTRAINTS {
            let sparse_column_index = coords::SparseColumn::new(column_index);
            let mut header = self.headers[column_index];
            header.column_size = 9;
            header.column_index = sparse_column_index;
            header.left = SparseColumn::new(if column_index == 0 { CONSTRAINTS - 1 } else { column_index - 1 });
            header.right = SparseColumn::new(if column_index == CONSTRAINTS - 1 { 0 } else { column_index + 1 });
            header.first_row = sparse_column_index.first_row().to_dense(&sparse_column_index);
        }

        // Set up matrix links.
        for row_index in 0..POSSIBILITIES {
            let mut row: &mut [Link] = &mut self.matrix[row_index];
            // Sparse and dense rows have same value.
            let dense_row = DenseRow::new(row_index);
            let sparse_row = SparseRow::new(row_index);

            reset_link(row, &dense_row, 0);
            reset_link(row, &dense_row, 1);
            reset_link(row, &dense_row, 2);
            reset_link(row, &dense_row, 3);
        }
    }

    pub fn print_row(&self, row_index: usize) {
        let row = &self.matrix[row_index];
        println!("row {}", row_index);
        for ref link in row {
            println!("link: {:?}", link);
        }
    }
}

fn reset_link(row: &mut [Link], dense_row: &DenseRow, dense_column_index: usize) {
    let mut constraint: &mut Link = &mut row[dense_column_index];
    let dense_column = DenseColumn::new(dense_column_index);
    constraint.left = dense_column.natural_left();
    constraint.right = dense_column.natural_right();
    constraint.up = dense_row.natural_up(&dense_column);
    constraint.down = dense_row.natural_down(&dense_column);
}

impl Solve for DLXSolver {
    fn solve(&mut self, board: &mut Board) -> Result<(), ()> {
        Err(())
    }
}