use std:: fs;
use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::iter;

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

#[derive(Debug, Clone)]
struct Node<'a> {
    sequence0: String,
    sequence1: String,
    robot1: Robot<'a>,
    robot2: Robot<'a>,
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

    fn get_char(&self) -> char {
        self.map.iter().find_map(|(&key, &val)| if val==self.loc { Some(key) } else { None } ).unwrap()
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
        let Node{ sequence0, sequence1, robot1, robot2 } = self;

        let neighbours: [Option<Node>; 5] = DIRECTIONS
            .map(|(next_chr, _)| { 
                let next_robot1 = robot1.spawn_move(&next_chr)?;
                let mut next_sequence0 = sequence0.clone();
                next_sequence0.push(next_chr);

                let mut next_sequence1 = sequence1.clone();

                let next_robot2 = match next_chr=='A' {
                    true => {
                        next_sequence1.push(robot1.get_char());
                        robot2.spawn_move(&robot1.get_char())?
                    },
                    false => robot2.clone(),
                };
                Some(Node { sequence0: next_sequence0, sequence1: next_sequence1, robot1: next_robot1, robot2: next_robot2})
            });
        neighbours
    }
}

fn make_pad_pad_map(pad1: &BTreeMap<char, [usize; 2]>, pad2: &BTreeMap<char, [usize; 2]>) -> BTreeMap<[char; 2], Vec<String>> {
    let mut arrow_num_map: BTreeMap<[char; 2], Vec<String>> = BTreeMap::new();

    for (&start_chr, &start_loc) in pad2.iter() {

        let robot1_loc = pad1.iter().find_map(|(&key, &val)| if key=='A' {Some(val)} else {None}).unwrap();
        let robot2_loc = start_loc;

        let start_node = Node { 
            sequence0: String::new(), 
            sequence1: String::new(), 
            robot1: Robot{ loc: robot1_loc, map: pad1 },
            robot2: Robot{ loc: robot2_loc, map: pad2 },
        };


        // depth-first search
        let pad1_cols: usize = pad1.values().map(|&[_row, col]| col).max().unwrap()+1;
        let pad1_rows: usize = pad1.values().map(|&[row, _col]| row).max().unwrap()+1;
        let pad2_cols: usize = pad2.values().map(|&[_row, col]| col).max().unwrap()+1;
        let pad2_rows: usize = pad2.values().map(|&[row, _col]| row).max().unwrap()+1;
        let mut record_strings: Vec<Vec<Vec<String>>> = (0..pad2_rows).map(|_| {
            (0..pad2_cols).map(|_| Vec::new()) .collect()
        }) .collect();

        let mut record_scores: Vec<Vec<Vec<Vec<usize>>>> = (0..pad1_rows).map(|_| {
            (0..pad1_cols).map(|_| {
                (0..pad2_rows).map(|_| {
                    (0..pad2_cols).map(|_| usize::MAX).collect()
                }).collect()
            }).collect()
        }) .collect();

        let mut queue: Vec<Node> = Vec::from([start_node.clone()]);
        while let Some(node) = queue.pop() {
            let ([row1, col1], [row2, col2]) = (node.robot1.loc, node.robot2.loc);
            let cost = node.sequence0.len();
            if cost > record_scores[row1][col1][row2][col2] { continue }
            if cost == record_scores[row1][col1][row2][col2] && node.robot1.get_char()=='A' { 
                let mut record_string = node.sequence1.clone();
                record_string.push('A');
                match record_strings[row2][col2].iter().position(|string| string==&record_string) {
                    Some(_) => (),
                    None => record_strings[row2][col2].push(record_string),
                }
                continue 
            }

            if cost < record_scores[row1][col1][row2][col2] {
                record_scores[row1][col1][row2][col2] = cost;
                let mut record_string = node.sequence1.clone();
                record_string.push('A');
                record_strings[row2][col2] = Vec::from([record_string]);
            }

            for next_node in node.next_nodes().into_iter().flatten() { queue.push(next_node) }
        }

        let best_sequences: Vec<Vec<Vec<String>>> = record_strings;

        println!("start: {start_loc:?}, {start_chr}");
        println!("{}", best_sequences.iter()
            // .map(|vec| vec.iter().map(|(string, _)| format!("{:<8}", *string) ).collect())
            .map(|vec| vec.iter().map(|string| format!("{:<14?}", string) ).collect())
            .collect::<Vec<String>>().join("\n"));
        println!("");
        // read all the records here
        for (&end_chr, &end_loc) in pad2.iter() {
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

    let arrow_num_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP, &NUMPAD_MAP);
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP, &ARROWPAD_MAP);

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 3, 3, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    println!("(Part  I) complexity sum: {complexity}");

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 26, 26, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    println!("(Part II) complexity sum: {complexity}");
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let filedata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let arrow_num_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP, &NUMPAD_MAP);
    let arrow_arrow_map: BTreeMap<[char; 2], Vec<String>> = make_pad_pad_map(&ARROWPAD_MAP, &ARROWPAD_MAP);

    let mut cache: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let complexity: usize = filedata.lines()
        .map(|code| (code, recursive_sublengths(code.to_string(), 3, 3, &arrow_num_map, &arrow_arrow_map, &mut cache)))
        .map(|(code, num)| code_to_num(code) * num)
        .sum();

    assert_eq!(126_384, complexity);
}

