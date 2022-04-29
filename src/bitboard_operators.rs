use std::ops::Not;

use chess::{BitBoard};

const NOT_A_FILE: u64 = 0xfefefefefefefefe; // ~0x0101010101010101
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f; // ~0x8080808080808080

fn east_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 1) & NOT_A_FILE)}
fn north_east_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 9) & NOT_A_FILE)}
fn south_east_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 7) & NOT_A_FILE)}
fn west_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 1) & NOT_H_FILE)}
fn south_west_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 9) & NOT_H_FILE)}
fn north_west_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 7) & NOT_H_FILE)}


fn sout_one (b: BitBoard) -> BitBoard {return  BitBoard::new(b.0 >> 8)}
fn nort_one (b:BitBoard) -> BitBoard {return   BitBoard::new(b.0 << 8)}


fn nort_fill(gen: BitBoard) -> BitBoard{
    let mut val = gen.0;
    val |= val <<  8;
    val |= val << 16;
    val |= val << 32;
    return BitBoard::new(val);
 }
 
fn sout_fill(gen: BitBoard) -> BitBoard {
    let mut val = gen.0;
    val |= val >>  8;
    val |= val >> 16;
    val |= val >> 32;
    return BitBoard::new(val);
 }


pub fn file_fill(gen: BitBoard)-> BitBoard{
    return nort_fill(gen) | sout_fill(gen);
 }


pub fn half_open_files(gen: BitBoard) ->BitBoard {return file_fill(gen).not();}
 

pub fn open_files(wpanws: BitBoard , bpawns: BitBoard) -> BitBoard {
    return file_fill(wpanws).not() & file_fill(bpawns).not();
 }

pub fn closed_files(wpanws: BitBoard, bpawns:BitBoard) -> BitBoard{
    return file_fill(wpanws) & file_fill(bpawns);
 }


pub fn east_attack_file_fill (pawns: BitBoard) -> BitBoard {return east_one(file_fill(pawns));}
pub fn west_attack_file_fill (pawns: BitBoard) -> BitBoard {return west_one(file_fill(pawns));}

pub fn white_front_spans(wpawns:BitBoard) -> BitBoard{return nort_one (nort_fill(wpawns));}
pub fn black_rear_spans (bpawns:BitBoard) -> BitBoard{return nort_one (nort_fill(bpawns));}
pub fn black_front_spans(bpawns:BitBoard) -> BitBoard{return sout_one (sout_fill(bpawns));}
pub fn white_rear_spans (wpawns:BitBoard) -> BitBoard{return sout_one (sout_fill(wpawns));}


pub fn white_pawn_east_attacks(wpawns: BitBoard) -> BitBoard {return north_east_one(wpawns);}
pub fn white_pawn_west_attacks(wpawns: BitBoard) -> BitBoard {return north_west_one(wpawns);}

pub fn black_pawn_east_attacks(bpawns: BitBoard) -> BitBoard {return south_east_one(bpawns);}
pub fn black_pawn_west_attacks(bpawns: BitBoard) -> BitBoard {return south_west_one(bpawns);}



pub fn white_pawn_any_attacks(wpawns: BitBoard) -> BitBoard {
    return white_pawn_east_attacks(wpawns) | white_pawn_west_attacks(wpawns);
}

 pub fn black_pawn_any_attacks(bpawns: BitBoard) -> BitBoard {
    return black_pawn_east_attacks(bpawns) | black_pawn_west_attacks(bpawns);
}




 // pawns with at least one pawn in front on the same file
pub fn white_pawns_behind_own(wpawns: BitBoard) -> BitBoard {return wpawns & white_rear_spans(wpawns);}

// pawns with at least one pawn behind on the same file
pub fn white_pawns_infront_own (wpawns: BitBoard) -> BitBoard {return wpawns & white_front_spans(wpawns);}

 // pawns with at least one pawn in front on the same file
pub fn black_pawns_behind_own(bpawns: BitBoard) -> BitBoard {return bpawns & black_rear_spans(bpawns);}

 // pawns with at least one pawn behind on the same file
pub fn black_pawns_infront_own (bpawns: BitBoard) -> BitBoard {return bpawns & black_front_spans(bpawns);}




pub fn no_neighbor_on_east_file (pawns: BitBoard) -> BitBoard {
    return pawns & west_attack_file_fill(pawns).not();
}

pub fn no_neighbor_on_west_file (pawns: BitBoard) -> BitBoard{
    return pawns & east_attack_file_fill(pawns).not();
}

pub fn isolanis(pawns: BitBoard) -> BitBoard{
   return  no_neighbor_on_east_file(pawns)
         & no_neighbor_on_west_file(pawns);
}

pub fn half_isolanis(pawns: BitBoard) -> BitBoard {
   return  no_neighbor_on_east_file(pawns)
         ^ no_neighbor_on_west_file(pawns);
}



pub fn white_backward(wpawns: BitBoard, bpawns: BitBoard) -> BitBoard {
    let  stops = BitBoard::new(wpawns.0 << 8);
    let white_attack_spans = file_fill(white_pawn_any_attacks(wpawns));
                     
    
    let black_attacks     = black_pawn_any_attacks(bpawns);
    return BitBoard::new((stops & black_attacks & white_attack_spans.not()).0 >> 8);
 }


pub fn king_attacks(king_set:BitBoard) -> BitBoard{
    let mut _king_set = king_set;
    let mut attacks: BitBoard = east_one(_king_set) | west_one(_king_set);
    _king_set |= attacks;
    attacks |= nort_one(_king_set) | sout_one(_king_set);
    return attacks;
 }


pub fn white_passed_pawns(wpawns:BitBoard, bpawns:BitBoard) -> BitBoard {
    let mut all_front_spans: BitBoard = black_front_spans(bpawns);
    all_front_spans |= east_one(all_front_spans)
                  |  west_one(all_front_spans);
    return wpawns & all_front_spans.not();
 }
 

pub fn black_passed_pawns(bpawns: BitBoard, wpawns: BitBoard) -> BitBoard {
    let mut all_front_spans = white_front_spans(wpawns);
    all_front_spans |= east_one(all_front_spans)
                  |  west_one(all_front_spans);
    return bpawns & all_front_spans.not();
 }