use std::{fs, io};
use std::time::Instant;
use std::sync::LazyLock;
use regex::Regex;

// HashMap implementation is slower; though we have an explicit array with all possible locations,
// summing over this array means summing over contiguous blocks of memory; this turns out to be
// ~10x faster than summing over (fewer) entries in the HashMap.

const STEP_MANUALLY: bool = false;

const STATE_PATTERN: &str = r"p=([-]*\d+),([-]*\d+) v=([-]*\d+),([-]*\d+)";
static RE_STATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(STATE_PATTERN)
    .unwrap_or_else(|err| panic!("Cannot make regex ({err})")));

#[derive(Debug, PartialEq)]
struct Vector<T> {
    x: T,  // col
    y: T,  // row
}

#[derive(Debug, PartialEq)]
struct Robot {
    pos: Vector<u32>,  // make invalid states irrepresentable!
    vel: Vector<i32>,
}

impl Robot {
    fn next_pos(&mut self, bounds: &Vector<u32>, time: i32) -> Vector<u32> {
        let Vector { x: pos_x, y: pos_y } = self.pos;
        let Vector { x: vel_x, y: vel_y } = self.vel;
        let &Vector { x: bound_x, y: bound_y } = bounds;
        let new_pos = Vector { 
            x: (pos_x as i32 + vel_x * time).rem_euclid(bound_x as i32).try_into().unwrap(), 
            y: (pos_y as i32 + vel_y * time).rem_euclid(bound_y as i32).try_into().unwrap(),
        };
        new_pos
    }

    fn propagate(&mut self, bounds: &Vector<u32>, time: i32) {
        self.pos = self.next_pos(bounds, time);
    }

    fn propagate_map(&mut self, bounds: &Vector<u32>, time:i32, map: &mut Vec<Vec<u32>>) {
        let new_pos = self.next_pos(bounds, time);
        let Vector { x: pos_x, y: pos_y } = self.pos;
        map[pos_y as usize][pos_x as usize] -= 1;
        map[new_pos.y as usize][new_pos.x as usize] += 1;
    }

    fn propagate_with_map(&mut self, bounds: &Vector<u32>, time:i32, map: &mut Vec<Vec<u32>>) {
        self.propagate_map(bounds, time, map);
        self.propagate(bounds, time);
    }
}

fn line_to_state(line: &str) -> Robot {
    let Some((_full, number_strings)) = RE_STATE.captures(line).map(|caps| caps.extract()) 
        else { panic!("Cannot find regex ({STATE_PATTERN}) for line {line}") };

    let [pos_x, pos_y, vel_x, vel_y]: [i32; 4] = number_strings
        .map(|string| string.parse::<i32>().unwrap()) ;
    let pos = Vector { x: pos_x as u32, y: pos_y as u32 };
    let vel = Vector { x: vel_x, y: vel_y };
    Robot { pos, vel }
}

fn position_map(robots: &Vec<Robot>, bounds: &Vector<u32>) -> Vec<Vec<u32>> {
    let &Vector { x: bound_x, y: bound_y } = bounds;
    let mut position_map: Vec<Vec<u32>> = (0..bound_y)
        .map(|_| (0..bound_x).map(|_| 0).collect::<Vec<u32>>())
        .collect();
    robots.into_iter()
        .for_each(|&Robot{ pos: Vector{x, y}, ..} | position_map[y as usize][x as usize] += 1);
    position_map
}

fn map_quadrant_count(map: &Vec<Vec<u32>>, bounds: &Vector<u32>) -> [u32; 4] {
    let &Vector { x: bound_x, y: bound_y } = bounds;
    let quadrant_bounds: [[u32; 4]; 4] = [
        [0, bound_x/2, 0, bound_y/2],
        [bound_x/2+1, bound_x, 0, bound_y/2],
        [0, bound_x/2, bound_y/2+1, bound_y],
        [bound_x/2+1, bound_x, bound_y/2+1, bound_y],
    ];

    let mut quadrant_count: [u32; 4] = [0; 4];
    for (i, condition) in quadrant_bounds.iter().enumerate() {
        let &[x_min, x_max, y_min, y_max] = condition;
        let quadrant_value: u32 = map[(y_min as usize)..(y_max as usize)].iter()
            .flat_map(|row| row[(x_min as usize)..(x_max as usize)].iter())
            .sum();
        quadrant_count[i] = quadrant_value;
        }
    quadrant_count
}

