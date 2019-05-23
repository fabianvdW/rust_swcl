use super::game_state::{GameState, GameColor};
use super::game_logic;
use super::constants::RAND;

pub const MAX_DIST: f64 = 6.36396103068;
pub const MID_X: f64 = 4.5;
pub const MID_Y: f64 = 4.5;

pub const ANZAHL_FISCHE: f64 = 1.1;
pub const ANZAHL_FISCHE_PHASE: f64 = 0.0;
pub const ANZAHL_FISCHE_NEGPHASE: f64 = 0.3;

pub const DISTANCE_TO_MID: f64 = 0.0;
pub const DISTANCE_TO_MID_PHASE: f64 = 0.0;
pub const DISTANCE_TO_MID_NEGPHASE: f64 = -16.0;

pub const DISTANCE_TO_BIGGEST: f64 = 0.0;
pub const DISTANCE_TO_BIGGEST_PHASE: f64 = -7.0;
pub const DISTANCE_TO_BIGGEST_NEGPHASE: f64 = 0.0;

pub const BIGGEST_SCHWARM: f64 = 1.0;
pub const BIGGEST_SCHWARM_PHASE: f64 = 4.0;
pub const BIGGEST_SCHWARM_NEGPHASE: f64 = 0.0;

pub const ABSOLUT_SCHWARM: f64 = 0.0;
pub const ABSOLUT_SCHWARM_PHASE: f64 = 8.0;
pub const ABSOLUT_SCHWARM_NEGPHASE: f64 = 0.0;

pub const RAND_FISCHE: f64 = 0.0;
pub const RAND_FISCHE_PHASE: f64 = 0.0;
pub const RAND_FISCHE_NEGPHASE: f64 = -0.4;

pub const DISTANCE_TO_ENEMY: f64 = 0.0;
pub const DISTANCE_TO_ENEMY_PHASE: f64 = -11.0;
pub const DISTANCE_TO_ENEMY_NEGPHASE: f64 = 0.0;

pub fn calculate_feature(input: f64, phase: f64, no_phase_feature: f64, phase_feature: f64, neg_phase_feature: f64) -> f64 {
    input * no_phase_feature + input * phase * phase_feature + input * (1.0 - phase) * neg_phase_feature
}

pub fn anzahl_fische_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, ANZAHL_FISCHE, ANZAHL_FISCHE_PHASE, ANZAHL_FISCHE_NEGPHASE)
}

pub fn distance_to_mid_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, DISTANCE_TO_MID, DISTANCE_TO_MID_PHASE, DISTANCE_TO_MID_NEGPHASE)
}

pub fn distance_to_biggest_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, DISTANCE_TO_BIGGEST, DISTANCE_TO_BIGGEST_PHASE, DISTANCE_TO_BIGGEST_NEGPHASE)
}

pub fn biggest_schwarm_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, BIGGEST_SCHWARM, BIGGEST_SCHWARM_PHASE, BIGGEST_SCHWARM_NEGPHASE)
}

pub fn absolut_schwarm_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, ABSOLUT_SCHWARM, ABSOLUT_SCHWARM_PHASE, ABSOLUT_SCHWARM_NEGPHASE)
}

pub fn rand_fische_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, RAND_FISCHE, RAND_FISCHE_PHASE, RAND_FISCHE_NEGPHASE)
}

pub fn distance_to_enemy_feature(input: f64, phase: f64) -> f64 {
    calculate_feature(input, phase, DISTANCE_TO_ENEMY, DISTANCE_TO_ENEMY_PHASE, DISTANCE_TO_ENEMY_NEGPHASE)
}

pub fn distance(x1: f64, x2: f64, y1: f64, y2: f64) -> f64 {
    ((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0)).sqrt()
}

pub struct Schwarm {
    gebiet: u128,
    pub size: usize,
    average_x: f64,
    average_y: f64,
    distance_to_mid: f64,
}

