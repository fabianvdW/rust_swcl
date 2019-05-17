use crate::search::{Search, TimeControl};
use crate::game_logic::{get_possible_moves, make_move};
use crate::game_state::GameState;
use std::env;
use crate::logging::Logger;

pub fn go() {
    let mut search = Search::new(TimeControl::MoveTime(1700));
    let mut my_state = GameState::standard();
    let args: Vec<String> = env::args().collect();
    let path_to_log = args[1].clone();
    let log = Logger::new(&format!("{}.txt", path_to_log), false);
    log.log("Started program!\n", false);
    println!("ready");
    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        if arg.len() == 0 {
            continue;
        }
        if arg[0] == "newgame" {
            log.log("Started new game\n", false);
            let l0 = GameState::my_u64(arg[1].parse::<i64>().unwrap());
            let l1 = GameState::my_u64(arg[2].parse::<i64>().unwrap());
            let kraken = ((l0 as u128) << 64) | (l1 as u128);
            my_state = GameState::standard_with_kraken(kraken);
        } else if arg[0] == "requestmove" {
            let pv = search.run(100, &mut my_state);
            let res = pv.stack[0];
            let mate_found = pv.score < -29000.0 || pv.score > 29000.0;
            println!("{} {} {}", res.from, res.to, mate_found);
            log.log(&format!("sent {} {}\n", res.from, res.to), false);
            log.log(&format!("Searched to depth: {}\n", pv.depth), false);
            log.log(&format!("Search result:  {}\n", pv.score), false);
            log.log(&format!("Nodes examined:  {}\n", search.nodes_analyzed), false);
        } else if arg[0] == "makemove" {
            let from = arg[1].parse::<u64>().unwrap();
            let to = arg[2].parse::<u64>().unwrap();
            let move_list = get_possible_moves(&my_state, &my_state.move_color);
            let mut found = false;
            for mv in move_list {
                if mv.from == from as u8 && mv.to == to as u8 {
                    my_state = make_move(&my_state, &mv);
                    found = true;
                    break;
                }
            }
            if !found {
                log.log("Move was not found in legal move list!\n", false);
                break;
            }
            log.log(&format!("FEN:\n{}\n", my_state.to_fen()), false);
        } else if arg[0] == "end" {
            break;
        }
    }
}