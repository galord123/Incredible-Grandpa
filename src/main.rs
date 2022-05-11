pub mod constants;
pub mod evaluation;
pub mod utils;
pub mod opening;
pub mod bitboard_operators;
pub mod search;
use std::io::{Write, Read};

use std::{io::{self}};
use chess::{self, ChessMove, Board, Game};

use rand::Rng;
use std::{time::Instant, str::FromStr};

use crate::{search::SearchInfo, search::Entry};


fn iterative_deepening(board: &Board, remaining_time: u128, depth: u32) -> (Option<chess::ChessMove>, SearchInfo){
    let mut table: Option<Vec<(ChessMove, i32)>> = None;
    let mut best_move:Option<chess::ChessMove> = None;
    let mut best_score = -9999; 
    let allowed_time =  remaining_time / 30; 
    let mut total_time = 0; 
    let mut info: SearchInfo = SearchInfo::new();
    let mut cachetable = chess::CacheTable::new(65536,  Entry{depth: 0, node_type: search::Nodetype::Pvnode, score: 0});
    let mut pawn_table = chess::CacheTable::new(65536,  0);

    let mut _depth = 1;

    let mut use_depth = true;
    if depth == 0{
        use_depth = false;
    }

    while ((use_depth && _depth <= depth) || (!use_depth && total_time < allowed_time)) && _depth < 100{
        let now = Instant::now();
        let time_left = allowed_time - total_time.min(allowed_time);
        
        let result = search::search_depth(&board, _depth, &table, time_left, (best_move, best_score), &mut cachetable, &mut pawn_table);
        if result.0.is_none() || result.4{
            println!("time took {}", total_time);
            println!("evaluated {} positions. {} transpostions recorded and {} used", info.nodes_searched, info.transpostions_recorded, info.transpostions_used);
            println!("recorded {} pawn stractures. {} used", info.pawn_hash_table_recorded, info.pawn_hash_table_used);
            if result.0.is_none(){
                println!("used last depth");
                return (best_move, info);
            }else{
                return (result.0, info);
            }
        }
        best_move = result.0;
        best_score = result.1;
        info.nodes_searched += result.3.nodes_searched;
        info.transpostions_used += result.3.transpostions_used;
        info.transpostions_recorded += result.3.transpostions_recorded;
        info.pawn_hash_table_used += result.3.pawn_hash_table_used;
        info.pawn_hash_table_recorded += result.3.pawn_hash_table_recorded;

        io::stdout().write((format!("info nodes {}\n", info.nodes_searched)).as_bytes()).ok();
      

        if best_score >= 9999{
            return (best_move, info);
        }
        table = Some(result.2);  
        let elapsed = now.elapsed();
        println!("info time {}", elapsed.as_millis());
        total_time += elapsed.as_millis(); 
        _depth += 1;
    }
    println!("time took {}", total_time);
    println!("evaluated {} positions. {} transpostions recorded and {} used", info.nodes_searched, info.transpostions_recorded, info.transpostions_used);
    println!("recorded {} pawn stractures. {} used", info.pawn_hash_table_recorded, info.pawn_hash_table_used);
    return (best_move, info);
}


fn choose_move(board: chess::Board, depth: u32, remaining_time: u128) -> (Option<chess::ChessMove>, SearchInfo){
    let count = chess::MoveGen::new_legal(&board);
    let moves: Vec<ChessMove> = count.collect();
    if moves.len() == 1{
        for chess_move in moves{
            return (Some(chess_move), SearchInfo::new());
        }
    }

    let (best_move, info) = iterative_deepening(&board, remaining_time, depth);
    
    if let Some(chess_move) = best_move {
        
        println!("best move found {} ", chess_move);
    }
    println!( "evaluated {} positions. {} transpostions recorded and {} used", info.nodes_searched, info.transpostions_recorded, info.transpostions_used);
    return (best_move, info);
}


fn play_random_move(board: chess::Board) -> Option<chess::ChessMove> {
    let movegen = chess::MoveGen::new_legal(&board);

    let moves: Vec<ChessMove> = movegen.collect();
    if moves.len() == 0{
        return None
    }
    Some(moves[rand::thread_rng().gen_range(0..moves.len())])
}


