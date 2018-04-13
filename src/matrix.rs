use std::fmt::{Display, Debug, Formatter};

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
// Each of the POSSIBILITIES rows has four nonzero column entries, plus one header node for
// every one of the CONSTRAINTS columns, plus one root.
const MATRIX_NODE_COUNT: usize = 1 + CONSTRAINTS + POSSIBILITIES * NONZERO_CONSTRAINTS_PER_POSSIBILITY;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NodeKind {
    Root,
    Header,
    Entry
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::Root
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
struct Node {
    kind: NodeKind,
    // How many nodes are in the column. Only set for header nodes, otherwise usize::MAX.
    column_size: usize,
    // 0-based column index. usize::MAX for the root.
    column_index: usize,
    // 0-based row index. usize::MAX for root and headers.
    row_index: usize,
    // 0-based offset into Node array of up neighbor.
    up: usize,
    // 0-based offset into Node array of down neighbor.
    down: usize,
    // 0-based offset into Node array of left neighbor.
    left: usize,
    // 0-based offset into Node array of right neighbor.
    right: usize
}

#[derive(Clone, Eq, PartialEq)]
pub struct Matrix {
    row_count: usize,
    column_count: usize,
    // Contains all nodes, including the root, column headers, and entries.
    nodes: Vec<Node>,
    row_fronts: Vec<Option<usize>>
}

impl Matrix {
    const ROOT_INDEX: usize = 0;

    pub fn new(row_count: usize, column_count: usize, entry_capacity: usize) -> Self {
        assert!(row_count > 0 && column_count < ::std::usize::MAX);
        assert!(column_count > 0 && row_count < ::std::usize::MAX);

        let nodes = Vec::with_capacity(1 + column_count + entry_capacity);
        let row_fronts = vec![None; row_count];
        let mut matrix = Matrix { row_count, column_count, nodes, row_fronts };
        matrix.clear();

        matrix
    }

    pub fn clear(&mut self) {
        // Clear nodes. Note this doesn't deallocate any memory.
        self.nodes.clear();

        // Create root.
        self.nodes.push(Node {
            kind: NodeKind::Root,
            column_size: ::std::usize::MAX,
            column_index: ::std::usize::MAX,
            row_index: ::std::usize::MAX,
            up: Matrix::ROOT_INDEX,
            down: Matrix::ROOT_INDEX,
            left: self.column_count,
            right: 1
        });

        // Create column headers.
        for column in 0..self.column_count {
            let column_index = column + 1;
            self.nodes.push(Node {
                kind: NodeKind::Header,
                column_size: 0,
                column_index: column,
                row_index: ::std::usize::MAX,
                up: column_index,
                down: column_index,
                // Note this is correct even when column == 0, because the left neighbor of the
                // leftmost 
                left: column,
                right: if column == self.column_count - 1 { 0 } else { column + 2 }
            });
        }

        // Reset all row_fronts to None.
        for front in &mut self.row_fronts {
            *front = None
        }
    }

    pub fn set_entry(&mut self, row_index: usize, column_index: usize) {
        assert!(row_index < self.row_count, "row_index ({}) must be less than self.row_count ({})", row_index, self.row_count);
        assert!(column_index < self.column_count, "column_index ({}) must be less than self.column_count ({})", column_index, self.column_count);

        // `nodes` index of the column header for this entry.
        let header_index = 1 + column_index;

        // Increase column size.
        self.nodes[header_index].column_size += 1;

        // `nodes` index of the new entry node.
        let entry_node_index = self.nodes.len();
        // The `nodes` index of the column header's existing `down` node. The new entry
        // node will be inserted between the header and this `down` node.
        let prev_header_down_index = self.nodes[header_index].down;

        // Set header's `down` pointer to the new entry node.
        self.nodes[header_index].down = entry_node_index;
        // Set what used to be the header.down node's `up` pointer to the new entry node.
        // Note that if this column was previously empty then header_index == prev_header_down_index,
        // which works fine.
        self.nodes[prev_header_down_index].up = entry_node_index;

        // Depending on whether there's already an entry in this row or not, we need to either initialize
        // this row with the new entry or link the new entry into the existing row.
        let (left, right) = match self.row_fronts[row_index] {
            Some(row_front_index) => {
                // If there is already an entry in this row, then link the new node in between the
                // row_front node and the row_front.right node. Note that if the row previously had
                // only one node then row_front == row_front.right, which works fine.
                let prev_row_front_right_index = self.nodes[row_front_index].right;

                // Modify the row_front.right pointer and the row_front.right.left pointer.
                self.nodes[row_front_index].right = entry_node_index;
                self.nodes[prev_row_front_right_index].left = entry_node_index;

                (row_front_index, prev_row_front_right_index)
            },
            None => {
                // Set the row_fronts[row_index] to the new entry node.
                self.row_fronts[row_index] = Some(entry_node_index);

                // If there was no entry in this row yet, then link the new entry node to itself.
                (entry_node_index, entry_node_index)
            }
        };

        // Finish by actually adding the new entry snode.
        self.nodes.push(Node {
            kind: NodeKind::Entry,
            column_size: ::std::usize::MAX,
            column_index,
            row_index,
            up: header_index,
            down: prev_header_down_index,
            left,
            right
        });
    }

    // https://arxiv.org/pdf/cs/0011047.pdf
    pub fn cover_column(&mut self, column_index: usize) {
        let header_index = column_index + 1;

        // Remove this column from the list of headers by making the left and right neighbors point to each other.
        let left_neighbor_index = self.nodes[header_index].left;
        let right_neighbor_index = self.nodes[header_index].right;
        self.nodes[right_neighbor_index].left = left_neighbor_index;
        self.nodes[left_neighbor_index].right = right_neighbor_index;

        // Go down to every node in this column. Stop once we reach the header node again.
        let mut current_down_index = self.nodes[header_index].down;
        while (current_down_index != header_index) {
            // Go right to every node in this row. For each node in the row EXCEPT (!) the one in
            // this covered column itself, unlink it from its respective column by making its up and
            // down nodes point to each other. Also remember to decrement the column size for the columns
            // that have nodes unlinked.
            let mut current_right_index = self.nodes[current_down_index].right;
            while (current_right_index != current_down_index) {
                let current_right_header_index = self.nodes[current_right_index].column_index + 1;
                assert!(current_right_header_index != header_index,
                    "When traversing right in cover_column, tried to unlink a node from the same column that is being covered");

                // Unlink from column.
                let up_neighbor_index = self.nodes[current_right_index].up;
                let down_neighbor_index = self.nodes[current_right_index].down;
                self.nodes[up_neighbor_index].down = down_neighbor_index;
                self.nodes[down_neighbor_index].up = up_neighbor_index;

                // Decrement column size.
                assert!(self.nodes[current_right_header_index].column_size >= 1,
                    "When traversing right in cover_column, tried to unlink a node from a column whose column_size was already 0");
                self.nodes[current_right_header_index].column_size -= 1;

                current_right_index = self.nodes[current_right_index].right;
            }

            // Continue traversal.
            current_down_index = self.nodes[current_down_index].down;
        }
    }

    // https://arxiv.org/pdf/cs/0011047.pdf
    // Note the importance of traversing in the opposite order of the cover_column method, ie
    // up then left instead of down then right
    pub fn uncover_column(&mut self, column_index: usize) {
        let header_index = column_index + 1;

        // Go up to every node in this column. Stop once we reach the header node again.
        let mut current_up_index = self.nodes[header_index].up;
        while (current_up_index != header_index) {
            // Go left to every node in this row. For each node in the row EXCEPT (!) the one in
            // this covered column itself, restore it to its respective column by making its up and
            // down nodes point to the node. Also remember to increment the column size for the columns
            // that have nodes restored.
            let mut current_left_index = self.nodes[current_up_index].left;
            while (current_left_index != current_up_index) {
                let current_left_header_index = self.nodes[current_left_index].column_index + 1;
                assert!(current_left_header_index != header_index,
                    "When traversing left in uncover_column, tried to restore a node from the same column that is being uncovered");

                // Restore to column.
                let up_neighbor_index = self.nodes[current_left_index].up;
                let down_neighbor_index = self.nodes[current_left_index].down;
                self.nodes[up_neighbor_index].down = current_left_index;
                self.nodes[down_neighbor_index].up = current_left_index;

                // Increment column size.
                self.nodes[current_left_header_index].column_size += 1;

                current_left_index = self.nodes[current_left_index].left;
            }

            // Continue traversal.
            current_up_index = self.nodes[current_up_index].up;
        }

        // Restore this column from to the list of headers by making the left and right neighbors point to this header node.
        let left_neighbor_index = self.nodes[header_index].left;
        let right_neighbor_index = self.nodes[header_index].right;
        self.nodes[right_neighbor_index].left = header_index;
        self.nodes[left_neighbor_index].right = header_index;
    }

    // This should probably return a Vec<Vec<usize>> (or better yet an iterator over solutions)
    // because there can be multiple solutions for a given puzzle, however for now we just return
    // the first one found.
    pub fn solve(&mut self) -> Result<Vec<usize>, ()> {
        let mut solution_rows = Vec::with_capacity(self.row_count);
        if self.search_first(&mut solution_rows) {
            Ok(solution_rows)
        } else {
            Err(())
        }
    }

    // https://arxiv.org/pdf/cs/0011047.pdf
    // Returns true if a solution was found, false otherwise. If a solution was found then
    // the solution_rows will contain the row indices of all rows in the solution, otherwise
    // the solution_rows will have the same contents it had when the function was called.
    fn search_first(&mut self, solution_rows: &mut Vec<usize>) -> bool {
        // If all columns are covered, then we've found a solution.
        if self.nodes[Matrix::ROOT_INDEX].right == Matrix::ROOT_INDEX {
            return true;
        }

        // Choose the column with the fewest nodes remaining in it.
        let (min_header_index, min_column_size) = {
            let mut min_column_size = ::std::usize::MAX;
            let mut min_header_index = Matrix::ROOT_INDEX;
            let mut current_index = self.nodes[Matrix::ROOT_INDEX].right;
            while current_index != Matrix::ROOT_INDEX {
                let column_size = self.nodes[current_index].column_size;
                if column_size < min_column_size {
                    min_column_size = column_size;
                    min_header_index = current_index;
                }
                current_index = self.nodes[current_index].right;
            }
            assert!(min_header_index != Matrix::ROOT_INDEX);
            (min_header_index, min_column_size)
        };

        // If we found a column with no nodes in it, then there is no exact cover solution.
        if min_column_size == 0 {
            return false;
        }

        // Cover the current column.
        let min_column_index = self.nodes[min_header_index].column_index;
        self.cover_column(min_column_index);

        // Go through every row in the minimum-sized column and try adding it to the solution.
        let mut current_down_index = self.nodes[min_header_index].down;
        while (current_down_index != min_header_index) {
            // Add the current row to the solution.
            solution_rows.push(self.nodes[current_down_index].row_index);

            // Traverse right across the row, covering all columns with an entry in this row.
            let mut current_right_index = self.nodes[current_down_index].right;
            while (current_right_index != current_down_index) {
                let column_index_to_cover = self.nodes[current_right_index].column_index;
                self.cover_column(column_index_to_cover);
                current_right_index = self.nodes[current_right_index].right;
            }

            // Recursively search the reduced matrix.
            if self.search_first(solution_rows) {
                // For now we're only interested in one solution, so we unwind the call stack
                // (importantly, without popping any solution rows) as soon as the first solution
                // is found. Future improvements would be to implement an efficient iterator over
                // solutions.
                return true;
            }

            solution_rows.pop();

            // Traverse left across the row, restoring all columns with an entry in this row.
            let mut current_left_index = self.nodes[current_down_index].left;
            while (current_left_index != current_down_index) {
                let column_index_to_cover = self.nodes[current_left_index].column_index;
                self.uncover_column(column_index_to_cover);
                current_left_index = self.nodes[current_left_index].left;
            }

            // Continue down the column.
            current_down_index = self.nodes[current_down_index].down;
        }

        // Restore the current column.
        self.uncover_column(min_column_index);

        false
    }
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        Display::fmt(self, f)
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        let mut current_header_index = self.nodes[Matrix::ROOT_INDEX].right;
        while (current_header_index != Matrix::ROOT_INDEX) {
            writeln!(f, "col {:?}", self.nodes[current_header_index]);
            current_header_index = self.nodes[current_header_index].right;
        }
        for row in 0..self.row_count {
            if let Some(row_front_index) = self.row_fronts[row] {
                write!(f, "row {}: [ ", row)?;
                let mut current_index = row_front_index;
                loop {
                    let node = self.nodes[current_index];
                    write!(f, "(i={};c={};l={};r={};u={};d={}) ",
                        current_index,
                        node.column_index,
                        node.left,
                        node.right,
                        node.up,
                        node.down)?;
                    current_index = node.right;
                    if (current_index == row_front_index) {
                        break;
                    }
                }
                writeln!(f, "]")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Matrix, Node, NodeKind};

    #[test]
    fn new_matrix() {
        let mut matrix = Matrix::new(10, 10, 10);

        matrix.set_entry(1, 1);
        matrix.set_entry(0, 0);
        matrix.set_entry(0, 1);
        println!("{}", matrix);
        // panic!("oh no!");
        // let root = matrix.root(); 
        // let root_up = matrix.up(root);
        // let root_down = matrix.down(root);
        // assert_eq!(root, root_up);
        // assert_eq!(root, root_down);
    }
}