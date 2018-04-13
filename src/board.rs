use std::fmt::{Display, Write, Formatter, Debug};

use matrix::Matrix;

// A possible choice in a Sudoku puzzle. A single Possibility represents the choice
// to place a certain number at a certain position (row and column) within the board.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Possibility {
    // [0, 9)
    row: usize,
    // [0, 9)
    column: usize,
    // [1, 9]
    number: usize
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Constraint {
    Cell,
    Row,
    Column,
    Box
}

/// Describes the state of a Sudoku puzzle board.
#[derive(Clone)]
pub struct Board {
    // Entries can be in the range [0, 9]. A value of 0 indicates the value is unknown. Stored
    // in row-major order (ie col + row * 9 calculates the cell index for a given row, column pair).
    entries: [usize; 9 * 9]
}

impl Possibility {
    fn new(row: usize, column: usize, number: usize) -> Possibility {
        assert!(row < 9);
        assert!(column < 9);
        assert!(number >= 1 && number <= 9);
        Possibility {
            row, column, number
        }
    }

    fn from_matrix_row(matrix_row: usize) -> Possibility {
        let row = matrix_row / (9 * 9);
        let column = matrix_row / 9 % 9;
        let number = matrix_row % 9 + 1;
        Possibility {
            row, column, number
        }
    }

    fn get_matrix_row(&self) -> usize {
        self.row * (9 * 9) + self.column * 9 + self.number - 1
    }

    fn get_matrix_column(&self, constraint: Constraint) -> usize {
        match constraint {
            Constraint::Cell => {
                self.column + self.row * 9
            },
            Constraint::Row => {
                81 * 1 + self.row * 9 + self.number - 1
            },
            Constraint::Column => {
                81 * 2 + self.column * 9 + self.number - 1
            },
            Constraint::Box => {
                let box_ = (self.column / 3) + (self.row / 3) * 3;
                81 * 3 + box_ * 9 + self.number - 1
            }
        }
    }
}

impl Board {
    pub fn new() -> Board {
        Board {
            entries: [0; 9 * 9]
        }
    }

    pub fn to_line<W: Write>(&self, to: &mut W) {
        for c in self.entries.iter() {
            write!(to, "{} ", c);
        }
    }

    pub fn get_entry(&self, row: usize, column: usize) -> Option<usize> {
        assert!(row < 9);
        assert!(column < 9);
        let entry = self.entries[column + row * 9];
        if entry == 0 { None } else { Some(entry) }
    }

    pub fn solve(&mut self) -> Result<(), ()> {
        let mut matrix = Matrix::new(9 * 9 * 9, 9 * 9 * 4, 9 * 9 * 9 * 4);
        // Initializes the exact cover matrix and removes entries corresponding to knowns.
        self.init_matrix(&mut matrix);
        // Note that these solution_rows do not include the givens, but that's ok because
        // the board already has the givens filled in.
        let solution_rows = matrix.solve()?;

        // Convert solution rows to Sudoku possibilities and record in the board.
        for matrix_row in solution_rows {
            let possibility = Possibility::from_matrix_row(matrix_row);
            self.entries[possibility.column + possibility.row * 9] = possibility.number;
        }

        Ok(())
    }

    pub fn init_matrix(&self, matrix: &mut Matrix) {
        // Reset matrix.
        matrix.clear();

        // First set up the full Sudoku exact cover matrix by adding entries for every combination of
        // row, column, and number
        for row in 0..9 {
            for column in 0..9 {
                for number in 1..10 {
                    let possibility = Possibility::new(row, column, number);
                    let matrix_row = possibility.get_matrix_row();
                    matrix.set_entry(matrix_row, possibility.get_matrix_column(Constraint::Cell));
                    matrix.set_entry(matrix_row, possibility.get_matrix_column(Constraint::Row));
                    matrix.set_entry(matrix_row, possibility.get_matrix_column(Constraint::Column));
                    matrix.set_entry(matrix_row, possibility.get_matrix_column(Constraint::Box));
                }
            }
        }

        // Next remove options from the full exact cover matrix by covering columns that correspond to
        // possibilities that are already known.
        for row in 0..9 {
            for column in 0..9 {
                let entry = self.entries[column + row * 9];
                if entry == 0 {
                    continue;
                }

                let possibility = Possibility::new(row, column, entry);
                matrix.cover_column(possibility.get_matrix_column(Constraint::Cell));
                matrix.cover_column(possibility.get_matrix_column(Constraint::Row));
                matrix.cover_column(possibility.get_matrix_column(Constraint::Column));
                matrix.cover_column(possibility.get_matrix_column(Constraint::Box));
            }
        }
    }

    pub fn from_singleline_str(input: &str) -> Result<Board, ParseBoardError> {
        let mut entries = [0usize; 9 * 9];
        let mut i = 0;
        for c in input.chars() {
            if i >= entries.len() {
                return Err(ParseBoardError::TooManyEntries);
            }
            if let Some(value) = c.to_digit(10) {
                assert!(value >= 0 && value <= 9);
                entries[i] = value as usize;
                i += 1;
            } else if ' ' != c {
                return Err(ParseBoardError::InvalidCharacter(c));
            }
        }
        Ok(Board { entries })
    }

