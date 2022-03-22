use chess::{self, Board};
use rand::{self, Rng};
use std::fs;

pub fn get_opening_move(path: String, board: &Board) -> Option<chess::ChessMove>{

    let contents = fs::read_to_string(path)
    .expect("Something went wrong reading the file");

    let splited_lines = contents.split("\n");
    for line in splited_lines{
        let parts = line.split(";");
        let converted = parts.collect::<Vec<&str>>();
        if converted[0] == board.to_string(){
            let moves: Vec<&str> = converted[1].split(" ").collect();
            return chess::ChessMove::from_san(board, moves[rand::thread_rng().gen_range(0..moves.len())]).ok();
        }
    }

    return None;

    
}