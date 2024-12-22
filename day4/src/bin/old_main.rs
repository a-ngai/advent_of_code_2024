use std::fs;
use std::time::Instant;

const XMAS_LEN: usize = 4;
const XMAS_ARRAY: [char; 4] = ['X', 'M', 'A', 'S'];

struct XmasCoor {
    coors: [[usize; 2]; 4],
}

fn forward_backward_match(test_array: [char; 4]) -> bool {
    let forward_match: bool = test_array.iter()
        .zip(XMAS_ARRAY.iter())
        .all(|(&chr1, &chr2)| chr1 == chr2 );

    let backward_match: bool = test_array.iter().rev()
        .zip(XMAS_ARRAY.iter())
        .all(|(&chr1, &chr2)| chr1 == chr2 );

    forward_match || backward_match
}

fn find_all_matches(word_search: &Vec<Vec<char>>) -> Vec<XmasCoor> {
    let nrows: usize = word_search.len();
    let ncols = word_search[0].len();
    word_search.iter().for_each(|vec| assert_eq!(vec.len(), ncols));

    // 4 alignment cases (8 orientation cases)
    // case 1: horizontal
    let row_range: Vec<usize> = (0..nrows).collect();
    let col_range: Vec<usize> = (0..(ncols-(XMAS_LEN-1))).collect();

    // let mut coors: Vec<XmasCoor> = Vec::new();
    let mut vec_coors: Vec<XmasCoor> = Vec::new();
    let mut coors: [[usize; 2]; 4] = [[0, 0],[0, 0],[0, 0],[0, 0],];
    let mut test_array: [char; 4] = ['A','A','A','A'];

    for &row in row_range.iter() {
        for &col in col_range.iter() {
            // let coors: [[usize; 2]; 4] = (0..XMAS_LEN).map(|i| [row, col+i] ).into();
            for i in 0..XMAS_LEN { coors[i] = [row, col+i]; }
            for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
            if forward_backward_match(test_array) { vec_coors.push(XmasCoor { coors: coors.clone()} ); }
        }
    }

    // case 2: vertical
    let row_range: Vec<usize> = (0..(nrows-(XMAS_LEN-1))).collect();
    let col_range: Vec<usize> = (0..ncols).collect();
    for &row in row_range.iter() {
        for &col in col_range.iter() {
            for i in 0..XMAS_LEN { coors[i] = [row+i, col]; }
            for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
            if forward_backward_match(test_array) { vec_coors.push(XmasCoor { coors: coors.clone()} ); }
        }
    }

    // case 3: backward-diagonal
    let row_range: Vec<usize> = (0..(nrows-(XMAS_LEN-1))).collect();
    let col_range: Vec<usize> = (0..(ncols-(XMAS_LEN-1))).collect();
    for &row in row_range.iter() {
        for &col in col_range.iter() {
            for i in 0..XMAS_LEN { coors[i] = [row+i, col+i]; }
            for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
            if forward_backward_match(test_array) { vec_coors.push(XmasCoor { coors: coors.clone()} ); }
        }
    }

    // case 4: forward-diagonal
    let row_range: Vec<usize> = ((XMAS_LEN-1)..nrows).collect();
    let col_range: Vec<usize> = (0..(ncols-(XMAS_LEN-1))).collect();
    for &row in row_range.iter() {
        for &col in col_range.iter() {
            for i in 0..XMAS_LEN { coors[i] = [row-i, col+i]; }
            for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
            if forward_backward_match(test_array) { vec_coors.push(XmasCoor { coors: coors.clone()} ); }
        }
    }

    vec_coors
}

fn find_xmas(word_search: Vec<Vec<char>>) -> Vec<XmasCoor> {
    let nrows: usize = word_search.len();
    let ncols = word_search[0].len();
    word_search.iter().for_each(|vec| assert_eq!(vec.len(), ncols));

    // 4 alignment cases (8 orientation cases)
    // case 1: horizontal
    let row_range: Vec<usize> = (1..(nrows-1)).collect();
    let col_range: Vec<usize> = (1..(ncols-1)).collect();

    // let mut coors: Vec<XmasCoor> = Vec::new();
    let mut vec_coors: Vec<XmasCoor> = Vec::new();
    let mut coors: [[usize; 2]; 4] = [[0, 0],[0, 0],[0, 0],[0, 0],];
    let mut test_array: [char; 4] = ['A','A','A','A'];

    for &row in row_range.iter() {
        for &col in col_range.iter() {
            // let coors: [[usize; 2]; 4] = (0..XMAS_LEN).map(|i| [row, col+i] ).into();
            if word_search[row][col] != 'A' { continue }
            for i in 0..XMAS_LEN { coors[i] = [row, col+i]; }
            coors[0] = [row-1, col-1];
            coors[1] = [row-1, col+1];
            coors[2] = [row+1, col-1];
            coors[3] = [row+1, col+1];
            for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
            let num_m: usize = test_array.iter().filter(|&&chr| chr == 'M').count();
            let num_s: usize = test_array.iter().filter(|&&chr| chr == 'S').count();
            let opposing = word_search[row-1][col-1] != word_search[row+1][col+1];
            if num_m == 2 && num_s == 2 && opposing {
                 vec_coors.push(XmasCoor { coors: coors.clone()} ); 
            }
        }
    }

    vec_coors
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let before = Instant::now();

    let word_search: Vec<Vec<char>> = filedata.lines()
        .map(|line| line.chars().collect())
        .collect();

    let matches: Vec<XmasCoor> = find_all_matches(&word_search);
    let num_matches: usize = matches.len();

    let after = before.elapsed();
    println!("Time elapsed (Part I): {after:2?}");
    println!("(Part I): # of matches: {num_matches}");
    let before = Instant::now();

    let matches: Vec<XmasCoor> = find_xmas(word_search);
    let num_matches: usize = matches.len();

    let after = before.elapsed();
    println!("Time elapsed (Part II): {after:2?}");
    println!("(Part II): # of matches: {num_matches}");

}

#[test]
fn test_small_word_search() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let word_search: Vec<Vec<char>> = filedata.lines()
        .map(|line| line.chars().collect())
        .collect();

    let matches: Vec<XmasCoor> = find_all_matches(&word_search);
    let num_matches: usize = matches.len();

    let coors: Vec<[[usize; 2]; 4]> = matches.iter().map(|xmas| xmas.coors).collect();
    println!("{coors:?}");
    assert_eq!(18, num_matches);
}

#[test]
fn test_small_word_search_xmas() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let word_search: Vec<Vec<char>> = filedata.lines()
        .map(|line| line.chars().collect())
        .collect();

    let matches: Vec<XmasCoor> = find_xmas(word_search);
    let num_matches: usize = matches.len();

    let coors: Vec<[[usize; 2]; 4]> = matches.iter().map(|xmas| xmas.coors).collect();
    println!("{coors:?}");
    assert_eq!(9, num_matches);
}

