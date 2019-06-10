pub mod game_state;
pub mod generate_u128_nums;
pub mod string_to_game_state_converter;
pub mod game_logic;
pub mod constants;
pub mod zobrist;
pub mod board_rating;
pub mod search;
pub mod localtesting;
pub mod online;
pub mod logging;

use self::game_state::{GameState, GameMove};

extern crate rand;
extern crate colored;

use rand::Rng;

fn main() {
    //localtesting::protocol::go();
    online::protocol::go();
}

fn play_rand_games() {
    let mut curr_state: GameState;
    let mut moves: Vec<GameMove>;
    for _ in 0..100000 {
        curr_state = GameState::standard();
        moves = game_logic::get_possible_moves(&curr_state, &curr_state.move_color, false);
        curr_state.analyze(&moves);
        while !curr_state.game_over() {
            curr_state = game_logic::make_move(&curr_state, &moves[rand::thread_rng().gen_range(0, moves.len())]);
            let moves = game_logic::get_possible_moves(&curr_state, &curr_state.move_color, false);
            curr_state.analyze(&moves);
        }
    }
}

fn perft_div(g: &mut GameState, depth: u8) -> u64 {
    if depth == 0u8 {
        return 1u64;
    }
    let moves = game_logic::get_possible_moves(&g, &g.move_color, false);
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
    let moves = game_logic::get_possible_moves(&g, &g.move_color, false);
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

