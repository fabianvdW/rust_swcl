use super::game_state;


#[allow(dead_code)]
pub fn rand() -> String {
    let mut res: u128 = 0u128;
    for y in 0..10 {
        for x in 0..10 {
            if y == 0 || y == 9 || x == 0 || x == 9 {
                res |= 1u128 << (10 * y + x);
            }
        }
    }
    format!("0x{:x}u128", res)
}

#[allow(dead_code)]
pub fn starting_position_red() -> String {
    let mut res: u128 = 0u128;
    for y in 0..10 {
        for x in 0..10 {
            let shift = 99 - (y * 10 + x);
            if y > 0 && y < 9 && (x == 0 || x == 9) {
                res |= 1u128 << shift;
            }
        }
    }
    format!("0x{:x}u128", res)
}

#[allow(dead_code)]
pub fn starting_position_blue() -> String {
    let mut res: u128 = 0u128;
    for y in 0..10 {
        for x in 0..10 {
            let shift = 99 - (y * 10 + x);
            if x > 0 && x < 9 && (y == 0 || y == 9) {
                res |= 1u128 << shift;
            }
        }
    }
    format!("0x{:x}u128", res)
}

#[allow(dead_code)]
pub fn nachbar_felder() -> String {
    let mut res_str: String = String::new();
    for y in 0..10 {
        for x in 0..10 {
            let shift = y * 10 + x;
            let mut res: u128 = 0u128;
            for dir in &game_state::DIRECTIONS {
                let new_shift = shift + dir;
                if new_shift <= 99 && new_shift >= 0 && (new_shift % 10 - shift % 10).abs() <= 1 && (new_shift / 10 - shift / 10).abs() <= 1 {
                    res |= 1u128 << new_shift;
                }
            }
            res_str.push_str(&format!("0x{:x}u128,", res));
        }
    }
    format!(": [u128;100]= [{}];", res_str)
}

#[allow(dead_code)]
pub fn attack_two_sided() -> String {
    let mut res_str: String = String::new();
    for y in 0..10 {
        for x in 0..10 {
            let shift = (y * 10 + x) as i8;
            res_str.push_str("[");
            for i in 0..4 {
                let mut res: u128 = 1u128 << shift;
                for j in 0..2 {
                    let plus_shift = game_state::DIRECTIONS[i] * if j == 0 { 1 } else { -1 };
                    let mut last_shift = shift;
                    let mut new_shift = shift + plus_shift;
                    while new_shift >= 0 && new_shift <= 99 && (new_shift % 10 - last_shift % 10).abs() <= 1 && (new_shift / 10 - last_shift / 10).abs() <= 1 {
                        res |= 1u128 << new_shift;
                        last_shift = new_shift;
                        new_shift += plus_shift;
                    }
                }
                res_str.push_str(&format!("0x{:x}u128,", res));
            }
            res_str.push_str("],");
        }
    }
    format!(": [[u128;4];100]= [{}];", res_str)
}

#[allow(dead_code)]
pub fn attack_one_sided_skipped_fields() -> String {
    let mut res_str: String = String::new();
    for y in 0..10 {
        for x in 0..10 {
            let shift = y * 10 + x;
            res_str.push_str("[");
            for i in 0..8 {
                res_str.push_str("[");
                for squares in 2..10 {
                    let mut res: u128 = 0u128;
                    let plus_shift = game_state::DIRECTIONS[i];
                    let mut last_shift = shift;
                    let mut new_shift = shift + plus_shift;
                    let mut count = 1;
                    while new_shift >= 0 && new_shift <= 99 && (new_shift % 10 - last_shift % 10).abs() <= 1 && (new_shift / 10 - last_shift / 10).abs() <= 1 && count + 1 <= squares {
                        count += 1;
                        res |= 1u128 << new_shift;
                        last_shift = new_shift;
                        new_shift += plus_shift;
                    }
                    res_str.push_str(&format!("0x{:x}u128,", res));
                }
                res_str.push_str("],");
            }
            res_str.push_str("],");
        }
    }
    format!(": [[[u128;8];8];100]= [{}];", res_str)
}