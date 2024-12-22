use std::fs;
use std::collections::{VecDeque, HashSet};
use std::thread::sleep;
use std::time;

static ALL_DIRECTIONS: [Direction; 4] = [Direction::U, Direction::R, Direction::D, Direction::L];

#[derive(Clone, Copy)]
enum Direction {
    U,
    R,
    D,
    L,
}

impl Direction {
    fn step(&self) -> [isize; 2] {
            match self {
                Direction::U => [-1isize,  0],
                Direction::R => [ 0isize,  1],
                Direction::D => [ 1isize,  0],
                Direction::L => [ 0isize, -1],
            }
    }
}

#[derive(Clone, Copy)]
struct Node {
    row: usize,
    col: usize,
    time: usize,
}

impl Node {
    fn make_new_nodes(&self, size: usize) -> [Option<Node>; 4] {
        let &Node { row, col, time } = self;
        ALL_DIRECTIONS.map(|dir| {
            let [step_row, step_col]: [isize; 2] = dir.step();
            let next_row: usize = match row.checked_add_signed(step_row) {
                Some(val) => val,
                None => return None
            };
            let next_col: usize = match col.checked_add_signed(step_col) {
                Some(val) => val,
                None => return None
            };
            let out_of_bounds = next_row >= size || next_col >= size;
            if out_of_bounds { return None }
            Some(Node { 
                row: next_row as usize,
                col: next_col as usize,
                time: time + 1
            })
        })

    }
}

fn find_least_time<const SIZE: usize>(map: &mut [[char; SIZE]; SIZE]) -> Option<usize> {
    let start: Node = Node { row: 0, col: 0, time: 0 }; //
    let end_loc: [usize; 2] = [SIZE-1, SIZE-1];
    let mut stack: VecDeque<Node> = VecDeque::from([start,]);
    let mut time_result: Option<usize> = None;
    map[0][0] = 'O';

    while let Some(node) = stack.pop_front() {
        let Node { row, col, time } = node;
        // reached the end?
        if [row, col] == end_loc {
            time_result = Some(time);
            break
        }

        let new_nodes: [Option<Node>; 4] = node.make_new_nodes(SIZE);

        for next_result in new_nodes {
            let next_node = match next_result {
                Some(node) => node,
                None => continue,
            };
            let Node { row: next_row, col: next_col, ..} = next_node;
            // println!("{next_row}, {next_col}");
            let already_visited = map[next_row][next_col] == 'O';
            let is_byte = map[next_row][next_col] == '#';
            if already_visited || is_byte { continue }
            // println!("\nNext map:\n{}", map.map(|rowdata| rowdata.iter().collect::<String>()).join("\n"));
            // sleep(time::Duration::from_millis(50));

            stack.push_back(next_node);
            map[next_row][next_col] = 'O';
        }
    }

    time_result
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot find file ({filename})"));

    let bytes: Vec<[usize; 2]> = filedata.lines()
        .map(|string| {
            let mut string_iter = string.split(",");
            [
                string_iter.next().unwrap().parse::<usize>().unwrap(), 
                string_iter.next().unwrap().parse::<usize>().unwrap()
            ]
        })
        .collect();

    let mut map: [[char; 71]; 71] = [['.'; 71]; 71];
    bytes.iter().take(1024).for_each(|&[col, row]| map[row][col]='#');
    println!("Filled map:\n{}", map.map(|rowdata| rowdata.iter().collect::<String>()).join("\n"));

    let time = find_least_time(&mut map).unwrap();
    println!("(Part  I) Min. steps needed: {time}");

    let mut map: [[char; 71]; 71] = [['.'; 71]; 71];
    let mut final_loc = [71, 71];
    for [block_col, block_row] in bytes {
        map.iter_mut().for_each(|rowdata| rowdata.iter_mut().for_each(|chr| if *chr == 'O' { *chr = '.' }));

        map[block_row][block_col] = '#';
        let time = find_least_time(&mut map);
        match time {
            Some(_) => (),
            None => {
                final_loc = [block_row, block_col];
                break
            }
        }
        // println!("\nNext map:\n{}", map.map(|rowdata| rowdata.iter().collect::<String>()).join("\n"));
        // sleep(time::Duration::from_millis(50));
    };
    let [block_row, block_col] = final_loc;
    println!("(Part II) (block col, block row): {block_col},{block_row}");

}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot find file ({filename})"));

    let bytes: Vec<[usize; 2]> = filedata.lines()
        .map(|string| {
            let mut string_iter = string.split(",");
            [
                string_iter.next().unwrap().parse::<usize>().unwrap(), 
                string_iter.next().unwrap().parse::<usize>().unwrap()
            ]
        })
        .collect();

    let mut map: [[char; 7]; 7] = [['.'; 7]; 7];
    bytes.iter().take(12).for_each(|&[col, row]| map[row][col]='#');
    println!("Filled map:\n{}", map.map(|rowdata| rowdata.iter().collect::<String>()).join("\n"));

    let time = find_least_time(&mut map).unwrap();
    assert_eq!(22, time);
}

