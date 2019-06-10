use super::game_state::{GameMove, GameState, GameStatus};
use super::game_logic::{get_possible_moves, make_move, get_schwarm, make_null_move};
use super::board_rating::{Schwarm, rating};
use std::time::Instant;
use crate::game_state::GameColor;
use crate::game_logic::get_schwarm_board;
use crate::constants;


pub const CACHE_MASK: i64 = 16 * 2097152 - 1;


pub struct PrincipialVariation {
    pub stack: Vec<GameMove>,
    pub hash_stack: Vec<i64>,
    pub score: f64,
    pub depth: usize,
}

impl PrincipialVariation {
    pub fn new(depth_left: usize) -> PrincipialVariation {
        PrincipialVariation {
            stack: Vec::with_capacity(depth_left),
            hash_stack: Vec::with_capacity(depth_left),
            score: -1000000.0,
            depth: depth_left,
        }
    }
}

pub enum TimeControl {
    Infinite,
    MoveTime(u64),
}

impl TimeControl {
    pub fn time_over(&self, time_spent: u64) -> bool {
        if let TimeControl::Infinite = self {
            return false;
        } else if let TimeControl::MoveTime(time) = self {
            return time_spent > *time;
        } else {
            panic!("Invalid TC");
        }
    }
}

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub hash: i64,
    pub score: f64,
    pub plies_played: u8,
    pub depth: u8,
    pub gm: GameMove,
    pub pv_node: bool,
    pub beta_node: bool,
    pub alpha_node: bool,
}

impl CacheEntry {
    pub fn new(hash: i64, score: f64, plies_played: u8, depth: u8, gm: GameMove, pv_node: bool, beta_node: bool, alpha_node: bool) -> CacheEntry {
        CacheEntry {
            hash,
            score,
            plies_played,
            depth,
            gm,
            pv_node,
            beta_node,
            alpha_node,
        }
    }
}

pub struct Search {
    pub stop: bool,
    pub tc: TimeControl,
    pub cache: Vec<Option<CacheEntry>>,
    pub killer_moves: [[Option<GameMove>; 3]; 100],
    pub hh_score: [[usize; 100]; 100],
    pub bf_score: [[usize; 100]; 100],
    pub nodes_analyzed: usize,
}

impl Search {
    pub fn new(tc: TimeControl) -> Search {
        Search {
            stop: false,
            tc,
            cache: vec![None; 16 * 2097152],
            killer_moves: [[None; 3]; 100],
            hh_score: [[8; 100]; 100],
            bf_score: [[8; 100]; 100],
            nodes_analyzed: 0,
        }
    }

    pub fn run(&mut self, depth: u8, game_state: &mut GameState) -> PrincipialVariation {
        //Reset killers and trim history scores
        self.nodes_analyzed = 0;
        self.killer_moves = [[None; 3]; 100];
        for i in 0..100 {
            for j in 0..100 {
                self.hh_score[i][j] /= 8;
                self.bf_score[i][j] /= 8;
            }
        }
        let mut best_pv: Option<PrincipialVariation> = None;
        let time = Instant::now();
        for d in 1..depth + 1 {
            let pv = alpha_beta(self, -100000.0, 100000.0, game_state, d, 0, if let GameColor::Red = game_state.move_color { 1 } else { -1 }, &time);
            if self.stop {
                break;
            }
            //Delete current best pv out of tt
            if let Some(last_pv) = best_pv {
                for i in 0..last_pv.stack.len() {
                    let mut entry = self.cache[(last_pv.hash_stack[i] & CACHE_MASK) as usize];
                    if let Some(content) = &mut entry {
                        content.pv_node = false;
                    } else {
                        panic!("Last pv was not in cache");
                    }
                }
            }
            /*println!("Depth {}", d);
            println!("Score {}", pv.score);
            println!("PV: ");
            for mv in &pv.stack {
                println!("{}\n-----", mv);
            }*/
            //Place pv in tt
            for i in 0..pv.stack.len() {
                self.cache[(pv.hash_stack[i] & CACHE_MASK) as usize] = Some(CacheEntry::new(pv.hash_stack[i], pv.score * if (i % 2) == 0 { 1.0 } else { -1.0 }, game_state.plies_played + 1, d - i as u8, pv.stack[i].clone(), true, false, false));
            }
            best_pv = Some(pv);
        }
        best_pv.unwrap()
    }
}

