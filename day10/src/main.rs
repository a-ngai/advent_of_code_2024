use std::fs;
use std::collections::{VecDeque, HashMap};
use std::time::Instant;

#[derive(Debug)]
struct Marker {
    row: usize,
    col: usize,
    num: usize,
}

fn locate_trailheads(map: &Vec<Vec<usize>>) -> Vec<[usize; 2]> {
    let num_cols = map[0].len();
    map.iter().for_each(|row| assert_eq!(num_cols, row.len()));

    let trailheads: Vec<[usize; 2]> = map.iter()
        .enumerate()
        .flat_map(|(row, row_data)| {
            row_data.iter()
            .enumerate()
            .filter(|(_col, &num)| num==0)
            .map(move |(col, _num)| [row, col])
            }
        )
        .collect();
    trailheads
}

fn get_new_locs(loc: [usize; 2], map: &Vec<Vec<usize>>) -> Vec<[usize; 2]> {
    let num_rows = map.len() as isize;
    let num_cols = map[0].len() as isize;
    let [row, col] = loc;
    let [row, col] = [row as isize, col as isize];
    let steps: [[isize; 2]; 4] = [[-1, 0], [1, 0], [0, 1], [0, -1]];

    let valid_new_locs = steps
        .iter()
        .filter(|&[row_step, col_step]| {
            (row+row_step) >= 0 && (row+row_step < num_rows)
            && (col+col_step) >= 0 && (col+col_step < num_cols)
        })
        .map(|&[row_step, col_step]| [(row+row_step) as usize, (col+col_step) as usize])
        .collect();
    valid_new_locs
}

fn trailstart_to_ends(trailhead: &[usize; 2], map: &Vec<Vec<usize>>) -> HashMap<[usize; 2], usize> {
    let &[row, col] = trailhead;
    let mut queue: VecDeque<Marker> = VecDeque::from(vec![Marker{row, col, num:0}]);
    let mut trailends: HashMap<[usize; 2], usize> = HashMap::new();
    while let Some(marker) = queue.pop_front() {
        let Marker { row, col, num} = marker;
        // println!("{:?}", marker);
        if num == 9 {
            *trailends.entry([row, col]).or_insert(0) += 1;
            continue
        }
        let down_locs: Vec<Marker> = get_new_locs([row, col], map)
            .iter()
            .filter(|&&[row, col]| map[row][col] == num+1)
            .map(|&[row, col]| Marker {row, col, num:num+1 })
            .collect();
        queue.extend(down_locs);
    }
    trailends
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot open file {filename}"));

    let before = Instant::now();

    let map: Vec<Vec<usize>> = textdata.lines()
        .map(|line| line.chars().map(|chr| chr.to_digit(10).unwrap() as usize).collect())
        .collect();

    let trailheads: Vec<[usize; 2]> = locate_trailheads(&map);
    let trailends_hashmaps: Vec<HashMap<[usize; 2], usize>> = trailheads.iter()
        .map(|loc| trailstart_to_ends(loc, &map))
        .collect();
    let trailhead_score: usize = trailends_hashmaps.iter()
        .map(|hashmap| hashmap.len())
        .sum();

    let after = before.elapsed();
    println!("(Part  I) trailhead score: {trailhead_score}");
    println!("(Part  I) elapsed time: {after:.2?}");
    let before = Instant::now();

    let unique_score: usize = trailends_hashmaps.iter()
        .map(|hashmap| hashmap.values().sum::<usize>())
        .sum();

    let after = before.elapsed();
    println!("(Part II) unique score: {unique_score}");
    println!("(Part II) elapsed time: {after:.2?}");
}

#[test]
fn test_locate_trailheads() {
    let test_vec = vec![
        vec![8,9,0,1,0,1,2,3,],
        vec![7,8,1,2,1,8,7,4,],
        vec![8,7,4,3,0,9,6,5,],
        vec![9,6,5,4,9,8,7,4,],
        vec![4,5,6,7,8,9,0,3,],
        vec![3,2,0,1,9,0,1,2,],
        vec![0,1,3,2,9,8,0,1,],
        vec![1,0,4,5,6,7,3,2,],
    ];
    let trailheads: Vec<[usize; 2]> = locate_trailheads(&test_vec);
    let num_of_trailheads: usize = trailheads.len();
    assert_eq!(9usize, num_of_trailheads);
}

#[test]
fn test_locate_trailends() {
    let map = vec![
        vec![8,9,0,1,0,1,2,3,],
        vec![7,8,1,2,1,8,7,4,],
        vec![8,7,4,3,0,9,6,5,],
        vec![9,6,5,4,9,8,7,4,],
        vec![4,5,6,7,8,9,0,3,],
        vec![3,2,0,1,9,0,1,2,],
        vec![0,1,3,2,9,8,0,1,],
        vec![1,0,4,5,6,7,3,2,],
    ];
    let trailheads: Vec<[usize; 2]> = locate_trailheads(&map);
    let num_trailheads: usize = trailheads.len();
    assert_eq!(9, num_trailheads);

    let trailends_hashmaps: Vec<HashMap<[usize; 2], usize>> = trailheads
        .iter()
        .map(|loc| trailstart_to_ends(loc, &map))
        .collect();

    let trailhead_score: usize = trailends_hashmaps
        .iter()
        .map(|hashmap| hashmap.len())
        .sum();
    assert_eq!(36, trailhead_score);
}
