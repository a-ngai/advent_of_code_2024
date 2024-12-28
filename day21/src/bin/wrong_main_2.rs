use std:: fs;
use std::collections::{BTreeMap, BinaryHeap};
use std::sync::LazyLock;
use std::iter;
use std::cmp::{Ordering, Reverse};

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

static NUMPAD_MAP: LazyLock<BTreeMap<char, [usize; 2]>> = LazyLock::new(|| 
    BTreeMap::from([
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

static ARROWPAD_MAP: LazyLock<BTreeMap<char, [usize; 2]>> = LazyLock::new(|| 
    BTreeMap::from([
        ('A', [0, 2]),
        ('^', [0, 1]),
        ('<', [1, 0]),
        ('v', [1, 1]),
        ('>', [1, 2]),
    ]));

fn transition_cost(transition: &Vec<[char; 2]>) -> usize {
    transition.into_iter()
        .map(|&[prev_char, next_char]| if prev_char==next_char {1} else {10})
        .sum()
}

fn sequence_to_transition(sequence: &str) -> impl Iterator<Item=[char; 2]> + '_ {
    // implicitly starts at 'A'
    iter::once('A').chain(sequence.chars())
        .zip(sequence.chars())
        .map(|(prev_char, next_char)| [prev_char, next_char])
}

fn arrowpad_distance(chr1: char, chr2: char) -> usize {
    let &[row1, col1] = ARROWPAD_MAP.get(&chr1)
        .unwrap_or_else(|| panic!("chr1 ({chr1}) is not recognized"));
    let &[row2, col2] = ARROWPAD_MAP.get(&chr2)
        .unwrap_or_else(|| panic!("chr2 ({chr2}) is not recognized"));
    // let dist = match chr2==chr2 {
    //     true => 1,
    //     false => 10 * ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize,
    // };
    let dist = match chr1==chr2 {
        true => 0,
        false => 1,
    };
    dist
}

fn sequence_cost(sequence: &str) -> usize {
    sequence_to_transition(sequence)
        //.map(|[prev_char, next_char]| if prev_char==next_char {1} else {10})
        .map(|[prev_char, next_char]| arrowpad_distance(prev_char, next_char))
        .sum()
}

#[derive(Debug, Eq, PartialEq)]
struct Node {
    loc: [usize; 2],
    sequence: String
}

static DIRECTIONS: [(char, [isize; 2]); 5] = [
    ('^', [-1,  0]),
    ('>', [ 0,  1]),
    ('v', [ 1,  0]),
    ('<', [ 0, -1]),
    ('A', [ 0,  0]),
];

impl Node {
    fn next_nodes(&self) -> [Option<Node>; 5] {
        let Node{ loc:[row, col], sequence} = self;

        let neighbours: [Option<Node>; 5] = DIRECTIONS
            .map(|(next_chr, [step_row, step_col])| { 
                let next_row = row.checked_add_signed(step_row)?;
                let next_col = col.checked_add_signed(step_col)?;
                let mut next_sequence = sequence.clone();
                next_sequence.push(next_chr);
                Some(Node { sequence: next_sequence, loc: [next_row, next_col] })
            });
        neighbours
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(sequence_cost(&self.sequence).cmp(&sequence_cost(&other.sequence)))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        sequence_cost(&self.sequence).cmp(&sequence_cost(&other.sequence))
    }
}

fn make_pad_pad_map(pad_map: &BTreeMap<char, [usize; 2]>) -> BTreeMap<[char; 2], Vec<[char; 2]>> {
    // the breadth-first search must actually be two-layers deep!
    // e.g. we must get the sequence v>A. It matters if we press 'A' on v first, or on v first. The
    // former takes fewer button presses!
    // The human->robot has equivalent paths, but for robot->robot those are now inequivalent!
    // So for the breadth-first search, we try human->robot->robot.
    let mut arrow_num_map: BTreeMap<[char; 2], Vec<[char; 2]>> = BTreeMap::new();

    let numpad_cols: usize = pad_map.values().map(|&[_row, col]| col).max().unwrap()+1;
    let numpad_rows: usize = pad_map.values().map(|&[row, _col]| row).max().unwrap()+1;
    for (&start_chr, &start_loc) in pad_map.iter() {
        let mut best_sequences: Vec<Vec<(String, usize)>> = (0..numpad_rows)
            .map(|_| (0..numpad_cols).map(|_| (String::new(), usize::MAX)).collect())
            .collect();
        let mut queue: BinaryHeap<Reverse<Node>> = BinaryHeap::from([Reverse(Node{sequence:String::new(), loc:start_loc}),]);
        while let Some(Reverse(node)) = queue.pop() {
            let next_nodes = node.next_nodes();
            for next_node in next_nodes.into_iter().flatten() {
                let next_cost = sequence_cost(&next_node.sequence);
                let [next_row, next_col] = next_node.loc;
                let is_empty_pad = ! pad_map.values().any(|loc| loc==&next_node.loc);
                let invalid_loc = {
                    next_row >= numpad_rows 
                    || next_col >= numpad_cols 
                    || is_empty_pad
                };
                if invalid_loc {continue}

                let record_cost = best_sequences[next_row][next_col].1;
                if next_cost >= record_cost { continue }
                if next_node.sequence.chars().last().unwrap() == 'A' {
                    best_sequences[next_row][next_col] = (next_node.sequence.clone(), next_cost);
                    continue
                }
                queue.push(Reverse(next_node));
            }
        }
        println!("start: {start_loc:?}, {start_chr}");
        println!("{}", best_sequences.iter()
            .map(|vec| vec.iter().map(|(string, _)| format!("{:<8}", *string) ).collect())
            .collect::<Vec<String>>().join("\n"));
        println!("");
        // read all the records here
        for (&end_chr, &end_loc) in pad_map.iter() {
            let [end_row, end_col] = end_loc;
            let new_vec: Vec<[char; 2]> = sequence_to_transition(&(best_sequences[end_row][end_col].0)).collect();
            arrow_num_map
                .entry([start_chr, end_chr])
                .and_modify(|val| *val=new_vec.clone() ).or_insert(new_vec);

        }

    }

    //println!("arrow_num_map: {arrow_num_map:?}");
    arrow_num_map
}

fn make_arrow_num_map() -> BTreeMap<[char; 2], Vec<[char; 2]>> {
    make_pad_pad_map(&NUMPAD_MAP)
}

fn make_arrow_arrow_map() -> BTreeMap<[char; 2], Vec<[char; 2]>> {
    make_pad_pad_map(&ARROWPAD_MAP)
}

fn pad_to_pad(movement: &[char; 2], pad_map: &BTreeMap<[char; 2], Vec<[char; 2]>>) -> Vec<[char; 2]> {
    let movement: Vec<[char; 2]> = pad_map.get(movement)
        .unwrap_or_else(|| panic!("pad map does not contain key {movement:?}"))
        .clone();
    movement
}

fn sequence_to_string(sequence: &Vec<[char; 2]>) -> String {
        sequence.iter()
        .map(|&[_, chr]| chr)
        .collect::<String>()
}

fn code_to_numeric_sequence(
    code: &str, 
    arrow_number_map: &BTreeMap<[char; 2], Vec<[char; 2]>>,
    arrow_arrow_map: &BTreeMap<[char; 2], Vec<[char; 2]>>
    ) -> (usize, String) {

    let numeric_part: usize = code.chars()
        .filter(|chr| chr.is_ascii_digit())
        .collect::<String>()
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("Cannot parse ({code}) to usize"));

    let sequence: Vec<[char; 2]> = ['A'].into_iter().chain(code.chars())
        .zip(code.chars())
        .map(|(prev_chr, next_chr)| [prev_chr, next_chr])
        .collect();
    println!("    len: {}, {}", sequence.len(), sequence_to_string(&sequence));

    let sequence: Vec<[char; 2]> = sequence.iter()
        .flat_map(|movement| pad_to_pad(movement, arrow_number_map))
        .collect();
    println!("    len: {}, {}", sequence.len(), sequence_to_string(&sequence));

    let sequence: Vec<[char; 2]> = sequence.iter()
        .flat_map(|movement| pad_to_pad(movement, arrow_arrow_map))
        .collect();
    println!("    len: {}, {}", sequence.len(), sequence_to_string(&sequence));

    let sequence: Vec<[char; 2]> = sequence.iter()
        .flat_map(|movement| pad_to_pad(movement, arrow_arrow_map))
        .collect();
    println!("    len: {}, {}", sequence.len(), sequence_to_string(&sequence));

    (numeric_part, sequence_to_string(&sequence))
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let arrow_num_map: BTreeMap<[char; 2], Vec<[char; 2]>> = make_arrow_num_map();
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<[char; 2]>> = make_arrow_arrow_map();

    let complexity: usize = filedata.lines()
        .map(|code| code_to_numeric_sequence(code, &arrow_num_map, &arrow_arrow_map))
        .map(|(num, sequence)| num * sequence.len())
        .sum();

    println!("(Part  I) complexity sum: {complexity}");
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let arrow_num_map: BTreeMap<[char; 2], Vec<[char; 2]>> = make_arrow_num_map();
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<[char; 2]>> = make_arrow_arrow_map();

    let complexity: usize = filedata.lines()
        .map(|code| code_to_numeric_sequence(code, &arrow_num_map, &arrow_arrow_map))
        .inspect(|(num, sequence)| println!("{num}, len: {}, {}", sequence.len(), sequence))
        .map(|(num, sequence)| num * sequence.len())
        .sum();

    assert_eq!(126_384, complexity);
}

