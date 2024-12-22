use std::fs;
use std::collections::HashMap;
use std::time::Instant;

fn get_child(rock: &usize) -> [Option<usize>; 2] {
    let is_zero: bool = rock==&0 ;
    if is_zero { return [Some(1), None] }
    let num_digits: u32 = match rock.checked_ilog10() {
        Some(val) => val + 1,
        None => 1,
    };
    let has_even_digits: bool = num_digits % 2 == 0;
    if has_even_digits {
        let power_ten = 10usize.pow((num_digits/2) as u32 );
        let right_rock = rock % power_ten as usize ;
        let left_rock = rock / power_ten as usize ;
        return [Some(left_rock), Some(right_rock)]
    }
    return [Some(rock * 2024), None]
}

fn sum_after_blinks(rocks: Vec<usize>, blinks: usize) -> usize {

    let mut parent_map: HashMap<usize, usize> = HashMap::new();
    rocks
        .iter()
        .for_each(|&rock| {parent_map.insert(rock, 1);} );

    for _ in 0..blinks {
        let mut child_map: HashMap<usize, usize> = HashMap::new();

        parent_map.iter()
            .flat_map(|(parent, &num)| get_child(parent)
                .into_iter()
                .filter_map(|child| child)
                .map(move|child| [child, num])
            )
            .for_each(|[child, num]| {*child_map.entry(child).or_insert(0) += num;});
        parent_map = child_map;
    }

    parent_map.values().sum()
}
 
fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read {filename}"));

    let before = Instant::now();

    let blinks = 25;
    let rocks: Vec<usize> = filedata
        .split_whitespace()
        .map(|string| string.parse::<usize>().unwrap())
        .collect();
    let num_after_blinks: usize = sum_after_blinks(rocks, blinks);

    let after = before.elapsed();
    println!("(Part  I) num of rocks after blinks: {num_after_blinks}");
    println!("(Part  I) elapsed time: {after:.2?}");
    let before = Instant::now();

    let blinks = 75;
    let rocks: Vec<usize> = filedata
        .split_whitespace()
        .map(|string| string.parse::<usize>().unwrap())
        .collect();
    let num_after_blinks: usize = sum_after_blinks(rocks, blinks);

    let after = before.elapsed();
    println!("(Part II) num of rocks after blinks: {num_after_blinks}");
    println!("(Part II) elapsed time: {after:.2?}");
}

#[test]
fn test_hashmap() {
    let blinks = 6;

    let start_rock: Vec<usize> = vec![125];
    let num_after_blinks: usize = sum_after_blinks(start_rock, blinks);
    assert_eq!(7, num_after_blinks);

    let start_rocks: Vec<usize> = vec![125, 17];
    let num_after_blinks: usize = sum_after_blinks(start_rocks, blinks);
    assert_eq!(22, num_after_blinks);
}
