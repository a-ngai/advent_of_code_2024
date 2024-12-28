use std:: fs;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::sync::LazyLock;
use std::iter;

// The order of the arrows matter; if starting from A, then ^< is better than <^ At each level,
// sort the clusters of arrows so that they have the closest distance to each other The only reason
// we can do it level by level, is because each robot has to press 'A' to finish an input. I also
// have to mind the gap; this I will fix later
//
// Or a different strategy: depth-first with memoization. If on part 2 I have to nest many more
// robots together, increasing the string length by 1 each time will be quite slow at O(a^n). If
// memoized, then the running time will be O(n), a much better alternative!
//
// Another benefit of the depth-first search; it's a "brute-force" method with the guarantee of
// finding the correct solution, and I don't have to worry missing edge cases.
//
// Another way is to do it like the splitting-rocks puzzle. What matters aren't the arrows
// themselves, but how to get *between* the arrows. i.e. we don't care about the sequence ^>>, we
// only care about ^ -> > and > -> >, because these instructions are functionally independent from
// each other. On each level we tabulate how many we need, and on the next level we calculate how
// these split off into sub-instructions. This is a mapping from sequences to sequences!

static NUMPAD_MAP: LazyLock<HashMap<char, [usize; 2]>> = LazyLock::new(|| 
    HashMap::from([
        ('0', [3, 1]),
        ('A', [3, 2]),
        ('1', [2, 0]),
        ('2', [2, 1]),
        ('3', [2, 2]),
        ('4', [1, 0]),
        ('5', [1, 1]),
        ('6', [1, 2]),
        ('7', [0, 0]),
        ('8', [0, 1]),
        ('9', [0, 2]),
    ]));

static ARROWPAD_MAP: LazyLock<HashMap<char, [usize; 2]>> = LazyLock::new(|| 
    HashMap::from([
        ('A', [0, 2]),
        ('^', [0, 1]),
        ('<', [1, 0]),
        ('v', [1, 1]),
        ('>', [1, 2]),
    ]));

struct Movement {
    u: usize,
    r: usize,
    d: usize,
    l: usize,
}

impl Default for Movement {
    fn default() -> Self {
        Movement { u: 0, r: 0, d: 0, l: 0}
    }
}

fn pad_distance(chr1: &char, chr2: &char, is_numpad: bool) -> usize {
    println!("{chr1}, {chr2}, {is_numpad}");
    let &[row1, col1] =  match is_numpad {
        true => NUMPAD_MAP.get(&chr1).unwrap(),
        false => ARROWPAD_MAP.get(&chr1).unwrap(),
    };
    let &[row2, col2] =  match is_numpad {
        true => NUMPAD_MAP.get(&chr2).unwrap(),
        false => ARROWPAD_MAP.get(&chr2).unwrap(),
    };

    let man_dist: usize = {
        (row1 as isize-row2 as isize).abs() 
            + (col1 as isize-col2 as isize).abs()
    } as usize;
    let switch_dist: usize = match row1==row2 || col1==col2 {
        true => 1,
        false => 0,
    };
    man_dist + switch_dist
}

fn order_instructions(unordered: Movement, curr_chr: char, is_numpad: bool) -> Vec<char> {
    // insert the 'A' instruction at the end

    let Movement { u: move_u, r: move_r, d: move_d, l: move_l} = unordered;
    let mut move_count: [(char, usize); 4] = [('^', move_u), ('>', move_r), ('v', move_d), ('<', move_l)];

    move_count.sort_by(|(chr1, ..), (chr2, ..)| {
        let dist_1 = pad_distance(chr1, &curr_chr, is_numpad);
        let dist_2 = pad_distance(chr2, &curr_chr, is_numpad);
        dist_1.cmp(&dist_2)

    });
    let mut instructions: Vec<char> = move_count.into_iter()
        .flat_map(|(chr, num)| iter::repeat(chr).take(num))
        .collect();
    
    instructions.push('A');
    // println!("{curr_chr}, instructions: {}", instructions.iter().collect::<String>());

    instructions
}