pub fn alpha_beta(search: &mut Search, mut alpha: f64, mut beta: f64, game_state: &mut GameState, mut depth_left: u8, current_depth: u8, maximizing_player: isize, start_time: &Instant) -> PrincipialVariation {
    if (search.nodes_analyzed % 1024) == 0 {
        search.stop = checkup(&start_time, &search.tc)
    }
    let original_alpha = alpha;
    let mut curr_pv = PrincipialVariation::new(depth_left as usize);
    if search.stop {
        return curr_pv;
    }

    search.nodes_analyzed += 1;

    //TODO experimental
    if depth_left == 0 && match game_state.move_color {
        GameColor::Red => false,
        GameColor::Blue => true
    } && is_one_fish_missing(game_state.blaue_fische) {
        depth_left += 1;
    }

    //Use early-return, if we are in d0 nodes, as we only need to know if there is one legal move
    let mut move_list = get_possible_moves(&game_state, &game_state.move_color, depth_left == 0);
    game_state.analyze(&move_list);
    //Early leafs
    if game_state.game_over() {
        if let Some(GameStatus::Draw) = game_state.game_status {
            curr_pv.score = 0.0;
            return curr_pv;
        } else if let Some(GameStatus::RedWin) = game_state.game_status {
            let diff = (Schwarm::berechne_schwaerme(game_state.rote_fische).1).size as isize - (Schwarm::berechne_schwaerme(game_state.blaue_fische).1).size as isize;
            curr_pv.score = maximizing_player as f64 * (30000.0 - game_state.plies_played as f64 + diff as f64 / 100.0);
            return curr_pv;
        } else if let Some(GameStatus::BlueWin) = game_state.game_status {
            let diff = (Schwarm::berechne_schwaerme(game_state.blaue_fische).1).size as isize - (Schwarm::berechne_schwaerme(game_state.rote_fische).1).size as isize;
            curr_pv.score = maximizing_player as f64 * (-30000.0 + game_state.plies_played as f64 - diff as f64 / 100.0);
            return curr_pv;
        } else {
            panic!("Invalid game over situation");
        }
    }
    //Probe TB
    let mut move_ordering_index = 0;
    let mut id_pv_move_found = false;
    {
        let ce: Option<CacheEntry> = search.cache[(game_state.hash & CACHE_MASK) as usize];
        if let Some(content) = ce {
            if content.hash == game_state.hash {
                if depth_left == 0 {
                    if content.depth == 0 {
                        curr_pv.score = content.score;
                        return curr_pv;
                    }
                } else {
                    //Cache-Hit
                    if content.depth >= depth_left && !(game_state.plies_played + depth_left >= 60 && content.plies_played + content.depth < 60) {
                        if !content.beta_node && !content.alpha_node {
                            curr_pv.stack.push(content.gm.clone());
                            curr_pv.hash_stack.push(content.hash);
                            curr_pv.score = content.score;
                            return curr_pv;
                        } else {
                            if content.beta_node {
                                if content.score > alpha {
                                    alpha = content.score;
                                }
                            } else if content.alpha_node {
                                if content.score < beta {
                                    beta = content.score;
                                }
                            }
                        }
                        if alpha >= beta {
                            curr_pv.score = alpha;
                            curr_pv.stack.push(content.gm.clone());
                            curr_pv.hash_stack.push(content.hash);
                            return curr_pv;
                        }
                    }
                    if content.depth != 0 {
                        //Move ordering
                        move_ordering_index = 1;
                        let mut index = 256;
                        for (i, mv) in move_list.iter().enumerate() {
                            if mv.from == content.gm.from && mv.to == content.gm.to {
                                index = i;
                                break;
                            }
                        }

                        id_pv_move_found = content.pv_node;
                        let mv0 = move_list[0];
                        move_list[0] = move_list[index];
                        move_list[index] = mv0;
                    }
                }
            }
        }
    }

    //Search ends
    if depth_left == 0 {
        curr_pv.score = rating(&game_state, false) * maximizing_player as f64;
        let content = search.cache[(game_state.hash & CACHE_MASK) as usize];
        if let None = content {
            let new_entry = CacheEntry::new(game_state.hash, curr_pv.score, 0, 0, GameMove::new(101, 101), false, false, false);
            search.cache[(game_state.hash & CACHE_MASK) as usize] = Some(new_entry);
        } else if content.unwrap().depth == 0 {
            let new_entry = CacheEntry::new(game_state.hash, curr_pv.score, 0, 0, GameMove::new(101, 101), false, false, false);
            search.cache[(game_state.hash & CACHE_MASK) as usize] = Some(new_entry);
        }
        return curr_pv;
    }

    let not_in_check = match game_state.move_color {
        GameColor::Red => true,
        GameColor::Blue => false
    } || get_schwarm(&game_state, &GameColor::Red) < game_state.rote_fische.count_ones() as u8;

    //Null Move Pruning
    if !id_pv_move_found && depth_left > 3 && current_depth > 0 && depth_left + game_state.plies_played < 60 && not_in_check {
        let rat = -alpha_beta(search, -beta, -beta + 0.0001, &mut make_null_move(&game_state), depth_left - 3, current_depth + 1, -maximizing_player, &start_time).score;
        if rat >= beta {
            curr_pv.score = rat;
            return curr_pv;
        }
    }

    //Killer heuristic
    {
        for i in move_ordering_index..move_list.len() {
            let mv = move_list[i];

            if let Some(other) = search.killer_moves[current_depth as usize][0] {
                if other == mv {
                    let mv_moi = move_list[move_ordering_index];
                    move_list[move_ordering_index] = mv;
                    move_list[i] = mv_moi;
                    move_ordering_index += 1;
                    continue;
                }
            }
            if let Some(other) = search.killer_moves[current_depth as usize][1] {
                if other == mv {
                    let mv_moi = move_list[move_ordering_index];
                    move_list[move_ordering_index] = mv;
                    move_list[i] = mv_moi;
                    move_ordering_index += 1;
                    continue;
                }
            }
            if let Some(other) = search.killer_moves[current_depth as usize][2] {
                if other == mv {
                    let mv_moi = move_list[move_ordering_index];
                    move_list[move_ordering_index] = mv;
                    move_list[i] = mv_moi;
                    move_ordering_index += 1;
                    continue;
                }
            }
            if current_depth >= 2 {
                if let Some(other) = search.killer_moves[(current_depth - 2) as usize][0] {
                    if other == mv {
                        let mv_moi = move_list[move_ordering_index];
                        move_list[move_ordering_index] = mv;
                        move_list[i] = mv_moi;
                        move_ordering_index += 1;
                        continue;
                    }
                }
                if let Some(other) = search.killer_moves[(current_depth - 2) as usize][1] {
                    if other == mv {
                        let mv_moi = move_list[move_ordering_index];
                        move_list[move_ordering_index] = mv;
                        move_list[i] = mv_moi;
                        move_ordering_index += 1;
                        continue;
                    }
                }
                if let Some(other) = search.killer_moves[(current_depth - 2) as usize][2] {
                    if other == mv {
                        let mv_moi = move_list[move_ordering_index];
                        move_list[move_ordering_index] = mv;
                        move_list[i] = mv_moi;
                        move_ordering_index += 1;
                        continue;
                    }
                }
            }
        }
    }

    //History heuristic
    let mut norm_score = 1.0;
    let mut ratings: Vec<f64> = vec![0.0; move_list.len()];
    for i in 0..move_list.len() {
        ratings[i] = search.hh_score[move_list[i].from as usize][move_list[i].to as usize] as f64 / search.bf_score[move_list[i].from as usize][move_list[i].to as usize] as f64;
        if ratings[i] > norm_score {
            norm_score = ratings[i];
        }
    }
    //Normalize score and add distance to mid
    for i in 0..move_list.len() {
        ratings[i] /= 0.3333*norm_score;
        ratings[i] += constants::DISTANCE_TO_MID[move_list[i].from as usize] - constants::DISTANCE_TO_MID[move_list[i].to as usize];
    }
    //Sort array
    /*for i in move_ordering_index..move_list.len() - 1 {
        for j in move_ordering_index..move_list.len() - 1 - i {
            if ratings[j] < ratings[j + 1] {
                let curr = ratings[j];
                ratings[j] = ratings[j + 1];
                ratings[j + 1] = curr;
                let curr = move_list[j];
                move_list[j] = move_list[j + 1];
                move_list[j + 1] = curr;
            }
        }
    }*/

    for i in 0..move_list.len() {
        //println!("{}", move_list[i]);
        if i >= move_ordering_index {
            sort_next_move(i, &mut move_list, &mut ratings);
        }
        let mut next_state = make_move(&game_state, &move_list[i]);
        let mut following_pv: PrincipialVariation;
        if depth_left <= 2 || !id_pv_move_found || i == 0 {
            following_pv = alpha_beta(search, -beta, -alpha, &mut next_state, depth_left - 1, current_depth + 1, -maximizing_player, &start_time);
        } else {
            following_pv = alpha_beta(search, -alpha - 0.00001, -alpha, &mut next_state, depth_left - 1, current_depth + 1, -maximizing_player, &start_time);
            let rat = following_pv.score * -1.0;
            if rat >= alpha && rat <= beta {
                following_pv = alpha_beta(search, -beta, -alpha, &mut next_state, depth_left - 1, current_depth + 1, -maximizing_player, &start_time);
            }
        }
        let rat = following_pv.score * -1.0;
        if rat > curr_pv.score {
            curr_pv.stack.clear();
            curr_pv.hash_stack.clear();
            curr_pv.stack.push(move_list[i].clone());
            curr_pv.hash_stack.push(game_state.hash);
            curr_pv.stack.append(&mut following_pv.stack);
            curr_pv.hash_stack.append(&mut following_pv.hash_stack);
            curr_pv.score = rat;
        }
        if curr_pv.score > alpha {
            alpha = curr_pv.score;
        }
        if alpha >= beta {

            //Place in Killer Heuristics
            if search.killer_moves[current_depth as usize][0] == None {
                search.killer_moves[current_depth as usize][0] = Some(move_list[i].clone());
            } else if search.killer_moves[current_depth as usize][1] == None {
                search.killer_moves[current_depth as usize][1] = Some(move_list[i].clone());
            } else if search.killer_moves[current_depth as usize][2] == None {
                search.killer_moves[current_depth as usize][2] = Some(move_list[i].clone());
            } else {
                //Check that it is not already in
                if !is_in_heuristics(&move_list[i], search, current_depth) {
                    //Place in heuristics then
                    search.killer_moves[current_depth as usize][2] = search.killer_moves[current_depth as usize][1];
                    search.killer_moves[current_depth as usize][1] = search.killer_moves[current_depth as usize][0];
                    search.killer_moves[current_depth as usize][0] = Some(move_list[i].clone());
                }
            }
            search.hh_score[move_list[i].from as usize][move_list[i].to as usize] += depth_left as usize;
            break;
        } else {
            search.bf_score[move_list[i].from as usize][move_list[i].to as usize] += depth_left as usize;
        }
    }
    //Make cache entry
    {
        let beta_node = curr_pv.score >= beta;
        let alpha_node = curr_pv.score <= original_alpha;

        let cache_index = (game_state.hash & CACHE_MASK) as usize;
        if let None = search.cache[cache_index] {
            search.cache[cache_index] = Some(CacheEntry::new(game_state.hash, curr_pv.score, game_state.plies_played as u8, depth_left, curr_pv.stack[0].clone(), false, beta_node, alpha_node));
        } else if let Some(other) = search.cache[cache_index] {
            if !other.pv_node && other.depth as isize - (game_state.plies_played as isize - other.plies_played as isize) <= depth_left as isize {
                search.cache[cache_index] = Some(CacheEntry::new(game_state.hash, curr_pv.score, game_state.plies_played as u8, depth_left, curr_pv.stack[0].clone(), false, beta_node, alpha_node));
            }
        }
    }
    curr_pv
}

