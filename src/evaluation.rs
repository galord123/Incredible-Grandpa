use std::ops::BitAnd;
use crate::bitboard_operators;
use crate::bitboard_operators::black_passed_pawns;
use crate::bitboard_operators::white_passed_pawns;
use crate::constants;
use crate::constants::Access;
use crate::utils;
use crate::bitboard_operators::{open_files, black_pawns_behind_own, white_pawns_behind_own, king_attacks, black_front_spans, file_fill};
use crate::utils::{get_piece_type, sum_by_table};
use chess::BitBoard;
use chess::Piece;
use chess::Square;

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
    let endgame = board.pieces(chess::Piece::Queen).count() == 0 && pieces_total_count <= 2;

    //evaluate white positional advantage
    let knightsq_w = sum_by_table(&white_knights, constants::KNIGHT_SQUARES_TABLE, false);
    let pawnsq_w = sum_by_table(&white_pawns, constants::PAWN_SQUARES_TABLE, false);
    let queensq_w = sum_by_table(&white_queens, constants::QUEEN_SQUARES_TABLE, false);
    let bishopsq_w = sum_by_table(&white_bishops, constants::BISHOP_SQUARES_TABLE, false);
    let rooksq_w = sum_by_table(&white_rooks, constants::ROOK_SQUARES_TABLE, false);

    // give bonus to rook on open file
    let white_rooks_on_open_file = constants::ROOK_ON_OPEN_FILE.access_endgame(endgame) as u32 * white_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();
    let black_rooks_on_open_file = constants::ROOK_ON_OPEN_FILE.access_endgame(endgame) as u32 * black_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();
    
    // evaluate pawns
    // evaluate double pawns
    let black_doubled_pawns = black_pawns_behind_own(black_pawns).popcnt() as i32 * constants::DOUBLED_PAWNS_DEBUFF.access_endgame(endgame);
    let white_doubled_pawns = white_pawns_behind_own(white_pawns).popcnt() as i32 * constants::DOUBLED_PAWNS_DEBUFF.access_endgame(endgame);

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

    // add bonus for knight in outpost 
    let mut white_outposted_knights = 0;

    for knight in white_knights{


        // print!("{}", white_pawns & bitboard_operators::black_pawn_any_attacks(BitBoard::from_square(knight)));
        if (white_pawns & bitboard_operators::black_pawn_any_attacks(BitBoard::from_square(knight))).popcnt() >= 1{

            if (black_pawns & bitboard_operators::white_pawn_any_attacks(BitBoard::from_square(knight))).popcnt() == 0{
                white_outposted_knights += constants::KNIGHT_OUTPOST_TABLE_WHITE[knight.to_index()];
            }
        }
    }

    let mut black_outposted_knights = 0;

    for knight in black_knights{
        if (black_pawns & bitboard_operators::white_pawn_any_attacks(BitBoard::from_square(knight))).popcnt() >= 1{
            if (black_pawns & bitboard_operators::black_pawn_any_attacks(BitBoard::from_square(knight))).popcnt() == 0{
                black_outposted_knights += constants::KNIGHT_OUTPOST_TABLE_BLACK[knight.to_index()];
            }
        }
    }
    // print!("white k-o: {}", white_outposted_knights);
    // print!("black k-o: {}", black_outposted_knights);


    // handle blocked bishops and rooks
    let mut w_blocked = 0;

    w_blocked += utils::blocked_bishop(&board, chess::Square::C1, chess::Square::D2, true);
    w_blocked += utils::blocked_bishop(&board, chess::Square::F1, chess::Square::E2, true);

    let mut b_blocked = 0;

    b_blocked += utils::blocked_bishop(&board, chess::Square::C8, chess::Square::D7, false);
    b_blocked += utils::blocked_bishop(&board, chess::Square::F8, chess::Square::E7, false);

    // handle blocked rooks
    if white_king == Square::B1 || white_king == Square::C1{
        if board.piece_on(Square::A1) == Some(Piece::Rook) || board.piece_on(Square::A2) == Some(Piece::Rook) || board.piece_on(Square::B1) == Some(Piece::Rook){
            w_blocked += -50;
        }
    }
    if white_king == Square::F1 || white_king == Square::G1{
        if board.piece_on(Square::H1) == Some(Piece::Rook) || board.piece_on(Square::H2) == Some(Piece::Rook) || board.piece_on(Square::G1) == Some(Piece::Rook){
            w_blocked += -50;
        }
    }

    if black_king == Square::B8 || white_king == Square::C8{
        if board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::B8) == Some(Piece::Rook){
            b_blocked += -50;
        }
    }
    if black_king == Square::F8 || white_king == Square::G8{
        if board.piece_on(Square::H8) == Some(Piece::Rook) || board.piece_on(Square::H8) == Some(Piece::Rook) || board.piece_on(Square::G8) == Some(Piece::Rook){
            b_blocked += -50;
        }
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
    if endgame{
        kingsq_w = sum_by_table(&BitBoard::from_square(white_king), constants::KING_ENDGAME_TABLE, false);
    }else{
        kingsq_w = sum_by_table(&BitBoard::from_square(white_king), constants::KING_SQUARES_TABLE, false);
    }

    let mut num_white: i32 = white_queens.count() as i32 * constants::QUEEN_VAL.access_endgame(endgame) + 
        white_bishops.count() as i32 * constants::BISHOP_VAL.access_endgame(endgame) + 
        white_knights.count() as i32 * constants::KNIGHT_VAL.access_endgame(endgame) + 
        white_pawns.count() as i32 * constants::PAWN_VAL.access_endgame(endgame) + 
        white_rooks.count() as i32 * constants::ROOK_VAL.access_endgame(endgame);
    num_white += knightsq_w + pawnsq_w + kingsq_w + queensq_w + bishopsq_w + rooksq_w;
    num_white += white_rooks_on_open_file as i32;
    num_white += white_doubled_pawns ;
    num_white += w_blocked;
    num_white += white_outposted_knights;

    if !endgame{
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
    if endgame{
        kingsq_b = sum_by_table(&BitBoard::from_square(black_king), constants::KING_ENDGAME_TABLE, true);
    }else{
        kingsq_b = sum_by_table(&BitBoard::from_square(black_king), constants::KING_SQUARES_TABLE, true);

    }


    let mut num_black: i32 = black_queens.count() as i32 * constants::QUEEN_VAL.access_endgame(endgame) + 
    black_bishops.count() as i32 * constants::BISHOP_VAL.access_endgame(endgame) + 
    black_knights.count() as i32 * constants::KNIGHT_VAL.access_endgame(endgame) + 
    black_pawns.count() as i32 * constants::PAWN_VAL.access_endgame(endgame) + 
    black_rooks.count() as i32 * constants::ROOK_VAL.access_endgame(endgame);
    num_black += knightsq_b + pawnsq_b + kingsq_b + queensq_b + bishopssq_b + rooksq_b;
    num_black += black_rooks_on_open_file as i32;
    num_black += black_doubled_pawns;
    num_black += b_blocked;
    num_black += black_outposted_knights;
    if !endgame{
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




