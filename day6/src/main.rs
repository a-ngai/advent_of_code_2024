use std::fs;
use std::time::Instant;

const UP: u8 = 2u8.pow(0) + 2u8.pow(4);
const RIGHT: u8 = 2u8.pow(1) + 2u8.pow(5);
const DOWN: u8 = 2u8.pow(2) + 2u8.pow(6);
const LEFT: u8 = 2u8.pow(3) + 2u8.pow(7);

const START_DIR: u8 = UP;

enum HistoryResult {
    Terminated(Vec<Vec<u8>>),
    Infinite(Vec<Vec<u8>>),
}

fn next_in_bound(pos: [usize; 2], dir: [isize; 2], map_size: [usize; 2]) -> Result<[usize; 2], &'static str> {
    let [num_rows, num_cols] = map_size;
    let (num_rows, num_cols) = (num_rows as isize, num_cols as isize);
    let [row, col] = pos;
    let [row_step, col_step] = dir;
    let next_row = (row as isize) + row_step;
    let next_col = (col as isize) + col_step;
    let out_bound = next_row < 0 || next_row >= num_rows || next_col < 0 || next_col >= num_cols;
    if !out_bound { 
        Ok([next_row as usize, next_col as usize])
    } else {
        Err("out of bounds")
    }
}

fn get_start_pos_dir(filedata: &String) -> ([usize; 2], u8) {
    // get current position of guard; assume initial direction is "up"
    let row: usize = filedata.lines().position(|string| { string.contains('^') }).expect("'^' not found");
    let col: usize = filedata.lines().nth(row).unwrap().chars().position(|chr| chr=='^').unwrap();
    let pos: [usize; 2] = [row, col];

    (pos, START_DIR)
}

fn simulate_history(start_pos: &[usize; 2], start_dir: u8, obstacles: &Vec<Vec<bool>>) -> HistoryResult {
    // initialize maze history
    let map_size: [usize; 2] = [obstacles.len(), obstacles[0].len()];
    let mut history: Vec<Vec<u8>> = (0..map_size[0])
        .map(|_| (0..map_size[1]).map(|_| 0u8).collect::<Vec<u8>>())
        .collect::<Vec<Vec<u8>>>();

    let mut current_dir = start_dir;
    let mut current_pos = *start_pos;

    // loop while guard has not exited maze or stuck in infinite loop
    // this is the complex part! Also interesting :)    
    loop {
        // when we loop, we've freshly arrive at the next point
        let [row, col] = current_pos;
        let check_history = &mut history[row][col];
        let is_inf_loop = *check_history & current_dir != 0u8; // same location AND direction
        if is_inf_loop { return HistoryResult::Infinite(history) }

        *check_history += current_dir;

        // prepare to move to next location
        let next_step = match current_dir {
            UP    => [-1,  0],
            RIGHT => [ 0,  1],
            DOWN  => [ 1,  0],
            LEFT  => [ 0, -1],
            _ => panic!("should not reach here!"), // invalid states are still representable
                                                   // use of enums will make them irrepresentable
        };

        // testing for bounds
        let [next_row, next_col] = match next_in_bound(current_pos, next_step, map_size) {
            Ok(val) => val,
            Err(_) => break,
        };

        // check for obstacles
        let next_is_obstacle = obstacles[next_row][next_col];
        if next_is_obstacle {
            current_dir = current_dir.rotate_left(1);  // set-up as [L, D, R, U, L, D, R, U]
        } else {
            current_pos = [next_row, next_col];
        }
    }
    HistoryResult::Terminated(history)
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot read file {filename}");

    let before = Instant::now();

    let obstacles: Vec<Vec<bool>> = filedata.lines()
        .map(|string| string.chars().map(|chr| chr=='#').collect())
        .collect();
    let (start_pos, start_dir) = get_start_pos_dir(&filedata);

    let history: Vec<Vec<u8>> = match simulate_history(&start_pos, start_dir, &obstacles) {
        HistoryResult::Terminated(val) => val,
        HistoryResult::Infinite(val) => val,
    };

    let num_visited: usize = history.iter()
        .map(|vec| vec.iter().filter(|&&val| val != 0u8).count())
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part I): {after:2?}");
    println!("(Part  I): num. of visited tiles: {num_visited}");
    let before = Instant::now();

    let pos_visited: Vec<[usize; 2]> = history.iter().enumerate()
        .flat_map(|(row, vec)| vec
            .iter().enumerate()
            .filter(|&(_, &val)| val != 0u8)  // ignore unvisited locations
            .map( move |(col, _)| [row, col])
        )
        .filter(|&[row, col]| [row, col] != start_pos)  // ignore starting position
        .collect();

    let [mut prev_row, mut prev_col] = start_pos;
    let mut new_obstacles = obstacles;
    let mut num_infinite_loops: usize = 0;
    for [new_row, new_col] in pos_visited {
        new_obstacles[prev_row][prev_col] = false;
        new_obstacles[new_row][new_col] = true;

        match simulate_history(&start_pos, start_dir, &new_obstacles) {
            HistoryResult::Terminated(_) => (),
            HistoryResult::Infinite(_) => num_infinite_loops += 1,
        };
        [prev_row, prev_col] = [new_row, new_col];
    }
    let after = before.elapsed();
    println!("Time elapsed (Part II): {after:2?}");
    println!("(Part II): num. of infinite loops: {num_infinite_loops}");
}

#[test]
fn test_input() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot read file {filename}");

    let obstacles: Vec<Vec<bool>> = filedata.lines()
        .map(|string| string.chars().map(|chr| chr=='#').collect())
        .collect();

    let (start_pos, start_dir) = get_start_pos_dir(&filedata);

    let history = match simulate_history(&start_pos, start_dir, &obstacles) {
        HistoryResult::Terminated(val) => val,
        HistoryResult::Infinite(val) => val,
    };

    let num_visited: usize = history.iter()
        .map(|vec| vec.iter().filter(|&&val| val != 0u8).count())
        .sum();
    assert_eq!(41, num_visited);
}
