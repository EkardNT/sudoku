use board::Constraint;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct SparseRow(usize);
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct SparseColumn(usize);
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DenseRow(usize);
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct DenseColumn(usize);

impl DenseRow {
    pub fn new(dense_row_index: usize) -> DenseRow {
        assert!(dense_row_index < 9 * 9 * 9);
        DenseRow(dense_row_index)
    }

    pub fn to_sparse(&self) -> SparseRow {
        SparseRow(self.0)
    }

    /// Determines the row of the top neighbor of this DenseRow when the exact cover sudoku 
    /// matrix is in its "natural" (aka clear) state.
    pub fn natural_up(&self, column: &DenseColumn) -> DenseRow {
        match column.constraint() {
            Constraint::Cell => {
                let row_position = self.0 % 9;
                if row_position == 0 {
                    DenseRow(self.0 + 8)
                } else {
                    DenseRow(self.0 - 1)
                }
            },
            Constraint::Row => {
                let row_group = self.0 / 9;
                if row_group % 9 == 0 {
                    DenseRow(self.0 + 9 * 8)
                } else {
                    DenseRow(self.0 - 9)
                }
            },
            Constraint::Column => {
                let row_group = self.0 / (9 * 9);
                if row_group % 9 == 0 {
                    DenseRow(self.0 + 9 * 9 * 8)
                } else {
                    DenseRow(self.0 - 9 * 9)
                }
            },
            Constraint::Box => {
                let row_group = self.0 / 9;
                if row_group % 3 == 0 {
                    let row_factor = (row_group % (9 * 3)) / 9;
                    if row_factor == 0 {
                        DenseRow(self.0 + 18 * 9 + 9 * 2)
                    } else {
                        DenseRow(self.0 - 9 * 7)
                    }
                } else {
                    DenseRow(self.0 - 9)
                }
            }
        }
    }

    /// Determines the row of the bottom neighbor of this DenseRow when the exact cover sudoku 
    /// matrix is in its "natural" (aka clear) state.
    pub fn natural_down(&self, column: &DenseColumn) -> DenseRow {
        match column.constraint() {
            Constraint::Cell => {
                let row_position = self.0 % 9;
                if row_position == 8 {
                    DenseRow(self.0 - 8)
                } else {
                    DenseRow(self.0 + 1)
                }
            },
            Constraint::Row => {
                let row_group = self.0 / 9;
                if row_group % 9 == 8 {
                    DenseRow(self.0 - 9 * 8)
                } else {
                    DenseRow(self.0 + 9)
                }
            },
            Constraint::Column => {
                let row_group = self.0 / (9 * 9);
                if row_group % 9 == 8 {
                    DenseRow(self.0 - 9 * 9 * 8)
                } else {
                    DenseRow(self.0 + 9 * 9)
                }
            },
            Constraint::Box => {
                // row_group will be in [0, 81)
                let row_group = self.0 / 9; // 18 / 9 = 2
                if row_group % 3 == 2 { // TRUE: 2 % 3 == 2
                    // row_factor will be in [0, 3)
                    let row_factor = (row_group % (9 * 3)) / 9; // (2 % 27) / 9 == 0
                    if row_factor == 2 {
                        DenseRow(self.0 - 18 * 9 - 9 * 2)
                    } else {
                        DenseRow(self.0 + 9 * 7)
                    }
                } else {
                    DenseRow(self.0 + 9)
                }
            }
        }
    }
}

impl SparseRow {
    pub fn new(sparse_row_index: usize) -> SparseRow {
        assert!(sparse_row_index < 9 * 9 * 9);
        SparseRow(sparse_row_index)
    }

    pub fn to_dense(&self, column: &SparseColumn) -> DenseRow {
        DenseRow(self.0)
    }
}

impl DenseColumn {
    pub fn new(dense_column_index: usize) -> DenseColumn {
        assert!(dense_column_index < 4);
        DenseColumn(dense_column_index)
    }

