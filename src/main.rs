#[macro_use]
extern crate lazy_static;
extern crate csv;

use std::fmt;
use std::fs::File;
use std::time::{Duration, Instant};

lazy_static! {
    // A cell's peers are all of the cells that are in the same row, column, or block
    // There are 81 cells, each of which has 20 peers, so this is a size 81*20 array
    static ref PEERS: [[usize; 20]; 81] = {
        let mut peers = [[0; 20]; 81];

        for row in 0..9 {
            for col in 0..9 {
                let mut p = &mut peers[9*row + col];
                let mut peer_i = 0;
                
                // Row peers
                for i in 0..9 {
                    if i == col { continue };

                    p[peer_i] = 9*row + i;
                    peer_i += 1;
                }

                // Column peers
                for i in 0..9 {
                    if i == row { continue };
                    
                    p[peer_i] = 9*i + col;
                    peer_i += 1;
                }

                // Block peers
                for block_row in (row/3*3)..(row/3*3+3) {
                    if block_row == row { continue };

                    for block_col in (col/3*3)..(col/3*3+3) {
                        if block_col == col { continue };

                        p[peer_i] = 9*block_row + block_col;
                        peer_i += 1;
                    }
                }
            }
        }

        peers
    };
}

const ALL_NUMS: u16 = (1 << 9) - 1;

// An iterator over the set bits in a u16, one-indexed
// Example: a BitIterator over the number 0b101101 will yield 1, 3, 4, 6
struct BitIterator(u16, u8);
impl Iterator for BitIterator {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        while self.0 != 0 {
            let bit = self.0 & 1;
            self.0 >>= 1;
            self.1 += 1;

            if bit > 0 {
                return Some(self.1)
            }
        }
        return None;
    }
}

#[derive(Clone, Copy)]
struct SudokuCell(u16);

impl SudokuCell {
    fn new() -> Self {
        SudokuCell(0)
    }
    fn new_all_set() -> Self {
        SudokuCell(ALL_NUMS)
    }

    fn set(&mut self, num: u8) {
        self.0 |= 1 << (num - 1);
    }

    fn unset(&mut self, num: u8) {
        self.0 &= !(1 << (num - 1));
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn get(&self, num: u8) -> bool {
        (self.0 & (1 << (num - 1))) > 0
    }

    fn size(&self) -> u32 {
        self.0.count_ones()
    }

    fn get_first(&self) -> u8 {
        self.0.trailing_zeros() as u8 + 1
    }

    fn remove_all(&mut self, other: &SudokuCell) {
        self.0 &= !other.0
    }

    fn iter(&self) -> BitIterator {
        BitIterator(self.0, 0)
    }
}
impl fmt::Display for SudokuCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for n in 1..=9 {
            if self.get(n) {
                write!(f, "{}", n)?;
            }
        }

        Ok(())
    }
}

type SudokuBoard = Vec<SudokuCell>;

fn try_solve(mut board: SudokuBoard) -> Option<SudokuBoard> {
    // Keeps track of whether any cell changed from uncertain to certain this iteration
    // If no cell has become certain, we won't be able to make any more progress using contraint propagation
    let mut made_progress = true;
    // Keeps track of whether the board has been solved or not
    // This is used to determine whether to continue with a search afterwards or not
    let mut solved = false;
    while made_progress {
        made_progress = false;
        solved = true;
        for i in 0..81 {
            // If the cell already has a known value, there isn't anything to do
            if board[i].size() == 1 { continue; }

            // This set will contain all of the values that this cell cannot be
            // (because it has a peer that is already that value)
            let mut peer_values = SudokuCell::new();

            for peer_i in PEERS[i].iter() {
                let peer = board[*peer_i];
                // Only if we are certain about the value of this peer:
                if peer.size() == 1 {
                    // Since the peer only has one possible value, get_first returns the value of the cell.
                    peer_values.set(peer.get_first());
                }
            }

            let mut cell = board.get_mut(i).unwrap();
            cell.remove_all(&peer_values);

            match cell.size() {
                0 => return None, // conflict found, board can't be solved.
                1 => made_progress = true, // cell wasn't certain before, now it is.
                _ => solved = false, // there are still multiple possibilites, so the board won't be solved this iteration.
            }
        }
    }

    if solved {
        Some(board)
    } else {
        // We can't make any more progress with contstring propagation, so it is time to start a search algorithm

        // Find the cell with the least number of possibilities
        let mut smallest_cell = 0;
        let mut smallest_count = 9;
        for i in 0..81 {
            if board[i].size() > 1 && board[i].size() < smallest_count {
                smallest_cell = i;
                smallest_count = board[i].size();
            }
        }

        // Try out every possible value of that cell
        for val in board[smallest_cell].iter() {
            let mut board2 = board.clone();
            // Set the cell to the value we're trying
            board2[smallest_cell].clear();
            board2[smallest_cell].set(val);

            // Try to solve the new board with the test value
            if let Some(solved) = try_solve(board2) {
                // If that value works, we're done. Otherwise we'll continue with the next value
                return Some(solved)
            }
        }

        None
    }
}

fn load_board(s: &str) -> SudokuBoard {
    let mut board = Vec::with_capacity(81);

    let mut i = 0;

    for c in s.chars() {
        if "0.".contains(c) {
            // Empty cell
            board.push(SudokuCell::new_all_set());
            i += 1;
        } else if "123456789".contains(c) {
            // Given cell
            let mut cell = SudokuCell::new();
            cell.set(c.to_digit(10).unwrap() as u8);
            board.push(cell);
            i += 1;
        }
        // Characters that aren't digits or '.' are ignored
    }

    if i != 81 {
        panic!("Too few cells in board {}", s)
    }


    board
}

fn serialize_board(board: &SudokuBoard) -> String {
    let mut out = String::new();
    for i in 0..81 {
        let val = board[i].get_first();
        out.push(::std::char::from_digit(val.into(), 10).unwrap());
    }
    out
}

fn print_board(board: &SudokuBoard) {
    let width = board.iter().map(|s| s.size()).max().unwrap() as usize;

    for row in 0..9 {
        for col in 0..9 {
            print!(" {:^width$}", format!("{}", board[9*row + col]), width = width);
            if col == 2 || col == 5 {
                print!(" |");
            }
        }
        println!("");
        if row == 2 || row == 5 {
            let line: String = (0..3*width + 4).map(|_| '-').collect();
            println!("{line}+{line}+{line}", line=line);
        }
    }
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();

    let mut total_time = Duration::from_secs(0);
    let mut num_boards = 0;
    let mut num_failed = 0;

    let mut rdr = csv::Reader::from_reader(File::open(&args[1]).unwrap());
    for result in rdr.records() {
        let record = result.unwrap();
        let board = load_board(record.get(0).unwrap());

        // Solve the board (while recording the time taken)
        let start = Instant::now();
        let maybe_solved = try_solve(board);
        let end = Instant::now();

        // Make sure the board was actually solved
        match maybe_solved {
            None => num_failed += 1,
            Some(solved) => {
                if serialize_board(&solved) != record.get(1).unwrap() {
                    num_failed += 1;
                }
            }
        }

        total_time += end - start;
        num_boards += 1;

        // Print timing info
        let time_as_secs = (total_time.as_secs() as f64) + (total_time.subsec_millis() as f64) / 1000.0;
        if time_as_secs > 0.0 {
            let rate = num_boards as f64 / time_as_secs;
            println!("{:.0} boards/s ({} boards, {} failed)", rate, num_boards, num_failed);
        }
    }
}
