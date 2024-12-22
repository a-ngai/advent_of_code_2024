use std::fs;
use std::collections::VecDeque;
use std::time::Instant;

const ALL_DIR: [Direction; 4] = [Direction::U, Direction::R, Direction::D, Direction::L];

enum Direction { U, R, D, L }

impl Direction {
    fn step(&self) -> [isize; 2] {
        match self {
            Direction::U => [ -1,  0],
            Direction::R => [  0,  1],
            Direction::D => [  1,  0],
            Direction::L => [  0, -1],
        }
    }
}

#[derive(Clone, Copy)]
struct TimeNode {
    row: usize,
    col: usize,
    bounds: [usize; 2],
    time: usize,
}

impl TimeNode {
    fn spawn_nodes(&self) -> [Option<TimeNode>; 4] {
        ALL_DIR.map(|dir| {
            let [row_step, col_step] = dir.step();
            let next_row: usize = self.row.checked_add_signed(row_step)?;
            let next_col: usize = self.col.checked_add_signed(col_step)?;
            let [row_lim, col_lim] = self.bounds;
            if next_row >= row_lim || next_col >= col_lim { return None }
            Some(TimeNode { row: next_row, col: next_col, bounds: self.bounds, time: self.time+1 })
        })
    }
}

fn bounds_from_map(map: &Vec<Vec<char>>) -> [usize; 2] {
    let nrows = map.len();
    let ncols = map[0].len();
    [nrows, ncols]
}

fn get_dist_map(row: usize, col: usize, map: &Vec<Vec<char>>) -> Vec<Vec<usize>> {
    let nrows = map.len();
    let ncols = map[0].len();
    let bounds: [usize; 2] = [nrows, ncols];
    let mut dist_map: Vec<Vec<usize>> = (0..nrows).map(|_| (0..ncols).map(|_| usize::MAX).collect()).collect();
    let start_node = TimeNode { row, col, time: 0, bounds };
    let mut queue = VecDeque::from([start_node,]);
    while let Some(node) = queue.pop_back() {
        let TimeNode { row, col, time, ..} = node;
        if map[row][col] == '#' { continue }
        let record = &mut (*dist_map)[row][col];
        if time > *record { continue }
        if time < *record { *record = time }
        for new_node in node.spawn_nodes().into_iter().filter_map(|node| node) {
            queue.push_front(new_node);
        }
    }
    dist_map
}

