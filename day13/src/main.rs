use std::fs;
use regex::Regex;
use std::time::Instant;

struct Game {
    button_a: [i64; 2],
    button_b: [i64; 2],
    prize: [i64; 2],
}

fn textdata_to_games(textdata: String) -> Vec<Game> {
    let mut lines = textdata.lines();
    let mut games: Vec<Game> = Vec::new();

    while let (Some(line1), Some(line2), Some(line3)) = (lines.next(), lines.next(), lines.next()) {
        let re_button: Regex = Regex::new("X\\+(\\d+), Y\\+(\\d+)").unwrap();

        let button_a_match: Vec<[&str; 2]> = re_button.captures_iter(line1)
            .map(|found| found.extract::<2>().1)  // [String; N]
            .collect();
        let button_b_match: Vec<[&str; 2]> = re_button.captures_iter(line2)
            .map(|found| found.extract::<2>().1)  // [String; N]
            .collect();

        let button_a: [i64; 2] = button_a_match.iter()
            .map(|[str1, str2]| [
                str1.parse::<i64>().unwrap(), 
                str2.parse::<i64>().unwrap() ]
            )
            .collect::<Vec<[i64; 2]>>()[0];

        let button_b: [i64; 2] = button_b_match.iter()
            .map(|[str1, str2]| [
                str1.parse::<i64>().unwrap(), 
                str2.parse::<i64>().unwrap() ]
            )
            .collect::<Vec<[i64; 2]>>()[0];

        let re_prize: Regex = Regex::new("X=(\\d+), Y=(\\d+)").unwrap();
        let prize_match: Vec<[&str; 2]> = re_prize.captures_iter(line3)
            .map(|found| found.extract::<2>().1)  // [String; N]
            .collect();
        let prize: [i64; 2] = prize_match.iter()
            .map(|[str1, str2]| [
                str1.parse::<i64>().unwrap(), 
                str2.parse::<i64>().unwrap() ]
            )
            .collect::<Vec<[i64; 2]>>()[0];
        games.push(Game { button_a, button_b, prize } );

        lines.next();
    }
    games
}

fn get_tokens_required(game: &Game) -> Option<usize> {
    let Game { button_a, button_b, prize } = game;
    let [prize_x, prize_y] = prize;
    let [button_a_x, button_a_y] = button_a;
    let [button_b_x, button_b_y] = button_b;

    let presses_a_numer: i64 = prize_x * button_b_y - prize_y * button_b_x;
    let presses_b_numer: i64 = -prize_x * button_a_y + prize_y * button_a_x;
    let press_denom: i64 = button_a_x * button_b_y - button_b_x * button_a_y;

    // check if integer multiple (i.e. prize lies on lattice points)
    let is_lattice_point: bool = (presses_a_numer % press_denom) == 0 && (presses_b_numer % press_denom == 0);
    if !is_lattice_point { return None }

    let presses_a: usize = (presses_a_numer / press_denom) as usize;
    let presses_b: usize = (presses_b_numer / press_denom) as usize;

    Some(tokens_from_presses(presses_a, presses_b))
}

fn num_digits(num: &i64) -> u64 { 
    let result = num.abs().checked_ilog10();
    let num_digits: u64 = match result {
        Some(val) => val as u64 + 1,
        None => 1,
    };
    num_digits
}

fn get_tokens_required_conversion(game: &Game) -> Option<usize> {
    let Game { button_a, button_b, prize } = game;
    let [prize_x, prize_y] = prize;
    let [button_a_x, button_a_y] = button_a;
    let [button_b_x, button_b_y] = button_b;

    let prize_x = &(10000000000000i64 + prize_x);
    let prize_y = &(10000000000000i64 + prize_y);
    if false {
        // for some reason, this is causing an overflow when I define "presses_a_numer", even when
        // I swap to i128???

        let prize_x_digits: u32 = num_digits(&prize_x) as u32;
        let prize_y_digits: u32 = num_digits(&prize_y) as u32;
        println!("prize x: {prize_x}");
        println!("prize y: {prize_y}");
        println!("prize x digits: {prize_x_digits}");
        println!("prize y digits: {prize_y_digits}");
        let prize_x = &((10000000000000i64 * (10u64.pow(prize_x_digits) as i64)) as i64);
        let prize_y = &((10000000000000i64 * (10u64.pow(prize_y_digits) as i64)) as i64);
        println!("");
        println!("not actually changing prize_x, this is scoped to within if-else block!");
    }

    let presses_a_numer: i64 = prize_x * button_b_y - prize_y * button_b_x;
    let presses_b_numer: i64 = -prize_x * button_a_y + prize_y * button_a_x;
    let press_denom: i64 = button_a_x * button_b_y - button_b_x * button_a_y;

    // check if integer multiple (i.e. prize lies on lattice points)
    let is_lattice_point: bool = (presses_a_numer % press_denom) == 0 && (presses_b_numer % press_denom == 0);
    if !is_lattice_point { return None }

    let presses_a: usize = (presses_a_numer / press_denom) as usize;
    let presses_b: usize = (presses_b_numer / press_denom) as usize;

    Some(tokens_from_presses(presses_a, presses_b))
}

fn tokens_from_presses(presses_a: usize, presses_b: usize) -> usize {
    presses_a * 3 + presses_b * 1
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot find file {filename}"));

    let before = Instant::now();

    let games: Vec<Game> = textdata_to_games(textdata);

    let total_tokens: usize = games.iter()
        .filter_map(get_tokens_required)
        .sum();

    let after = before.elapsed();
    println!("(Part  I) Total tokens required: {total_tokens}");
    println!("(Part  I) elapsed time: {after:.2?}");
    let before = Instant::now();

    let total_tokens: usize = games.iter()
        .filter_map(get_tokens_required_conversion)
        .sum();

    let after = before.elapsed();
    println!("(Part II) Total tokens required: {total_tokens}");
    println!("(Part II) elapsed time: {after:.2?}");

}

#[test]
fn small_test() {
    let filename: &str = "small_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot find file {filename}"));

    let games: Vec<Game> = textdata_to_games(textdata);

    let total_tokens: usize = games.iter()
        .filter_map(get_tokens_required)
        .sum();

    assert_eq!(480, total_tokens);
}

#[test]
fn test_game() {

    let game1 = Game {button_a: [94, 34], button_b: [22, 67], prize: [8400, 5400] };
    let game1_tokens = get_tokens_required(&game1);
    assert!(game1_tokens.is_some());
    assert_eq!(280, game1_tokens.unwrap());

    let game2 = Game {button_a: [22, 66], button_b: [67, 21], prize: [12748, 12176] };
    let game2_tokens = get_tokens_required(&game2);
    assert!(game2_tokens.is_none());

    let game3 = Game {button_a: [17, 86], button_b: [84, 37], prize: [7870, 6450] };
    let game3_tokens = get_tokens_required(&game3);
    assert!(game3_tokens.is_some());
    assert_eq!(200, game3_tokens.unwrap());

    let game4 = Game {button_a: [69, 23], button_b: [27, 71], prize: [18641, 10279] };
    let game4_tokens = get_tokens_required(&game4);
    assert!(game4_tokens.is_none());

}