    pub fn to_sparse(&self, row: &DenseRow) -> SparseColumn {
        // These formulas were all discovered by looking at the table at http://www.stolaf.edu/people/hansonr/sudoku/exactcovermatrix.htm
        match self.constraint() {
            Constraint::Cell => SparseColumn(row.0 / 9),
            Constraint::Row => SparseColumn(81 * 1 + row.0 % 9 + 9 * (row.0 / 81)),
            Constraint::Column => SparseColumn(81 * 2 + row.0 % 81),
            Constraint::Box => SparseColumn(81 * 3 + row.0 % 9 + 9 * ((row.0 / (9 * 3)) % 3) + 3 * 9 * (row.0 / (9 * 9 * 3)))
        }
    }

    fn constraint(&self) -> Constraint {
        match self.0 {
            0 => Constraint::Cell,
            1 => Constraint::Row,
            2 => Constraint::Column,
            3 => Constraint::Box,
            _ => panic!("Illegal DenseColumn value {}", self.0)
        }
    }

    /// Determines the column of the left neighbor of this DenseColumn when the exact cover sudoku 
    /// matrix is in its "natural" (aka clear) state.
    pub fn natural_left(&self) -> DenseColumn {
        DenseColumn::new(match self.0 {
            0 => 3,
            1 => 0,
            2 => 1,
            3 => 2,
            _ => panic!("Unexpected DenseColumn value {}", self.0)
        })
    }

    /// Determines the column of the right neighbor of this DenseColumn when the exact cover sudoku 
    /// matrix is in its "natural" (aka clear) state.
    pub fn natural_right(&self) -> DenseColumn {
        DenseColumn::new(match self.0 {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 0,
            _ => panic!("Unexpected DenseColumn value {}", self.0)
        })
    }
}

impl SparseColumn {
    pub fn new(sparse_column_index: usize) -> SparseColumn {
        assert!(sparse_column_index < 9 * 9 * 4);
        SparseColumn(sparse_column_index)
    }

    pub fn to_dense(&self) -> DenseColumn {
        DenseColumn::new(self.0 / (9 * 9))
    }

