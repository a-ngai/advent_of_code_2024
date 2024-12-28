use std::fs;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::iter;
use std::time::Instant;
use std::cmp::Ordering;

// Possibilities for optimizations:
// - For the memoized depth-first search, some of the sequences e.g. "<v<A" will always be smaller
// than sequences which don't have this back-and-forth e.g. "<<vA". Eliminate these patterns to
// curtail the search space
// - Another possible way (which is not certain) is to find for each transition (e.g. from "1" to
// "9" on the arrow pad) the optimal path (e.g. ">>^^"). I presume that on the first level,
// multiple alternative sequences have the same length, but on deeper levels they one will be
// shorter. By the time we recurse down to 25, I'm quite sure there are no more equivalent paths.
// We can then take these now-uniquely-determined paths, and do a simple mapping procedure where we
// consider not the sequence itself, but the transitions e.g. for the sequence ^^>, we consider (^
// -> ^), (^ -> >). This way, we can map individual transitions to multiple transitions when we go
// down a layer. If we consider a vector space of transitions, then going down a layer is a linear
// transformation in this vector space, and we can use simple matrix multiplation to go to
// arbitrary depths. This only works if the linear transformation is unique i.e. we've recursed
// enough to completely disambiguate the paths which intially appear to have the same length!

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

#[derive(Debug, Clone)]
struct Node<'a> {
    sequence: String,
    robot: Robot<'a>,
}

#[derive(Debug, Clone, Copy)]
struct Robot<'a> {
    loc: [usize; 2],
    map: &'a BTreeMap<char, [usize; 2]>,
}

impl Robot<'_> {
    fn check_in_bounds(&self, loc: [usize; 2]) -> bool {
        self.map.values().any(|&val| loc==val)
    }

    fn spawn_move(self, chr: &char) -> Option<Self> {
        let [row, col] = self.loc;
        let &[step_row, step_col] = DIRECTIONS.iter().find_map(|(next_chr, step_loc)| if next_chr==chr { Some(step_loc)} else { None }).unwrap();
        let next_loc = [row.checked_add_signed(step_row)?, col.checked_add_signed(step_col)?];
        match self.check_in_bounds(next_loc) {
            true => Some(Self{ loc: next_loc.clone(), map: self.map}),
            false => None,
        }
    }
}

static DIRECTIONS: [(char, [isize; 2]); 5] = [
    ('^', [-1,  0]),
    ('>', [ 0,  1]),
    ('v', [ 1,  0]),
    ('<', [ 0, -1]),
    ('A', [ 0,  0]),
];

impl<'a> Node<'a> {
    fn next_nodes(&self) -> [Option<Node<'a>>; 5] {
        let Node{ sequence, robot} = self;

        let neighbours: [Option<Node>; 5] = DIRECTIONS
            .map(|(next_chr, _)| { 
                let next_robot = robot.spawn_move(&next_chr)?;
                let mut next_sequence = sequence.clone();
                next_sequence.push(next_chr);

                Some(Node { sequence: next_sequence, robot: next_robot })
            });
        neighbours
    }
}

fn make_pad_pad_map(pad: &BTreeMap<char, [usize; 2]>) -> BTreeMap<[char; 2], Vec<String>> {
    let mut arrow_num_map: BTreeMap<[char; 2], Vec<String>> = BTreeMap::new();

    for (&start_chr, &start_loc) in pad.iter() {

        let start_node = Node { 
            sequence: String::new(), 
            robot: Robot{ loc: start_loc, map: pad },
        };

        // depth-first search
        let pad_cols: usize = pad.values().map(|&[_row, col]| col).max().unwrap()+1;
        let pad_rows: usize = pad.values().map(|&[row, _col]| row).max().unwrap()+1;
        let mut record_strings: Vec<Vec<Vec<String>>> = (0..pad_rows).map(|_| {
            (0..pad_cols).map(|_| Vec::new()) .collect()
        }) .collect();

        let mut record_scores: Vec<Vec<usize>> = (0..pad_rows).map(|_| {
            (0..pad_cols).map(|_| usize::MAX).collect()
        }) .collect();

        let mut queue: Vec<Node> = Vec::from([start_node.clone()]);
        while let Some(node) = queue.pop() {
            let [row, col] = node.robot.loc;
            let cost = node.sequence.len();
            match cost.cmp(&record_scores[row][col]) {
                Ordering::Greater => { continue },
                Ordering::Equal => {
                    let record_string: String = node.sequence.clone() + &'A'.to_string();
                    if let None = record_strings[row][col].iter().position(|string| string==&record_string) {
                        record_strings[row][col].push(record_string)
                    }
                }
                Ordering::Less => {
                    record_scores[row][col] = cost;
                    let record_string: String = node.sequence.clone() + &'A'.to_string();
                    record_strings[row][col] = Vec::from([record_string]);
                }
            }

            node.next_nodes()
                .into_iter()
                .flatten()
                .for_each(|next_node| queue.push(next_node));
        }

        let best_sequences: Vec<Vec<Vec<String>>> = record_strings;
        for (&end_chr, &end_loc) in pad.iter() {
            let [end_row, end_col] = end_loc;
            let hand_sequence = (best_sequences[end_row][end_col]).clone();
            arrow_num_map
                .entry([start_chr, end_chr])
                .and_modify(|val| *val=hand_sequence.clone() ).or_insert(hand_sequence);
        }
    }

    arrow_num_map
}

fn paths(chr1: &char, chr2: &char, pad_pad_map: &BTreeMap<[char; 2], Vec<String>>) -> Vec<String> {
    pad_pad_map.get(&[*chr1, *chr2]).unwrap().clone()
}

fn recursive_sublengths(cycle: String, 
    depth: usize, 
    target_depth: usize,
    arrow_number_map: &BTreeMap<[char; 2], Vec<String>>,
    arrow_arrow_map: &BTreeMap<[char; 2], Vec<String>>,
    cache: &mut BTreeMap<(usize, String), usize>) -> usize {
    if depth == 0 { return cycle.len() }

    match cache.get(&(depth, cycle.clone())) {
        Some(&num) => return num,
        None => (),
    }

    let mut cumul_sum = 0;
    for (prev_chr, next_chr) in iter::once('A') .chain(cycle.chars()) .zip(cycle.chars()) {
        let possible_subsequences = if depth == target_depth {
            paths(&prev_chr, &next_chr, arrow_number_map)
        } else {
            paths(&prev_chr, &next_chr, arrow_arrow_map) 
        };
        let min_sequence: usize = possible_subsequences.into_iter().map(|string| {
            recursive_sublengths(string, depth-1, target_depth, arrow_number_map, arrow_arrow_map, cache)
        }).min().unwrap();
        cumul_sum += min_sequence;
    }

    cache.entry((depth, cycle.clone())).or_insert(cumul_sum);

    cumul_sum

}

fn code_to_num(code: &str) -> usize {
    let numeric_part: usize = code.chars()
        .filter(|chr| chr.is_ascii_digit())
        .collect::<String>()
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("Cannot parse ({code}) to usize"));
    numeric_part
}

fn main() {
    let filename: &str = "input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let arrow_num_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&NUMPAD_MAP);
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP);

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 3, 3, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    let after = before.elapsed();
    println!("(Part  I) complexity sum: {complexity}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 26, 26, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    let after = before.elapsed();
    println!("(Part II) complexity sum: {complexity}");
    println!("(Part II) time elapsed: {after:.2?}");
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let arrow_num_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&NUMPAD_MAP);
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP);

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 3, 3, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    assert_eq!(126_384, complexity);
}