fn get_cheat_time_save(cheat_time: usize, cheat_interval: usize, map: &Vec<Vec<char>>, dist_map: &Vec<Vec<usize>>) -> Vec<usize> {
    let bounds = bounds_from_map(map);
    let start_nodes: Vec<TimeNode> = dist_map.iter().enumerate()
        .flat_map(|(row, rowdata)| rowdata.iter().enumerate().map(move |(col, &time)| (row, col, time)))
        .filter(|&(_, _, time)| time==cheat_time)
        .map(|(row, col, time)| TimeNode { row, col, time, bounds} )
        .collect();
    let mut time_saves: Vec<usize> = Vec::new();
    for start_node in start_nodes {
        //let mut visited: HashSet<[usize; 2]> = HashSet::new();

        let [nrows, ncols] = bounds;
        let mut visited: Vec<Vec<bool>> = (0..nrows).map(|_| (0..ncols).map(|_| false).collect()).collect();

        let mut queue = VecDeque::from([start_node,]);
        let mut finished: Vec<TimeNode> = Vec::new();
        while let Some(node) = queue.pop_back() {
            let TimeNode { row, col, time, ..} = node;
            if map[row][col] != '#' { finished.push(node) }
            if time == cheat_time + cheat_interval { continue }
            for new_node in node.spawn_nodes().into_iter().filter_map(|node| node) {
                let TimeNode { row, col, ..} = new_node;
                // if visited.contains(&[row, col]) { continue };
                // visited.insert([row, col]);
                if visited[row][col] { continue };
                visited[row][col] = true;
                queue.push_front(new_node);
            }
        }
        let time_save: Vec<usize> = finished.into_iter()
            .filter_map(|TimeNode { row, col, time, ..} | dist_map[row][col].checked_sub(time))
            .collect();
        time_saves.extend(time_save)
    }
    time_saves
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|row| row.chars().collect::<Vec<char>>())
        .collect();

    let start_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='S')).unwrap();
    let start_col: usize = map[start_row].iter().position(|chr| *chr=='S').unwrap();

    let end_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='E')).unwrap();
    let end_col: usize = map[end_row].iter().position(|chr| *chr=='E').unwrap();

    // first run without cheats to get distances from end, for each tile
    let dist_map = get_dist_map(end_row, end_col, &map);
    let finish_time = dist_map[start_row][start_col];

    // loop with cheats, each loop has the specific time for the cheat
    let time_saves: Vec<usize> = (0..(finish_time-2))
        .flat_map(|cheat_time| get_cheat_time_save(cheat_time, 2, &map, &dist_map))
        .filter(|&num| num != 0)
        .collect();
    let num_saves_greater_100 = time_saves.iter().filter(|&&num| num>=100).count();

    let after = before.elapsed();
    println!("(Part  I) Number of time saves: {num_saves_greater_100}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    let time_saves: Vec<usize> = (0..(finish_time-20))
        .flat_map(|cheat_time| get_cheat_time_save(cheat_time, 20, &map, &dist_map))
        .filter(|&num| num != 0)
        .collect();
    let num_saves_greater_100 = time_saves.iter().filter(|&&num| num>=100).count();

    let after = before.elapsed();
    println!("(Part II) Number of time saves: {num_saves_greater_100}");
    println!("(Part II) time elapsed: {after:.2?}");
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|row| row.chars().collect::<Vec<char>>())
        .collect();

    let start_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='S')).unwrap();
    let start_col: usize = map[start_row].iter().position(|chr| *chr=='S').unwrap();

    let end_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='E')).unwrap();
    let end_col: usize = map[end_row].iter().position(|chr| *chr=='E').unwrap();

    // first run without cheats to get distances from end, for each tile
    let dist_map = get_dist_map(end_row, end_col, &map);
    let finish_time = dist_map[start_row][start_col];
    assert_eq!(84, finish_time);

    let mut _temp_saves: Vec<(usize, Vec<usize>)> = (0..(finish_time-2))
        .map(|cheat_time| (cheat_time, get_cheat_time_save(cheat_time, 2, &map, &dist_map)))
        // .chain(get_cheat_time_save(0, 1, &map, &dist_map).into_iter())
        .collect();

    let mut time_saves: Vec<usize> = (0..(finish_time-2))
        .flat_map(|cheat_time| get_cheat_time_save(cheat_time, 2, &map, &dist_map))
        .chain(get_cheat_time_save(0, 1, &map, &dist_map).into_iter())
        .filter(|&num| num != 0)
        .collect();
    time_saves.sort();
    time_saves.reverse();
    let save_nums = [2, 4, 6, 8, 10, 12, 20, 36, 38, 40, 64];
    let reference = [14, 14, 2, 4, 2, 3, 1, 1, 1, 1, 1];
    let count_save = save_nums.map(|num| time_saves.iter().filter(|&&item| item==num).count());
    assert_eq!(reference, count_save);
    assert_eq!(reference.iter().sum::<usize>(), time_saves.len());
}

#[test]
fn small_test_part_two() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|row| row.chars().collect::<Vec<char>>())
        .collect();

    let start_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='S')).unwrap();
    let start_col: usize = map[start_row].iter().position(|chr| *chr=='S').unwrap();

    let end_row: usize = map.iter().position(|vec| (*vec).iter().any(|&chr| chr=='E')).unwrap();
    let end_col: usize = map[end_row].iter().position(|chr| *chr=='E').unwrap();

    // first run without cheats to get distances from end, for each tile
    let dist_map = get_dist_map(end_row, end_col, &map);
    let finish_time = dist_map[start_row][start_col];
    assert_eq!(84, finish_time);

    let mut _temp_saves: Vec<(usize, Vec<usize>)> = (0..(finish_time-2))
        .map(|cheat_time| (cheat_time, get_cheat_time_save(cheat_time, 2, &map, &dist_map)))
        // .chain(get_cheat_time_save(0, 1, &map, &dist_map).into_iter())
        .collect();
    let mut time_saves: Vec<usize> = (0..(finish_time-20))
        .flat_map(|cheat_time| get_cheat_time_save(cheat_time, 20, &map, &dist_map))
        .chain(get_cheat_time_save(0, 1, &map, &dist_map).into_iter())
        .filter(|&num| num != 0)
        .collect();
    time_saves.sort();
    time_saves.reverse();
    let save_nums = [50, 52, 54, 56, 58, 60, 62, 64, 66, 68, 70, 72, 74, 76];
    let reference = [32, 31, 29, 39, 25, 23, 20, 19, 12, 14, 12, 22, 4, 3];
    let count_save = save_nums.map(|num| time_saves.iter().filter(|&&item| item==num).count());
    assert_eq!(reference, count_save);
}

