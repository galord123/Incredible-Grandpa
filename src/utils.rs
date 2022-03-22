use chess::{Square, BitBoard, Board};
use std::ops::BitAnd;


pub fn print_board(fen:String){
    let split = fen.split("/");
    
    for s in split {
        let mut count = 0;
        for c in s.chars() {
            if count < 8{
                if c.is_digit(10){
                    let times = c. to_digit(10);
                    match times{
                        None=>{},
                        Some(a)=>{
                            for _ in 0..a{
                                print!(" ");
                                count += 1;
                            }
                        }
                    }
                    
                }
                else
                {
                    print!("{}", c);
                    count += 1;
                }
            }
            

        }
        println!("");
    }

}


pub fn get_piece_type(board: &chess::Board, piece: chess::Piece, color: chess::Color) -> chess::BitBoard{
    board.color_combined(color).bitand(board.pieces(piece))
}


pub fn mirror_square(square: &Square) -> Square{
    let f = square.get_file();
    let r = square.get_rank();
    let new_rank = 7 - r.to_index();
    Square::make_square(chess::Rank::from_index(new_rank), f)
}


pub fn distance(a: Square, b:Square) -> i32{
    std::cmp::max(((b.get_rank().to_index() as i32- a.get_rank().to_index() as i32) as i32 ).abs(), ((b.get_file().to_index() as i32- a.get_file().to_index() as i32)as i32).abs())
} 


pub fn blocked_bishop(board: &Board, bishop_square: Square, square: Square, white: bool) -> i32{
    if let Some(piece) = board.piece_on(bishop_square){
        if piece == chess::Piece::Bishop{
            if let Some(piece) = board.piece_on(square){
                if piece == chess::Piece::Pawn{
                    if white{
                        if board.piece_on(square.uup()).is_some(){
                            return -50
                        }
                    }else{
                        if board.piece_on(square.udown()).is_some(){
                            return -50
                        }
                    }
                }
            }
        }
    }
    return 0;
}


pub fn sum_by_table(pieces: &BitBoard, table: &[i32], mirror: bool) -> i32{
    let mut sum = 0;
    if mirror {
        for i in *pieces{
            sum += table[mirror_square(&i).to_int() as usize];
        }
    }else{
        for i in *pieces{
            sum += table[i.to_int() as usize];
        }
    }
    sum
}