    pub fn from_multiline_str(input: &str) -> Result<Board, ParseBoardError> {
        let mut entries = [0usize; 9 * 9];
        let mut i = 0;
        for c in input.chars() {
            if i >= entries.len() {
                return Err(ParseBoardError::TooManyEntries);
            }
            if let Some(value) = c.to_digit(10) {
                assert!(value >= 1 && value <= 9);
                entries[i] = value as usize;
                i += 1;
            } else if '_' == c {
                entries[i] = 0;
                i += 1;
            } else if ' ' != c {
                return Err(ParseBoardError::InvalidCharacter(c));
            }
        }
        Ok(Board { entries })
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        Display::fmt(self, f)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        // Write one line at a time so it can be padded/indented appropriately
        let mut buf = String::new();
        for row in 0..9 {
            fmt_row(self, f, &mut buf, row);
        }
        Ok(())
    }
}

fn fmt_row(board: &Board, f: &mut Formatter, buf: &mut String, row: usize) -> Result<(), ::std::fmt::Error> {
    buf.clear();
    for col in 0..8 {
        let i = col + row * 9;
        write!(buf, "{} ", board.entries[col + row * 9])?;
    }
    let last_row = row == 8;
    if last_row {
        write!(buf, "{}", board.entries[8 + row * 9])?;
    } else {
        writeln!(buf, "{}", board.entries[8 + row * 9])?;
    }
    f.pad(&buf)?;
    Ok(())
}

#[derive(Debug)]
pub enum ParseBoardError {
    TooManyEntries,
    InvalidCharacter(char)
}

impl Eq for Board {}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..9*9 {
            if self.entries[i] != other.entries[i] {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Possibility, Constraint, Board};
    
    #[test]
    fn possibility_from_matrix_row_consistent_with_get_matrix_row() {
        for sudoku_row in 0..9 {
            for sudoku_column in 0..9 {
                for sudoku_number in 1..10 {
                    let initial_possibility = Possibility::new(sudoku_row, sudoku_column, sudoku_number);
                    let initial_matrix_row = initial_possibility.get_matrix_row();
                    let final_possibility = Possibility::from_matrix_row(initial_matrix_row);
                    let final_matrix_row = final_possibility.get_matrix_row();
                    assert_eq!(initial_possibility, final_possibility);
                    assert_eq!(initial_matrix_row, final_matrix_row);
                }
            }
        }
    }

    #[test]
    fn possibility_get_matrix_row() {
        // (possibility, expected_matrix_row)
        let cases = vec![
            (Possibility::new(0, 0, 1), 0),
        ];
        for (possibility, expected_matrix_row) in cases {
            let actual_matrix_row = possibility.get_matrix_row();
            assert_eq!(expected_matrix_row, actual_matrix_row);
        }
    }

    #[test]
    fn possibility_get_matrix_column_cell() {
        for sudoku_row in 0..9 {
            for sudoku_column in 0..9 {
                let first_possibility = Possibility::new(sudoku_row, sudoku_column, 1);
                let first_matrix_column = first_possibility.get_matrix_column(Constraint::Cell);
                for sudoku_number in 2..10 {
                    let possibility = Possibility::new(sudoku_row, sudoku_column, sudoku_number);
                    let matrix_column = possibility.get_matrix_column(Constraint::Cell);
                    assert_eq!(first_matrix_column, matrix_column);
                }
            }
        }
    }

    #[test]
    fn possibility_get_matrix_column_row() {
        for sudoku_row in 0..9 {
            for sudoku_number in 1..10 {
                let first_possibility = Possibility::new(sudoku_row, 0, sudoku_number);
                let first_matrix_column = first_possibility.get_matrix_column(Constraint::Row);
                for sudoku_column in 1..9 {
                    let possibility = Possibility::new(sudoku_row, sudoku_column, sudoku_number);
                    let matrix_column = possibility.get_matrix_column(Constraint::Row);
                    assert_eq!(first_matrix_column, matrix_column);
                }
            }
        }
    }

    #[test]
    fn possibility_get_matrix_column_column() {
        for sudoku_column in 0..9 {
            for sudoku_number in 1..10 {
                let first_possibility = Possibility::new(0, sudoku_column, sudoku_number);
                let first_matrix_column = first_possibility.get_matrix_column(Constraint::Column);
                for sudoku_row in 1..9 {
                    let possibility = Possibility::new(sudoku_row, sudoku_column, sudoku_number);
                    let matrix_column = possibility.get_matrix_column(Constraint::Column);
                    assert_eq!(first_matrix_column, matrix_column);
                }
            }
        }
    }

