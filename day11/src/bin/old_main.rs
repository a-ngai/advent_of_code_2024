use std::fs;
use std::collections::HashMap;
use std::time::Instant;

fn left_split(rock: &usize) -> usize {
    let is_zero: bool = *rock==0 ;
    let num_digits: u32 = match rock.checked_ilog10() {
        Some(val) => val + 1,
        None => 1,
    };
    let has_even_digits: bool = num_digits % 2 == 0;
    if has_even_digits { return rock / 10usize.pow((num_digits/2) as u32 ) as usize }
    else if is_zero { return 1 }
    else { return rock * 2024 }
}

fn right_split(rock: &usize) -> Option<usize> {
    let num_digits = rock.to_string().len();
    let has_even_digits: bool = num_digits % 2 == 0;
    
    if has_even_digits { return Some(rock % 10usize.pow((num_digits/2) as u32 ) as usize) }
    else { return None }
}

fn sum_after_blinks_recursion(rock: usize, blinks: usize) -> usize {
    let mut sum = 1;
    if blinks == 0 { return sum }

    let left_sum = sum_after_blinks_recursion(left_split(&rock), blinks-1);
    let right_sum = match right_split(&rock) {
        Some(val) => sum_after_blinks_recursion(val, blinks-1),
        None => 0,
    };

    sum = left_sum + right_sum;
    sum
}

fn sum_after_blinks(rocks: Vec<usize>, blinks: usize) -> usize {

    let mut parent_map: HashMap<usize, usize> = HashMap::new();
    rocks
        .iter()
        .for_each( |&rock| {parent_map.insert(rock, 1);} );

    for _ in 0..blinks {
        let mut child_map: HashMap<usize, usize> = HashMap::new();

        for parent_rock in parent_map.keys() {
            let mut child: Vec<usize> = Vec::from([left_split(&parent_rock)]);
            match right_split(parent_rock) {
                Some(val) => child.push(val),
                None => (),
            };
            for child_rock in child {
                *child_map.entry(child_rock).or_insert(0) 
                    += parent_map.get(&parent_rock).unwrap();
            }
        }
        parent_map = child_map;

    }

    parent_map
        .values().sum()
}
 
fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read {filename}"));

    let before = Instant::now();

    let blinks = 25;
    let num_after_blinks: usize = filedata
        .split_whitespace()
        .map(|string| string.parse::<usize>().unwrap())
        .map(|rock| sum_after_blinks_recursion(rock, blinks))
        .sum();

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
fn small_test() {
    let blinks = 6;

    let start_rock: usize = 125;
    let num_after_blinks: usize = sum_after_blinks_recursion(start_rock, blinks);
    assert_eq!(7, num_after_blinks);

    let start_rocks: Vec<usize> = vec![125, 17];
    let num_after_blinks: usize = start_rocks
        .iter()
        .map(|&rock| sum_after_blinks_recursion(rock, blinks))
        .sum();
    assert_eq!(22, num_after_blinks);
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
