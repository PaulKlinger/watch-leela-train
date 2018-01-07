#![feature(inclusive_range_syntax)]
extern crate regex;

use std::process::{Command, Stdio};
use std::io::BufReader;
use std::str;
use std::ops::{Index, IndexMut};
use std::env;
use std::collections::HashSet;

use regex::Regex;

mod read_until_multiple;
use read_until_multiple::read_until_multiple;

const SIZE: usize = 19;
const ROW_INDICES: &'static str = "ABCDEFGHJKLMNOPQRST";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Coord(usize, usize);

#[derive(Debug, Clone, Copy)]
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
    size: usize,
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

    fn to_string(&self) -> String {
        let mut out = String::new();
        for row in 1..=self.size {
            out.push('\n');
            for col in 1..=self.size {
                out.push(self[Coord(row, col)]);
                out.push(' ');
            }
        }
        return out;
    }
}

fn get_index(size: usize, Coord(row, col): Coord) -> usize {
    return (row - 1) * size + (col - 1);
}

fn update_board(
    board: &mut Board,
    row_str: &str,
    col_str: &str,
    current_player: Player,
) -> Vec<u16> {
    let row_index = ROW_INDICES.find(row_str).unwrap() + 1;
    let col_index: usize = col_str.parse().unwrap();
    board[Coord(row_index, col_index)] = current_player.symbol();

    resolve_capture(board)
}

fn resolve_capture(board: &mut Board) -> Vec<u16> {
    let mut out = Vec::new();
    let mut processed_coords: HashSet<Coord> = HashSet::new();
    let mut chain_coords: HashSet<Coord> = HashSet::new();
    let mut liberties: HashSet<Coord> = HashSet::new();
    for row in 1..=board.size {
        for col in 1..=board.size {
            let coord = Coord(row, col);
            if board[coord] != '.' && !processed_coords.contains(&coord) {
                process_chain(board, coord, &mut chain_coords, &mut liberties);
                if liberties.len() == 0 {
                    out.push(chain_coords.len() as u16);
                    for &chain_coord in &chain_coords {
                        board[chain_coord] = '.';
                    }
                }
                processed_coords.extend(chain_coords.drain()); // this clears chain_coords
                liberties.clear()
            }
        }
    }
    out
}

fn process_chain(
    board: &Board,
    coord: Coord,
    chain_coords: &mut HashSet<Coord>,
    liberties: &mut HashSet<Coord>,
) {
    let Coord(row, col) = coord;
    if row < 1 || col < 1 || row > board.size || col > board.size {
        return;
    }

    let val = board[coord];
    if val == '.' {
        liberties.insert(coord);
        return;
    } else if !chain_coords.contains(&coord)
        && (chain_coords.is_empty() || val == board[*chain_coords.iter().next().unwrap()])
    {
        chain_coords.insert(coord);
        process_chain(board, Coord(row - 1, col), chain_coords, liberties);
        process_chain(board, Coord(row + 1, col), chain_coords, liberties);
        process_chain(board, Coord(row, col - 1), chain_coords, liberties);
        process_chain(board, Coord(row, col + 1), chain_coords, liberties);
    }
}

fn get_autogtp_version() -> String {
    let output = match Command::new("./autogtp").arg("--version").output() {
        Ok(o) => o,
        Err(e) => {
            println!("Error executing \"autogtp --version\": {:?}", e);
            std::process::exit(1);
        }
    };
    let version_string = String::from_utf8_lossy(&output.stdout);
    match version_string.trim_right().split(' ').last() {
        Some(s) => s.into(),
        None => {
            println!(
                "Failed to determine autogtp version! Got version string \"{}\"",
                version_string
            );
            std::process::exit(1);
        }
    }
}

fn main() {
    let autogtp_version = get_autogtp_version();

    let mut arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.len() < 2 {
        arguments = vec!["-k".to_string(), "sgfs".to_string()];
        if !vec!["v1", "v2", "v3", "v4", "v5"].contains(&autogtp_version.as_str()) {
            arguments.push("-g".to_string());
            arguments.push("1".to_string());
        }
    }
    let mut child = match Command::new("./autogtp")
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            println!("Error starting autogtp: {:?}", e);
            std::process::exit(1);
        }
    };


    let mut child_out = BufReader::new(child.stdout.as_mut().unwrap());
    let mut child_err = BufReader::new(child.stderr.as_mut().unwrap());

    let mut buffer: Vec<u8> = Vec::new();
    let mut line: String;

    let mut board: Board = Board::new(SIZE);
    let mut current_player = Player::Black;
    let move_regex = Regex::new(r"^ ?\d+ \((?:[BW] )?([A-Z])(\d+)\)").unwrap();
    let end_regex = Regex::new(r"Game has ended").unwrap();
    let move_or_pass_regex = Regex::new(r"^ ?\d+ \((?:[BW] )?(?:[A-Z]\d+)|(?:pass)\)").unwrap();
    assert!(move_regex.is_match(" 245 (F18)"));

    enum Stream {
        Stdout,
        Stderr,
    }
    // Early autogtp versions printed to stderr instead of stdout
    let move_stream = match vec!["v1", "v2", "v3", "v4"].contains(&autogtp_version.as_str()) {
        true => Stream::Stderr,
        false => Stream::Stdout,
    };

    let delims = [')' as u8, '\n' as u8];

    loop {
        match match move_stream {
            Stream::Stderr => read_until_multiple(&mut child_err, &delims, &mut buffer),
            Stream::Stdout => read_until_multiple(&mut child_out, &delims, &mut buffer),
        } {
            Ok(_) => {}
            Err(e) => {
                println!("Error reading from autogtp: {:?}", e);
                std::process::exit(1);
            }
        };
        if buffer.len() == 0 {
            // autogtp has exited
            break;
        }

        line = String::from_utf8_lossy(&buffer).into();
        let mut out = line.clone();
        buffer.clear();

        if end_regex.is_match(&line) {
            board = Board::new(SIZE);
            current_player = Player::Black;
        }

        match move_regex.captures(&line) {
            Some(caps) => {
                let captures = update_board(
                    &mut board,
                    caps.get(1).unwrap().as_str(),
                    caps.get(2).unwrap().as_str(),
                    current_player,
                );
                if !captures.is_empty() {
                    out.push_str(&format!(
                        " Captured {} stone{}.",
                        captures
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                        if captures.iter().sum::<u16>() > 1 {
                            "s"
                        } else {
                            ""
                        }
                    ))
                }
            }
            _ => {}
        }

        if move_or_pass_regex.is_match(&line) {
            out.insert_str(0, &format!("{}: ", current_player.symbol()));
            out.push_str(&board.to_string());

            current_player = match current_player {
                Player::White => Player::Black,
                Player::Black => Player::White,
            };
            out.push('\n');
        }
        print!("{}", out);
    }
}