fn diff_to_movement(row_diff: i8, col_diff: i8) -> Movement {
    let mut instructions = Movement::default();
    match row_diff.cmp(&0) {
        Ordering::Greater => {instructions.d = row_diff.abs() as usize},
        Ordering::Less => {instructions.u = row_diff.abs() as usize},
        Ordering::Equal => (),
    }
    match col_diff.cmp(&0) {
        Ordering::Greater => {instructions.r = col_diff.abs() as usize},
        Ordering::Less => {instructions.l = col_diff.abs() as usize},
        Ordering::Equal => (),
    }

    instructions
}

fn arrowpad_to_arrowpad(curr_chr: char, in_chr: char) -> Vec<char> {
    let &[in_row, in_col] = ARROWPAD_MAP.get(&in_chr)
        .unwrap_or_else(|| panic!("Cannot parse ({in_chr}) to usize"));
    let &[curr_row, curr_col] = ARROWPAD_MAP.get(&curr_chr)
        .unwrap_or_else(|| panic!("Cannot parse ({curr_chr}) to usize"));

    let row_diff: i8 = in_row as i8 - curr_row as i8;
    let col_diff: i8 = in_col as i8 - curr_col as i8;

    let unordered_instructions = diff_to_movement(row_diff, col_diff);
    order_instructions(unordered_instructions, curr_chr, false)
}

fn arrowpad_to_numpad(curr_chr: char, in_chr: char) -> Vec<char> {
    let &[in_row, in_col] = NUMPAD_MAP.get(&in_chr)
        .unwrap_or_else(|| panic!("Cannot parse ({in_chr}) to usize"));
    let &[curr_row, curr_col] = NUMPAD_MAP.get(&curr_chr)
        .unwrap_or_else(|| panic!("Cannot parse ({curr_chr}) to usize"));

    let row_diff: i8 = in_row as i8 - curr_row as i8;
    let col_diff: i8 = in_col as i8 - curr_col as i8;

    let unordered_instructions = diff_to_movement(row_diff, col_diff);
    // do the closest arrow first!
    order_instructions(unordered_instructions, curr_chr, true)
}

fn code_to_numeric_sequence(code: &str) -> (usize, String) {
    let numeric_part: usize = code.chars()
        .filter(|chr| chr.is_ascii_digit())
        .collect::<String>()
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("Cannot parse ({code}) to usize"));

    let instructions: String = ['A'].into_iter().chain(code.chars())
        .zip(code.chars())
        .flat_map(|(chr1, chr2)| arrowpad_to_numpad(chr1, chr2))
        .collect();
    println!("    len: {}, {}", instructions.len(), instructions);
    let instructions: String = ['A'].into_iter().chain(instructions.chars())
        .zip(instructions.chars())
        .flat_map(|(chr1, chr2)| arrowpad_to_arrowpad(chr1, chr2))
        .collect();
    println!("    len: {}, {}", instructions.len(), instructions);
    let instructions: String = ['A'].into_iter().chain(instructions.chars())
        .zip(instructions.chars())
        .flat_map(|(chr1, chr2)| arrowpad_to_arrowpad(chr1, chr2))
        .collect();
    println!("    len: {}, {}", instructions.len(), instructions);

    (numeric_part, instructions)
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let complexity: usize = filedata.lines()
        .map(code_to_numeric_sequence)
        .map(|(num, sequence)| num * sequence.len())
        .sum();

    println!("(Part  I) complexity sum: {complexity}");
}


#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let complexity: usize = filedata.lines()
        .map(code_to_numeric_sequence)
        .inspect(|(num, sequence)| println!("{num}, len: {}, {}", sequence.len(), sequence))
        .map(|(num, sequence)| num * sequence.len())
        .sum();

    assert_eq!(126_384, complexity);
}



