use std::fs;
use std::collections::HashMap;
use std::time::Instant;

fn distance(num1: i32, num2: i32) -> u32 {
    (num2 - num1).abs() as u32
}

fn filedata_to_lists(filedata: String) -> [Vec<i32>; 2] {
    let list1 = filedata
        .lines()
        .map(|line| line
            .split_whitespace()
            .nth(0).unwrap()
            .parse::<i32>().expect("cannot parse i32!"))
        .collect();

    let list2 = filedata
        .lines()
        .map(|line| line
            .split_whitespace()
            .nth(1).unwrap()
            .parse::<i32>().expect("cannot parse to i32!"))
        .collect();

    [list1, list2]
}

fn update_hashmap(occurances: &mut HashMap<i32, i32>, num: i32) {
    let value = occurances.entry(num).or_insert(0);
    *value += 1;
}

fn main() {

    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot find file {filename}");

    let before = Instant::now();

    let two_lists: [Vec<i32>; 2] = filedata_to_lists(filedata);
    let [mut list1, mut list2] = two_lists;
    list1.sort();  // sort them independently
    list2.sort();

    let distance_sum: u32 = list1
        .iter()
        .zip(list2.iter())
        .map(|(&num1, &num2)| distance(num2, num1))
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part I): {:2?}", after);
    println!("    (Part  I) Sum of distances: {distance_sum}");
    let before = Instant::now();

    let mut occurances: HashMap<i32, i32> = HashMap::new();
    let keys = list1;
    let appearances = list2;  // or "multiplicities"

    appearances
        .iter()
        .for_each(|&item| update_hashmap(&mut occurances, item));

    let similarity_sum: i32 = keys
        .iter()
        .map(|&key| key * *occurances.entry(key).or_insert(0))
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part II): {:2?}", after);
    println!("    (Part II) Sum of similaries: {similarity_sum}");
}