pub fn is_in_heuristics(mv: &GameMove, search: &mut Search, depth: u8) -> bool {
    if let Some(other) = search.killer_moves[depth as usize][0] {
        if other == *mv {
            return true;
        }
    }
    if let Some(other) = search.killer_moves[depth as usize][1] {
        if other == *mv {
            return true;
        }
    }
    if let Some(other) = search.killer_moves[depth as usize][2] {
        if other == *mv {
            return true;
        }
    }
    false
}

pub fn checkup(start_time: &Instant, tc: &TimeControl) -> bool {
    let now = Instant::now();
    let dur = now.duration_since(*start_time).as_millis();
    tc.time_over(dur as u64)
}

pub fn is_one_fish_missing(fische: u128) -> bool {
    let fisch_anz = fische.count_ones() as usize;
    let board_one = get_schwarm_board(fische);
    let board_fische = board_one.count_ones() as usize;
    board_fische >= fisch_anz - 1 || (board_fische == 1 && get_schwarm_board(fische & !board_one).count_ones() as usize == fisch_anz - 1)
}

pub fn sort_next_move(to_position: usize, from_array: &mut Vec<GameMove>, ratings_array: &mut Vec<f64>) {
    let mut max_index = to_position;
    let mut max_val = ratings_array[max_index];
    for i in max_index + 1..from_array.len() {
        if ratings_array[i] > max_val {
            max_val = ratings_array[i];
            max_index = i;
        }
    }
    //Swap max_index with to_position
    let saved = from_array[to_position];
    let saved_rating = ratings_array[to_position];
    from_array[to_position] = from_array[max_index];
    ratings_array[to_position] = ratings_array[max_index];
    from_array[max_index] = saved;
    ratings_array[max_index] = saved_rating;
}