#![feature(inclusive_range_syntax)]
extern crate regex;

use std::process::{Command,Stdio};
use std::io::{BufRead, BufReader};
use std::str;
use std::ops::{Index, IndexMut};
use std::env;

use regex::Regex;

const SIZE: usize = 19;
const ROW_INDICES: &'static str = "ABCDEFGHJKLMNOPQRST";

#[derive(PartialEq, Clone, Copy)]
struct Coord(usize, usize);

#[derive(Clone, Copy)]
enum Player {
    White,
    Black,
}

impl Player {
    fn symbol(&self) -> char {
        match *self {
            Player::White => 'o',
            Player::Black => 'x',
        }
    }
}

struct Board {
    board: Vec<char>,
    size: usize
}

impl Index<Coord> for Board {
    type Output = char;

    fn index(&self, coord: Coord) -> &char {
        &self.board[get_index(self.size, coord)]
    }
}

impl IndexMut<Coord> for Board {
    fn index_mut(&mut self, coord: Coord) -> &mut char {
        &mut self.board[get_index(self.size, coord)]
    }
}

impl Board {
    fn new(size: usize) -> Board {
        Board {
            board: vec!['.'; size * size],
            size: size,
        }
    }
}

fn get_index(size: usize, Coord(row, col): Coord) -> usize {
    return (row - 1) * size + (col - 1)
}

fn update_board(board: &mut Board, row_str: &str, col_str: &str, current_player: Player) {
    let row_index = ROW_INDICES.find(row_str).unwrap() + 1;
    let col_index: usize = col_str.parse().unwrap();
    board[Coord(row_index, col_index)] = current_player.symbol();

    resolve_capture(board);
}

fn resolve_capture(board: &mut Board) {
    let mut processed_coords: Vec<Coord> = Vec::new();
    let mut chain_coords: Vec<Coord> = Vec::new();
    let mut liberties: Vec<Coord> = Vec::new();
    for row in 1..=board.size {
        for col in 1..=board.size {
            if board[Coord(row, col)] != '.' && ! processed_coords.contains(&Coord(row,col)) {
                process_chain(board, row, col, &mut chain_coords, &mut liberties);
                if liberties.len() == 0 {
                    println!("{} stones captured!", chain_coords.len());
                    for &Coord(c_row, c_col) in &chain_coords {
                        board[Coord(c_row, c_col)] = '.';
                    }
                }
                processed_coords.append(&mut chain_coords); // this clears chain_coords
                liberties.clear()
            }
        }
    }
}

fn process_chain(board: &Board, row: usize, col: usize, chain_coords: &mut Vec<Coord>, liberties: &mut Vec<Coord>) {
    if row < 1 || col < 1 || row > board.size || col > board.size {
        return;
    }
    let val = board[Coord(row,col)];
    let coord = Coord(row, col);
    if val == '.' {
        liberties.push(coord);
        return;
    }
    else if ! chain_coords.contains(&coord) 
            && (chain_coords.len() == 0
                || val == board[*chain_coords.get(0).unwrap()]
                )
    {
        chain_coords.push(coord);
        process_chain(board, row - 1, col, chain_coords, liberties);
        process_chain(board, row + 1, col, chain_coords, liberties);
        process_chain(board, row, col - 1, chain_coords, liberties);
        process_chain(board, row, col + 1, chain_coords, liberties);
    }
    
}

fn print_board(board: &Board) {
    for row in 1..=board.size {
        let mut out_line = String::new();
        for col in 1..=board.size {
            out_line.push(board[Coord(row, col)]);
        }
        println!("{}",out_line);
    }
}

fn main() {
    let mut arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.len() < 2 {
        arguments = vec!["-k".to_string(), "sgfs".to_string()];
    }
    let mut child = Command::new("./autogtp")
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start autogtp");
    let mut child_out = BufReader::new(child.stderr.as_mut().unwrap());
    let mut buffer: Vec<u8> = Vec::new();
    let mut line: String;

    let mut board: Board = Board::new(SIZE);
    let mut current_player = Player::White;
    let move_regex = Regex::new(r"[ \n]\d+ \(([A-Z])(\d+)\)").unwrap();
    let end_regex = Regex::new(r"Game has ended").unwrap();
    assert!(move_regex.is_match(" 245 (F18)"));

    loop {
        child_out.read_until(')' as u8, &mut buffer).unwrap();
        line = String::from_utf8_lossy(&buffer).into();
        buffer.clear();

        if end_regex.is_match(&line) {
            board = Board::new(SIZE);
            current_player = Player::White;
        }

        println!("{}",line);
        match move_regex.captures(&line) {
            Some(caps) => {
                update_board(
                &mut board,
                caps.get(1).unwrap().as_str(),
                caps.get(2).unwrap().as_str(),
                current_player);
                print_board(&board);},
            _ => {},
        }

        current_player = match current_player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}
