use std::borrow::BorrowMut;
use std::ops::BitAnd;
use std::ops::BitOr;
use crate::bitboard_operators;
use crate::bitboard_operators::black_passed_pawns;
use crate::bitboard_operators::white_front_spans;
use crate::bitboard_operators::white_passed_pawns;
use crate::constants;
use crate::constants::Access;
use crate::constants::BISHOP_SQUARES_TABLE;
use crate::constants::KING_ENDGAME_TABLE;
use crate::constants::KING_SQUARES_TABLE;
use crate::utils;
use crate::bitboard_operators::{open_files, black_pawns_behind_own, white_pawns_behind_own, king_attacks, black_front_spans, file_fill, half_open_files};
use crate::utils::{get_piece_type, sum_by_table};
use chess::BitBoard;
use chess::Color;
use chess::Piece;
use chess::Rank;
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
    
    let pieces_total_count = board.pieces(chess::Piece::Bishop).popcnt() + board.pieces(chess::Piece::Knight).popcnt() + board.pieces(chess::Piece::Rook).popcnt();
    let endgame = board.pieces(chess::Piece::Queen).popcnt() == 0 && pieces_total_count <= 2;

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


    // handle blocked bishops
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

    if black_king == Square::B8 || black_king == Square::C8{
        if board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::B8) == Some(Piece::Rook){
            b_blocked += -50;
        }
    }
    if black_king == Square::F8 || black_king == Square::G8{
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
    let white_storming_pawns_files = white_front_spans(white_pawns).bitand(black_king_zone);
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

    let mut num_white: i32 = white_queens.popcnt() as i32 * constants::QUEEN_VAL.access_endgame(endgame) + 
        white_bishops.popcnt() as i32 * constants::BISHOP_VAL.access_endgame(endgame) + 
        white_knights.popcnt() as i32 * constants::KNIGHT_VAL.access_endgame(endgame) + 
        white_pawns.popcnt() as i32 * constants::PAWN_VAL.access_endgame(endgame) + 
        white_rooks.popcnt() as i32 * constants::ROOK_VAL.access_endgame(endgame);
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


    let mut num_black: i32 = black_queens.popcnt() as i32 * constants::QUEEN_VAL.access_endgame(endgame) + 
    black_bishops.popcnt() as i32 * constants::BISHOP_VAL.access_endgame(endgame) + 
    black_knights.popcnt() as i32 * constants::KNIGHT_VAL.access_endgame(endgame) + 
    black_pawns.popcnt() as i32 * constants::PAWN_VAL.access_endgame(endgame) + 
    black_rooks.popcnt() as i32 * constants::ROOK_VAL.access_endgame(endgame);
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


pub fn evaluate_rework(board: &chess::Board) -> i32{
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

    let white_king_zone = king_attacks(BitBoard::from_square( white_king));
    let black_king_zone = king_attacks(BitBoard::from_square( black_king));



    let mut total_black_score = 0;
    let mut total_white_score = 0;
    
    let pieces_total_count = board.pieces(chess::Piece::Bishop).popcnt() + board.pieces(chess::Piece::Knight).popcnt() + board.pieces(chess::Piece::Rook).popcnt();
    let endgame = board.pieces(chess::Piece::Queen).popcnt() == 0 && pieces_total_count <= 2;

    let black_pawns = get_piece_type(&board, chess::Piece::Pawn, chess::Color::Black);
    let white_pawns = get_piece_type(&board, chess::Piece::Pawn, chess::Color::White);

    let all_pieces = board.combined();
    let mut pieces_attacking_white_king = 0;
    let mut pieces_attacking_black_king = 0;

    let mut pieces_attacking_white_king_units = 0;
    let mut pieces_attacking_black_king_units = 0;

    for square in *all_pieces{
        let piece_type = board.piece_on(square);
        let piece_color = board.color_on(square);
        let mut cur_score = 0;
        match piece_type{
            None=>{},
            Some(piece)=>{
                if let Some(color) = piece_color{
                    match piece {
                        Piece::Bishop=>{
                            cur_score += constants::BISHOP_VAL.access_endgame(endgame);
                            if color == Color::White{
                                cur_score += BISHOP_SQUARES_TABLE[square.to_index()];

                                if chess::get_bishop_rays(square).bitand(black_king_zone).popcnt() > 0{
                                    // check if the queen is attacking the black king on the real board within 2 squares
                            
                                    pieces_attacking_black_king += 1;
                                    pieces_attacking_black_king_units += 1;
                                    
                                }
                            }else{
                                cur_score += constants::BISHOP_SQUARES_TABLE_BLACK[square.to_index()];

                                if chess::get_bishop_rays(square).bitand(white_king_zone).popcnt() > 0{
                                    // check if the queen is attacking the black king on the real board within 2 squares
                                        pieces_attacking_white_king += 1;
                                        pieces_attacking_white_king_units += 1;
                                    
                                }
                            }
                            
                        },
                        Piece::King=>{
                            if endgame{
                                if color == Color::White{
                                    cur_score += KING_ENDGAME_TABLE[square.to_index()];

                                }else{
                                    cur_score += KING_ENDGAME_TABLE[utils::mirror_square(&square).to_index()];

                                }
                            }else{
                                if color == Color::White{
                                    cur_score += KING_SQUARES_TABLE[square.to_index()];

                                }else{
                                    cur_score += constants::KING_SQUARES_TABLE_BLACK[square.to_index()];

                                }
                            }
                        },
                        Piece::Knight=>{
                            cur_score += constants::KNIGHT_VAL.access_endgame(endgame);
                            
                            if color == Color::White{
                                cur_score += constants::KNIGHT_SQUARES_TABLE[square.to_index()];
                                // check if the knight is attacking the king
                                if chess::get_knight_moves(square).bitand(black_king_zone).popcnt() > 0{
                                    pieces_attacking_black_king += 1;
                                    pieces_attacking_black_king_units += 1;
                                }

                            }else{
                                cur_score += constants::KNIGHT_SQUARES_TABLE_BLACK[square.to_index()];
                                // check if the knight is attacking the king
                                if chess::get_knight_moves(square).bitand(white_king_zone).popcnt() > 0{
                                    pieces_attacking_white_king += 1;
                                    pieces_attacking_white_king_units += 1;
                                }
                            }

                            if let Some(color) = piece_color{
                                match color{
                                Color::White=>{
                                    if (white_pawns & bitboard_operators::black_pawn_any_attacks(BitBoard::from_square(square))).popcnt() >= 1{

                                        if (black_pawns & bitboard_operators::white_pawn_any_attacks(BitBoard::from_square(square))).popcnt() == 0{
                                            cur_score += constants::KNIGHT_OUTPOST_TABLE_WHITE[square.to_index()];
                                        }
                                    }
                                }
                                Color::Black=>{
                                    if (black_pawns & bitboard_operators::white_pawn_any_attacks(BitBoard::from_square(square))).popcnt() >= 1{
                                        if (black_pawns & bitboard_operators::black_pawn_any_attacks(BitBoard::from_square(square))).popcnt() == 0{
                                            cur_score += constants::KNIGHT_OUTPOST_TABLE_BLACK[square.to_index()];
                                        }
                                    }
                                }
                            }
                            // add bonus for outpost
                        }
                        
                        },
                        Piece::Rook=>{
                            cur_score += constants::ROOK_VAL.access_endgame(endgame);
                            if color == Color::White{
                                cur_score += constants::ROOK_SQUARES_TABLE[square.to_index()];
                                // check if the rook is attacking the king
                                if chess::get_rook_rays(square).bitand(black_king_zone).popcnt() > 0{
                                    pieces_attacking_black_king += 1;
                                    pieces_attacking_black_king_units += 2;
                                }

                            }else{
                                cur_score += constants::ROOK_SQUARES_TABLE[utils::mirror_square(&square).to_index()];
                                // check if the rook is attacking the king
                                if chess::get_rook_rays(square).bitand(white_king_zone).popcnt() > 0{
                                    pieces_attacking_white_king += 1;
                                    pieces_attacking_white_king_units += 2;
                                }
                            }

                        },
                        Piece::Pawn=>{
                            cur_score += constants::PAWN_VAL.access_endgame(endgame);
                            
                            if color == Color::White{
                                cur_score += constants::PAWN_SQUARES_TABLE[square.to_index()];
                            }else{
                                cur_score += constants::PAWN_SQUARES_TABLE_BLACK[square.to_index()];
                            }

                            // calculate pawn storm and shelter
                            if !endgame{
                                if let Some(color) = piece_color {
                                    let file = square.get_file();
                                    let rank = square.get_rank();
                                    match color{
                                        Color::Black=>{
                                            let king_file = white_king.get_file(); 
                                            let own_king_file = black_king.get_file(); 
                                            if file == king_file || file.left() == king_file || file.right() == king_file{
                                                if rank == Rank::Fifth{
                                                    total_white_score -= 10;
                                                }else if rank == Rank::Fourth{
                                                    total_white_score -= 30;
                                                }else if rank == Rank::Third{
                                                    total_white_score -= 60;
                                                }
                                            }

                                            // calcualte pawn shelter for black king
                                            if file == own_king_file || file.left() == own_king_file || file.right() == own_king_file{
                                                if rank != Rank::Seventh{
                                                    let score = rank.to_index() as i32 + 1;
                                                    let black_shlter =  36 - score*score;
                                                    total_black_score -= black_shlter;
                                                }
                                            }
                                        }
                                        Color::White=>{
                                            let king_file = black_king.get_file(); 
                                            let own_king_file = white_king.get_file(); 
                                            if file == king_file || file.left() == king_file || file.right() == king_file{
                                                if rank == Rank::Fourth{
                                                    total_black_score -= 10;
                                                }else if rank == Rank::Fifth{
                                                    total_black_score -= 30;
                                                }else if rank == Rank::Sixth{
                                                    total_black_score -= 60;
                                                }
                                            }

                                            // calcualte pawn shelter for white king
                                            if file == own_king_file || file.left() == own_king_file || file.right() == own_king_file{
                                                if rank != Rank::Second{
                                                    let score = 8-(rank.to_index() as i32);
                                                    let white_shelter = 36 - score*score;
                                                    total_white_score -= white_shelter
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        
                        },
                        Piece::Queen=>{
                            cur_score += constants::QUEEN_VAL.access_endgame(endgame);
                            if color == Color::White{
                                cur_score += constants::QUEEN_SQUARES_TABLE[square.to_index()];
                                // check if the queen is attacking the black king
                                
                                if chess::get_rook_rays(square).bitor(chess::get_bishop_rays(square)).bitand(black_king_zone).popcnt() > 0{
                                    // check if the queen is attacking the black king on the real board within 2 squares

                                    pieces_attacking_black_king += 1;
                                    pieces_attacking_black_king_units += 4;
                                    
                                }

                            }else{
                                cur_score += constants::QUEEN_SQUARES_TABLE_BLACK[square.to_index()];
                                if chess::get_rook_rays(square).bitor(chess::get_bishop_rays(square)).bitand(white_king_zone).popcnt() > 0{
                                    // check if the queen is attacking the white king on the real board within 2 squares
                                    
                                    pieces_attacking_white_king += 1;
                                    pieces_attacking_white_king_units += 4;
                                    
                                }
                            }
                        },
                    }
                }
                if let Some(color) = piece_color {
                    match color{
                        Color::Black=>{
                            total_black_score += cur_score;
                        }
                        Color::White=>{
                            total_white_score += cur_score;
                        }
                    }
                }
            }
        }
    }

    // handle blocked bisops and rooks
    let (white_blocked, black_blocked) = handle_blocked_pieces(board, black_king, white_king);

    // give bonus to rook on open file
    let white_rooks = get_piece_type(&board, chess::Piece::Rook, chess::Color::White);
    let black_rooks = get_piece_type(&board, chess::Piece::Rook, chess::Color::Black);
    let white_rooks_on_open_file = constants::ROOK_ON_OPEN_FILE.access_endgame(endgame) as u32 * white_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();
    let black_rooks_on_open_file = constants::ROOK_ON_OPEN_FILE.access_endgame(endgame) as u32 * black_rooks.bitand(open_files(white_pawns, black_pawns)).popcnt();

    // give bonus to rook on half-open file
    let white_rooks_on_half_open_file = constants::ROOK_ON_HALF_OPEN_FILE.access_endgame(endgame) as u32 * white_rooks.bitand(half_open_files(white_pawns)).popcnt();
    let black_rooks_on_half_open_file = constants::ROOK_ON_HALF_OPEN_FILE.access_endgame(endgame) as u32 * black_rooks.bitand(half_open_files(black_pawns)).popcnt();

    // Give bonus to bishop pair
    let white_bishops = get_piece_type(&board, chess::Piece::Bishop, chess::Color::White);
    let black_bishops = get_piece_type(&board, chess::Piece::Bishop, chess::Color::Black);
    let white_bishop_pair = constants::BISHOP_PAIR.access_endgame(endgame) as u32 * white_bishops.popcnt() / 2;
    let black_bishop_pair = constants::BISHOP_PAIR.access_endgame(endgame) as u32 * black_bishops.popcnt() / 2;

    total_black_score += black_rooks_on_open_file as i32 + black_rooks_on_half_open_file as i32 + black_bishop_pair as i32 + black_blocked as i32;
    total_white_score += white_rooks_on_open_file as i32 + white_rooks_on_half_open_file as i32 + white_bishop_pair as i32 + white_blocked as i32;
    
    // pieces attacking the other king 
    total_black_score += (-20. * pieces_attacking_black_king_units as f32 * constants::PIECES_ATTACKING_KING[pieces_attacking_black_king]) as i32;
    total_white_score += (-20. * pieces_attacking_white_king_units as f32 * constants::PIECES_ATTACKING_KING[pieces_attacking_white_king]) as i32;


    match board.side_to_move(){
        chess::Color::Black=>return total_black_score - total_white_score,// + constants::TEMPO_BONUS.access_endgame(endgame),
        chess::Color::White=>return total_white_score - total_black_score// + constants::TEMPO_BONUS.access_endgame(endgame),
    }
}


fn handle_blocked_pieces(board: &chess::Board, black_king: Square, white_king: Square) -> (i32, i32){
    // handle blocked bishops
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

    if black_king == Square::B8 || black_king == Square::C8{
        if board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::A8) == Some(Piece::Rook) || board.piece_on(Square::B8) == Some(Piece::Rook){
            b_blocked += -50;
        }
    }
    if black_king == Square::F8 || black_king == Square::G8{
        if board.piece_on(Square::H8) == Some(Piece::Rook) || board.piece_on(Square::H8) == Some(Piece::Rook) || board.piece_on(Square::G8) == Some(Piece::Rook){
            b_blocked += -50;
        }
    }
    return (w_blocked, b_blocked)
}


pub fn evaluate_pawn_structure(black_pawns: BitBoard, white_pawns: BitBoard, endgame: bool) -> i32{
    let mut score = 0;
    // evaluate pawns
    // evaluate double pawns
    let black_doubled_pawns = black_pawns_behind_own(black_pawns).popcnt() as i32 * constants::DOUBLED_PAWNS_DEBUFF.access_endgame(endgame);
    let white_doubled_pawns = white_pawns_behind_own(white_pawns).popcnt() as i32 * constants::DOUBLED_PAWNS_DEBUFF.access_endgame(endgame);

    // evaluate isolated pawns
    let black_isolated_pawns = bitboard_operators::isolanis(black_pawns).popcnt() as i32 * constants::ISOLATED_PAWNS_DEBUFF.access_endgame(endgame);
    let white_isolated_pawns = bitboard_operators::isolanis(white_pawns).popcnt() as i32 * constants::ISOLATED_PAWNS_DEBUFF.access_endgame(endgame);

    // evaluate backward pawns
    let black_backward_pawns = bitboard_operators::white_backward(white_pawns, black_pawns).popcnt() as i32 * constants::BACKWARD_PAWNS_DEBUFF.access_endgame(endgame);
    let white_backward_pawns = bitboard_operators::white_backward(white_pawns, black_pawns).popcnt() as i32 * constants::BACKWARD_PAWNS_DEBUFF.access_endgame(endgame);


    // evaluate passed pawns
    let mut white_passed_pawns_value = 0;
    let mut black_passed_pawns_value = 0;
    if endgame{
        let black_passed_pawns = black_passed_pawns(black_pawns, white_pawns);
        
        for black_pawn in black_passed_pawns{
            black_passed_pawns_value += constants::PASSED_PAWNS_BLACK[black_pawn.get_rank().to_index()]
        } 

        let white_passed_pawns = white_passed_pawns(white_pawns, black_pawns);
        
        for white_pawn in white_passed_pawns{
            white_passed_pawns_value += constants::PASSED_PAWNS_WHITE[white_pawn.get_rank().to_index()]
        } 

    }else{
        let black_passed_pawns = black_passed_pawns(black_pawns, white_pawns);
        
        for black_pawn in black_passed_pawns{
            black_passed_pawns_value += (10. + 60.* constants::PASSED_PAWNS_BLACK_OPENING[black_pawn.get_rank().to_index()]) as i32;
        }

        let white_passed_pawns = white_passed_pawns(white_pawns, black_pawns);
        
        for white_pawn in white_passed_pawns{
            white_passed_pawns_value += (10. + 60.* constants::PASSED_PAWNS_WHITE_OPENING[white_pawn.get_rank().to_index()]) as i32;
        }
    }
    score += white_doubled_pawns - black_doubled_pawns + white_passed_pawns_value - black_passed_pawns_value + white_isolated_pawns - black_isolated_pawns + white_backward_pawns - black_backward_pawns;
    score
}

pub fn material_balance(board: &chess::Board) -> i32{
    let mut score = 0;
    let mut white_material = 0;
    let mut black_material = 0;
    let squares = board.combined();
    for square in *squares{
        match board.color_on(square){
            Some(chess::Color::White) => {
                match board.piece_on(square){
                    Some(chess::Piece::Pawn) => {
                        white_material += constants::PAWN_VAL.0;
                    },
                    Some(chess::Piece::Knight) => {
                        white_material += constants::KNIGHT_VAL.0;
                    },
                    Some(chess::Piece::Bishop) => {
                        white_material += constants::BISHOP_VAL.0;
                    },
                    Some(chess::Piece::Rook) => {
                        white_material += constants::ROOK_VAL.0;
                    },
                    Some(chess::Piece::Queen) => {
                        white_material += constants::QUEEN_VAL.0;
                    },
                    Some(chess::Piece::King) => {
                        
                    },
                    _ => {
                        panic!("Invalid piece on square");
                    }
                }
            },
            Some(chess::Color::Black) => {
                match board.piece_on(square){
                    Some(chess::Piece::Pawn) => {
                        black_material += constants::PAWN_VAL.0;
                    },
                    Some(chess::Piece::Knight) => {
                        black_material += constants::KNIGHT_VAL.0;
                    },
                    Some(chess::Piece::Bishop) => {
                        black_material += constants::BISHOP_VAL.0;
                    },
                    Some(chess::Piece::Rook) => {
                        black_material += constants::ROOK_VAL.0;
                    },
                    Some(chess::Piece::Queen) => {
                        black_material += constants::QUEEN_VAL.0;
                    },
                    Some(chess::Piece::King) => {
                       
                    },
                    _ => {
                        panic!("Invalid piece on square");
                    }
                }
            },
            _ => {
                panic!("Invalid color on square");
            }
        }
    }
    match board.side_to_move(){
        chess::Color::White => {
            score += white_material - black_material;
        },
        chess::Color::Black => {
            score -= white_material - black_material;
        }
    }
    
    score
}


// fn get_smallest_attacker(board: &chess::Board, square: Square, side: chess::Color) -> chess::Piece{
//     let mut smallest_attacker = chess::Piece::Pawn;
//     let mut smallest_attacker_value = constants::PAWN_VAL.0;
//     for attacker in board.attackers_to(square, side){
//         let attacker_value = constants::PIECE_VAL[attacker.to_index()];
//         if attacker_value < smallest_attacker_value{
//             smallest_attacker = attacker;
//             smallest_attacker_value = attacker_value;
//         }
//     }
//     smallest_attacker
// }

// // static exchange evaluation
// pub fn static_exchange(board: &chess::Board, square: Square, side: chess::Color) -> i32{
//     let mut value = 0;
//     let piece = get_smallest_attacker(board, square);
//     if let Some(piece) = piece{
//         let passed_board = board.make_move_new(chess::ChessMove::new(square, square, piece, None));
//         value = cmp::max(value, passed_board.static_exchange(square, side));

// get piece value
// fn get_piece_value(piece: chess::Piece) -> i32{
//     match piece{
//         chess::Piece::Pawn => {
//             constants::PAWN_VAL.0
//         },
//         chess::Piece::Knight => {
//             constants::KNIGHT_VAL.0
//         },
//         chess::Piece::Bishop => {
//             constants::BISHOP_VAL.0
//         },
//         chess::Piece::Rook => {
//             constants::ROOK_VAL.0
//         },
//         chess::Piece::Queen => {
//             constants::QUEEN_VAL.0
//         },
//         chess::Piece::King => {
//             999
//         },
//         _ => {
//             panic!("Invalid piece");
//         }
//     }
// }




// fn see ( board: &chess::Board, toSq: Square,  target: Piece, frSq: Square,  aPiece: Piece) -> i32{
// {
//     let gain: [i32; 32] = [0; 32];
//     let mut d = 0;
//     let mayXray: BitBoard = board.pieces(Piece::Pawn) | board.pieces(Piece::Bishop) | board.pieces(Piece::Rook) | board.pieces(Piece::Queen);
//     let fromSet = BitBoard::from_square(frSq);
//     let mut occ     = board.combined();
//     let mut attadef = attacksTo( occ, toSq );
//     gain[d]     = get_piece_value(target);
//     loop {
//         d += 1; // next depth and side
//         gain[d]  = value[aPiece] - gain[d-1]; // speculative store, if defended
//         if std::cmp::max(-gain[d-1], gain[d]) < 0{ 
//             break;
//         } // pruning does not influence the result
//         attadef ^= fromSet; // reset bit in set to traverse
//         *occ     ^= fromSet; // reset bit in temporary occupancy (for x-Rays)
//         if ( fromSet & mayXray )
//             attadef |= considerXrays(occ, ..);
//         fromSet  = getLeastValuablePiece (attadef, d & 1, aPiece);
//         if (fromSet.0 == 0){ break;}
//     } 
//     while (--d){
//         gain[d-1]= -max (-gain[d-1], gain[d])
//     }
//     return gain[0];
// }