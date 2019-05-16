mod game_state;
mod generate_u128_nums;
mod string_to_game_state_converter;
mod game_logic;
mod constants;
mod zobrist;
mod board_rating;
mod search;

use std::time::Instant;
use self::game_state::{GameState, GameStatus, GameColor, GameMove};

extern crate rand;
extern crate colored;

use rand::Rng;
use crate::search::{Search, TimeControl};

fn main() {
    //let mut state = string_to_game_state_converter::string_to_game_state(string_to_game_state_converter::STANDARD_GAME_STATE,0,0,GameColor::Red);
    //println!("{}",perft_div(&mut state,5));
    let mut state = GameState::from_fen("8 180179238186385408 10241 18023332108566528 0 72075186223972352 r 50 25");
    //println!("{}", state.to_fen());
    //println!("{}", board_rating::rating(&state, true));
    let mut s = Search::new(TimeControl::Infinite);
    let now = Instant::now();
    println!("Score {}",s.run(7,&mut state).score);
    println!("{}",s.nodes_analyzed);
    //play_rand_games();
    let new_now = Instant::now();
    println!("Time: {}ms", new_now.duration_since(now).as_secs() * 1000 + new_now.duration_since(now).subsec_millis() as u64);
}

fn play_rand_games() {
    let mut curr_state: GameState;
    let mut moves: Vec<GameMove>;
    for i in 0..100000 {
        curr_state = GameState::standard();
        moves = game_logic::get_possible_moves(&curr_state, &curr_state.move_color);
        curr_state.analyze(&moves);
        while !curr_state.game_over() {
            curr_state = game_logic::make_move(&curr_state, &moves[rand::thread_rng().gen_range(0, moves.len())]);
            let moves = game_logic::get_possible_moves(&curr_state, &curr_state.move_color);
            curr_state.analyze(&moves);
        }
    }
}

fn perft_div(g: &mut GameState, depth: u8) -> u64 {
    if depth == 0u8 {
        return 1u64;
    }
    let moves = game_logic::get_possible_moves(&g, &g.move_color);
    g.analyze(&moves);
    if g.game_over() {
        return 1;
    }
    let mut count: u64 = 0u64;
    for i in &moves {
        let z = perft(&mut game_logic::make_move(g, i), depth - 1);
        println!("{}:{}", i, z);
        count += z;
    }
    count
}

fn perft(g: &mut GameState, depth: u8) -> u64 {
    if depth == 0u8 {
        return 1u64;
    }
    let moves = game_logic::get_possible_moves(&g, &g.move_color);
    g.analyze(&moves);
    if g.game_over() {
        return 1;
    }
    let mut count: u64 = 0u64;
    for i in &moves {
        count += perft(&mut game_logic::make_move(g, i), depth - 1);
    }
    count
}