fn play_bot_move( board: chess::Board, depth: u32, book_moves: u32, remaining_time: u128) -> ChessMove{
    if book_moves > 0{
        let file = std::fs::File::open("C:\\Users\\משתמש\\Documents\\projects\\RustChess\\target\\release\\book.bin").unwrap(); 
        let book = opening::read_polyglot_book(file).unwrap();
        // print the first few entrys of the book
        // println!("{:#?}", &book.iter().take(20).collect::<Vec<_>>());

        match book.get(&board.get_hash()) {
            None=>{
                // println!("no book entry found for this position {}", board.get_hash());
            },
            Some(moves)=>{
                // choose a random move from the book
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..moves.len());
                let move_ = moves[index];
                println!("book move found {:#?}", move_);
                let mut s = String::new();
                s.push(opening::FILE_NAMES[move_.move_.end_file() as usize]);
                s.push(opening::RANK_NAMES[move_.move_.end_row() as usize]);
                s.push(opening::FILE_NAMES[move_.move_.start_file() as usize]);
                s.push(opening::RANK_NAMES[move_.move_.start_row() as usize]);
                // println!("{}", s);
                match move_.move_.promotion_piece() {
                    Some(opening::PromotionPiece::Knight) => s.push('n'),
                    Some(opening::PromotionPiece::Bishop) => s.push('b'),
                    Some(opening::PromotionPiece::Rook) => s.push('r'),
                    Some(opening::PromotionPiece::Queen) => s.push('q'),
                    None => {}
                }
                
                return ChessMove::from_str(s.as_str()).unwrap();


            }
        }
    }
    
    match choose_move(board, depth, remaining_time).0{
        None=>{return play_random_move(board).expect("error_board has no moves")},
        Some(chess_move)=>{
            return chess_move;
        }
    }
}


fn play_game(starting_position: Option<String>, verbose: bool, bot_white: bool, depth: u32, terminate_after: Option<u32>) -> i32{
    
    let both = true;
    let mut game = chess::Game::new();
    let mut book_moves = 6;
    match starting_position{
        None=>{},
        Some(pos)=>{
            match chess::Game::from_str(&pos).ok(){
                None=>{},
                Some(g)=>{
                    game = g
                }
            }
        }   
    }
    let mut move_count = 0;
    while game.result().is_none()
    {
        move_count += 1;
        let board = game.current_position();
        let now = Instant::now();
        match board.side_to_move(){
            chess::Color::White=>{
                if verbose{
                    println!("White");
                }
                if bot_white{
                    game.make_move(play_bot_move(board, depth, book_moves, 4*60*1000));
                }else{
                    if both{
                        game.make_move(play_bot_move(board, depth, book_moves, 4*60*1000));
                    }else{
                        game.make_move(play_random_move(board).expect("error_board has no moves"));
                    }

                }
            }
            chess::Color::Black=>{
                if verbose{
                    println!("Black")
                }
                if !bot_white{
                    game.make_move(play_bot_move(board, depth, book_moves, 4*60*1000));
                }else{
                    if both{
                        game.make_move(play_bot_move(board, depth, book_moves, 4*60*1000));
                    } else{
                        game.make_move(play_random_move(board).expect("error_board has no moves"));
                    }


                }
            }
        }
        if book_moves > 0 {
            book_moves -= 1;
        }    
        if verbose { 
            // print!("\x1B[2J\x1B[1;1H");
            let elapsed = now.elapsed();
            
            println!("time per move: {:.2?}",  elapsed);
            println!("{}",&game.current_position());
            utils::print_board(game.current_position().to_string());
        }

        if let Some(a) = terminate_after{
            if a < move_count{
                break;
            }
        }
        
    }
    if verbose{
        match game.result(){
            None=>{},
            Some(result)=>{
                match result{
                    chess::GameResult::BlackCheckmates=>{
                        println!("black won!")
                    },
                    chess::GameResult::WhiteCheckmates=>{
                        println!("white won!")
                    },
                    chess::GameResult::WhiteResigns=>{
                        println!("black won!")
                    },
                    chess::GameResult::BlackResigns=>{
                        println!("white won!")
                    },
                    chess::GameResult::Stalemate=>{
                        println!("stalemate!")
                    },
                    chess::GameResult::DrawAccepted=>{
                        println!("draw accepted!")
                    },
                    chess::GameResult::DrawDeclared=>{
                        println!("draw declared!")
                    }
                }
            }
            
        }
    }

    let mut count = 0;
    for action in game.actions(){
        
        match action{
            chess::Action::MakeMove(m)=>{
                if verbose{
                print!("{} ", m);
                }
                count += 1
            },
            chess::Action::AcceptDraw=>{},
            chess::Action::DeclareDraw=>{},
            chess::Action::OfferDraw(_)=>{},
            chess::Action::Resign(_)=>{},
        }

    }
    return count;
}


