use std::ops::BitAnd;
use crate::bitboard_operators::black_passed_pawns;
use crate::bitboard_operators::white_passed_pawns;
use crate::constants;
use crate::utils;
use crate::bitboard_operators::{open_files, black_pawns_behind_own, white_pawns_behind_own, king_attacks, black_front_spans, file_fill};
use crate::utils::{get_piece_type, sum_by_table};
use chess::BitBoard;


pub fn evaluate(board: &chess::Board) -> i32{
    
    let status = board.status();
    if status == chess::BoardStatus::Checkmate{
        match board.side_to_move(){
            chess::Color::Black=>{
                return 9999;
            }
            chess::Color::White=>{
                return -9999;
            }
        }
    }else if status == chess::BoardStatus::Stalemate{
        return 0;
    }

    let white_king = board.king_square(chess::Color::White);
    let black_king = board.king_square(chess::Color::Black);

    
    // evaluate white material advantage
    let white_queens = get_piece_type(&board, chess::Piece::Queen, chess::Color::White);
    let white_bishops = get_piece_type(&board, chess::Piece::Bishop, chess::Color::White);
    let white_knights = get_piece_type(&board, chess::Piece::Knight, chess::Color::White);
    let white_rooks = get_piece_type(&board, chess::Piece::Rook, chess::Color::White);
    let white_pawns = get_piece_type(&board, chess::Piece::Pawn, chess::Color::White);
    
    
    // evaluate black material advantage
    let black_queens = get_piece_type(&board, chess::Piece::Queen, chess::Color::Black);
    let black_bishops = get_piece_type(&board, chess::Piece::Bishop, chess::Color::Black);
    let black_knights = get_piece_type(&board, chess::Piece::Knight, chess::Color::Black);
    let black_rooks = get_piece_type(&board, chess::Piece::Rook, chess::Color::Black);
    let black_pawns = get_piece_type(&board, chess::Piece::Pawn, chess::Color::Black);
    
    let pieces_total_count = board.pieces(chess::Piece::Bishop).count() + board.pieces(chess::Piece::Knight).count() + board.pieces(chess::Piece::Rook).count();
    let end_game = board.pieces(chess::Piece::Queen).count() == 0 && pieces_total_count <= 2;

    //evaluate white positional advantage
    let knightsq_w = sum_by_table(&white_knights, constants::KNIGHT_SQUARES_TABLE, false);
    let pawnsq_w = sum_by_table(&white_pawns, constants::PAWN_SQUARES_TABLE, false);
    let queensq_w = sum_by_table(&white_queens, constants::QUEEN_SQUARES_TABLE, false);
    let bishopsq_w = sum_by_table(&white_bishops, constants::BISHOP_SQUARES_TABLE, false);
    let rooksq_w = sum_by_table(&white_rooks, constants::ROOK_SQUARES_TABLE, false);

    // give bonus to rook on open file
    let white_rooks_on_open_file = 10 * white_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();
    let black_rooks_on_open_file = 10 * black_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();
    
    // evaluate pawns
    // evaluate double pawns
    let black_doubled_pawns = black_pawns_behind_own(black_pawns).popcnt() as i32* constants::DOUBLED_PAWNS_DEBUFF;
    let white_doubled_pawns = white_pawns_behind_own(white_pawns).popcnt() as i32* constants::DOUBLED_PAWNS_DEBUFF;

    //evaluate passed pawns
    let black_passed_pawns = black_passed_pawns(black_pawns, white_pawns);
    let mut black_passed_pawns_value = 0;
    for black_pawn in black_passed_pawns{
        black_passed_pawns_value += constants::PASSED_PAWNS_BLACK[black_pawn.get_rank().to_index()]
    } 

    let white_passed_pawns = white_passed_pawns(white_pawns, black_pawns);
    let mut white_passed_pawns_value = 0;
    for white_pawn in white_passed_pawns{
        white_passed_pawns_value += constants::PASSED_PAWNS_WHITE[white_pawn.get_rank().to_index()]
    } 


    // evaluate king safety
    // calculate pawn shield for white
    let white_king_zone = king_attacks(BitBoard::from_square( white_king));
    let mut white_pawn_shield = 3 as i32 - white_pawns.bitand(white_king_zone).popcnt() as i32;
    white_pawn_shield *= -25;
    
    // calculate pawns going for the king white
    let black_storming_pawns_files = black_front_spans(black_pawns).bitand(white_king_zone);
    let black_storming_pawns = file_fill(black_storming_pawns_files).bitand(black_pawns);
    // print!("{}", black_storming_pawns);
    let mut black_storming_score = 0;
    for pawn in black_storming_pawns{
        black_storming_score += (7 - utils::distance(pawn, white_king))* -10;
    }

    
    // calculate pawn shield for black
    let black_king_zone = king_attacks(BitBoard::from_square( black_king));
    let mut black_pawn_shield = 3 as i32 - black_pawns.bitand(black_king_zone).popcnt() as i32;
    black_pawn_shield *= -25;

    // calculate pawns going for the king white
    let white_storming_pawns_files = black_front_spans(white_pawns).bitand(black_king_zone);
    let white_storming_pawns = file_fill(white_storming_pawns_files).bitand(white_pawns);
    // print!("{}", black_storming_pawns);
    let mut white_storming_score = 0;
    for pawn in white_storming_pawns{
        white_storming_score += (7 - utils::distance(pawn, black_king)) * -10;
    }


    let kingsq_w;
    if end_game{
        kingsq_w = sum_by_table(&BitBoard::from_square(white_king), constants::KING_ENDGAME_TABLE, false);
    }else{
        kingsq_w = sum_by_table(&BitBoard::from_square(white_king), constants::KING_SQUARES_TABLE, false);
    }

    let mut num_white: i32 = white_queens.count() as i32 * constants::QUEEN_VAL + white_bishops.count() as i32 * constants::BISHOP_VAL + white_knights.count() as i32 * constants::KNIGHT_VAL + white_pawns.count() as i32 * constants::PAWN_VAL + white_rooks.count() as i32 * constants::ROOK_VAL;
    num_white += knightsq_w + pawnsq_w + kingsq_w + queensq_w + bishopsq_w + rooksq_w;
    num_white += white_rooks_on_open_file as i32;
    num_white += white_doubled_pawns ;
    if !end_game{
        num_white += white_pawn_shield;
        num_white += black_storming_score;
    }else{
        num_white += white_passed_pawns_value;
    }
    
    
    //evaluate black positional advantage
    let knightsq_b = sum_by_table(&black_knights, constants::KNIGHT_SQUARES_TABLE, true);
    let pawnsq_b = sum_by_table(&black_pawns, constants::PAWN_SQUARES_TABLE, true);
    let queensq_b = sum_by_table(&black_queens, constants::QUEEN_SQUARES_TABLE, true);
    let bishopssq_b = sum_by_table(&black_bishops, constants::BISHOP_SQUARES_TABLE, true);
    let rooksq_b = sum_by_table(&black_rooks, constants::ROOK_SQUARES_TABLE, true);
    
    
    let kingsq_b;
    if end_game{
        kingsq_b = sum_by_table(&BitBoard::from_square(black_king), constants::KING_ENDGAME_TABLE, true);
    }else{
        kingsq_b = sum_by_table(&BitBoard::from_square(black_king), constants::KING_SQUARES_TABLE, true);

    }


    let mut num_black: i32 = black_queens.count() as i32 * constants::QUEEN_VAL + black_bishops.count() as i32 * constants::BISHOP_VAL + black_knights.count() as i32 * constants::KNIGHT_VAL + black_pawns.count() as i32 * constants::PAWN_VAL + black_rooks.count() as i32 * constants::ROOK_VAL;
    num_black += knightsq_b + pawnsq_b + kingsq_b + queensq_b + bishopssq_b + rooksq_b;
    num_black += black_rooks_on_open_file as i32;
    num_black += black_doubled_pawns ;
    if !end_game{
        num_black += black_pawn_shield;
        num_black += white_storming_score;
    }else{
        num_black += black_passed_pawns_value
    }

    match board.side_to_move(){
        chess::Color::Black=>return num_black - num_white ,
        chess::Color::White=>return num_white - num_black
    }
    
}