fn string_from_map(map: &Vec<Vec<u32>>) -> String {
    let string: String = map.iter()
        .map(|vec| vec.iter().map(|&num| {
            if num == 0 { '.' }
            else { char::from_digit(num, 16).unwrap() }
        }).collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
    string
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    let bounds: Vector<u32> = Vector { x: 101, y: 103 };
    let mut robots: Vec<Robot> = textdata.lines()
        .map(line_to_state)
        .collect();

    let time: i32 = 100;
    robots.iter_mut()
        .for_each(|robot| robot.propagate(&bounds, time));
    let prop_map = position_map(&robots, &bounds);
    let quadrant_counts: [u32; 4] = map_quadrant_count(&prop_map, &bounds);
    let safety_factor: u32 = quadrant_counts.iter().product();

    let after = before.elapsed();
    println!("(Part  I) Safety factor: {safety_factor}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    robots.iter_mut()
        .for_each(|robot| robot.propagate(&bounds, -time));  // reset to original positions

    let increment_time = 1;
    let mut checkpoint_count = 0;
    let mut count: usize = 0;
    let mut min_safety_score = u32::MAX;
    let mut checkpoint_map: Vec<Vec<u32>> = Vec::new();
    let mut map = position_map(&robots, &bounds);

    loop {
        robots.iter_mut()
            .for_each(|robot| robot.propagate_with_map(&bounds, increment_time, &mut map));
        count += 1;

        let quadrant_counts: [u32; 4] = map_quadrant_count(&map, &bounds);
        let safety_score: u32 = quadrant_counts.iter().product();
        let evaluate_condition = safety_score <= min_safety_score;
        if !evaluate_condition { continue }
        min_safety_score = safety_score;

        if STEP_MANUALLY {
            let mut input = String::new();
            println!("{}", string_from_map(&map));
            println!("time: {count}, quadrant counts: {quadrant_counts:?}, safety_score: {safety_score}");
            io::stdin().read_line(&mut input).expect("Failed to read line");
            if input.len() >= 2 { break }
        }
        if !STEP_MANUALLY && checkpoint_map == map { break }

        checkpoint_count = count;
        checkpoint_map = map.clone();
    }

    let after = before.elapsed();
    println!("{}", string_from_map(&map));
    println!("(Part II) time: {checkpoint_count}");
    println!("(Part II) time elapsed: {after:.2?}");
}

#[test]
fn test_propagation() {
    let bounds: Vector<u32> = Vector { x: 11, y: 7 };
    let mut robot: Robot = Robot{ pos: Vector{x:2, y:4}, vel: Vector{x:2, y:-3}};

    let time: i32 = 1;
    robot.propagate(&bounds, time);
    assert_eq!(robot, Robot{ pos: Vector{x:4, y:1}, vel: Vector{x:2, y:-3}});

    robot.propagate(&bounds, time);
    assert_eq!(robot, Robot{ pos: Vector{x:6, y:5}, vel: Vector{x:2, y:-3}});
}

#[test]
fn test_small_input() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let bounds: Vector<u32> = Vector { x: 11, y: 7 };

    let mut robots: Vec<Robot> = textdata.lines()
        .map(line_to_state)
        .collect();

    let time: i32 = 100;
    robots.iter_mut()
        .for_each(|robot| robot.propagate(&bounds, time));
    let map = position_map(&robots, &bounds);
    assert_eq!(2, map[0][6]);

    let quadrant_counts: [u32; 4] = map_quadrant_count(&map, &bounds);
    assert_eq!([1, 3, 4, 1], quadrant_counts);
}