impl Schwarm {
    pub fn new(gebiet: u128, size: usize, average_x: f64, average_y: f64, distance_to_mid: f64) -> Self {
        Schwarm { gebiet, size, average_x, average_y, distance_to_mid }
    }
    pub fn calculate_sichere_fische(&self) -> usize {
        let mut gebiet = self.gebiet;
        let mut res = 0;
        while gebiet != 0u128 {
            let fisch_pos = gebiet.trailing_zeros() as usize;
            if game_logic::get_schwarm_board(self.gebiet ^ (1u128 << fisch_pos)).count_ones() as usize + 1 == self.size {
                res += 1;
            }
            gebiet ^= 1u128 << fisch_pos;
        }
        res
    }

    pub fn average_coordinates(mut gebiet: u128, size: usize) -> (f64, f64) {
        let mut average_x: f64 = 0.0;
        let mut average_y: f64 = 0.0;
        while gebiet != 0u128 {
            let fisch_pos = gebiet.trailing_zeros() as usize;
            average_x += (fisch_pos % 10) as f64;
            average_y += (fisch_pos / 10) as f64;
            gebiet ^= 1u128 << fisch_pos;
        }
        average_x /= size as f64;
        average_y /= size as f64;
        (average_x, average_y)
    }

    pub fn make_schwarm(board: u128) -> Schwarm {
        let board_size = board.count_ones() as usize;
        let (average_x, average_y) = Schwarm::average_coordinates(board, board_size);
        let distance_to_mid = distance(average_x, MID_X, average_y, MID_Y);
        Schwarm::new(board, board_size, average_x, average_y, distance_to_mid)
    }
    pub fn berechne_schwaerme(mut fische: u128) -> (Vec<Schwarm>, Schwarm) {
        let mut res = Vec::with_capacity(5);
        let mut biggest_schwarm: Schwarm;
        biggest_schwarm = Schwarm::make_schwarm(game_logic::get_schwarm_board(fische));
        fische ^= biggest_schwarm.gebiet;
        while fische != 0u128 {
            let board = game_logic::get_schwarm_board(fische);
            let schwarm = Schwarm::make_schwarm(board);
            if biggest_schwarm.size < schwarm.size || biggest_schwarm.size == schwarm.size && schwarm.distance_to_mid < biggest_schwarm.distance_to_mid {
                res.push(biggest_schwarm);
                biggest_schwarm = schwarm;
            } else {
                res.push(schwarm);
            }
            fische ^= board;
        }
        (res, biggest_schwarm)
    }
}

pub fn rating(game_state: &GameState, verbose: bool) -> f64 {
    let (rote_schwaerme, biggest_roter_schwarm) = Schwarm::berechne_schwaerme(game_state.rote_fische);
    let (blaue_schwaerme, biggest_blauer_schwarm) = Schwarm::berechne_schwaerme(game_state.blaue_fische);
    eval(game_state.plies_played as usize, game_state.rote_fische, &rote_schwaerme, &biggest_roter_schwarm, GameColor::Red, &biggest_blauer_schwarm, game_state.blaue_fische.count_ones() as usize, verbose)
        - eval(game_state.plies_played as usize, game_state.blaue_fische, &blaue_schwaerme, &biggest_blauer_schwarm, GameColor::Blue, &biggest_roter_schwarm, game_state.rote_fische.count_ones() as usize, verbose)
}

