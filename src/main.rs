pub mod constants;
pub mod evaluation;
pub mod utils;
pub mod opening;
pub mod bitboard_operators;

use std::{io::{self, Write}};
use evaluation::evaluate;
use chess::{self, ChessMove, Board, Game};

use rand::Rng;
use std::{time::Instant, str::FromStr};


fn is_capture(board: &Board, chess_move: &ChessMove) -> bool{
    let new_board = board.make_move_new(*chess_move);
    if board.combined().count() > new_board.combined().count(){
        return true;
    }

    return false;
}


fn quiesce(board: &Board, alpha: i32, beta:i32, depth:u32) -> i32{
    let stand_pat = evaluate(&board);
    let mut _alpha = alpha;
    if stand_pat >= beta{
        return beta;
    }

    if _alpha < stand_pat{
        _alpha = stand_pat
    }

    if depth == 0{
        return stand_pat
    }
    let movegen = chess::MoveGen::new_legal(&board);

    for chess_move in movegen{

        if is_capture(&board, &chess_move){
            let board = board.make_move_new(chess_move);
            let score = -quiesce(&board, -beta, -_alpha, depth - 1);

            if score >= beta{
                return beta
            }
            if score > _alpha{
                _alpha = score
            }
        }
    }
    return _alpha

}


fn alphabeta(board: &chess::Board, depth: u32, alpha: i32, beta: i32, cache: &mut chess::CacheTable<(i32, u32)>) -> i32{
    
    match cache.get(board.get_hash()){
        None=>{},
        Some(desc)=>{ 
            let (eval, depth_s) = desc;
            if depth_s >= depth{
                // println!("used cache!!");
                return eval
            }
        }
    }

    if depth == 0{
        let eval = quiesce(board, alpha, beta, 6);
        //cache.add(board.get_hash(), eval);
        return eval;
    }
    
    let movegen = chess::MoveGen::new_legal(&board);
    let count = chess::MoveGen::new_legal(&board);
    let mut best_score = -9999;  
    let mut _alpha = alpha;

    
    if count.count() == 0{
        if board.status() == chess::BoardStatus::Stalemate{
            return 0;
        }
        match board.side_to_move(){
            chess::Color::White=>{
                // print!("white");
                return best_score - depth as i32

            },
            chess::Color::Black=>{
                return best_score - depth as i32

            }
        }
        
    }

    for chess_move in movegen{
        let passed_board = board.make_move_new(chess_move); 
        
        let score = -alphabeta(&passed_board, depth-1, -beta, -_alpha, cache);
        
        if score >= beta{
            cache.add(passed_board.get_hash(), (score, depth));
            return score;
        }
        if score > best_score{
            best_score = score
        }
        if score > alpha{
            _alpha = score
        }
    }

    return best_score
}


fn search_depth(board: &Board, depth: u32, sorted_moves: &Option<Vec<(ChessMove, i32)>>, max_time: u128) -> (Option<chess::ChessMove>, Vec<(ChessMove, i32)>){
    let movegen = chess::MoveGen::new_legal(&board);
    let mut best_move:Option<chess::ChessMove> = None;
    let mut best_score = -9999;  
    let debug = false;
    let mut alpha = -100000;
    let beta = 100000;

    let mut table: Vec<(ChessMove, i32)> = Vec::new();
    let mut cachetable = chess::CacheTable::new(65536,  (0, 0));


    let mut moves: Vec<ChessMove> = movegen.collect();

    match sorted_moves {
        None=>{    
        },
        Some(moves_list)=>{
            let mut new_moves: Vec<ChessMove> = Vec::new(); 
            for chess_move in moves_list{
                new_moves.push(chess_move.0);

            }
            moves = new_moves;
        }
    }
    let mut time_spent = 0;
    for chess_move in moves{
        let now = Instant::now();
        let passed_board = board.make_move_new(chess_move); 
        let board_value = -alphabeta(&passed_board, depth, -beta, -alpha, &mut cachetable);
        if debug{
            println!("{} - {}",chess_move, board_value);
        }
        if let Some(best_line) = best_move{
            io::stdout().write((format!("info depth {} score cp {} pv {}\n",depth, board_value, best_line)).as_bytes()).ok();

        }
        
        table.push((chess_move, board_value));

        if board_value > best_score{
            best_score = board_value;
            best_move = Some(chess_move); 
        }
        if board_value > alpha{
            alpha = board_value
        }
        let elapsed = now.elapsed();
        time_spent += elapsed.as_millis();
        if time_spent > max_time{
            return (None, table);
        }
    }
    
    

    table.sort_by_key(|x| -x.1);

    return (best_move, table)
}


fn choose_move(board: chess::Board, depth: u32, remaining_time: u128) ->Option<chess::ChessMove>{
    let mut table: Option<Vec<(ChessMove, i32)>> = None;
    let mut best_move:Option<chess::ChessMove> = None;
    let allowed_time =  remaining_time / 30; 
    let mut total_time = 0; 
    
    for _depth in 0..depth{
        let now = Instant::now();
        let result = search_depth(&board, _depth, &table, 5000);
        if result.0.is_none(){
            return best_move
        }
        best_move = result.0;
        table = Some(result.1);  
        let elapsed = now.elapsed();
        total_time += elapsed.as_millis(); 

    }
    let mut _depth = depth;
    while total_time < allowed_time{
        let now = Instant::now();
        let time_left = allowed_time - total_time.min(allowed_time);
        let result = search_depth(&board, _depth, &table, time_left);
        if result.0.is_none(){
            return best_move
        }
        best_move = result.0;
        _depth += 1; 
        table = Some(result.1);  
        let elapsed = now.elapsed();
        total_time += elapsed.as_millis(); 
    }

    if let Some(chess_move) = best_move {

        println!("best move found {} ", chess_move);
    }
    
    return best_move
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
        match opening::get_opening_move("C:\\Users\\משתמש\\Documents\\projects\\RustChess\\src\\openings.txt".to_string(), &board){
            None=>{},
            Some(m)=>{
                return m;
            }
        }
    }
    
    match choose_move(board, depth, remaining_time){
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
                    game.make_move(play_bot_move(board, depth, book_moves, 60000));
                }else{
                    if both{
                        game.make_move(play_bot_move(board, depth, book_moves, 60000));
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
                    game.make_move(play_bot_move(board, depth, book_moves, 60000));
                }else{
                    if both{
                        game.make_move(play_bot_move(board, depth, book_moves, 60000));
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
    let turns = play_game(None, true, false, 5, Some(200));
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
            let mut wtime: u128 = 0; 
            let mut btime: u128 = 0; 
            while tokens.len() > idx{
                current_token = tokens[idx];
                if current_token == "wtime"{
                    idx += 1;
                    wtime = FromStr::from_str(tokens[idx]).unwrap();
                }else if current_token == "btime"{
                    idx += 1;
                    btime = FromStr::from_str(tokens[idx]).unwrap();
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
            
            let chess_move = play_bot_move(board , 5, 10, remaining_time);
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


fn main() {
    let debug = false;
    if debug{
        test_match()
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

            let count = play_game(Some(test_pos.pos.to_string()), false, test_pos.mating_side_white, 5, Some(50));
            let elapsed = now.elapsed();
            let turns = (count as f32/2 as f32).ceil();
            assert_eq!(turns, test_pos.mate_in as f32);
            
            println!("mate found in {} move, should be in {}, Elapsed time: {:.2?}", turns, test_pos.mate_in, elapsed);

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
    

}