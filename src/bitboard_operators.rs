use std::ops::{Not};

use chess::{BitBoard};

const NOT_A_FILE: u64 = 0xfefefefefefefefe; // ~0x0101010101010101
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f; // ~0x8080808080808080

fn east_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 1) & NOT_A_FILE)}
fn noEaOne (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 9) & NOT_A_FILE)}
fn soEaOne (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 7) & NOT_A_FILE)}
fn west_one (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 1) & NOT_H_FILE)}
fn soWeOne (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 >> 9) & NOT_H_FILE)}
fn noWeOne (b: BitBoard) -> BitBoard {return BitBoard::new((b.0 << 7) & NOT_H_FILE)}


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

pub fn open_files(wpanws: BitBoard , bpawns: BitBoard) -> BitBoard {
    return file_fill(wpanws).not() & file_fill(bpawns).not();
 }

pub fn closed_files(wpanws: BitBoard, bpawns:BitBoard) -> BitBoard{
    return file_fill(wpanws) & file_fill(bpawns);
 }


pub fn white_front_spans(wpawns:BitBoard) -> BitBoard{return nort_one (nort_fill(wpawns));}
pub fn black_rear_spans (bpawns:BitBoard) -> BitBoard{return nort_one (nort_fill(bpawns));}
pub fn black_front_spans(bpawns:BitBoard) -> BitBoard{return sout_one (sout_fill(bpawns));}pub fn white_rear_spans (wpawns:BitBoard) -> BitBoard{return sout_one (sout_fill(wpawns));}


 // pawns with at least one pawn in front on the same file
pub fn white_pawns_behind_own(wpawns: BitBoard) -> BitBoard {return wpawns & white_rear_spans(wpawns);}

// pawns with at least one pawn behind on the same file
pub fn white_pawns_infront_own (wpawns: BitBoard) -> BitBoard {return wpawns & white_front_spans(wpawns);}

 // pawns with at least one pawn in front on the same file
pub fn black_pawns_behind_own(bpawns: BitBoard) -> BitBoard {return bpawns & black_rear_spans(bpawns);}

 // pawns with at least one pawn behind on the same file
pub fn black_pawns_infront_own (bpawns: BitBoard) -> BitBoard {return bpawns & black_front_spans(bpawns);}

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