pub fn eval(plies_played: usize, meine_fische: u128, meine_schwaerme: &Vec<Schwarm>, my_biggest_schwarm: &Schwarm, my_color: GameColor, biggest_gegner_schwarm: &Schwarm, gegner_fische: usize, verbose: bool) -> f64 {
    let unskewed_phase = plies_played as f64 / 60.0;
    let phase = 1.0 - (1.0 - unskewed_phase).powf(2.0);


    //FISCHE FEATURE
    let fisch_anzahl = meine_fische.count_ones() as usize;
    let mut fisch_eval = anzahl_fische_feature(fisch_anzahl as f64, phase);
    if fisch_anzahl == 2 {
        fisch_eval += 20.0;
    } else if fisch_anzahl == 3 {
        fisch_eval += 10.0;
    } else if fisch_anzahl == 4 {
        fisch_eval += 5.0;
    }

    let mut spielfeld_mitte_distanzen: f64 = 0.0;
    let mut gegner_biggest_schwarm_distanzen: f64 = 0.0;
    for s in meine_schwaerme {
        spielfeld_mitte_distanzen += (s.distance_to_mid / MAX_DIST).powf(2.0) * s.size as f64;
        gegner_biggest_schwarm_distanzen += (distance(s.average_x, biggest_gegner_schwarm.average_x, s.average_y, biggest_gegner_schwarm.average_y) / MAX_DIST).powf(2.0) * s.size as f64;
    }
    {
        let distance_biggest_mid = (my_biggest_schwarm.distance_to_mid / MAX_DIST).powf(2.0);
        spielfeld_mitte_distanzen += distance_biggest_mid * my_biggest_schwarm.size as f64;
        spielfeld_mitte_distanzen /= fisch_anzahl as f64;
        //TODO test sensibility
        if my_biggest_schwarm.size * 2 > fisch_anzahl {
            spielfeld_mitte_distanzen += distance_biggest_mid;
        }

        gegner_biggest_schwarm_distanzen += (distance(my_biggest_schwarm.average_x, biggest_gegner_schwarm.average_x, my_biggest_schwarm.average_y, biggest_gegner_schwarm.average_y) / MAX_DIST).powf(2.0) * my_biggest_schwarm.size as f64;
        gegner_biggest_schwarm_distanzen /= fisch_anzahl as f64;
        gegner_biggest_schwarm_distanzen *= (biggest_gegner_schwarm.size as f64 / gegner_fische as f64).powf(3.0);
    }
    let gegner_distance_eval = distance_to_enemy_feature(gegner_biggest_schwarm_distanzen, phase);
    let distance_to_mid_eval = distance_to_mid_feature(spielfeld_mitte_distanzen, phase);

    let mut abstand_zu_biggest_distanzen: f64 = 0.0;
    if my_biggest_schwarm.size < fisch_anzahl {
        let missing_fische = fisch_anzahl - my_biggest_schwarm.size;
        for s in meine_schwaerme {
            abstand_zu_biggest_distanzen += (distance(s.average_x, my_biggest_schwarm.average_x, s.average_y, my_biggest_schwarm.average_y) / MAX_DIST).powf(2.0) * s.size as f64;
        }
        abstand_zu_biggest_distanzen /= missing_fische as f64;
    }
    let distance_to_biggest_schwarm_eval = distance_to_biggest_feature(abstand_zu_biggest_distanzen, phase);

    let ratio = my_biggest_schwarm.size as f64 / fisch_anzahl as f64;
    let mut biggest_schwarm_relative_input = ratio.powf(2.0);
    if ratio > 0.5 {
        biggest_schwarm_relative_input *= 1.0 + (my_biggest_schwarm.calculate_sichere_fische() as f64 / my_biggest_schwarm.size as f64);
    }
    let biggest_schwarm_eval = biggest_schwarm_feature(biggest_schwarm_relative_input, phase);
    let absolute_schwarm_eval = absolut_schwarm_feature((my_biggest_schwarm.size as f64 / 16.0 + 0.5).powf(2.0), unskewed_phase.powf(3.0));
    let rand_fisch_eval = rand_fische_feature((meine_fische & RAND).count_ones() as f64, phase);
    if verbose {
        println!("Phase: {}", phase);
        println!("Eval for {}", if let GameColor::Red = my_color { "Red" } else { "Blue" });
        println!("FischEval: {}", fisch_eval);
        println!("Abstand zu Mitte: {}", distance_to_mid_eval);
        println!("Abstand zu BS: {}", distance_to_biggest_schwarm_eval);
        println!("Biggest Schwarm: {}", biggest_schwarm_eval);
        println!("Maximal Schwarm Size: {}", absolute_schwarm_eval);
        println!("Rand Fische: {}", rand_fisch_eval);
        println!("Abstand zu Gegner Schwarm: {}", gegner_distance_eval);
    }
    let res = fisch_eval + distance_to_mid_eval + distance_to_biggest_schwarm_eval + biggest_schwarm_eval + absolute_schwarm_eval + rand_fisch_eval + gegner_distance_eval;
    res
}