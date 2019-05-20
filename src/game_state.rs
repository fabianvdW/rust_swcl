use std::fmt::{self, Formatter, Display};
use super::game_logic;
use super::zobrist;

extern crate rand;

use rand::Rng;
use colored::*;

pub const DIRECTIONS: [i8; 8] = [10, 11, 1, -9, -10, -11, -1, 9];

#[allow(dead_code)]
#[derive(PartialEq)]
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

#[derive(Copy, Clone, PartialEq)]
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
    pub hash: i64,
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
                blau_biggest_schwarm = blau_biggest_schwarm.max(schwarm_count);
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

    pub fn calculate_hash(rote_fische: u128, blaue_fische: u128, kraken: u128, move_color: &GameColor) -> i64 {
        let mut hash = 0;
        for y in 0..10 {
            for x in 0..10 {
                let shift = 10 * y + x;
                if ((rote_fische >> shift) & 1) != 0u128 {
                    hash ^= zobrist::ZOBRIST_KEYS[y][x][0];
                } else if ((blaue_fische >> shift) & 1) != 0u128 {
                    hash ^= zobrist::ZOBRIST_KEYS[y][x][1];
                } else if ((kraken >> shift) & 1) != 0u128 {
                    hash ^= zobrist::ZOBRIST_KEYS[y][x][2];
                }
            }
        }
        if let GameColor::Blue = move_color {
            hash ^= zobrist::SIDE_TO_MOVE_IS_BLUE;
        }
        hash
    }

    pub fn my_u64(myi64: i64) -> u64 {
        let mut res = 0u64;
        for i in 0..64 {
            res |= (((myi64 >> i) & 1i64) as u64) << i;
        }
        res
    }

    pub fn my_i64(myu64: u64) -> i64 {
        let mut res = 0i64;
        for i in 0..64 {
            res |= (((myu64 >> i) & 1u64) as i64) << i;
        }
        res
    }

    pub fn from_fen(fen: &str) -> Self {
        let arr: Vec<&str> = fen.split(" ").collect();
        let rlinks = arr[0].parse::<i64>().unwrap();
        let rrechts = arr[1].parse::<i64>().unwrap();
        let blinks = arr[2].parse::<i64>().unwrap();
        let brechts = arr[3].parse::<i64>().unwrap();
        let klinks = arr[4].parse::<i64>().unwrap();
        let krechts = arr[5].parse::<i64>().unwrap();
        let rote_fische = ((GameState::my_u64(rlinks) as u128) << 64) | (GameState::my_u64(rrechts) as u128);
        let blaue_fische = ((GameState::my_u64(blinks) as u128) << 64) | (GameState::my_u64(brechts) as u128);
        let kraken = ((GameState::my_u64(klinks) as u128) << 64) | (GameState::my_u64(krechts) as u128);
        let move_color: GameColor;
        if arr[6].eq_ignore_ascii_case("r") {
            move_color = GameColor::Red;
        } else {
            move_color = GameColor::Blue;
        }
        let plies_played = arr[7].parse::<u8>().unwrap();
        let rounds_played = arr[8].parse::<u8>().unwrap();
        let hash = GameState::calculate_hash(rote_fische, blaue_fische, kraken, &move_color);
        GameState::new(rote_fische, blaue_fische, kraken, plies_played, rounds_played, move_color, hash)
    }

    pub fn to_fen(&self) -> String {
        let mut res_str = String::new();
        let rlinks = GameState::my_i64((self.rote_fische >> 64) as u64);
        let rrechts = GameState::my_i64(self.rote_fische as u64);
        let blinks = GameState::my_i64((self.blaue_fische >> 64) as u64);
        let brechts = GameState::my_i64(self.blaue_fische as u64);
        let klinks = GameState::my_i64((self.kraken >> 64) as u64);
        let krechts = GameState::my_i64(self.kraken as u64);
        let mc = if let GameColor::Red = self.move_color { "r" } else { "b" };
        res_str.push_str(&format!("{} {} {} {} {} {} {} {} {}", rlinks, rrechts, blinks, brechts, klinks, krechts, mc, self.plies_played, self.rounds_played));
        res_str
    }

    pub fn new(rote_fische: u128, blaue_fische: u128, kraken: u128, plies_played: u8, rounds_played: u8, move_color: GameColor, hash: i64) -> GameState {
        GameState {
            rote_fische,
            blaue_fische,
            kraken,
            plies_played,
            rounds_played,
            game_status: None,
            move_color,
            hash,
        }
    }

    pub fn standard() -> GameState {
        GameState::standard_with_kraken(GameState::generate_random_kraken())
    }
    pub fn standard_with_kraken(kraken: u128) -> GameState {
        let hash = GameState::calculate_hash(0x20180601806018060180400u128, 0x7f800000000000000000001feu128, kraken, &GameColor::Red);
        GameState {
            rote_fische: 0x20180601806018060180400u128,
            blaue_fische: 0x7f800000000000000000001feu128,
            kraken,
            plies_played: 0,
            rounds_played: 0,
            move_color: GameColor::Red,
            game_status: Some(GameStatus::Ingame),
            hash,
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
                    res_str.push_str(&format!("{}", "🐟".red()));
                } else if ((&self.blaue_fische >> shift) & 1u128) != 0u128 {
                    res_str.push_str(&format!("{}", "🐟".blue()));
                } else if ((&self.kraken >> shift) & 1u128) != 0u128 {
                    res_str.push_str(&format!("{}", "🐙".green()));
                }
                res_str.push_str("\t|");
            }
            res_str.push_str("\n");
        }
        res_str.push_str(&format!("Rounds played: {}\n", self.rounds_played));
        res_str.push_str(&format!("Plies played: {}\n", self.plies_played));
        res_str.push_str(&format!("Hash: {}", self.hash));
        write!(formatter, "{}", res_str)
    }
}