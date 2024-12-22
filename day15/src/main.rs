use std::fs;
use std::io;
use std::thread::sleep;
use std::{time::Instant, iter};
use std::collections::VecDeque;

// implementation with an explicit stack instead of recursion

const MANUAL_STEP: bool = true;

#[derive(Copy, Clone)]
enum Direction {
    U,
    R,
    D,
    L,
}

struct Robot {
    row: usize,
    col: usize,
    dir: Direction,
}

fn dir_to_step(dir: Direction) -> [isize; 2] {
    let [step_row, step_col] = match dir {
        Direction::U => [-1,  0],
        Direction::R => [ 0,  1],
        Direction::D => [ 1,  0],
        Direction::L => [ 0, -1],
    };
    [step_row, step_col]
}

fn get_next_loc(loc: [usize; 2], dir: Direction) -> [usize; 2] {
    let [row, col] = loc;
    let [step_row, step_col] = dir_to_step(dir);
    [ ((row as isize) + step_row) as usize, ((col as isize) + step_col) as usize]
}

impl Robot {
    fn update_dir(&mut self, new_dir: Direction) { self.dir = new_dir }
    fn loc(&self) -> [usize; 2] { [self.row, self.col] }
    fn update_loc(&mut self) { [self.row, self.col] = get_next_loc(self.loc(), self.dir) }
}

fn parse_from_textdata(textdata: &String) -> (Vec<Vec<char>>, String) {
    let mut textdata_iter = textdata.lines();

    let map_textdata: Vec<Vec<char>> = textdata_iter.by_ref()
        .map(|string| string.chars().collect::<Vec<char>>())
        .take_while(|line| line.len() > 0)
        .collect();

    let direction_string: String = textdata_iter
        .map(|slice| slice.to_string())
        .collect::<Vec<String>>()
        .join("");
    (map_textdata, direction_string)
}

fn parse_from_textdata_wide(textdata: &String) -> (Vec<Vec<char>>, String) {
    let mut textdata_iter = textdata.lines();

    let map_textdata: Vec<Vec<char>> = textdata_iter.by_ref()
        .map(|string| string.chars().collect::<Vec<char>>())
        .take_while(|line| line.len() > 0)
        .collect();
    let map_textdata_strings: Vec<Vec<String>> = map_textdata
        .iter()
        .map(|vec_chars| 
            vec_chars.iter().map(|chr| {
                match chr {
                    '#' => "##".to_string(),
                    'O' => "[]".to_string(),
                    '.' => "..".to_string(),
                    '@' => "@.".to_string(),
                    _other => panic!("({_other}) chr case not covered!")
                }
            })
            .collect::<Vec<String>>()
        )
        .collect();
    let map_textdata: Vec<Vec<char>> = map_textdata_strings
        .iter()
        .map(|vec_strings| 
            vec_strings.iter()
            .flat_map(|string| string.chars())
            .collect::<Vec<char>>()
        )
        .collect();

    let direction_string: String = textdata_iter
        .map(|slice| slice.to_string())
        .collect::<Vec<String>>()
        .join("");
    (map_textdata, direction_string)
}

fn dir_string_to_list(string: String) -> Vec<Direction> {
    let dir_list: Vec<Direction> = string.chars()
        .map(|chr| match chr {
            '^' => Direction::U,
            '>' => Direction::R,
            'v' => Direction::D,
            '<' => Direction::L,
            _ => panic!("not recognized!")
        })
        .collect();
    dir_list
}

fn stack_movement(curr_loc: [usize; 2], dir: Direction, map: &mut Vec<Vec<char>>) -> bool {
    // recurse until find either wall (trivial, nothing moves) or empty space (everything
    // shuffles!)

    let move_up = match dir {
        Direction::U => true,
        Direction::D => true,
        Direction::R => false,
        Direction::L => false,
        };

    let mut stack: VecDeque<[usize; 2]> = VecDeque::from([curr_loc,]);
    let mut history_stack: Vec<([usize; 2], char)> = Vec::new();
    let can_move: bool = loop {
        let curr_loc = match stack.pop_front() {
            Some(loc) => loc,
            None => break true,
        };

        let [curr_row, curr_col] = curr_loc;
        let curr_char = map[curr_row][curr_col];
        let next_loc = get_next_loc(curr_loc, dir);
        let [next_row, next_col] = next_loc;

        match (move_up, map[next_row][next_col]) {
            (_, '#') => { break false },
            (_, '.') => (),
            (_, 'O') => {stack.push_back(next_loc)},
            (false, '[') => {stack.push_back(next_loc)},
            (false, ']') => {stack.push_back(next_loc)},
            (true, '[') => {
                let next_left_loc = next_loc;
                let next_right_loc = get_next_loc([curr_row, curr_col+1], dir);
                stack.extend([next_left_loc, next_right_loc]);
            },
            (true, ']') => {
                let next_left_loc = get_next_loc([curr_row, curr_col-1], dir);
                let next_right_loc = next_loc;
                stack.extend([next_left_loc, next_right_loc]);
            },
            _other => panic!("({:?}) char not recognized!", _other),
        };
        history_stack.push((curr_loc, curr_char));
    };
    if !can_move { return can_move }

    while let Some((loc, curr_char)) = history_stack.pop() {
        let [row, col] = loc;
        let [next_row, next_col] = get_next_loc(loc, dir);
        map[next_row][next_col] = curr_char;
        map[row][col] = '.';
    }
    can_move
}

