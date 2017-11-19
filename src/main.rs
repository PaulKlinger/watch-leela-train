extern crate regex;

use std::process::{Command,Stdio};
use std::io::{BufRead, BufReader};
use std::str;

use regex::Regex;

const SIZE: usize = 19;
const ROW_INDICES: &'static str = "ABCDEFGHJKLMNOPQRST";

fn get_index((row, col): (usize, usize)) -> usize {
    return (row - 1) * SIZE + (col - 1)
}

fn update_board(board: &mut Vec<char>, row_str: &str, col_str: &str, current_player: char) {
    let row_index = ROW_INDICES.find(row_str).unwrap() + 1;
    let col_index: usize = col_str.parse().unwrap();
    board[get_index((row_index, col_index))] = current_player;

    resolve_capture(board);
}

fn resolve_capture(board: &mut Vec<char>) {
    let mut processed_coords: Vec<(usize, usize)> = Vec::new();
    let mut chain_coords: Vec<(usize, usize)> = Vec::new();
    let mut liberties: Vec<(usize, usize)> = Vec::new();
    for row in 1..SIZE+1 {
        for col in 1..SIZE+1 {
            if board[get_index((row, col))] != '.' && ! processed_coords.contains(&(row,col)) {
                process_chain(board, row, col, &mut chain_coords, &mut liberties);
                if liberties.len() == 0 {
                    println!("{} stones captured!", chain_coords.len());
                    for &(c_row, c_col) in &chain_coords {
                        board[get_index((c_row, c_col))] = '.';
                    }
                }
                processed_coords.append(&mut chain_coords); // this clears chain_coords
                liberties.clear()
            }
        }
    }
}

fn process_chain(board: &Vec<char>, row: usize, col: usize, chain_coords: &mut Vec<(usize, usize)>, liberties: &mut Vec<(usize, usize)>) {
    if row < 1 || col < 1 || row > SIZE || col > SIZE {
        return;
    }
    let val = board[get_index((row,col))];
    let coord = (row, col);
    if val == '.' {
        liberties.push(coord);
        return;
    }
    else if ! chain_coords.contains(&coord) 
            && (chain_coords.len() == 0
                || val == board[get_index(*chain_coords.get(0).unwrap())]
                )
    {
        chain_coords.push(coord);
        process_chain(board, row - 1, col, chain_coords, liberties);
        process_chain(board, row + 1, col, chain_coords, liberties);
        process_chain(board, row, col - 1, chain_coords, liberties);
        process_chain(board, row, col + 1, chain_coords, liberties);
    }
    
}

fn print_board(board: &Vec<char>) {
    for row in 1..SIZE+1 {
        let mut out_line = String::new();
        for col in 1..SIZE+1 {
            out_line.push(board[get_index((row, col))]);
        }
        println!("{}",out_line);
    }
}

fn main() {
    //println!("Hello, world!");
    let mut child = Command::new("./autogtp.exe")
        .arg("-k")
        .arg("sgfs")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start autogtp.exe");
    let mut child_out = BufReader::new(child.stderr.as_mut().unwrap());
    let mut buffer: Vec<u8> = Vec::new();
    let mut line: String;

    let mut board: Vec<char> = vec!['.'; SIZE * SIZE];
    let mut current_player = 'o';
    let move_regex = Regex::new(r"[ \n]\d+ \(([A-Z])(\d+)\)").unwrap();
    let end_regex = Regex::new(r"Game has ended").unwrap();
    assert!(move_regex.is_match(" 245 (F18)"));

    loop {
        child_out.read_until(')' as u8, &mut buffer).unwrap();
        line = String::from_utf8_lossy(&buffer).into();
        buffer.clear();

        if end_regex.is_match(&line) {
            board = vec!['.'; SIZE * SIZE];
            current_player = 'o';
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

        if current_player == 'o' {current_player = 'x'}
        else {current_player = 'o'}
    }
}
