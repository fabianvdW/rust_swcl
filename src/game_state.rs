use std::fmt::{self, Formatter, Display};
use super::game_logic;

extern crate rand;

use rand::Rng;
use colored::*;

pub const DIRECTIONS: [i8; 8] = [10, 11, 1, -9, -10, -11, -1, 9];

#[allow(dead_code)]
pub enum GameColor {
    Red,
    Blue,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum GameStatus {
    Ingame,
    Draw,
    BlueWin,
    RedWin,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        (match *self {
            GameStatus::Ingame => {
                write!(f, "Ingame")
            }
            GameStatus::Draw => {
                write!(f, "Draw")
            }
            GameStatus::RedWin => {
                write!(f, "RedWin")
            }
            GameStatus::BlueWin => {
                write!(f, "BlueWin")
            }
        })
    }
}

pub struct GameMove {
    pub from: u8,
    pub to: u8,
}

impl Display for GameMove {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({xone},{yone}) --> ({xtwo},{ytwo})", xone = (9 - self.from % 10), yone = (9 - self.from / 10), xtwo = (9 - self.to % 10), ytwo = (9 - self.to / 10))
    }
}

impl GameMove {
    pub fn new(from: u8, to: u8) -> GameMove {
        GameMove { from, to }
    }
}

pub struct GameState {
    pub rote_fische: u128,
    pub blaue_fische: u128,
    pub kraken: u128,

    pub plies_played: u8,
    pub rounds_played: u8,

    pub game_status: Option<GameStatus>,
    pub move_color: GameColor,
}

impl GameState {
    pub fn analyze(&mut self, possible_moves: &Vec<GameMove>) {
        if self.plies_played % 2 == 0 {
            let rote_fische: u8 = self.rote_fische.count_ones() as u8;
            let roter_schwarm = game_logic::get_schwarm(&self, &GameColor::Red);
            let blaue_fische: u8 = self.blaue_fische.count_ones() as u8;
            let blauer_schwarm = game_logic::get_schwarm(&self, &GameColor::Blue);
            if roter_schwarm == rote_fische && blauer_schwarm == blaue_fische {
                if rote_fische > blaue_fische {
                    self.game_status = Some(GameStatus::RedWin);
                    return;
                } else if blaue_fische > rote_fische {
                    self.game_status = Some(GameStatus::BlueWin);
                    return;
                } else {
                    self.game_status = Some(GameStatus::Draw);
                    return;
                }
            } else if roter_schwarm == rote_fische {
                self.game_status = Some(GameStatus::RedWin);
                return;
            } else if blauer_schwarm == blaue_fische {
                self.game_status = Some(GameStatus::BlueWin);
                return;
            }
        }
        if self.rounds_played == 30 {
            //Ermittle größten Schwarm
            let mut rot_biggest_schwarm = 0;
            let mut blau_biggest_schwarm = 0;
            let mut rote_fische = self.rote_fische;
            while rote_fische != 0u128 {
                let schwarm = game_logic::get_schwarm_board(rote_fische);
                let schwarm_count = schwarm.count_ones() as usize;
                rot_biggest_schwarm = rot_biggest_schwarm.max(schwarm_count);
                rote_fische &= !schwarm;
            }
            let mut blaue_fische: u128 = self.blaue_fische;
            while blaue_fische != 0u128 {
                let schwarm = game_logic::get_schwarm_board(blaue_fische);
                let schwarm_count = schwarm.count_ones() as usize;
                blau_biggest_schwarm = blau_biggest_schwarm.max(blau_biggest_schwarm);
                blaue_fische &= !schwarm;
            }
            if rot_biggest_schwarm > blau_biggest_schwarm {
                self.game_status = Some(GameStatus::RedWin);
            } else if blau_biggest_schwarm > rot_biggest_schwarm {
                self.game_status = Some(GameStatus::BlueWin);
            } else {
                self.game_status = Some(GameStatus::Draw);
            }
            return;
        }
        if possible_moves.len() == 0 {
            match self.move_color {
                GameColor::Red => {
                    self.game_status = Some(GameStatus::BlueWin);
                    return;
                }
                GameColor::Blue => {
                    self.game_status = Some(GameStatus::RedWin);
                    return;
                }
            }
        }
        self.game_status = Some(GameStatus::Ingame);
    }