    pub fn first_row(&self) -> SparseRow {
        // Again, this page is invaluable: http://www.stolaf.edu/people/hansonr/sudoku/exactcovermatrix.htm
        match self.to_dense().constraint() {
            Constraint::Cell => {
                // Note constraint_offset == self.0
                let constraint_offset = self.0 - 0 * 9 * 9;
                SparseRow::new(constraint_offset * 9)
            },
            Constraint::Row => {
                let constraint_offset = self.0 - 1 * 9 * 9;
                SparseRow::new(constraint_offset + (constraint_offset / 9) * 9 * 9 - (constraint_offset / 9) * 9)
            },
            Constraint::Column => {
                let constraint_offset = self.0 - 2 * 9 * 9;
                SparseRow::new(constraint_offset)
            },
            Constraint::Box => {
                // (252, 24)
                // constraint_offset = 252 - 3 * 9 * 9 = 9
                let constraint_offset = self.0 - 3 * 9 * 9; // [0, 81)
                // major_group = 0
                let major_group = constraint_offset / (9 * 3); // [0, 3)
                // minor_group = 1
                let minor_group = constraint_offset % (9 * 3) / 9; // [0, 3)
                // stagger = 0
                let stagger = constraint_offset % 9; // [0, 9) - this gives the finest level of sawtooth pattern
                // row = (0 + )
                SparseRow::new(major_group * (9 * 9 * 3) + minor_group * (9 * 3) + stagger)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SparseColumn, SparseRow, DenseColumn, DenseRow};

    #[test]
    fn dense_column_to_sparse() {
        // (dense_row, dense_column, expected_sparse_column)
        let cases = vec![
            (0, 0, 0),
            (0, 1, 81),
            (0, 2, 162),
            (0, 3, 243),
            (8, 0, 0),
            (8, 1, 89),
            (8, 2, 170),
            (8, 3, 251),
            (26, 0, 2),
            (26, 1, 89),
            (26, 2, 188),
            (26, 3, 251),
            (30, 0, 3),
            (30, 1, 84),
            (30, 2, 192),
            (30, 3, 255),
            (163, 0, 18),
            (163, 1, 100),
            (297, 0, 33),
            (297, 1, 108),
            (297, 2, 216),
            (297, 3, 288),
            (540, 0, 60),
            (540, 1, 135),
            (540, 2, 216),
            (540, 3, 315),
            (720, 0, 80),
            (720, 1, 153),
            (720, 2, 234),
            (720, 3, 315),
            (728, 0, 80),
            (728, 1, 161),
            (728, 2, 242),
            (728, 3, 323)
        ];
        for &(ref dense_row, ref dense_column, ref expected_sparse_column) in &cases {
            assert_eq!(SparseColumn::new(*expected_sparse_column), DenseColumn::new(*dense_column).to_sparse(&DenseRow::new(*dense_row)));
        }
    }

    #[test]
    fn natural_left() {
        // (initial_dense_column, expected_dense_column)
        let cases = vec![
            (0, 3),
            (1, 0),
            (2, 1),
            (3, 2)
        ];
        for &(ref initial_dense_column, ref expected_dense_column) in &cases {
            for dense_row_index in 0..9*9*9 {
                let initial_column = DenseColumn::new(*initial_dense_column);
                let expected_column = DenseColumn::new(*expected_dense_column);
                let actual_column = initial_column.natural_left();
                assert_eq!(expected_column, actual_column);
            }
        }
    }

    #[test]
    fn natural_right() {
        // (initial_dense_column, expected_dense_column)
        let cases = vec![
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0)
        ];
        for &(ref initial_dense_column, ref expected_dense_column) in &cases {
            let initial_column = DenseColumn::new(*initial_dense_column);
            let expected_column = DenseColumn::new(*expected_dense_column);
            let actual_column = initial_column.natural_right();
            assert_eq!(expected_column, actual_column);
        }
    }

    #[test]
    fn natural_up() {
        // (initial_dense_row, initial_dense_column, expected_dense_row) 
        let cases = vec![
            (0, 0, 8),
            (8, 0, 7),
            (9, 0, 17),
            (4, 0, 3),
            (270, 0, 278),
            (273, 0, 272),
            (278, 0, 277),
            (720, 0, 728),
            (725, 0, 724),
            (728, 0, 727),
            (0, 1, 72),
            (8, 1, 80),
            (9, 1, 0),
            (4, 1, 76),
            (72, 1, 63),
            (80, 1, 71),
            (73, 1, 64),
            (270, 1, 261),
            (273, 1, 264),
            (278, 1, 269),
            (720, 1, 711),
            (725, 1, 716),
            (728, 1, 719),
            (0, 2, 648),
            (4, 2, 652),
            (8, 2, 656),
            (162, 2, 81),
            (170, 2, 89),
            (720, 2, 639),
            (728, 2, 647),
            (0, 3, 180),
            (4, 3, 184),
            (8, 3, 188),
            (81, 3, 18),
            (180, 3, 171),
            (185, 3, 176),
            (188, 3, 179),
            (459, 3, 396),
        ];
        for &(ref initial_dense_row, ref initial_dense_column, ref expected_dense_row) in &cases {
            let initial_row = DenseRow::new(*initial_dense_row);
            let initial_column = DenseColumn::new(*initial_dense_column);
            let expected_row = DenseRow::new(*expected_dense_row);
            let actual_row = initial_row.natural_up(&initial_column);
            assert_eq!(expected_row, actual_row);
        }
    }

    #[test]
    fn natural_down() {
        // (initial_dense_row, initial_dense_column, expected_dense_row) 
        let cases = vec![
            (0, 0, 1),
            (8, 0, 0),
            (9, 0, 10),
            (4, 0, 5),
            (270, 0, 271),
            (273, 0, 274),
            (278, 0, 270),
            (720, 0, 721),
            (725, 0, 726),
            (728, 0, 720),
            (0, 1, 9),
            (8, 1, 17),
            (9, 1, 18),
            (4, 1, 13),
            (72, 1, 0),
            (80, 1, 8),
            (73, 1, 1),
            (270, 1, 279),
            (273, 1, 282),
            (278, 1, 287),
            (720, 1, 648),
            (725, 1, 653),
            (728, 1, 656),
            (0, 2, 81),
            (4, 2, 85),
            (8, 2, 89),
            (162, 2, 243),
            (170, 2, 251),
            (720, 2, 72),
            (728, 2, 80),
            (0, 3, 9),
            (4, 3, 13),
            (8, 3, 17),
            (18, 3, 81),
            (171, 3, 180),
            (175, 3, 184),
            (179, 3, 188),
            (396, 3, 459),
            (404, 3, 467),
            (720, 3, 540),
            (728, 3, 548)
        ];
        for &(ref initial_dense_row, ref initial_dense_column, ref expected_dense_row) in &cases {
            let initial_row = DenseRow::new(*initial_dense_row);
            let initial_column = DenseColumn::new(*initial_dense_column);
            let expected_row = DenseRow::new(*expected_dense_row);
            let actual_row = initial_row.natural_down(&initial_column);
            assert_eq!(expected_row, actual_row);
        }
    }

    #[test]
    fn natural_up_down_reflexive() {
        for dense_row_index in 0..9*9*9 {
            for dense_column_index in 0..4 {
                let initial_row = DenseRow::new(dense_row_index);
                let initial_column = DenseColumn::new(dense_column_index);
                // Test going up then down is reflexive
                let up_row = initial_row.natural_up(&initial_column);
                let up_down_row = up_row.natural_down(&initial_column);
                assert_eq!(initial_row, up_down_row);
                // Test going down then up is reflexive
                let down_row = initial_row.natural_down(&initial_column);
                let down_up_row = down_row.natural_up(&initial_column);
                assert_eq!(initial_row, down_up_row);
            }
        }
    }

    #[test]
    fn natural_left_right_reflexive() {
        for dense_column_index in 0..4 {
            let initial_column = DenseColumn::new(dense_column_index);
            // Test going left then right is reflexive
            let left_row = initial_column.natural_left();
            let left_right_row = left_row.natural_right();
            assert_eq!(initial_column, left_right_row);
            // Test going right then left is reflexive
            let right_row = initial_column.natural_right();
            let right_left_row = right_row.natural_left();
            assert_eq!(initial_column, right_left_row);
        }
    }

    #[test]
    fn first_row() {
        // (initial_sparse_column, expected_sparse_row)
        let cases = vec![
            (0, 0),
            (1, 9),
            (2, 18),
            (80, 720),
            (81, 0),
            (82, 1),
            (83, 2),
            (87, 6),
            (90, 81),
            (91, 82),
            (97, 88),
            (98, 89),
            (99, 162),
            (153, 648),
            (160, 655),
            (161, 656),
            (162, 0),
            (170, 8),
            (171, 9),
            (179, 17),
            (242, 80),
            (243, 0),
            (244, 1),
            (248, 5),
            (251, 8),
            (252, 27),
            (253, 28),
            (260, 35),
            (261, 54),
            (269, 62),
            (270, 243),
            (278, 251),
            (323, 548)
        ];
        for &(initial_sparse_column, expected_sparse_row) in &cases {
            let initial_column = SparseColumn::new(initial_sparse_column);
            let expected_row = SparseRow::new(expected_sparse_row);
            let actual_row = initial_column.first_row();
            assert_eq!(expected_row, actual_row);
        }
    }
}