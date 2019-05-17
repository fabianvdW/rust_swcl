use crate::logging::Logger;
use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use crate::game_state::{GameState, GameColor};
use crate::search::{Search, TimeControl};


pub enum FieldState {
    Empty,
    Obstructed,
    Red,
    Blue,
}

pub fn go() {
    let mut search = Search::new(TimeControl::MoveTime(1700));
    let log = Logger::new("client_log.txt", false);
    log.log("Finished initializing!\n", false);
    let args: Vec<String> = env::args().collect();
    log.log(&format!("Arguments: {:?}\n", args), false);
    let mut host = "localhost";
    let mut port = 13050;
    let mut reservation = "";
    let mut index: usize = 0;
    while index < args.len() {
        match &args[index][..] {
            "-r" | "--reservation" => {
                reservation = &args[index + 1][..];
                index += 2;
            }
            "-h" | "--host" => {
                host = &args[index + 1][..];
                index += 2;
            }
            "-p" | "--port" => {
                port = args[index + 1].parse::<i64>().unwrap();
                index += 2;
            }
            _ => { index += 1; }
        }
    }
    let mut my_gamestate = GameState::standard();

    let mut stream = TcpStream::connect(&format!("{}:{}", host, port)).expect("Could not connect!");
    write_to_stream(&mut stream, &log, "<protocol>");
    write_to_stream(&mut stream, &log, &format!("<joinPrepared reservationCode=\"{}\"/>", reservation));
    let mut current_parsing = String::new();
    loop {
        let mut buf = [0; 2048];
        let bytes = stream.read(&mut buf).expect("Could not read from stream!");
        let mut line = std::str::from_utf8(&buf[0..bytes]).expect("Could not convert to line");
        current_parsing.push_str(line);
        //log.log(&format!("Read from Stream:\n{}\n", line), false);
        if line.contains("<data class=\"result\">") {
            break;
        }
        while current_parsing.contains("<state") && current_parsing.contains("</state>") {
            let x = current_parsing.clone();
            let halves: Vec<&str> = x.split("</state>").collect::<Vec<&str>>();
            current_parsing = String::from(halves[1]);
            let first_state = halves[0];
            let lines: Vec<&str> = first_state.split("\n").collect::<Vec<&str>>();
            let mut rote_fische = 0u128;
            let mut blaue_fische = 0u128;
            let mut kraken = 0u128;
            let mut plies_played = 0;
            let mut rounds_played = 0;
            let mut move_color = GameColor::Blue;
            for line in lines {
                //log.log(&format!("Parsing Line:\n{}\n", line), false);
                if line.contains("<state") {
                    let stripped = line.trim().replace("<state class=\"sc.plugin2019.GameState\" ", "").replace("startPlayerColor=\"RED\"", "").replace("\"", "").replace(">", "");
                    let args: Vec<&str> = stripped.split(" ").collect::<Vec<&str>>();
                    for arg in args {
                        if arg.contains("turn") {
                            plies_played = arg.split("=").collect::<Vec<&str>>()[1].parse::<u64>().unwrap();
                            rounds_played = plies_played / 2;
                        } else if arg.contains("currentPlayerColor") {
                            let color = arg.split("=").collect::<Vec<&str>>()[1];
                            if color == "RED" {
                                move_color = GameColor::Red;
                            } else if color == "BLUE" {
                                move_color = GameColor::Blue;
                            } else {
                                panic!("invalid color!");
                            }
                        }
                    }
                } else if line.contains("<field ") {
                    let stripped = line.trim().replace("<field ", "").replace("\"", "").replace("/>", "");
                    let args: Vec<&str> = stripped.split(" ").collect::<Vec<&str>>();
                    let x = 9 - args[0].split("=").collect::<Vec<&str>>()[1].parse::<u64>().unwrap();
                    let y = args[1].split("=").collect::<Vec<&str>>()[1].parse::<u64>().unwrap();
                    let field_desc = args[2].split("=").collect::<Vec<&str>>()[1];
                    let field_type =
                        if field_desc == "EMPTY" {
                            FieldState::Empty
                        } else if field_desc == "RED" {
                            FieldState::Red
                        } else if field_desc == "BLUE" {
                            FieldState::Blue
                        } else if field_desc == "OBSTRUCTED" {
                            FieldState::Obstructed
                        } else {
                            panic!("Invalid field desc!");
                        };
                    let index = 10 * y + x;
                    if let FieldState::Red = field_type {
                        rote_fische |= 1u128 << index;
                    } else if let FieldState::Blue = field_type {
                        blaue_fische |= 1u128 << index;
                    } else if let FieldState::Obstructed = field_type {
                        kraken |= 1u128 << index;
                    }
                }
            }
            let hash = GameState::calculate_hash(rote_fische, blaue_fische, kraken, &move_color);
            my_gamestate = GameState::new(rote_fische, blaue_fische, kraken, plies_played as u8, rounds_played as u8, move_color, hash);
            log.log("Succesfully read GameState!\n", false);
            log.log(&format!("FEN:\n{}\n", my_gamestate.to_fen()), false);
        }
        if current_parsing.contains("MoveRequest") {
            log.log("Got a move request!\n", false);
            let x = current_parsing.clone();
            let lines: Vec<&str> = x.split("\n").collect::<Vec<&str>>();
            let mut id = String::new();
            for line in lines {
                //log.log(&format!("Parsing line:\n{}\n", line), false);
                if line.contains("<room roomId=\"") {
                    let x = line.trim().replace("\t", "").replace(" ", "").replace(">", "").replace("\"", "").replace("<roomroomId=", "");
                    //log.log(&format!("\n{:?}\n", x), false);
                    id = format!("{}", x);
                }
            }
            current_parsing.clear();
            //Search
            let result = search.run(100, &mut my_gamestate);
            let mv = result.stack[0];
            let x = 9 - mv.from % 10;
            let y = mv.from / 10;
            let newx = 9 - mv.to % 10;
            let newy = mv.to / 10;
            let mut direction = "";
            if newx > x {
                if newy > y {
                    direction = "UP_RIGHT";
                } else if newy == y {
                    direction = "RIGHT";
                } else if newy < y {
                    direction = "DOWN_RIGHT";
                }
            } else if newx == x {
                if newy > y {
                    direction = "UP";
                } else if newy < y {
                    direction = "DOWN";
                } else {
                    direction = "INVALID";
                }
            } else if newx < x {
                if newy > y {
                    direction = "UP_LEFT";
                } else if newy == y {
                    direction = "LEFT";
                } else if newy < y {
                    direction = "DOWN_LEFT";
                }
            }
            let inner_statement = &format!("\t<data class=\"move\" x=\"{}\" y=\"{}\" direction=\"{}\"/>", x, y, direction);
            let statement = &format!("<room roomId=\"{}\">\n{}\n</room>", id, inner_statement);
            write_to_stream(&mut stream, &log, statement);
            log.log("Succesfully sent move!\n", false);
            log.log(&format!("Nodes analyzed: {}\n", search.nodes_analyzed), false);
            log.log(&format!("Searched to depth: {}\n", result.depth), false);
            log.log(&format!("Score: {}\n", result.score), false);
            log.log("PV:\n", false);
            for mv in result.stack {
                log.log(&format!("{}\n", mv), false);
            }
        }

        std::thread::sleep(Duration::from_millis(5));
    }
}

pub fn write_to_stream(stream: &mut TcpStream, log: &Logger, msg: &str) {
    stream.write(msg.as_bytes()).expect("Could not write to stream!");
    log.log(&format!("Wrote to stream:\n{}\n", msg), false);
}