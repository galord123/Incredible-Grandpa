use chess::{self, Board};
use rand::{self, Rng};
use std::fs;
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read};
use std::collections::HashMap;
use std::str::FromStr;

pub const FILE_NAMES: &'static [char]  = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANK_NAMES: &'static [char]  = &['1', '2', '3', '4', '5', '6', '7', '8'];


#[derive(Debug, Clone, Copy)]
pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Clone, Copy)]
pub struct Move(u16);

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct PolyglotEntry {
    pub key: u64,
    pub move_: Move,
    pub weight: u16,
    pub learn: u32,
}

impl Move {
    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn end_file(&self) -> u8 {
        (self.0 & 0b111) as u8
    }

    pub fn end_row(&self) -> u8 {
        ((self.0 >> 3) & 0b111) as u8
    }

    pub fn start_file(&self) -> u8 {
        ((self.0 >> 6) & 0b111) as u8
    }

    pub fn start_row(&self) -> u8 {
        ((self.0 >> 9) & 0b111) as u8
    }

    pub fn promotion_piece(&self) -> Option<PromotionPiece> {
        // `promotion piece` is encoded as follows
        // none       0
        // knight     1
        // bishop     2
        // rook       3
        // queen      4
        let n = ((self.0 >> 12) & 0b111) as u8;
        match n {
            0 => None,
            1 => Some(PromotionPiece::Knight),
            2 => Some(PromotionPiece::Bishop),
            3 => Some(PromotionPiece::Rook),
            4 => Some(PromotionPiece::Queen),
            _ => panic!("invalid promotion piece"),
        }
    }


}
pub fn convert_str_to_move(s: &str) -> Move {
    let mut move_ = Move(0);
    let mut i = 0;
    for c in s.chars() {
        match c {
            'a'..='h' => {
                move_.0 |= ((c as u8 - 'a' as u8) as u16) << (i * 3);
                // println!("{:b}", (c as u8 - 'a' as u8) as u16);
                },
            '1'..='8' => {
                move_.0 |= ((c as u8 - '1' as u8) as u16) << (i * 3);
                // println!("{:b}", (c as u8 - '1' as u8) as u16);
            },
            'N' => move_.0 |= 0b0001_0000_0000_0000,
            'B' => move_.0 |= 0b0010_0000_0000_0000,
            'R' => move_.0 |= 0b0100_0000_0000_0000,
            'Q' => move_.0 |= 0b1000_0000_0000_0000,
            _ => panic!("invalid move"),
        }
        i += 1;
        // println!("{:b}", move_.0);
    }
    move_
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

pub fn convert_move_to_str(move_: Move) -> String {
    let mut s = String::new();
    s.push(FILE_NAMES[move_.end_file() as usize]);
    s.push(RANK_NAMES[move_.end_row() as usize]);
    s.push(FILE_NAMES[move_.start_file() as usize]);
    s.push(RANK_NAMES[move_.start_row() as usize]);
    

    // println!("{}", s);
    match move_.promotion_piece() {
        Some(PromotionPiece::Knight) => s.push('n'),
        Some(PromotionPiece::Queen) => s.push('q'),
        Some(PromotionPiece::Bishop) => s.push('b'),
        Some(PromotionPiece::Rook) => s.push('r'),
        None => {}
    }
    
    s
}
        



// pub fn convert_move_to_str(move_: Move)-> String {
    
//     let mut s = String::new();
//     s.push(FILE_NAMES[move_.start_file() as usize]);
//     s.push((move_.start_row() + 1) as char);
//     s.push(FILE_NAMES[move_.end_file() as usize]);
//     s.push((move_.end_row() + 1)as char);
//     match move_.promotion_piece() {
//         Some(PromotionPiece::Knight) => s.push('n'),
//         Some(PromotionPiece::Bishop) => s.push('b'),
//         Some(PromotionPiece::Rook) => s.push('r'),
//         Some(PromotionPiece::Queen) => s.push('q'),
//         None => {}
//     }
//     s
    
// }



impl Debug for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let start_file = self.start_file();
        let start_row = self.start_row();
        let end_file = self.end_file();
        let end_row = self.end_row();
        let promotion_piece = self.promotion_piece();
        write!(
            f,
            "{start_file}{start_row}-{end_file}{end_row}",
            start_file = start_file + 1,
            start_row = start_row + 1,
            end_file = end_file + 1,
            end_row = end_row + 1,
        )?;
        if let Some(promotion_piece) = promotion_piece {
            write!(f, "={:?}", promotion_piece)?;
        }
        Ok(())
    }
}

pub fn read_polyglot_book<R: Read>(mut reader: R) -> io::Result<HashMap<u64, Vec<PolyglotEntry>>> {
    let mut buf = [0u8; 16];
    let mut entries = HashMap::new();
    loop {
        match reader.read(&mut buf)? {
            0 => break Ok(entries),
            16 => {
                let entry = PolyglotEntry {
                    key: u64::from_be_bytes(buf[0..8].try_into().unwrap()),
                    move_: Move(u16::from_be_bytes(buf[8..10].try_into().unwrap())),
                    weight: u16::from_be_bytes(buf[10..12].try_into().unwrap()),
                    learn: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
                };
                // if the key is already in the map, append the entry to the vector
                if let Some(entries) = entries.get_mut(&entry.key) {
                    entries.push(entry);
                } else {
                    entries.insert(entry.key, vec![entry]);
                }
                
            }
            n => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("read {} bytes, expected 16", n),
                ));
            }
        }
    }
}


pub fn write_entry<W: std::io::Write>(mut writer: W, entry: &PolyglotEntry) -> io::Result<()> {

    let bytes = &entry.key.to_be_bytes();
    writer.write_all(bytes).unwrap();
    
    let bytes = &entry.move_.value().to_be_bytes();
    writer.write_all(bytes).unwrap();
    
    let bytes = &entry.weight.to_be_bytes();
    writer.write_all(bytes).unwrap();
    
    let bytes = &entry.learn.to_be_bytes();
    writer.write_all(bytes).unwrap();
    // flush the buffer
    writer.flush().unwrap();
    return Ok(());
}

pub fn write_entrys(){
    let path = "C:\\Users\\משתמש\\Documents\\projects\\ChessAi\\ChessAi\\book.txt";
    let contents = fs::read_to_string(path)
    .expect("Something went wrong reading the file");
    // create a new file
    let _ = std::fs::File::create("book.bin").expect("Something went wrong opening the file");
    
    let file = std::fs::OpenOptions::new()
    .write(true)
    .append(true)
    .open("book.bin")
    .unwrap();
    let mut writer = std::io::BufWriter::new(file);

    let splited_lines = contents.split("\n");
    for line in splited_lines {
        let parts = line.split(";");
        let converted = parts.collect::<Vec<&str>>();
        let b = Board::from_str(converted[0]).unwrap();
        let moves: Vec<&str> = converted[1].split(" ").collect();
        
        for m in moves {
            // make a polyglot entry
            println!("{}", m);
            match chess::ChessMove::from_san(&b, m){
                Ok(m) => {
                    let entry = PolyglotEntry{
                        key: b.get_hash(),
                        move_: convert_str_to_move(m.to_string().as_str()),
                        weight: 0,
                        learn: 0,
                    };
                    // write the entry
                    write_entry(&mut writer, &entry).unwrap();
                }
                Err(e) => {
                    println!("{} - {}", m, e);
                }
            
            }
        }

    }

}


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