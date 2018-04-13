extern crate sudoku;

use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut line = String::with_capacity(9*9*2+1);
    while let Ok(bytes_read) = stdin_lock.read_line(&mut line) {
        if (bytes_read == 0) {
            break;
        }
        line.trim();
        let board = sudoku::Board::from_singleline_str(&line);
        if let Err(err) = board {
            eprintln!("Invalid board: {:?}", err);
            process::exit(1);
        }
        let mut board = board.unwrap();
        let solved = board.solve();
        // writeln!(stdout_lock, "{}", board);
        if let Ok(_) = solved {
            board.to_line(&mut stdout_lock);
            writeln!(stdout_lock);
            stdout_lock.flush();
        } else {
            eprintln!("No solution");
        }
        line.clear();
    }
}
