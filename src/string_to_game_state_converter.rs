use super::game_state;

#[allow(dead_code)]
pub const STANDARD_GAME_STATE: [[&str; 10]; 10] = [
    [" ", "b", "b", "b", "b", "b", "b", "b", "b", " "],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", "k", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", "k", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    ["r", " ", " ", " ", " ", " ", " ", " ", " ", "r"],
    [" ", "b", "b", "b", "b", "b", "b", "b", "b", " "],
];

//pub const STANDARD_GAME_STATE: [[&str;10]; 10] = [
//    [" ","b"," "," "," "," "," "," ","b"," "],
//    [" ","r"," ","b","b","b","b"," ","r"," "],
//    [" "," "," ","b","r","r","b"," "," "," "],
//    [" "," ","r"," "," ","k","r"," ","b"," "],
//    [" "," ","r"," "," "," "," "," ","r"," "],
//    [" "," "," "," ","k"," ","r"," "," "," "],
//    [" "," "," "," ","r"," "," "," "," "," "],
//    [" "," "," ","b"," ","b"," "," "," "," "],
//    [" "," "," "," ","b"," ","b"," "," "," "],
//    [" "," "," "," "," "," "," "," "," "," "],
//];
pub fn string_to_game_state(arr: [[&str; 10]; 10], plies_played: u8, rounds_played: u8, move_color: game_state::GameColor) -> game_state::GameState
{
    let mut rote_fische: u128 = 0u128;
    let mut blaue_fische: u128 = 0u128;
    let mut kraken: u128 = 0u128;
    for y in 0..10 {
        for x in 0..10 {
            let s: String = arr[y][x].to_lowercase();
            let shift = 99 - (y * 10 + x);
            if s == "b" {
                blaue_fische |= 1u128 << shift;
            } else if s == "r" {
                rote_fische |= 1u128 << shift;
            } else if s == "k" {
                kraken |= 1u128 << shift;
            }
        }
    }
    let hash= game_state::GameState::calculate_hash(rote_fische,blaue_fische,kraken,&move_color);
    game_state::GameState::new(rote_fische, blaue_fische, kraken, plies_played, rounds_played, move_color,hash)
}