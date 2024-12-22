use std::fs;
use std::time::Instant;

const XMAS_ARRAY: [char; 4] = ['X', 'M', 'A', 'S'];
const XMAS_LEN: usize = XMAS_ARRAY.len();

fn find_all_matches(word_search: &Vec<Vec<char>>) -> usize {
    let nrows: usize = word_search.len();
    let ncols: usize = word_search[0].len();
    word_search.iter().for_each(|vec| assert_eq!(vec.len(), ncols));

    let coor_steps: [[i32; 2]; 8] = [
        [0, 1], [1, 1], [1, 0], [1, -1], [0, -1], [-1, -1], [-1, 0], [-1, 1]
    ];

    let coor_with_x = (0..nrows).flat_map(|i| (0..ncols).map(move |j| (i, j)))
        .filter(|&(row, col)| word_search[row][col]=='X');

    let bounded_cases = coor_with_x
        .flat_map(|(row, col)| coor_steps.iter()
            .map(move |&[inc_row, inc_col]| (row, col, inc_row, inc_col))
        )
        .filter(|&(row, _, inc_row, _)| row as i32 + (XMAS_LEN-1) as i32 * inc_row >= 0i32)
        .filter(|&(row, _, inc_row, _)| row as i32 + (XMAS_LEN-1) as i32 * inc_row < nrows as i32)
        .filter(|&(_, col, _, inc_col)| col as i32 + (XMAS_LEN-1) as i32 * inc_col >= 0i32)
        .filter(|&(_, col, _, inc_col)| col as i32 + (XMAS_LEN-1) as i32 * inc_col < ncols as i32);

     let num_matches: usize = bounded_cases
         .filter(|&(row, col, step_row, step_col)| {
             XMAS_ARRAY.iter().enumerate().all(|(i, &chr)| {
                 let row_i = row+i * step_row as usize;
                 let col_i = col+i * step_col as usize;
                 word_search[row_i][col_i] == chr
             })
         })
         .count();

    num_matches
}

fn find_xmas(word_search: Vec<Vec<char>>) -> usize {
    let nrows: usize = word_search.len();
    let ncols = word_search[0].len();
    word_search.iter().for_each(|vec| assert_eq!(vec.len(), ncols));

    let mut coors: [[usize; 2]; 4] = [[0, 0],[0, 0],[0, 0],[0, 0],];
    let mut test_array: [char; 4] = ['A','A','A','A'];

    let product_range = (1..(nrows-1))
        .flat_map(|i| (1..(ncols-1)).map(move |j| (i, j))) ;

    let coor_with_a = product_range
        .filter(|&(row, col)| word_search[row][col] == 'A');

    let mut num_matches: usize = 0;
    for (row, col) in coor_with_a {
        coors[0] = [row-1, col-1];  // code isn't cleaner with a for-loop
        coors[1] = [row-1, col+1];
        coors[2] = [row+1, col-1];
        coors[3] = [row+1, col+1];
        for (i, &[row, col]) in coors.iter().enumerate() { test_array[i] = word_search[row][col]; }
        let num_m: usize = test_array.iter().filter(|&&chr| chr == 'M').count();
        let num_s: usize = test_array.iter().filter(|&&chr| chr == 'S').count();
        let opposing = word_search[row-1][col-1] != word_search[row+1][col+1];
        if num_m == 2 && num_s == 2 && opposing { num_matches += 1 }
    }

    num_matches
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let before = Instant::now();

    let word_search: Vec<Vec<char>> = filedata.lines()
        .map(|line| line.chars().collect())
        .collect();

    let num_matches: usize = find_all_matches(&word_search);

    let after = before.elapsed();
    println!("Time elapsed (Part I): {after:2?}");
    println!("(Part I): # of matches: {num_matches}");
    let before = Instant::now();

    let num_matches: usize = find_xmas(word_search);

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

    let num_matches: usize = find_all_matches(&word_search);

    assert_eq!(18, num_matches);
}

#[test]
fn test_small_word_search_xmas() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename).expect("Cannot open file {filename}");

    let word_search: Vec<Vec<char>> = filedata.lines()
        .map(|line| line.chars().collect())
        .collect();

    let num_matches: usize = find_xmas(word_search);

    assert_eq!(9, num_matches);
}
