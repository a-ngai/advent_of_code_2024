use std::fs;
use std::collections::HashMap;
use std::time::Instant;

fn recurse_pattern<'a>(
    towel: &'a str,
    patterns: &Vec<&'a str>,
    hashmap: &mut HashMap<&'a str, usize>,
    ) -> usize {

    match hashmap.get(towel) {
        Some(&num) => return num,
        None => ()
    }

    let mut count = 0;
    for &pattern in patterns {
        let len = pattern.len();
        if len > towel.len() { continue }
        if towel == pattern { count += 1 }
        if towel[..len] == *pattern {
            count += recurse_pattern(&towel[len..], patterns, hashmap);
        }
    }

    hashmap.insert(towel, count);
    count
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let mut file_lines = filedata.lines();
    let mut patterns: Vec<&str> = file_lines.next().unwrap().split(", ").collect();
    file_lines.next();
    let towels: Vec<&str> = file_lines.collect();

    patterns.sort_by(|str1, str2| str1.len().cmp(&str2.len()));
    patterns.reverse();
    let patterns = patterns;

    let mut hashmap: HashMap<&str, usize> = HashMap::new();
    let num_designs: usize = towels.iter()
        .map(|towel| { recurse_pattern(towel, &patterns, &mut hashmap) })
        .filter(|&num| num > 0)
        .count();

    let after = before.elapsed();
    println!("(Part  I) Number of possible designs: {num_designs}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    let num_variants: usize = towels.iter()
        .map(|towel| { recurse_pattern(towel, &patterns, &mut hashmap) })
        .sum();

    let after = before.elapsed();
    println!("(Part II) Number of possible variants: {num_variants}");
    println!("(Part II) time elapsed: {after:.2?}");
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let mut file_lines = filedata.lines();
    let mut patterns: Vec<&str> = file_lines.next().unwrap().split(", ").collect();
    file_lines.next();
    let towels: Vec<&str> = file_lines.collect();

    patterns.sort_by(|str1, str2| str1.len().cmp(&str2.len()));
    patterns.reverse();
    let patterns = patterns;

    let mut hashmap: HashMap<&str, usize> = HashMap::new();

    assert_eq!(2, recurse_pattern("brwrr", &patterns, &mut hashmap));
    assert_eq!(1, recurse_pattern("bggr", &patterns, &mut hashmap));
    assert_eq!(4, recurse_pattern("gbbr", &patterns, &mut hashmap));
    assert_eq!(6, recurse_pattern("rrbgbr", &patterns, &mut hashmap));
    assert_eq!(0, recurse_pattern("ubwu", &patterns, &mut hashmap));
    assert_eq!(1, recurse_pattern("bwurrg", &patterns, &mut hashmap));
    assert_eq!(2, recurse_pattern("brgr", &patterns, &mut hashmap));
    assert_eq!(0, recurse_pattern("bbrgwb", &patterns, &mut hashmap));

    let num_designs: usize = towels.iter()
        .map(|towel| recurse_pattern(towel, &patterns, &mut hashmap))
        .filter(|&fragments| fragments > 0)
        .count();
    assert_eq!(6, num_designs);

    let num_variants: usize = towels.iter()
        .map(|towel| { recurse_pattern(towel, &patterns, &mut hashmap) })
        .sum();
    assert_eq!(16, num_variants);
}