fn test_match(){
    let now = Instant::now();
    // 6k1/1p3pp1/p7/8/r2n4/8/3K4/7q b - - 0 1
    // 4rrk1/1pp2ppp/p7/3n4/8/P7/1P3PPP/R1BR2K1 w - - 0 1 =====> test not getting to mate!!!
    // test hard end game -> 8/p3k3/Pp4p1/1P4P1/4K3/8/8/8 w - - 0 1
    // medium endgame Some("8/p3k2p/Pp4p1/1Pn3P1/3RK3/8/8/8 w - - 0 1".to_string())
    let turns = play_game(None, true, false, 0, Some(200));
    let elapsed = now.elapsed();
            
    println!("avarage time per move: {:.2?}",  elapsed / (turns) as u32);
}


fn handle_uci(){
    let mut game = Game::new(); 
    let mut book_moves = 10;
    let mut buffer=String::new();
    let _=io::stdout().flush();
    io::stdin().read_line(&mut buffer).expect("Did not enter a correct string");
    if let Some('\n')=buffer.chars().next_back() {
        buffer.pop();
    }
    if let Some('\r')=buffer.chars().next_back() {
        buffer.pop();
    }
    while buffer != "" {
        
        if buffer == "uci"{
            // print!("heyyyy");
            io::stdout().write((format!("id name {} \n", constants::NAME)).as_bytes()).ok();
            io::stdout().write((format!("id auther {} \n", constants::NAME)).as_bytes()).ok();
            io::stdout().write(("uciok\n").as_bytes()).ok();

            
        }else if buffer == "quit"{
            io::stdout().write(("Bye Bye!\n").as_bytes()).ok();
            
        }else if buffer == "isready"{
            io::stdout().write(("readyok\n").as_bytes()).ok();
        }else if buffer == "ucinewgame"{
            game = Game::new();
            book_moves = 10;
        }
        if buffer.starts_with("position "){
            let parts: Vec<&str> = buffer.split(" ").collect(); 
            if parts[1] == "startpos"{
                game = Game::new();
                book_moves = 10;
                for chess_move in &parts[2..]{
                    if chess_move != &"moves"{
                    game.make_move(ChessMove::from_str(chess_move).ok().expect("illigal move"));
                    if book_moves > 0{ book_moves -= 1}
                    }
                }

            }else {
                let mut fen = String::new();
                let mut idx: usize = 1;
                while parts.len() > idx && parts[idx] != "moves"{
                    if parts[idx] != "fen"{
                        if idx!=2{
                            fen.push_str(" ");
                        }
                        fen.push_str(parts[idx]);
                    }
                    idx += 1;
                }
                
                game = Game::new_with_board(Board::from_str(&fen).expect(&format!("failed to load pos {}", fen)[..]));
                
                for chess_move in &parts[idx..]{
                    if chess_move != &"moves"{
                    game.make_move(ChessMove::from_str(chess_move).ok().expect("illigal move"));
                    }
                }
            }

        }else if buffer.starts_with("go "){
            let tokens: Vec<&str> = buffer.strip_prefix("go ").expect("not going to happen...").split(" ").collect();
            let mut current_token: &str;
            let mut idx:usize = 0;
            let mut wtime: u128 = 100000000000;
            let mut btime: u128 = 100000000000; 
            let mut max_depth: u32 = 0;
            while tokens.len() > idx{
                current_token = tokens[idx];
                if current_token == "wtime"{
                    idx += 1;
                    wtime = FromStr::from_str(tokens[idx]).unwrap();
                }else if current_token == "btime"{
                    idx += 1;
                    btime = FromStr::from_str(tokens[idx]).unwrap();
                }else if current_token == "depth"{
                    idx += 1;
                    max_depth = FromStr::from_str(tokens[idx]).unwrap();
                }
                idx += 1;
            }
            

            let board = game.current_position();
            
            let remaining_time:u128;
            match board.side_to_move(){
                chess::Color::White=>{
                    remaining_time = wtime;
                },
                chess::Color::Black=>{
                    remaining_time = btime;
                }
            }
            
            let chess_move = play_bot_move(board , max_depth, 10, remaining_time);
            io::stdout().write(format!("bestmove {}\n", chess_move).as_bytes()).ok();
            game.make_move(chess_move);
            if book_moves> 0 {book_moves -= 1;}
        }
        buffer.clear();
        let _=io::stdout().flush();
        io::stdin().read_line(&mut buffer).expect("Did not enter a correct string");
        if let Some('\n')=buffer.chars().next_back() {
            buffer.pop();
        }
        if let Some('\r')=buffer.chars().next_back() {
            buffer.pop();
        }
    }
}


