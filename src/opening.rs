use chess::{self, Board};
use rand::{self, Rng};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Entry{
    pub key: u64,
    pub chess_move: u16,
    pub wieght: u16,
    pub learn: u32 
}

impl Entry{
}


// pub fn load_opening_dataset(file_name: String){
//     let opening_table = chess::CacheTable::new(65536, Entry{key: 0, chess_move:0, wieght: 0, learn: 0});
//     let file = std::fs::read(file_name).ok().expect("error_loading data");
    
    
//     for byte_group in file{



//     }


// }



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