use std::fs;
use std::collections::HashMap;
use regex::Regex;
use std::io;

// possible optimizations:
//
// instead of creating a new Robot instance, just update the values of the previous Robot
//

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
struct Vector {
    x: i32,  // col
    y: i32,  // row
}

#[derive(Debug, PartialEq)]
struct Robot {
    pos: Vector,
    vel: Vector,
    bounds: Vector,
}

fn line_to_state(line: &str, bounds: Vector) -> Robot {

    let state_pattern: &str = "p=([-]*\\d+),([-]*\\d+) v=([-]*\\d+),([-]*\\d+)";
    let re_state = Regex::new(state_pattern)
        .unwrap_or_else(|err| panic!("Cannot make regex ({err})"));

    let Some((_full, number_strings)) = re_state.captures(line).map(|caps| caps.extract()) 
        else { panic!("Cannot find regex ({state_pattern}) for line {line}") };

    let [pos_x, pos_y, vel_x, vel_y]: [i32; 4] = number_strings
        .map(|string| string.parse::<i32>().unwrap()) ;
    let pos = Vector { x: pos_x, y: pos_y };
    let vel = Vector { x: vel_x, y: vel_y };
    Robot { pos, vel, bounds}
}

fn propagate_state(pos: Vector, vel: Vector, bounds: Vector, time: i32) -> (Vector, Vector) {
    let Vector { x: pos_x, y: pos_y } = pos;
    let Vector { x: vel_x, y: vel_y } = vel;
    let Vector { x: bound_x, y: bound_y } = bounds;
    let new_pos = Vector { 
        x: (pos_x + vel_x * time).rem_euclid(bound_x), 
        y: (pos_y + vel_y * time).rem_euclid(bound_y)
    };
    let new_vel = vel;
    (new_pos, new_vel)
}

fn propagate_robot(robot: Robot, time: i32) -> Robot {
    let Robot { pos, vel, bounds} = robot;
    let (new_pos, new_vel) = propagate_state(pos, vel, bounds, time);
    Robot {pos: new_pos, vel: new_vel, bounds}
}

fn position_map(robots: &Vec<Robot>) -> HashMap<Vector, usize> {
    let mut position_map: HashMap<Vector, usize> = HashMap::new();
    robots.into_iter()
        .for_each(|Robot { pos, .. } | { *position_map.entry(*pos).or_insert(0) += 1; });
    position_map
}

fn map_quadrant_count(map: &HashMap<Vector, usize>, bounds: Vector) -> [usize; 4] {
    let Vector { x: bound_x, y: bound_y } = bounds;
    let quadrant_bounds: [[i32; 4]; 4] = [
        [0, bound_x/2, 0, bound_y/2],
        [bound_x/2+1, bound_x, 0, bound_y/2],
        [0, bound_x/2, bound_y/2+1, bound_y],
        [bound_x/2+1, bound_x, bound_y/2+1, bound_y],
    ];

    let mut quadrant_count: [usize; 4] = [0; 4];
    'outer: for (key, val) in map.iter() {
        let Vector { x, y } = key;
        for (i, condition) in quadrant_bounds.iter().enumerate() {
            let [x_min, x_max, y_min, y_max] = condition;
            if x_min <= x && x < x_max && y_min <= y && y < y_max {
                quadrant_count[i] += val;
                continue 'outer
            }
        }
    }
    quadrant_count
}

