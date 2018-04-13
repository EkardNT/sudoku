extern crate sudoku;

fn main() {
    // let mut puzzle = sudoku::Puzzle::from_string(
        // "5 3 _ _ 7 _ _ _ _\
        //  6 _ _ 1 9 5 _ _ _\
        //  _ 9 8 _ _ _ _ 6 _\
        //  8 _ _ _ 6 _ _ _ 3\
        //  4 _ _ 8 _ 3 _ _ 1\
        //  7 _ _ _ 2 _ _ _ 6\
        //  _ 6 _ _ _ _ 2 8 _\
        //  _ _ _ 4 1 9 _ _ 5\
        //  _ _ _ _ 8 _ _ 7 9"
    // );
    let mut board = sudoku::Board::from_multiline_str(
        "5 3 _ _ 7 _ _ _ _\
         6 _ _ 1 9 5 _ _ _\
         _ 9 8 _ _ _ _ 6 _\
         8 _ _ _ 6 _ _ _ 3\
         4 _ _ 8 _ 3 _ _ 1\
         7 _ _ _ 2 _ _ _ 6\
         _ 6 _ _ _ _ 2 8 _\
         _ _ _ 4 1 9 _ _ 5\
         _ _ _ _ 8 _ _ 7 9").unwrap();
    println!("{:?}", board);
    board.solve();
    println!("-----------------");
    // let solution_rows = board.init_solution_rows();
    // println!("{:?}", solution_rows);
    println!("{:?}", board);
    // let mut string = format!("{}", board1);
    // let mut board2 = sudoku::Board::from_str(&string).unwrap();
    // assert_eq!(board1, board2);
    // println!("{}", board1);
    // println!("{}", board2);
    // let mut line = String::new();
    // board1.to_line(&mut line);
    // println!("{}", line);
}