    #[test]
    fn possibility_get_matrix_column_box() {
        for sudoku_box_row in 0..3 {
            for sudoku_box_column in 0..3 {
                for sudoku_number in 1..10 {
                    let first_possibility = Possibility::new(sudoku_box_row * 3, sudoku_box_column * 3, sudoku_number);
                    let first_matrix_column = first_possibility.get_matrix_column(Constraint::Box);
                    for sudoku_row in (sudoku_box_row * 3)..(sudoku_box_row * 3 + 3) {
                        for sudoku_column in (sudoku_box_column * 3)..(sudoku_box_column * 3 + 3) {
                            let possibility = Possibility::new(sudoku_row, sudoku_column, sudoku_number);
                            let matrix_column = possibility.get_matrix_column(Constraint::Box);
                            assert_eq!(first_matrix_column, matrix_column, 
                                "Matrix column {} for possibility {:?} did not match \
                                matrix column {} for possibility {:?}",
                                first_matrix_column, first_possibility,
                                matrix_column, possibility);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn board_from_multiline_str() {
        let board = Board::from_multiline_str(
        //   0 1 2 3 4 5 6 7 8
            "5 3 _ _ 7 _ _ _ _\
             6 _ _ 1 9 5 _ _ _\
             _ 9 8 _ _ _ _ 6 _\
             8 _ _ _ 6 _ _ _ 3\
             4 _ _ 8 _ 3 _ _ 1\
             7 _ _ _ 2 _ _ _ 6\
             _ 6 _ _ _ _ 2 8 _\
             _ _ _ 4 1 9 _ _ 5\
             _ _ _ _ 8 _ _ 7 9").unwrap();
        assert_eq!(Some(5), board.get_entry(0, 0));
        assert_eq!(Some(3), board.get_entry(0, 1));
        assert_eq!(None, board.get_entry(0, 2));
        assert_eq!(None, board.get_entry(0, 3));
        assert_eq!(Some(7), board.get_entry(0, 4));
        assert_eq!(Some(9), board.get_entry(1, 4));
        assert_eq!(None, board.get_entry(2, 6));
        assert_eq!(Some(6), board.get_entry(6, 1));
        assert_eq!(Some(7), board.get_entry(8, 7));
        assert_eq!(Some(9), board.get_entry(8, 8));
        assert_eq!(Some(6), board.get_entry(3, 4));
        assert_eq!(Some(8), board.get_entry(4, 3));
        assert_eq!(None, board.get_entry(4, 4));
        assert_eq!(Some(3), board.get_entry(4, 5));
        assert_eq!(Some(2), board.get_entry(5, 4));
    }

    #[test]
    fn board_from_singleline_str() {
        let board = Board::from_singleline_str("5 3 0 0 7 0 0 0 0 6 0 0 1 9 5 0 0 0 0 9 8 0 0 0 0 6 0 8 0 0 0 6 0 0 0 3 4 0 0 8 0 3 0 0 1 7 0 0 0 2 0 0 0 6 0 6 0 0 0 0 2 8 0 0 0 0 4 1 9 0 0 5 0 0 0 0 8 0 0 7 9").unwrap();
        assert_eq!(Some(5), board.get_entry(0, 0));
        assert_eq!(Some(3), board.get_entry(0, 1));
        assert_eq!(None, board.get_entry(0, 2));
        assert_eq!(None, board.get_entry(0, 3));
        assert_eq!(Some(7), board.get_entry(0, 4));
        assert_eq!(Some(9), board.get_entry(1, 4));
        assert_eq!(None, board.get_entry(2, 6));
        assert_eq!(Some(6), board.get_entry(6, 1));
        assert_eq!(Some(7), board.get_entry(8, 7));
        assert_eq!(Some(9), board.get_entry(8, 8));
        assert_eq!(Some(6), board.get_entry(3, 4));
        assert_eq!(Some(8), board.get_entry(4, 3));
        assert_eq!(None, board.get_entry(4, 4));
        assert_eq!(Some(3), board.get_entry(4, 5));
        assert_eq!(Some(2), board.get_entry(5, 4));
    }

    #[test]
    fn singleline_and_multiline_board_equivalency() {
        let a = Board::from_multiline_str(
        //   0 1 2 3 4 5 6 7 8
            "5 3 _ _ 7 _ _ _ _\
             6 _ _ 1 9 5 _ _ _\
             _ 9 8 _ _ _ _ 6 _\
             8 _ _ _ 6 _ _ _ 3\
             4 _ _ 8 _ 3 _ _ 1\
             7 _ _ _ 2 _ _ _ 6\
             _ 6 _ _ _ _ 2 8 _\
             _ _ _ 4 1 9 _ _ 5\
             _ _ _ _ 8 _ _ 7 9").unwrap();
        let b = Board::from_singleline_str("5 3 0 0 7 0 0 0 0 6 0 0 1 9 5 0 0 0 0 9 8 0 0 0 0 6 0 8 0 0 0 6 0 0 0 3 4 0 0 8 0 3 0 0 1 7 0 0 0 2 0 0 0 6 0 6 0 0 0 0 2 8 0 0 0 0 4 1 9 0 0 5 0 0 0 0 8 0 0 7 9").unwrap();
        for row in 0..9 {
            for col in 0..9 {
                assert_eq!(a.get_entry(row, col), b.get_entry(row, col));
            }
        }
    }
}