fn string_from_map(map: &HashMap<Vector, usize>, bounds: Vector) -> String {
    let Vector { x: bound_x, y: bound_y } = bounds;
    let mut number_map: Vec<Vec<char>> = (0..bound_y)
        .map(|_| (0..bound_x).map(|_| '.').collect::<Vec<char>>())
        .collect();
    for (&Vector{ x, y}, &val) in map.iter() {
        number_map[y as usize][x as usize] = char::from_digit(val as u32, 16).unwrap();
    }
    let string: String = number_map.iter()
        .map(|vec| vec.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
    string
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let bounds: Vector = Vector { x: 101, y: 103 };

    let robots: Vec<Robot> = textdata.lines()
        .map(|line| line_to_state(line, bounds))
        .collect();

    let time: i32 = 100;
    let prop_robots = robots.into_iter()
        .map(|robot| propagate_robot(robot, time))
        .collect();
    let prop_map = position_map(&prop_robots);
    let quadrant_counts: [usize; 4] = map_quadrant_count(&prop_map, bounds);

    let safety_factor: usize = quadrant_counts.iter().product();
    println!("(Part  I) Safety factor: {safety_factor}");

    // Make CLI which shows the time + current map, and lets us either increment the time
    // forward/backwards or quit.

    let mut robots: Vec<Robot> = textdata.lines()
        .map(|line| line_to_state(line, bounds))
        .collect();
    let increment_time = 1;
    let mut checkpoint_count = 0;
    let mut checkpoint_string_map = String::from("");
    let mut count: usize = 0;
    let mut min_safety_score = usize::MAX;
    loop {
        let map = position_map(&robots);
        let quadrant_counts: [usize; 4] = map_quadrant_count(&map, bounds);

        // let [upleft, upright, downleft, downright] = quadrant_counts;
        // let evaluate_condition = upleft == upright && upright == downleft && downleft == downright;
        // let evaluate_condition = 
        //     (upleft as i32 - upright as i32).abs() < 5 
        //     && (upright as i32 - downleft as i32).abs() < 5 
        //     && (downleft as i32 - downright as i32).abs() < 5 ;
        // let evaluate_condition = true;

        let safety_score: usize = quadrant_counts.iter().product();
        let evaluate_condition = {
            if safety_score <= min_safety_score {
                min_safety_score = safety_score;
                true
            } else { false }
        };

        let step_manually = false;

        let mut input = String::new();
        if evaluate_condition {
            let string_map = string_from_map(&map, bounds);
            if step_manually {
                println!("{string_map}");
                println!("time: {count}, quadrant counts: {quadrant_counts:?}, safety_score: {safety_score}");
                io::stdin().read_line(&mut input).expect("Failed to read line");
                if input.len() >= 2 { break }

            } else {
                if checkpoint_string_map == string_map { break }
            }
            checkpoint_count = count;
            checkpoint_string_map = string_map.clone();
        }

        robots = robots.into_iter()
            .map(|robot| propagate_robot(robot, increment_time))
            .collect();
        count += 1;
    }
    println!("{checkpoint_string_map}");
    println!("(Part II) time: {checkpoint_count}");
}

#[test]
fn test_propagation() {
    let bounds: Vector = Vector { x: 11, y: 7 };
    let robot: Robot = Robot{ pos: Vector{x:2, y:4}, vel: Vector{x:2, y:-3}, bounds};

    let time: i32 = 1;
    let robot: Robot = propagate_robot(robot, time);
    assert_eq!(robot, Robot{ pos: Vector{x:4, y:1}, vel: Vector{x:2, y:-3}, bounds});

    let robot: Robot = propagate_robot(robot, time);
    assert_eq!(robot, Robot{ pos: Vector{x:6, y:5}, vel: Vector{x:2, y:-3}, bounds});
}

#[test]
fn test_small_input() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let bounds: Vector = Vector { x: 11, y: 7 };

    let robots: Vec<Robot> = textdata.lines()
        .map(|line| line_to_state(line, bounds))
        .collect();

    let time: i32 = 100;
    let prop_robots = robots.into_iter()
        .map(|robot| propagate_robot(robot, time))
        .collect();
    let prop_map = position_map(&prop_robots);
    println!("{prop_robots:?}");
    assert_eq!(2, *prop_map.get(&Vector{x:6, y: 0}).unwrap());

    let quadrant_counts: [usize; 4] = map_quadrant_count(&prop_map, bounds);
    assert_eq!([1, 3, 4, 1], quadrant_counts);
}