    pub fn game_over(&self) -> bool {
        match &self.game_status {
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

    pub fn from_fen(fen: &str)->Self {
        let arr: Vec<&str> = fen.split(" ").collect();
        let rote_fische = (arr[0].parse::<u128>().unwrap() << 64) | (arr[1].parse::<u128>().unwrap());
        let blaue_fische = (arr[2].parse::<u128>().unwrap() << 64) | (arr[3].parse::<u128>().unwrap());
        let kraken = (arr[4].parse::<u128>().unwrap() << 64) | (arr[5].parse::<u128>().unwrap());
        let move_color:GameColor;
        if arr[6].eq_ignore_ascii_case("r"){
            move_color=GameColor::Red;
        }else{
            move_color=GameColor::Blue;
        }
        let plies_played=arr[7].parse::<u8>().unwrap();
        let rounds_played=arr[8].parse::<u8>().unwrap();
        //TODO Hash
        GameState::new(rote_fische,blaue_fische,kraken,plies_played,rounds_played,move_color)
    }

    pub fn new(rote_fische: u128, blaue_fische: u128, kraken: u128, plies_played: u8, rounds_played: u8, move_color: GameColor) -> GameState {
        GameState {
            rote_fische,
            blaue_fische,
            kraken,
            plies_played,
            rounds_played,
            game_status: None,
            move_color,
        }
    }
    pub fn standard() -> GameState {
        GameState::standard_with_kraken(GameState::generate_random_kraken())
    }
    pub fn standard_with_kraken(kraken: u128) -> GameState {
        GameState {
            rote_fische: 0x20180601806018060180400u128,
            blaue_fische: 0x7f800000000000000000001feu128,
            kraken,
            plies_played: 0,
            rounds_played: 0,
            move_color: GameColor::Red,
            game_status: Some(GameStatus::Ingame),
        }
    }
    pub fn generate_random_kraken() -> u128 {
        let mut pos1: i8 = GameState::get_random_kraken_position();
        while pos1 % 10 == 9 || pos1 % 10 == 8 || pos1 % 10 == 1 || pos1 % 10 == 0 {
            pos1 = GameState::get_random_kraken_position();
        }
        let mut pos2: i8 = GameState::get_random_kraken_position();
        let mut diff: i8 = pos1 - pos2;
        while pos2 % 10 == 9 || pos2 % 10 == 8 || pos2 % 10 == 1 || pos2 % 10 == 0 || diff % 10 == 0 || pos1 / 10 == pos2 / 10 || diff % 11 == 0 || diff % 9 == 0 {
            pos2 = GameState::get_random_kraken_position();
            diff = pos1 - pos2;
        }
        return 1u128 << pos1 | 1u128 << pos2;
    }
    pub fn get_random_kraken_position() -> i8 {
        rand::thread_rng().gen_range(22, 78)
    }

    #[allow(dead_code)]
    pub fn to_string_u128(num: u128) -> String {
        let mut res_str: String = String::new();
        for y in 0..10 {
            res_str.push_str("|");
            for x in 0..10 {
                let shift: u128 = 99 - (y * 10 + x);
                res_str.push_str(&format!("\t{}", shift));
                if ((num >> shift) & 1u128) != 0u128 {
                    res_str.push_str("X");
                } else {
                    res_str.push_str(" ");
                }
                res_str.push_str("\t|");
            }
            res_str.push_str("\n");
        }
        res_str
    }
}

impl Display for GameState {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut res_str: String = String::new();
        for y in 0..10 {
            res_str.push_str("|");
            for x in 0..10 {
                res_str.push_str("\t");
                let shift: u128 = 99 - (y * 10 + x);
                if ((&self.rote_fische >> shift) & 1u128) != 0u128 {
                    res_str.push_str(&format!("{}","🐟".red()));
                } else if ((&self.blaue_fische >> shift) & 1u128) != 0u128 {
                    res_str.push_str(&format!("{}","🐟".blue()));
                } else if ((&self.kraken >> shift) & 1u128) != 0u128 {
                    res_str.push_str(&format!("{}","🐙".green()));
                }
                res_str.push_str("\t|");
            }
            res_str.push_str("\n");
        }
        res_str.push_str(&format!("Rounds played: {}\n", self.rounds_played));
        res_str.push_str(&format!("Plies played: {}", self.plies_played));
        write!(formatter, "{}", res_str)
    }
}