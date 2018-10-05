/*

A Rust solver for a simple numerical game.

See the inital article at [simple-number].

# Rules

The rules are simple: on an empty 10x10 grid (100 squares in total) you put a
number 1 on an arbitrary square. Starting from that square you can move
horizontally or vertically jumping over two squares or diagonally jumping over
one square. There you can place number 2. Your task is to reach number 100,
filling all squares. You can not visit already visited squares.

Here is an example of a solved game with a reduced 5x5 grid, starting at
top-left corner

     1 24 14  2 25
    16 21  5  8 20
    13 10 18 23 11
     4  7 15  3  6
    17 22 12  9 19

[simple-number]: https://www.nurkiewicz.com/2018/09/brute-forcing-seemingly-simple-number.html
 */

#![allow(dead_code)]

use std::slice::Iter;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MyError {
    details: String
}

impl MyError{
    fn new(msg: &str) -> MyError {
        MyError{
            details: msg.to_string()
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        &self.details
    }
}

const HV_OFFSET: i32 = 3;
const DIAG_OFFSET: i32 = 2;

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Down,
    DownRight,
    Right,
    UpRight,
    Up,
    UpLeft,
    Left,
    DownLeft,
}

impl Direction {
    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 8] = [
            Direction::Down,
            Direction::DownRight,
            Direction::Right,
            Direction::UpRight,
            Direction::Up,
            Direction::UpLeft,
            Direction::Left,
            Direction::DownLeft,
        ];
        DIRECTIONS.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    size: usize,
    cells: usize,
    values: Vec<u8>,
    x: usize, // last move location
    y: usize,
}

impl Board {
    pub fn new(size: usize) -> Self {
        let mut size = size;
        if size < 5 {
            size = 5;
        }
        if size > 16 {
            size = 16;
        }

        Board {
            size,
            cells: size * size,
            values: vec![0; size * size],
            x: 0,
            y: 0,
        }
    }

    /// Return true if the board has been started.
    pub fn is_started(&self) -> bool {
        self.values[self.y * self.size + self.x] > 0
    }