fn run_test_position(position: &str, remaining_time: u128){
    let now = Instant::now();
    let test = Board::from_str(position).ok().expect("invalid position");
    let chess_move = choose_move(test, 10, remaining_time);
    let elapsed = now.elapsed();
    println!("{}, {:?}", chess_move.0.expect("msg"), chess_move.1);
    println!("time to complete {:?}", elapsed);
}



fn check_eval(position: &str, pawn_table: &mut chess::CacheTable<i32>){
    let board = Board::from_str(position).ok().expect("msg"); 
    let now = Instant::now();
    let eval = evaluation::evaluate(&board);
    let elapsed = now.elapsed();
    println!("{} - {:?}", eval, elapsed);
    
    let mut info = search::SearchInfo::new();
    let now = Instant::now();
    
    let eval = evaluation::evaluate_rework(&board) + search::pawn_table_lookup(&board, pawn_table, &mut info);
    let elapsed = now.elapsed();
    println!("{} - {:?}", eval, elapsed);
}

fn main() {
    let debug = false;
    // //let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").ok().expect("invalid position");
    // //println!("{:x}",board.get_hash());
    // //let board = Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").ok().expect("invalid position");
    // // print the board hash in hexadecimal
    // //println!("{:x1}",board.get_hash() );
    


    if debug{
        // 2r5/2q4k/8/8/2K5/8/8/8 w - - 0 1
        // 8/8/8/5k2/8/8/K4Q2/5R2 b - - 0 1
        let b = &Board::from_str("k7/7Q/8/8/8/8/7q/1K6 w - - 0 1").unwrap();
        let score = evaluation::evaluate_rework(b);
        println!("{}", score);
        // // let b = Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").ok().expect("invalid position");
        // // opening::write_entrys();

        // // check how many bytes were written
        // // let mut buffer = vec![0; 80];
        // let file = std::fs::File::open("test.bin").unwrap(); 
        // // file.read_to_end(&mut buffer).unwrap();
        // // println!("{:?}", buffer);
        

        // let book = opening::read_polyglot_book(file).unwrap();
        // // print the first few entrys of the book
        // println!("{:#?}", &book.iter().take(1).collect::<Vec<_>>());

        // match book.get(&b.get_hash()) {
        //     None=>{
        //         println!("no book entry found for this position {}", b.get_hash());
        //     },
        //     Some(moves)=>{
        //         // choose a random move from the book
        //         let mut rng = rand::thread_rng();
        //         let index = rng.gen_range(0..moves.len());
        //         let move_ = moves[index];
        //         println!("book move found {:#?}", move_);
        //         let mut s = String::new();
        //         s.push(opening::FILE_NAMES[move_.move_.end_file() as usize]);
        //         s.push(opening::RANK_NAMES[move_.move_.end_row() as usize]);
        //         s.push(opening::FILE_NAMES[move_.move_.start_file() as usize]);
        //         s.push(opening::RANK_NAMES[move_.move_.start_row() as usize]);

        //         // println!("{}", s);
        //         match move_.move_.promotion_piece() {
        //             Some(opening::PromotionPiece::Knight) => s.push('n'),
        //             Some(opening::PromotionPiece::Bishop) => s.push('b'),
        //             Some(opening::PromotionPiece::Rook) => s.push('r'),
        //             Some(opening::PromotionPiece::Queen) => s.push('q'),
        //             None => {}
        //         }
                
        //         println!("{:?}", ChessMove::from_str(s.as_str()).unwrap());


        //     }
        // }

        
        








        // let mut s=String::new();
        // while s != "exit" {
        //     print!("Please enter some text: ");
        //     let _=stdout().flush();
        //     stdin().read_line(&mut s).expect("Did not enter a correct string");
        //     if let Some('\n')=s.chars().next_back() {
        //         s.pop();
        //     }
        //     if let Some('\r')=s.chars().next_back() {
        //         s.pop();
        //     }
        //     run_test_position(s.as_str(), 10*60*1000)
        // };
        
        // let mut pawn_table = chess::CacheTable::new(65536,  0);
        // check_eval("rnbqkb1r/ppp1pppp/5n2/3p4/3P4/2N5/PPP1PPPP/R1BQKBNR w KQkq - 1 3", &mut pawn_table);
        // check_eval("rnbqkbnr/ppp1pppp/8/3p4/3P4/2N5/PPP1PPPP/R1BQKBNR b KQkq - 0 2", &mut pawn_table);
        // check_eval("rnbqkb1r/ppp1pppp/5n2/3p4/3P4/2N5/PPP1PPPP/R1BQKBNR w KQkq - 1 3", &mut pawn_table);
        // check_eval("rnbqkbnr/ppp1pppp/8/3p4/3P4/2N5/PPP1PPPP/R1BQKBNR b KQkq - 0 2", &mut pawn_table);
        // check_eval("rnbqkb1r/ppp1pppp/5n2/3p4/3P4/2N5/PPP1PPPP/R1BQKBNR w KQkq - 1 3", &mut pawn_table);


        // run_test_position("rnbqkb1r/p3pppp/1p6/2ppP3/3N4/2P5/PPP1QPPP/R1B1KB1R w KQkq -", 10*60*1000);
        // run_test_position("8/8/7p/3KNN1k/2p4p/8/3P2p1/8 w - -", 10*60*1000);
        // run_test_position("6k1/1p3ppp/4b3/2p4q/3p4/P2PB1QP/2P3PK/8 b - - 0 26", 10*60*1000);
        
        // test_match()
    }else{
        handle_uci();
    }
    
}

