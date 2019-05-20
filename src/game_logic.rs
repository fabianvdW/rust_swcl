use super::game_state::{GameState, GameColor, DIRECTIONS, GameMove};
use super::constants;
use super::zobrist;

#[inline(always)]
pub fn get_schwarm(gs: &GameState, gc: &GameColor) -> u8 {
    let mut meine_fische: u128 = match gc {
        GameColor::Red => gs.rote_fische,
        GameColor::Blue => gs.blaue_fische,
    };
    if meine_fische == 0u128 {
        return 0u8;
    }
    let mut result: u8 = 0u8;
    let mut neighboring_fields: u128 = 0u128;
    let mut neighboring_fields_and_meine_fische: u128 = meine_fische;
    loop {
        result += 1;
        let fisch_bit: usize = neighboring_fields_and_meine_fische.trailing_zeros() as usize;
        meine_fische &= !(1u128 << fisch_bit);
        neighboring_fields |= constants::NACHBARN[fisch_bit];
        neighboring_fields_and_meine_fische = neighboring_fields & meine_fische;
        if neighboring_fields_and_meine_fische == 0u128 {
            break;
        }
    }
    result
}

#[inline(always)]
pub fn get_schwarm_board(mut meine_fische: u128) -> u128 {
    if meine_fische == 0u128 {
        return 0u128;
    }
    let mut result: u128 = 0u128;
    let mut neighboring_fields: u128 = 0u128;
    let mut neighboring_fields_and_meine_fische: u128 = meine_fische;
    loop {
        let fisch_bit: usize = neighboring_fields_and_meine_fische.trailing_zeros() as usize;
        meine_fische ^= 1u128 << fisch_bit;
        result |= 1u128 << fisch_bit;
        neighboring_fields |= constants::NACHBARN[fisch_bit];
        neighboring_fields_and_meine_fische = neighboring_fields & meine_fische;
        if neighboring_fields_and_meine_fische == 0u128 {
            break;
        }
    }
    result
}

#[inline(always)]
pub fn get_possible_moves(gs: &GameState, gc: &GameColor, early_return: bool) -> Vec<GameMove> {
    let mut res: Vec<GameMove> = Vec::with_capacity(90);
    let (meine_fische, gegner_fische) = match gc {
        GameColor::Red => (gs.rote_fische, gs.blaue_fische),
        GameColor::Blue => (gs.blaue_fische, gs.rote_fische),
    };
    let mut fisch_iterator = meine_fische;
    while fisch_iterator != 0u128 {
        let fisch_pos = fisch_iterator.trailing_zeros() as usize;
        for i in 0..4 {
            let squares: usize = (constants::ATTACK_TWO_SIDED[fisch_pos][i] & (meine_fische | gegner_fische)).count_ones() as usize;
            for j in 0..2 {
                let destination: isize = fisch_pos as isize + DIRECTIONS[i] as isize * if j == 0 { 1isize } else { -1isize } * squares as isize;
                if destination <= 99 && destination >= 0 {
                    let destination_square: u128 = 1u128 << destination;
                    if (destination_square & (meine_fische | gs.kraken)) == 0 && (constants::ATTACK_TWO_SIDED[fisch_pos][i] & destination_square) != 0 {
                        if squares < 2 || (constants::ATTACK_ONE_SIDED_SKIPPED_SQUARES[fisch_pos][i + if j == 0 { 0usize } else { 4usize }][squares - 2] & gegner_fische) == 0u128 {
                            res.push(GameMove::new(fisch_pos as u8, destination as u8));
                            if early_return {
                                return res;
                            }
                        }
                    }
                }
            }
        }
        fisch_iterator &= !(1u128 << fisch_pos);
    }
    res
}

#[inline(always)]
pub fn make_move(gs: &GameState, gm: &GameMove) -> GameState {
    match gs.move_color {
        GameColor::Red => {
            let mut new_red: u128 = gs.rote_fische & !(1u128 << gm.from);
            new_red |= 1u128 << gm.to;
            let new_blau: u128 = gs.blaue_fische & !(1u128 << gm.to);
            //Update hash
            let mut hash = gs.hash;
            hash ^= zobrist::ZOBRIST_KEYS[(gm.from / 10) as usize][(gm.from % 10) as usize][0];
            hash ^= zobrist::ZOBRIST_KEYS[(gm.to / 10) as usize][(gm.to % 10) as usize][0];
            if new_blau != gs.blaue_fische {
                hash ^= zobrist::ZOBRIST_KEYS[(gm.to / 10) as usize][(gm.to % 10) as usize][1];
            }
            hash ^= zobrist::SIDE_TO_MOVE_IS_BLUE;
            GameState::new(new_red, new_blau, gs.kraken, gs.plies_played + 1, gs.rounds_played, GameColor::Blue, hash)
        }
        GameColor::Blue => {
            let mut new_blau: u128 = gs.blaue_fische & !(1u128 << gm.from);
            new_blau |= 1u128 << gm.to;
            let new_red: u128 = gs.rote_fische & !(1u128 << gm.to);
            let mut hash = gs.hash;
            hash ^= zobrist::ZOBRIST_KEYS[(gm.from / 10) as usize][(gm.from % 10) as usize][1];
            hash ^= zobrist::ZOBRIST_KEYS[(gm.to / 10) as usize][(gm.to % 10) as usize][1];
            if new_red != gs.rote_fische {
                hash ^= zobrist::ZOBRIST_KEYS[(gm.to / 10) as usize][(gm.to % 10) as usize][0];
            }
            hash ^= zobrist::SIDE_TO_MOVE_IS_BLUE;
            GameState::new(new_red, new_blau, gs.kraken, gs.plies_played + 1, gs.rounds_played + 1, GameColor::Red, hash)
        }
    }
}

#[inline(always)]
pub fn make_null_move(gs: &GameState) -> GameState {
    GameState::new(gs.rote_fische, gs.blaue_fische, gs.kraken, gs.plies_played + 1, gs.rounds_played + if let GameColor::Blue = gs.move_color { 1 } else { 0 }, if let GameColor::Blue = gs.move_color { GameColor::Red } else { GameColor::Blue }, gs.hash ^ zobrist::SIDE_TO_MOVE_IS_BLUE)
}