    /// Return a list of all possible moves from the current location.
    /// Returns an empty list if there are no moves, or the board is empty.
    pub fn possible_moves(&self) -> Vec<&'static Direction> {
        Direction::iterator().filter(|&x| self.valid_move(*x).is_some()).collect()
    }

    /// Determines if a move in the given direction is valid. A move is valid
    /// if the resulting position is valid, and if the the resulting position
    /// is an empty cell. If the move is valid, it returns `Some((x, y))` 
    /// where (x, y) is the cell location resulting from the move. Otherwise,
    /// it returns `None`.
    fn valid_move(&self, dir: Direction) -> Option<(usize, usize)> {
        let x: i32 = self.x as i32;
        let y: i32 = self.y as i32;
        let size: i32 = self.size as i32;
        if self.is_started() {
            let (x, y) = match dir {
                Direction::Down => (x, y + HV_OFFSET),
                Direction::DownRight => (x + DIAG_OFFSET, y + DIAG_OFFSET),
                Direction::Right => (x + HV_OFFSET, y),
                Direction::UpRight => (x + DIAG_OFFSET, y - DIAG_OFFSET),
                Direction::Up => (x, y - HV_OFFSET),
                Direction::UpLeft => (x - DIAG_OFFSET, y - DIAG_OFFSET),
                Direction::Left => (x - HV_OFFSET, y),
                Direction::DownLeft => (x - DIAG_OFFSET, y + DIAG_OFFSET),
            };
            if x>= 0 && y >= 0 && x < size && y < size && self.values[(y * size + x) as usize] == 0 {
                return Some((x as usize, y as usize));
            }
        }
        None
    }

    /// Make the next move on the board using a given direction.
    pub fn next_move(&mut self, dir: Direction) -> Result<(), MyError> {
        if !self.is_started() {
            return Err(MyError::new("Attempt to move with an empty board"));
        }
        let val = self.values[self.y * self.size + self.x];
        match self.valid_move(dir) {
            Some((x, y)) => self.set_cell(x, y, val + 1),
            None => Err(MyError::new(&format!("Moving in direction: {:?} is invalid", dir))),
        }
    }

    /// Return true if the board is complete. A board is complete if the value
    /// of the last move equals the maximum number of cells, and there are no
    /// empty cells in the board.
    pub fn is_won(&self) -> bool {
        static ZERO: u8 = 0 as u8;
        self.values[self.y * self.size + self.x] == self.cells as u8 && !self.values.contains(&ZERO)
    }

    /// Return `true` if there are no possible moves for the current board.
    pub fn is_blocked(self) -> bool {
        self.is_started() && self.possible_moves().len() == 0 
    }

    /// Start the puzzle by placing a 1 in the given location.
    pub fn start_at(&mut self, x: usize, y: usize) -> Result<(), MyError> {
        self.set_value(x, y, 1)
    }

    /// The score is simply the highest value on the board.
    pub fn score(&self) -> usize {
        self.values.iter().cloned().fold(0, u8::max) as usize
    }

    /// Set the value of location on the board to `value`.
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), MyError> {
        if value < 1 {
            return Err(MyError::new(&format!("cannot clear cell [{}, {}]", x, y)));
        }
        if value as usize > self.cells {
            return Err(MyError::new(&format!(
                "cannot set cell [{}, {}] to {} (max: {})",
                x, y, value, self.cells
            )));
        }
        self.set_cell(x, y, value)
    }

    // TODO(markcol): use Index, IndexRef traits instead?
    fn set_cell(&mut self, x: usize, y: usize, value: u8) -> Result<(), MyError> {
        if x >= self.size || y >= self.size {
            return Err(MyError::new(&format!(
                "index [{}, {}] out of range (max: {})",
                x, y, self.size
            )));
        }
        if self.values[y * self.size + x] != 0 {
            return Err(MyError::new(&format!("cannot change value of cell [{}, {}]", x, y)));
        }
        self.x = x;
        self.y = y;
        self.values[y * self.size + x] = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_board() {
        let board = Board::new(10);
        assert_eq!(board.size, 10);
        // newly created board has a score of 0
        assert_eq!(board.score(), 0);
        // newly created board is not started
        assert_eq!(board.is_started(), false);
        // unstarted board cannot be won
        assert_eq!(board.is_won(), false);
        // no possible moves because board isn't started
        assert_eq!(board.possible_moves().len(), 0);
    }

    #[test]
    fn start_board() {
        let mut board = Board::new(10);
        board.start_at(5, 5).unwrap();
        assert_eq!(board.values[55], 1);
        assert_eq!(board.score(), 1);
        // board is started
        assert_eq!(board.is_started(), true);
        // board isn't won
        assert_eq!(board.is_won(), false);
        // all moves should be possible
        assert_eq!(board.possible_moves().len(), 8);
    }

    #[test]
    fn win_5() {
            
        //  1 24 14  2 25
        // 16 21  5  8 20
        // 13 10 18 23 11
        //  4  7 15  3  6
        // 17 22 12  9 19

        let moves = [
            Direction::Right, Direction::Down, Direction::Left, 
            Direction::UpRight, Direction::DownRight, Direction::Left,
            Direction::UpRight, Direction::Down, Direction::UpLeft,
            Direction::Right, Direction::DownLeft, Direction::UpLeft,
            Direction::UpRight, Direction::Down, Direction::UpLeft,
            Direction::Down, Direction::UpRight, Direction::DownRight,
            Direction::Up, Direction::Left, Direction::Down,
            Direction::UpRight, Direction::UpLeft, Direction::Right
        ];

        let mut board = Board::new(5);
        board.start_at(0, 0).unwrap();
        for m in moves.iter() {
            assert_eq!(board.next_move(*m).is_ok(), true);
        }
        assert_eq!(board.is_won(), true);
        assert_eq!(board.score(), 25);
        assert_eq!(board.possible_moves().len(), 0);
        assert_eq!(board.is_blocked(), true);
    }
}