#[cfg(test)]
mod test{
    use super::*;
    
    struct TestPositon{
        pos: String,
        mate_in: u32,
        mating_side_white: bool 
    }
    
    fn run_mate(test_pos: TestPositon){
        let now = Instant::now();

            let count = play_game(Some(test_pos.pos.to_string()), false, test_pos.mating_side_white, 9, Some(10));
            let elapsed = now.elapsed();
            let turns = (count as f32/2 as f32).ceil();
            println!("mate found in {} move, should be in {}, Elapsed time: {:.2?}", turns, test_pos.mate_in, elapsed);
            assert_eq!(turns, test_pos.mate_in as f32);
            

    }


    #[test]
    fn test_mate_black_2(){
    
        run_mate(TestPositon{ pos: "8/4K3/2b5/3kp3/8/8/1n6/b4r2 w - - 1 10".to_string(),mate_in: 2, mating_side_white: false})
    }

    #[test]
    fn test_mate_black_3(){
        run_mate(TestPositon{pos: "6k1/1p3pp1/p7/8/r2n4/8/3K4/7q b - - 0 1".to_string(), mate_in: 3, mating_side_white: false })
    }

    #[test]
    fn test_mate_white_2(){
        run_mate(TestPositon{ pos: "b1B3Q1/5K2/5NP1/n7/2p2k1P/3pN2R/1B1P4/4qn2 w - - 0 1".to_string(),mate_in: 2, mating_side_white: true})
    }
    #[test]
    fn test_mate_white_3(){
        run_mate(TestPositon{ pos: "1k6/1P5Q/8/7B/8/5K2/8/8 w - - 0 1".to_string(),mate_in: 3, mating_side_white: true});
    }
    #[test]
    fn test_mate_white_2_(){
        run_mate(TestPositon{pos: "6k1/1p3ppp/4b3/2p4q/8/P2Pp1QP/2P3PK/8 w - - 0 26 ".to_string(), mate_in: 2, mating_side_white: true})
    }

    fn test_better_evaluation(position: &str){
        let board = Board::from_str(position).ok().expect("msg"); 
        let now = Instant::now();
        let eval = evaluation::evaluate(&board);
        let elapsed = now.elapsed();
        println!("{} - {:?}", eval, elapsed);

        let now = Instant::now();
        let eval_other = evaluation::evaluate_rework(&board);
        let elapsed = now.elapsed();
        println!("{} - {:?}", eval_other, elapsed);

        assert_eq!(eval_other, eval);
    }



    #[test]
    fn test_better_evaluation_startpos(){
        test_better_evaluation("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }
    #[test]
    fn test_opening_better_eval(){
        test_better_evaluation("rnbqkb1r/ppp1pppp/3p1n2/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 1 3")
    }
    #[test]
    fn test_opening_better_eval2(){
        test_better_evaluation("rn1qkb1r/ppp1pppp/8/3p1bB1/3Pn3/3Q1N2/PPP1PPPP/RN2KB1R w KQkq - 4 5")
    }
}