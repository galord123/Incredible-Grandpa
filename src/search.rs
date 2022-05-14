use chess::{self, Board, ChessMove, Piece, Square, BitBoard, Color, Game, BoardStatus};
use crate::{evaluation, constants::{self, Access}, utils::get_piece_type};
use std::{time::Instant, io::{self, Write}, hash::{Hash, Hasher}};
use std::collections::hash_map::DefaultHasher;
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Nodetype {
    Pvnode,
    CutNode,
    AllNode
}

#[derive(Debug)]
pub struct SearchInfo{
    pub nodes_searched: u64,
    pub transpostions_used: u64,
    pub transpostions_recorded: u64,
    pub pawn_hash_table_used: u64,
    pub pawn_hash_table_recorded: u64
} 

impl SearchInfo{
    pub fn new() -> SearchInfo{
        SearchInfo{
            nodes_searched: 0,
            transpostions_used: 0,
            transpostions_recorded: 0,
            pawn_hash_table_used: 0,
            pawn_hash_table_recorded: 0
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct PawnKey{
    pub endgame: bool,
    pub white_pawns: BitBoard,
    pub black_pawns: BitBoard
}


#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Entry{
    pub node_type: Nodetype,
    pub depth: u32,
    pub score: i32,
}

pub trait ChessData<T> {
    fn make_new(&self, chess_move: ChessMove) -> Box<dyn ChessData<T>>;
    fn get_draw(&self) -> bool;
    fn state(&self) -> &Board;
    fn get_hash(&self) -> u64;
}

impl ChessData<Board> for Board{
    fn make_new(&self, chess_move: ChessMove) -> Box<dyn ChessData<Board>> {
        Box::new(self.make_move_new(chess_move))
    }
    fn state(&self) -> &Board{
        self
    }
    fn get_hash(&self) -> u64 {
        self.get_hash()
    }
    fn get_draw(&self) -> bool {
        if self.status() == BoardStatus::Stalemate{
            return true;
        }
        return false;
    }
}

pub fn search_depth(game: &chess::Game, depth: u32, sorted_moves: &Option<Vec<(ChessMove, i32)>>, max_time: u128, best_previous: ( Option<ChessMove>, i32), cachetable: &mut chess::CacheTable<Entry>, pawn_table: &mut chess::CacheTable<i32>) -> (Option<chess::ChessMove>, i32, Vec<(ChessMove, i32)>, SearchInfo, bool){
    let movegen = chess::MoveGen::new_legal(&game.current_position());
    let mut best_move:Option<chess::ChessMove> = None;
    let mut best_score = -9999;  
    let debug = false;
    let mut alpha = -1000000;
    let beta = 1000000;
    let mut pvline: Vec<ChessMove> = Vec::new();
    let mut line: Vec<ChessMove> = Vec::new();

    let mut table: Vec<(ChessMove, i32)> = Vec::new();
    // let mut cachetable = chess::CacheTable::new(65536,  (0, 0));

    let mut moves: Vec<ChessMove> = movegen.collect();
    if depth < 4{
        match sorted_moves {
            None=>{    
            },
            Some(moves_list)=>{
                let mut new_moves: Vec<ChessMove> = Vec::new(); 
                for chess_move in moves_list{
                    new_moves.push(chess_move.0);
                    if debug{
                        println!("{:?}", chess_move.1);
                    }

                }
                moves = new_moves;
            }
        }
    }
    else { 
        let mut new_moves: Vec<ChessMove> = Vec::new(); 
        // best move first
        if best_previous.0.is_some(){
            new_moves.push(best_previous.0.unwrap());
        }
        // remove the best move from moves list
        moves.retain(|&x| x != best_previous.0.unwrap());
        
        let board = game.current_position();
        // sort moves by score and then by capture and checks 
        moves.sort_by(|a, b| {
            let a_score = table.iter().find(|&x| x.0 == *a).unwrap_or(&(ChessMove::default(), 0)).1;
            let b_score = table.iter().find(|&x| x.0 == *b).unwrap_or(&(ChessMove::default(), 0)).1;
            
            if a_score > b_score{
                return Ordering::Less;
            }
            else if a_score < b_score{
                return Ordering::Greater;
            }
            else{
                if is_capture(&board, a) && !is_capture(&board, b){
                    return Ordering::Less;
                }
                else if !is_capture(&board, a) && is_capture(&board, b) {
                    return Ordering::Greater;
                }
                else if is_capture(&board, a) && is_capture(&board, b){
                    return Ordering::Equal;
                }else {
                    return Ordering::Equal;
                }
            }    
        
        });
        new_moves.append(&mut moves);
        moves = new_moves;
        
    }





    
    let mut time_spent = 0;
    let mut total_nodes = 0;
    let mut total_transpositions_recorded = 0;
    let mut total_transpositions_used = 0;
    let mut total_pawn_hash_table_used = 0;
    let mut total_pawn_hash_table_recorded = 0;
    let mut checked_previous_best_move = false;
    let mut bad_last_move = false;
    let best_previous_move = best_previous.0;
    let best_previous_score = best_previous.1;

    for chess_move in moves{
        let now = Instant::now();
        let mut passed_board = game.clone();
        passed_board.make_move(chess_move);// .make_move_new(chess_move); 
        
        let mut search_info = SearchInfo::new();
        let board_value = -pv_search(&passed_board, -beta, -alpha, depth, cachetable, &mut search_info, pawn_table, &mut line);
        
        
        if ! checked_previous_best_move && chess_move == best_previous_move.unwrap_or(ChessMove::default()){
            if best_previous_score > board_value + 100{
                bad_last_move = true;
            }
            checked_previous_best_move = true;
        }


        total_nodes += search_info.nodes_searched;
        total_transpositions_recorded += search_info.transpostions_recorded;
        total_transpositions_used += search_info.transpostions_used;
        total_pawn_hash_table_used += search_info.pawn_hash_table_used;
        total_pawn_hash_table_recorded += search_info.pawn_hash_table_recorded;

        if debug{
            println!("{} - {}",chess_move, board_value);
        }
        
        
        table.push((chess_move, board_value));

        if board_value > best_score{
            best_score = board_value;
            if board_value > alpha{
                best_move = Some(chess_move); 
                // pvline edit
                pvline.clear();
                pvline.push(chess_move);
                pvline.extend(&line);
                if alpha < beta{
                    alpha = board_value

                }
            }
        }
        

        if let Some(best_line) = best_move{
            let joined = pvline.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ");
            io::stdout().write((format!("info depth {} score cp {} pv {}\n",depth, best_score, joined)).as_bytes()).ok();

        }

        let elapsed = now.elapsed();
        time_spent += elapsed.as_millis();
        if !bad_last_move && time_spent > max_time{
            if debug{
            println!("ended on time in depth {} ", depth);
            println!("found {}, last {}", best_score, best_previous_score);
            }
            if best_score >= best_previous_score + 100{
                return (best_move, best_score, table, SearchInfo{nodes_searched: total_nodes, transpostions_recorded: total_transpositions_recorded, transpostions_used: total_transpositions_used, pawn_hash_table_recorded: total_pawn_hash_table_recorded, pawn_hash_table_used: total_pawn_hash_table_used}, true)
            }
            return (None, -9999, table, SearchInfo{nodes_searched: total_nodes, transpostions_recorded: total_transpositions_recorded, transpostions_used: total_transpositions_used, pawn_hash_table_recorded: total_pawn_hash_table_recorded, pawn_hash_table_used: total_pawn_hash_table_used}, true);
        }
    }
    
    

    table.sort_by_key(|x| -x.1);

    return (best_move, best_score, table, SearchInfo{nodes_searched: total_nodes, transpostions_recorded: total_transpositions_recorded, transpostions_used: total_transpositions_used, pawn_hash_table_recorded: total_pawn_hash_table_recorded, pawn_hash_table_used: total_pawn_hash_table_used}, false)
}

fn is_interesting(board: &Board, chess_move: ChessMove) -> bool{
    let move_is_capture = is_capture(board, &chess_move);
        let source = chess_move.get_source(); 
        let dest = chess_move.get_dest();
        let pawn_push = board.piece_on(source) == Some(Piece::Pawn);
        let black_pawn_push = pawn_push && board.color_on(source) == Some(chess::Color::Black) && dest <= Square::H3;
        let white_pawn_push = pawn_push &&board.color_on(source) == Some(chess::Color::White) && dest >= Square::A6;
        let is_castle = board.piece_on(source) == Some(Piece::King) && (dest == Square::G1 || dest == Square::C1 || dest == Square::G8 || dest == Square::C8);
        let board_after_move = board.make_move_new(chess_move);
        let interesting = board.checkers().popcnt() > 0 || move_is_capture || chess_move.get_promotion().is_some() 
        || board_after_move.checkers().popcnt() > 0 || black_pawn_push || white_pawn_push || is_castle;
        interesting
}

fn late_move_reduction(board: &Board, chess_move: ChessMove, depth: u32) -> u32{
    let mut red = 0;
    if depth >= 3{
        let interesting = is_interesting(board, chess_move);
        
        if !interesting{

            red = 1;
            if depth >= 5{
                red = depth / 4; //TODO: maybe to much?
            }
            
        }
        
    }
    return red;
}


pub fn pv_search<T>(data: &impl ChessData<T> ,alpha: i32, beta:i32, depth:u32, cache: &mut chess::CacheTable<Entry>, info: &mut SearchInfo, pawn_table: &mut chess::CacheTable<i32>, pvline: &mut Vec<ChessMove>) -> i32{
    let mut line:Vec<ChessMove> = Vec::new();
    let late_move_reduction_enabled = true;
    let null_pruning = true;
    let using_cache = false;
    info.nodes_searched += 1;
    let mut alpha = alpha;
    let beta = beta;

    // look for the position in the cache
    if using_cache{
        match cache.get(data.get_hash()){
            None=>{},
            Some(desc)=>{ 
                let entry = desc;
                match entry.node_type {
                    // if we had a high enough value to set the lower-bound.
                    Nodetype::AllNode=>{
                        alpha = entry.score;
                        info.transpostions_used += 1;
                    }
                    Nodetype::CutNode=>{
                        // if we have a lower enough value to set the upper-bound
                        // _beta = entry.score;
                        // info.transpostions_used +=1;
                    }
                    Nodetype::Pvnode=>{
                        // if we have the exact score then we'll return it.
                        if entry.depth >= depth{
                            // println!("used cache!!");
                            info.transpostions_used += 1;
                            return entry.score
                        }
                    }
                }
                
            }
        }
    }
    
    // if we reached the max depth then we'll return the score.
    if depth <= 0 { 
        pvline.clear();
        return quiesce(data, alpha, beta, 6, info, pawn_table);
    }


    let in_check = data.state().checkers().popcnt() > 0;
    
    // null move pruning 
    if null_pruning && depth >= 3 && !in_check{
        if let Some(passed_board) = data.state().null_move(){
            
            let score =  -pv_search(&passed_board,-beta, -beta + 1, depth - 2 - 1, cache, info, pawn_table, &mut line);
            if score >= beta{
                return beta;
            }
        }
    }

    let extend = 0;
    let mut fprune = false;
    let mut fmax = 9999;
    let mut razoring = false;
    

    /* decide about limited razoring at the pre-pre-frontier nodes */
    let board_balance = evaluation::material_balance(&data.state());
    let mut fscore = board_balance + constants::RAZORING_MARGIN;
    if !in_check && extend != 0 && depth == 3 && fscore <= alpha
        { fprune = true;  fmax = fscore; razoring = true; }
    /* decide about extended futility pruning at pre-frontier nodes */
    fscore = board_balance + constants::EXTENDED_FUTILITY_MARGIN;
    if !in_check && extend != 0 && depth == 2 && fscore <= alpha
        { fprune = true; fmax = fscore; }
    /* decide about selective futility pruning at frontier nodes */
    fscore = board_balance + constants::FUTILITY_MARGIN;
    if !in_check && depth == 1 && fscore <= alpha
        { fprune = true; fmax = fscore; }
 

    


        
    // if depth <= 3{
    //     // razor pruning
    //     let eval = evaluation::evaluate_rework(board) + pawn_table_lookup(board, pawn_table, info);
    //     if razoring_enabled && !in_check && eval < alpha - 348 - 258 * depth as i32* depth as i32
    //     {
    //         let value = quiesce(board, alpha -1, alpha, 6, info, pawn_table);
    //         if value < alpha{
    //             return value;
    //         }
    //     }


    //     // futility pruning
    //     let futility_margin =  -100;
    //     if depth == 1 && futility_pruning_enabled && !in_check{
    //         let score = quiesce(board, alpha -1, alpha, 1, info, pawn_table);
    //         if score + futility_margin < alpha{
    //             // save the score in the cache
    //             cache.add(board.get_hash(), Entry{ depth, node_type: Nodetype::CutNode, score: alpha });
    //             info.transpostions_recorded += 1;

    //             return alpha;
    //         }
    //     }

    // }

    
    
    
    let mut first_search_pv: bool  = true;

    let count = chess::MoveGen::new_legal(&data.state());

    // if this positon has no moves then its mate or a stalemate
    if count.count() == 0{
        if data.get_draw() {
            return 0;
        }
        match data.state().side_to_move(){
            chess::Color::White=>{
                // print!("white mated");
                return -9999 - depth as i32

            },
            chess::Color::Black=>{
                // print!("black mated");
                return -9999 - depth as i32

            }
        }
        
    }
    
    
    
    

    // //try to make a Futility pruning

    // //let mut retry = false;
    // //if futility_pruning_enabled && depth == 1 && board.checkers().popcnt() == 0{
    // //    let eval = evaluation::evaluate_rework(board) + pawn_table_lookup(board, pawn_table, info);
    // //    let movegen = chess::MoveGen::new_legal(board);
    // //    for chess_move in movegen{
    // //        if is_capture(board, &chess_move) || is_check(board, &chess_move){
    // //            let passed_board = board.make_move_new(chess_move);
    // //            let val =  evaluation::evaluate_rework(&passed_board) + pawn_table_lookup(&passed_board, pawn_table, info);
    // //            if val > _alpha{
    // //                retry = true;
    // //                break;
    // //            }
    // //        }
    // //    } 
    // //    if !retry && eval - constants::FUTOLITY_MARGIN < _alpha{
    // //        return _alpha;
    // //    } 
    // //}


    let movegen = chess::MoveGen::new_legal(&data.state());
    
    // sort the moves by checks and captures
    let mut moves: Vec<ChessMove> = Vec::new();
    for chess_move in movegen{
        if is_capture(&data.state(), &chess_move) || is_check(&data.state(), &chess_move){
            moves.push(chess_move);
        }
    }
    let movegen = chess::MoveGen::new_legal(&data.state());
    for chess_move in movegen{
        if !is_capture(&data.state(), &chess_move) && !is_check(&data.state(), &chess_move){
            moves.push(chess_move);
        }
    }

    // let futility_pruning = false;
    // let margin = -100;
    for chess_move in moves  {

        // // futility pruning in child nodes
        // if futility_pruning && depth == 1 && board.checkers().popcnt() == 0{
        //     let passed_board = board.make_move_new(chess_move);
        //     let eval = evaluation::evaluate_rework(&passed_board) + pawn_table_lookup(board, pawn_table, info);
        //     if eval + margin + gain(board, &chess_move) <= _alpha{
        //         continue;
        //     }
        // }
        
        let passed_board = data.make_new(chess_move);
        
        let mut score;
        // late move reduction
        let moves_to_reduce;
        if late_move_reduction_enabled{
            moves_to_reduce = late_move_reduction(&data.state(), chess_move, depth);
        }else{
            moves_to_reduce = 0;
        }

        if first_search_pv{
            score = -pv_search(&passed_board,-beta, -alpha, depth - 1, cache, info, pawn_table, &mut line);
        } else {
            if !fprune || is_check(&data.state(), &chess_move) || fmax + gain(&data.state(), &chess_move) > alpha{
                score = -zero_window_search(&passed_board, -alpha, depth - 1 - moves_to_reduce, info, cache, pawn_table);
                // in fail-soft ... && score < beta ) is common
                if  score > alpha {
                    score = -pv_search(&passed_board, -beta, -alpha, depth - 1, cache, info, pawn_table, &mut line); // re-search
                }

            }else{
                if razoring && is_interesting(&data.state(), chess_move){
                    score = -zero_window_search(&passed_board.as_ref(), -alpha, depth - 1 - moves_to_reduce, info, cache, pawn_table);
                    // in fail-soft ... && score < beta ) is common
                    if  score > alpha {
                        score = -pv_search(&passed_board.as_ref(), -beta, -alpha, depth - 1, cache, info, pawn_table, &mut line); // re-search
                    }
                }else{
                    score = quiesce(data, -beta, alpha, 6, info, pawn_table);
                }
                    
                
            }
        }
        // the move is un-made because we created a copy of the board.
        if score >= beta {
            cache.add(data.state().get_hash(), Entry{ depth, node_type: Nodetype::AllNode, score: alpha });
            info.transpostions_recorded += 1;
                return beta;   // fail-hard beta-cutoff
            }
        if score > alpha {
            alpha = score; // alpha acts like max in MiniMax

            //score is between alpha and beta so it is a pv-node
            cache.add(passed_board.get_hash(), Entry{ depth, node_type: Nodetype::Pvnode, score });
            pvline.clear();
            pvline.push(chess_move);
            pvline.extend(&line);
            // println!("{:?}", pvline);

            info.transpostions_recorded += 1;
            first_search_pv = false;   // *1)
        }
    }
    cache.add(data.get_hash(), Entry{ depth, node_type: Nodetype::CutNode, score: alpha });
    info.transpostions_recorded += 1;
    // cache.add(board.get_hash(), (_alpha, depth));
    return alpha;
 }
 
 // fail-hard zero window search, returns either beta-1 or beta
fn zero_window_search<T>(data: &impl ChessData<T>, beta:i32, depth: u32, info: &mut SearchInfo, cache: &mut chess::CacheTable<Entry>, pawn_table: &mut chess::CacheTable<i32>) -> i32 {
    // alpha == beta - 1
    // this is either a cut- or all-node
    let using_cache = false;
    if depth <= 0 { return quiesce(data, beta-1, beta, 6, info, pawn_table);}

    let mut _beta = beta;
    if using_cache{
        match cache.get(data.get_hash()){
            None=>{},
            Some(desc)=>{ 
                let entry = desc;
                match entry.node_type {
                    Nodetype::AllNode=>{
                        
                    }
                    Nodetype::CutNode=>{
                        _beta = entry.score;
                        info.transpostions_used += 1;
                    }
                    Nodetype::Pvnode=>{
                        if entry.depth >= depth{
                            // println!("used cache!!");
                            info.transpostions_used += 1;
                            return entry.score
                        }
                    }
                }
                
            }
        }
    }
    let movegen = chess::MoveGen::new_legal(&data.state());
    // sort the moves by checks and captures
    let mut moves: Vec<ChessMove> = Vec::new();
    for chess_move in movegen{
        if is_capture(&data.state(), &chess_move) || is_check(&data.state(), &chess_move){
            moves.push(chess_move);
        }
    }
    let movegen = chess::MoveGen::new_legal(&data.state());
    for chess_move in movegen{
        if !is_capture(&data.state(), &chess_move) && !is_check(&data.state(), &chess_move){
            moves.push(chess_move);
        }
    }

    for chess_move in moves {
        let mut passed_board = data.make_new(chess_move);
        let score = -zero_window_search(&passed_board, 1-_beta, depth - 1, info, cache, pawn_table);

        if score >= _beta {
            cache.add(data.get_hash(), Entry{ depth, node_type: Nodetype::AllNode, score: _beta });
            info.transpostions_recorded += 1;
            return _beta;   // fail-hard beta-cutoff
        }
    }
    cache.add(data.get_hash(), Entry{ depth, node_type: Nodetype::CutNode, score: _beta-1 });
    info.transpostions_recorded += 1;
    return _beta-1; // fail-hard, return alpha
 }


 fn quiesce<T>(data: &impl ChessData<T>, alpha: i32, beta:i32, depth:u32, info: &mut SearchInfo, pawn_table: &mut chess::CacheTable<i32>) -> i32{
    info.nodes_searched += 1;
    let board = data.state();
    let stand_pat = evaluation::evaluate_rework(game) + pawn_table_lookup(&board, pawn_table, info);

    let mut _alpha = alpha;
    if stand_pat >= beta{
        return beta;
    }

    let big_delta = 975; // queen value
    if stand_pat < alpha - big_delta {
        return alpha;
    }

    if _alpha < stand_pat{
        _alpha = stand_pat
    }

    if depth == 0{
        return stand_pat
    }
    let movegen = chess::MoveGen::new_legal(&board);

    for chess_move in movegen{

        if is_capture(&board, &chess_move) || is_check(&board, &chess_move){
            let mut board = data.make_new(chess_move);
            let score = -quiesce(&board, -beta, -_alpha, depth - 1, info, pawn_table);

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


fn is_check(board: &Board, chess_move: &ChessMove) -> bool{
    let new_board = board.make_move_new(*chess_move);
    if new_board.checkers().popcnt() > 0{
        return true;
    }
    return false;
}


fn is_capture(board: &Board, chess_move: &ChessMove) -> bool{
    let new_board = board.make_move_new(*chess_move);
    if board.combined().popcnt() > new_board.combined().popcnt(){
        return true;
    }

    return false;
}


fn gain(board: &Board, chess_move: &ChessMove) -> i32{
    let mut gain = 0;
    if is_capture(board, chess_move){
        let captured_piece = board.piece_on(chess_move.get_dest());
        if let Some(piece) = captured_piece{
            match piece{
                Piece::Pawn => gain = constants::PAWN_VAL.access_endgame(false),
                Piece::Knight => gain = constants::KNIGHT_VAL.access_endgame(false),
                Piece::Bishop => gain = constants::BISHOP_VAL.access_endgame(false),
                Piece::Rook => gain = constants::ROOK_VAL.access_endgame(false),
                Piece::Queen => gain = constants::QUEEN_VAL.access_endgame(false),
                Piece::King => gain = 900,
            } 
        }
    }
    // adjust the gain for the side to move
    if board.side_to_move() == Color::White{
        gain = -gain;
    }
    return gain;
}


pub fn pawn_table_lookup(board: &Board, pawn_table: &mut chess::CacheTable<i32>, info: &mut SearchInfo) -> i32{
    let mut score = 0;
    let black_pawns = get_piece_type(board, Piece::Pawn,chess::Color::Black);
    let white_pawns = get_piece_type(board, Piece::Pawn,chess::Color::White);
    
    // get endgame score
    let pieces_total_count = board.pieces(chess::Piece::Bishop).popcnt() + board.pieces(chess::Piece::Knight).popcnt() + board.pieces(chess::Piece::Rook).popcnt();
    let endgame = board.pieces(chess::Piece::Queen).popcnt() == 0 && pieces_total_count <= 2;
    
    // get the pawn hash key
    let mut hasher = DefaultHasher::new();
    PawnKey{endgame, black_pawns, white_pawns}.hash(&mut hasher);
    let key = hasher.finish();
    
    // get pawn scores for both sides
    match pawn_table.get(key) {
        None =>{
            

            let structure_score = evaluation::evaluate_pawn_structure(black_pawns, white_pawns, endgame);
            // save to pawn table
            pawn_table.add(key, structure_score);
            score += structure_score;
            info.pawn_hash_table_recorded += 1;

        },
        Some(desc) =>{
            score += desc;
            info.pawn_hash_table_used += 1;
        }
    }
    match board.side_to_move(){
        chess::Color::White=>{
            return score
        },
        chess::Color::Black=>{
            return -score
        }
    }
    
}
