use std::fs;
use regex::{Regex, Match};
use std::time::Instant;

fn string_to_muls(string: &String) -> Vec<[i32; 2]> {
    let re_pattern: Regex = Regex::new("mul\\(([0-9]+),([0-9]+)\\)").unwrap();

    let found_pairs: Vec<[&str; 2]> = re_pattern.captures_iter(string.as_str())
        .map(|found| found.extract::<2>().1)  // [String; N]
        .collect();

    found_pairs.iter()
        .map(|[str1, str2]| [
            str1.parse::<i32>().unwrap(), 
            str2.parse::<i32>().unwrap() ]
        )
        .collect()
}

fn string_to_dos_donts(string: &String) -> Vec<Match> {
    let do_dont_pattern: Regex = Regex::new("(do\\(\\)|don't\\(\\))").unwrap();
    let matches: Vec<Match> = do_dont_pattern.find_iter(string.as_str())
        .collect::<Vec<Match>>();
    matches
}

fn matches_to_ranges(matches: Vec<Match>) -> Vec<[usize; 2]> {
    matches.iter()
        .zip(matches.iter().skip(1))
        .filter(|&(match1, _)| (match1.as_str()=="do()") | (match1.range().next().unwrap()==0usize))
        .map(|(&match1, &match2)| [match1.range().last().unwrap()+1, match2.range().next().unwrap()])
        .collect()
}

fn standardize_string(string: String) -> String {
    String::from("do()") + string.as_str() + "do()"
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let before = Instant::now();

    let mul_pairs: Vec<[i32; 2]> = string_to_muls(&filedata);
    let sum_of_muls: i32 = mul_pairs.iter()
        .map(|[val1, val2]| val1 * val2)
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part I): {after:2?}");
    println!("(Part  I): Sum of mul pairs: {sum_of_muls}");
    let before = Instant::now();

    let prepended_filedata: String = standardize_string(filedata);
    let do_dont_matches: Vec<Match> = string_to_dos_donts(&prepended_filedata);
    let do_ranges: Vec<[usize; 2]> = matches_to_ranges(do_dont_matches);

    let do_strings: Vec<String> = do_ranges.iter()
        .map(|&[start, end]| prepended_filedata[start..end].to_owned())
        .collect();

    let sum_of_filtered_muls: i32 = do_strings.iter()
        .flat_map(|string| string_to_muls(string))
        .map(|[val1, val2]| val1 * val2)
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part II): {after:2?}");
    println!("(Part II): Sum of filtered mul pairs: {sum_of_filtered_muls}");

}

#[test]
fn test_simple() {
    let test_string: String = String::from("xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))");

    let mul_pairs: Vec<[i32; 2]> = string_to_muls(&test_string);
    let ref_vec: Vec<[i32; 2]> = Vec::from([[2, 4], [5, 5], [11, 8], [8, 5]]);
    assert_eq!(ref_vec, mul_pairs);

    let sum_of_muls: i32 = ref_vec.iter()
        .map(|[val1, val2]| val1 * val2)
        .sum();
    assert_eq!(161 as i32, sum_of_muls);
}

#[test]
fn test_do_dont() {
    let test_string: String = String::from("xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))");
    let prepended_test_string = standardize_string(test_string);
    let do_dont_matches: Vec<Match> = string_to_dos_donts(&prepended_test_string);
    let do_ranges: Vec<[usize; 2]> = matches_to_ranges(do_dont_matches);
    let do_strings: Vec<String> = do_ranges.iter()
        .map(|&[start, end]| prepended_test_string[start..end].to_owned())
        .collect();
    let sum_of_filtered_muls: i32 = do_strings.iter()
        .flat_map(|string| string_to_muls(string))
        .map(|[val1, val2]| val1 * val2)
        .sum();

    assert_eq!(48, sum_of_filtered_muls);

    assert_eq!(String::from("do()12345do()"), standardize_string(String::from("12345")));

    let test_string: String = String::from("012mul(9,7)12don't()");
    let prepended_test_string = standardize_string(test_string);
    let do_dont_matches: Vec<Match> = string_to_dos_donts(&prepended_test_string);
    let do_ranges: Vec<[usize; 2]> = matches_to_ranges(do_dont_matches);
    let shift = "do()".len();
    assert_eq!(vec![[0 + shift, 13 + shift],], do_ranges);

}
