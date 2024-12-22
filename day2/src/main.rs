use std::fs;

struct Report {
    numbers: Vec<i32>
}

const UPPER_BOUND: i32 = 3;
const LOWER_BOUND: i32 = 1;

fn diff(vec: &Vec<i32>) -> Vec<i32> {
    vec.iter()
        .zip(vec.iter().skip(1))  // or use .windows(2) instead of .iter().zip(...)
        .map(|(&num1, &num2)| num2 - num1)
        .collect()
}

impl Report {
    fn is_safe(&self) -> bool {
        // check if monotically increasing/decreasing
        let differences: Vec<i32> = diff(&self.numbers);

        let diff_len = differences.len();
        let num_increasing = differences.iter()
            .filter(|&&num| num>=0).count();
        let num_decreasing = differences.iter()
            .filter(|&&num| num<=0).count();
        if num_increasing != diff_len && num_decreasing != diff_len { return false };
        // could also use Iterator::is_sorted_by

        // check if finite difference is within bounds
        let num_exceeds_bounds = differences.iter()
            .filter(|&&diff| (diff.abs() < LOWER_BOUND)  || (UPPER_BOUND < diff.abs()))
            .count();
        if num_exceeds_bounds > 0 { return false };

        return true
    }

    fn is_safe_dampener(&self) -> bool {
        // brute force approach with O(n^2)
        if self.is_safe() { return true }
        for remove_index in 0..self.numbers.len() {
            let vector_without_index: Vec<i32> = self.numbers.iter()
                .enumerate()
                .filter(|&(index, _)| index != remove_index)
                .map(|(_, &num)| num)
                .collect();
            let test_report = Report {numbers:vector_without_index};
            if test_report.is_safe() { return true };
        }
        return false

    }

    fn is_safe_dampener_quick(&self) -> bool {
        let differences: Vec<i32> = diff(&self.numbers);

        let diff_len = differences.len();
        let num_increasing = differences.iter()
            .filter(|&&num| num>=0).count();
        let num_decreasing = differences.iter()
            .filter(|&&num| num<=0).count();

        let mostly_increasing: bool = num_increasing >= num_decreasing;
        let num_bad_points = if mostly_increasing {
            diff_len - num_increasing
        } else {
            diff_len - num_decreasing
        };

        // three categories: no "bad" point (trivial), one "bad" point, two or more bad points
        // (return early)
        let deleted: Vec<i32>;
        let is_corrected: bool = num_bad_points != 0;

        if num_bad_points == 0 {
            deleted = self.numbers.clone();
        } 

        else if num_bad_points == 1 {
            let sign_change = match mostly_increasing {
                true => 1,
                false => -1,
            };
            let index_of_bad_point = differences.iter()
                .position(|&num| sign_change * num < 0).unwrap();
            deleted = self.numbers.iter().enumerate()
                .filter(|&(index, _)| index != (index_of_bad_point+1))
                .map(|(_, &item)| item)
                .collect();

        } else { return false };

        let corrected: Vec<i32> = diff(&deleted);

        let num_exceeds_upper_bounds = corrected.iter()
            .filter(|num|  UPPER_BOUND < num.abs())
            .count();

        let index_upper_bound: usize;
        if num_exceeds_upper_bounds > 0 {
            index_upper_bound = corrected.iter()
                .position(|num|  UPPER_BOUND < num.abs()).unwrap();
        } else {
            index_upper_bound = 0;
        }
        let num_exceeds_lower_bounds = corrected.iter()
            .filter(|num| num.abs() < LOWER_BOUND )
            .count();
        let num_exceeds_bounds = num_exceeds_lower_bounds + num_exceeds_upper_bounds;

        if is_corrected && num_exceeds_bounds > 0 { return false }

        // only non-corrected cases left
        if num_exceeds_bounds > 1 { return false };
        if num_exceeds_lower_bounds == 1 { return true };

        // non-corrected case with one exceeded upper bound
        if num_exceeds_upper_bounds == 1 { 
            let on_edge = [0, corrected.len()-1].contains(&index_upper_bound);
            if on_edge { return true }
            return false
        };

        return true
    }
}

fn into_report(line: &str) -> Report {
    let numbers: Vec<i32> = line.split_whitespace()
        .map(|string| string.parse().unwrap())
        .collect();
    Report { numbers }
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename).expect("Cannot find file {filename}");

    let reports: Vec<Report> = textdata.lines()
        .map(|line| into_report(line))
        .collect();

    let num_safe: usize = reports.iter()
        .filter(|report| report.is_safe())
        .count();

    println!("Number of safe reports: {num_safe}");

    let num_safe_dampener: usize = reports.iter()
        .filter(|report| report.is_safe_dampener())
        .count();

    println!("Number of safe reports w/ dampener: {num_safe_dampener}")
}

#[test]
fn test_is_safe() {
    let test_report = Report { numbers : Vec::from([7, 6, 4, 2, 1])};
    assert_eq!(test_report.is_safe(), true);
    let test_report = Report { numbers : Vec::from([1, 2, 7, 8, 9])};
    assert_eq!(test_report.is_safe(), false);
    let test_report = Report { numbers : Vec::from([9, 7, 6, 2, 1])};
    assert_eq!(test_report.is_safe(), false);
    let test_report = Report { numbers : Vec::from([1, 3, 2, 4, 5])};
    assert_eq!(test_report.is_safe(), false);
    let test_report = Report { numbers : Vec::from([8, 6, 4, 4, 1])};
    assert_eq!(test_report.is_safe(), false);
    let test_report = Report { numbers : Vec::from([1, 3, 6, 7, 9])};
    assert_eq!(test_report.is_safe(), true);
}

#[test]
fn test_is_safe_dampener() {
    let test_report = Report { numbers : Vec::from([7, 6, 4, 2, 1])};
    assert_eq!(test_report.is_safe_dampener(), true);
    let test_report = Report { numbers : Vec::from([1, 2, 7, 8, 9])};
    assert_eq!(test_report.is_safe_dampener(), false);
    let test_report = Report { numbers : Vec::from([9, 7, 6, 2, 1])};
    assert_eq!(test_report.is_safe_dampener(), false);
    let test_report = Report { numbers : Vec::from([1, 3, 2, 4, 5])};
    assert_eq!(test_report.is_safe_dampener(), true);
    let test_report = Report { numbers : Vec::from([8, 6, 4, 4, 1])};
    assert_eq!(test_report.is_safe_dampener(), true);
    let test_report = Report { numbers : Vec::from([1, 3, 6, 7, 9])};
    assert_eq!(test_report.is_safe_dampener(), true);
}

