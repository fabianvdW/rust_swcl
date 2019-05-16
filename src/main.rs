mod game_state;
mod generate_u128_nums;
mod string_to_game_state_converter;
mod game_logic;
mod constants;

use std::time::Instant;
use self::game_state::{GameState, GameStatus, GameColor, GameMove};

extern crate rand;

use rand::Rng;

fn main() {
    let now = Instant::now();
    //let mut state = string_to_game_state_converter::string_to_game_state(string_to_game_state_converter::STANDARD_GAME_STATE,0,0,GameColor::Red);
    //println!("{}",perft_div(&mut state,5));
    play_rand_games();
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
        while !matches(&curr_state) {
            curr_state = game_logic::make_move(&curr_state, &moves[rand::thread_rng().gen_range(0, moves.len())]);
            let moves = game_logic::get_possible_moves(&curr_state, &curr_state.move_color);
            curr_state.analyze(&moves);
        }
    }
}

fn matches(gs: &GameState) -> bool {
    match &gs.game_status {
        Some(x) => {
            match x {
                GameStatus::RedWin | GameStatus::BlueWin | GameStatus::Draw => {
                    true
                }
                _ => false,
            }
        }
        None => {
            panic!("This should not happen!");
        }
    }
}

fn perft_div(g: &mut GameState, depth: u8) -> u64 {
    if depth == 0u8 {
        return 1u64;
    }
    let moves = game_logic::get_possible_moves(&g, &g.move_color);
    g.analyze(&moves);
    if matches(g) {
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
    if matches(g) {
        return 1;
    }
    let mut count: u64 = 0u64;
    for i in &moves {
        count += perft(&mut game_logic::make_move(g, i), depth - 1);
    }
    count
}