fn increment_robot_map(robot: &mut Robot, map: &mut Vec<Vec<char>>) {
    let can_move = stack_movement(robot.loc(), robot.dir, map);
    if can_move { robot.update_loc() }
}

fn gps_sum_map(map: &Vec<Vec<char>>) -> usize {
    let gps_sum: usize = map
        .iter().enumerate()
        .flat_map(|(row, rowdata)| rowdata.iter().enumerate().map(move |(col, &chr)| (row, col, chr)))
        .filter(|(_row, _col, chr)| *chr == 'O')
        .map(|(row, col, _chr)|  row * 100 + col * 1 )
        .sum();

    gps_sum
}

fn gps_sum_map_wide(map: &Vec<Vec<char>>) -> usize {
    let left_edge_locs: Vec<[usize; 2]> = map
        .iter().enumerate()
        .flat_map(|(row, rowdata)| rowdata.iter().enumerate().map(move |(col, &chr)| (row, col, chr)))
        .filter(|(_row, _col, chr)| *chr == '[')
        .map(|(row, col, _chr)| [row, col])
        .collect();

    left_edge_locs.iter()
        .map(|[row, col]|  row * 100 + col * 1 )
        .sum()
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let (map_textdata, move_string): (Vec<Vec<char>>, String) = parse_from_textdata(&textdata);
    let move_list: Vec<Direction> = dir_string_to_list(move_string);

    let robot_row = map_textdata.iter()
        .position(|vec| vec.contains(&'@'))
        .unwrap_or_else(|| panic!("Cannot find (@) in textdata!"));
    let robot_col = map_textdata[robot_row].iter().position(|&chr| chr=='@')
        .unwrap_or_else(|| panic!("Cannot find (@) in textdata row ({:?})!", map_textdata[robot_row]));
    let mut robot: Robot = Robot {row: robot_row, col: robot_col, dir: Direction::U};

    let mut map = map_textdata;

    for dir in move_list {

        // let map_string: String = map
        //     .iter().map(|vec_chr| vec_chr.iter().collect::<String>())
        //     .collect::<Vec<String>>()
        //     .join("\n");
        // println!("{map_string}");
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).expect("Failed to read line");
        // if input.len() >= 2 { break }

        robot.update_dir(dir);
        increment_robot_map(&mut robot, &mut map);

    }
    
    let after = before.elapsed();

    let gps_sum = gps_sum_map(&map);
    println!("(Part  I) GPS sum: {gps_sum}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    let (map_textdata, move_string): (Vec<Vec<char>>, String) = parse_from_textdata_wide(&textdata);
    let move_list: Vec<Direction> = dir_string_to_list(move_string);

    let robot_row = map_textdata.iter()
        .position(|vec| vec.contains(&'@'))
        .unwrap_or_else(|| panic!("Cannot find (@) in textdata!"));
    let robot_col = map_textdata[robot_row].iter().position(|&chr| chr=='@')
        .unwrap_or_else(|| panic!("Cannot find (@) in textdata row ({:?})!", map_textdata[robot_row]));
    let mut robot: Robot = Robot {row: robot_row, col: robot_col, dir: Direction::U};

    let mut map = map_textdata;

    // for dir in iter::repeat(move_list).flat_map(|vec| vec.into_iter()) {
    //     robot.update_dir(dir);
    //     increment_robot_map(&mut robot, &mut map);

    //     if MANUAL_STEP {
    //         let map_string: String = map
    //             .iter().map(|vec_chr| vec_chr.iter().collect::<String>())
    //             .collect::<Vec<String>>()
    //             .join("\n");

    //         println!("{map_string}");
    //         sleep(time::Duration::from_millis(100));

    //         // let mut input = String::new();
    //         // io::stdin().read_line(&mut input).expect("Failed to read line");
    //         // if input.len() >= 2 { break }
    //     }
    // }

    for dir in move_list {
        
        // let map_string: String = map
        //     .iter().map(|vec_chr| vec_chr.iter().collect::<String>())
        //     .collect::<Vec<String>>()
        //     .join("\n");
        // println!("{map_string}");
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).expect("Failed to read line");
        // if input.len() >= 2 { break }

        robot.update_dir(dir);
        increment_robot_map(&mut robot, &mut map);
    }

    let after = before.elapsed();
    let gps_sum = gps_sum_map_wide(&map);
    println!("(Part II) GPS sum: {gps_sum}");
    println!("(Part  I) time elapsed: {after:.2?}");
}


#[test]
fn test_gps_sum_wide() {
let map: Vec<Vec<char>> = vec![
    vec!['#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#',],
    vec!['#','#','[',']','.','.','.','.','.','.','.','[',']','.','[',']','[',']','#','#',],
    vec!['#','#','[',']','.','.','.','.','.','.','.','.','.','.','.','[',']','.','#','#',],
    vec!['#','#','[',']','.','.','.','.','.','.','.','.','[',']','[',']','[',']','#','#',],
    vec!['#','#','[',']','.','.','.','.','.','.','[',']','.','.','.','.','[',']','#','#',],
    vec!['#','#','.','.','#','#','.','.','.','.','.','.','[',']','.','.','.','.','#','#',],
    vec!['#','#','.','.','[',']','.','.','.','.','.','.','.','.','.','.','.','.','#','#',],
    vec!['#','#','.','.','@','.','.','.','.','.','.','[',']','.','[',']','[',']','#','#',],
    vec!['#','#','.','.','.','.','.','.','[',']','[',']','.','.','[',']','.','.','#','#',],
    vec!['#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#','#',],
    ];
    assert_eq!(9021, gps_sum_map_wide(&map));
}
