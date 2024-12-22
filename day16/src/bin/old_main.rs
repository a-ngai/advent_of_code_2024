use std::fs;
use std::collections::VecDeque;
use std::time::Instant;

// Currently Part I is very slow! ~800ms for --release mode.
// Future optimization:
// - not sure, I don't know which part is taking so long
#[derive(PartialEq, Clone, Debug, Copy)]
enum Direction {
    U,
    R,
    D,
    L,
}

const ALL_DIRECTIONS: [Direction; 4] = [Direction::U, Direction::R, Direction::D, Direction::L];

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::U => Direction::D,
            Direction::R => Direction::L,
            Direction::D => Direction::U,
            Direction::L => Direction::R,
        }
    }

    fn step(&self) -> [isize; 2] {
        match self {
            Direction::U => [-1,  0],
            Direction::R => [ 0,  1],
            Direction::D => [ 1,  0],
            Direction::L => [ 0, -1],
        }
    }

    fn loc(&self) -> usize {
        match self {
            Direction::U => 0,
            Direction::R => 1,
            Direction::D => 2,
            Direction::L => 3,
        }
    }
}

#[derive(Clone, Debug)]
struct Path {
    row: usize,
    col: usize,
    score: u32,
    dir: Direction,
}

fn next_steps(item: &Path) -> [Path; 4] {
    // either a new direction, or a step in the current direction
    let Path { row, col, dir, score, .. } = item;
    let next_steps = ALL_DIRECTIONS.map(|next_dir| {
        let [row_step, col_step] = next_dir.step();
        let [next_row, next_col]: [usize; 2] = match next_dir == *dir {
            true => [ ((*row as isize) + row_step) as usize, ((*col as isize) + col_step) as usize ],
            false => [ *row, *col ]
        };
        let opposite_dir = *dir == next_dir.opposite();
        let perpendicular_dir = *dir != next_dir && !opposite_dir;
        let same_dir = *dir == next_dir;
        let next_score = match [opposite_dir, perpendicular_dir, same_dir] {
            [true, false, false] => score + 2000,
            [false, true, false] => score + 1000,
            [false, false, true] => score + 1,
            [_, _, _] => panic!("more than one mutually exclusive condition met!"),
        };
        Path { row: next_row, col: next_col, dir: next_dir, score: next_score }
    });
    next_steps
}

fn next_steps_backwards(item: &Path) -> [Path; 4] {
    let Path { row, col, dir, .. } = item;
    let next_steps = ALL_DIRECTIONS.map(|next_dir| {
        let [row_step, col_step] = next_dir.step();
        let [next_row, next_col]: [usize; 2] = match next_dir == *dir {
            true => [ ((*row as isize) - row_step) as usize, ((*col as isize) - col_step) as usize ],
            false => [ *row, *col ]
        };
        Path { row: next_row, col: next_col, dir: next_dir, score: 0 } // score is irrelevant
    });
    next_steps
}

fn update_score_all_dir(row: &usize, col: &usize, score: &u32, dir: &Direction, score_record: &mut Vec<Vec<[u32; 4]>>) {
    for next_dir in ALL_DIRECTIONS {
        let opposite_dir = *dir == next_dir.opposite();
        let perpendicular_dir = *dir != next_dir && !opposite_dir;
        let same_dir = *dir == next_dir;
        let next_score = match [opposite_dir, perpendicular_dir, same_dir] {
            [true, false, false] => *score + 2000,
            [false, true, false] => *score + 1000,
            [false, false, true] => *score,
            [_, _, _] => panic!("more than one mutually exclusive condition met!"),
        };
        let memory = &mut score_record[*row][*col][next_dir.loc()];
        if *memory > next_score { *memory = next_score }
    }
}

fn next_new_steps(item: &Path, map: &Vec<Vec<char>>, record_score: &mut Vec<Vec<[u32; 4]>>) -> [Option<Path>; 4] {
    // directions if '.' or 'E', and if not opposite self.dir
    // not checking opposite self.dir, since that is handled when checking with the HashSet
    let next_items = next_steps(item);
    let Path {row, col, score, dir } = item;
    update_score_all_dir(&row, &col, &score, dir, record_score);
    let next_new_items: [Option<Path>; 4] = next_items.map(|item| {
        let Path { row: next_row, col: next_col, dir: next_dir, score: next_score} = item;
        let next_chr = map[next_row][next_col];
        let valid_char: bool = next_chr == '.' || next_chr == 'E' || next_chr == 'S';
        let memory_score = &mut record_score[next_row][next_col][next_dir.loc()];

        let has_low_score_for_next_dir = next_score <= *memory_score;

        let not_opposite = next_dir != dir.opposite();
        if has_low_score_for_next_dir && valid_char && not_opposite  {
            // update_score_all_dir(&next_row, &next_col, &next_score, dir, record_score);
            *memory_score = next_score;
            Some(item)
        } else { None }
    });
    next_new_items
}

fn best_path_score(map: &Vec<Vec<char>>) -> (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) {
    // Make two stacks. The idea is to do a Dijkstra-like algorithm. The only difference, is that
    // the distance is determed by how many turns, which makes things slight more complicated. This
    // can be circumvented by having two stacks:
    // - first stack has all propagations that don't require a direction change
    // - second stack has all propagations that require a direction change

    let start_row = map.iter().position(|rowdata| rowdata.contains(&'S'))
        .unwrap_or_else(|| panic!("map does not contain 'S'"));
    let start_col = map[start_row].iter().position(|chr| chr == &'S')
        .unwrap();
    let start_dir = Direction::R;

    let num_rows = map.len();
    let num_cols = map[0].len();
    let mut exit_paths: Vec<Path> = Vec::new();
    let start_path: Path = Path { row: start_row, col: start_col, score: 0, dir: start_dir};
    let mut score_record: Vec<Vec<[u32; 4]>> = (0..num_rows)
        .map(|_| (0..num_cols)
            .map(|_| [u32::MAX; 4]).collect()
        )
        .collect();

    score_record[start_row][start_col][start_dir.loc()] = 0;

    let mut curr_stack: VecDeque<Path> = VecDeque::from([start_path,]);
    let mut lowest_score: u32 = u32::MAX;  // default value, final loop when changed
    while curr_stack.len() > 0 && lowest_score == u32::MAX {
        curr_stack.make_contiguous().sort_by(|path1, path2| path1.score.cmp(&path2.score));

        let mut next_stack: VecDeque<Path> = VecDeque::new();
        while let Some(item) = curr_stack.pop_back() {

            let Path { row, col, ..} = item;
            let chr = map[row][col];
            if chr == 'E' { 
                if item.score <= lowest_score { 
                    lowest_score = item.score;
                    exit_paths.push(item.clone());
                }
            }

            let new_steps: [Option<Path>; 4] = next_new_steps(&item, &map, &mut score_record);
            for next_item in new_steps.into_iter().filter_map(|item| item) {
                let is_new_dir: bool = item.dir != next_item.dir;
                if item.score > lowest_score { continue }
                match is_new_dir {
                    true => { next_stack.push_front(next_item) },
                    false => { curr_stack.push_front(next_item) },
                }
            }
        }
        curr_stack = next_stack;
    }
    if lowest_score == u32::MAX { panic!("algorithm cannot find the end point!") }

    (lowest_score, score_record, exit_paths)
}

fn make_scalar_score_map(score_record: &Vec<Vec<[u32; 4]>>) -> Vec<Vec<u32>> {
    score_record
        .into_iter()
        .map(|rowdata| rowdata.into_iter()
            .map(|memory| *memory.iter().min().unwrap())
            .collect()
        )
        .collect()
}

fn next_steps_lower_score(item: &Path, score_record: &Vec<Vec<[u32; 4]>>, visited: &mut Vec<Vec<bool>>) -> [Option<Path>; 4] {
    let &Path { row, col, dir, ..} = item;
    let score = score_record[row][col][dir.loc()];
    
    let next_items = next_steps_backwards(item);
    let next_new_items: [Option<Path>; 4] = next_items.map(|item| {
        let Path { row: next_row, col: next_col, dir: next_dir, .. } = item;
        let next_score = score_record[next_row][next_col][next_dir.loc()];
        let next_path = Path { row: next_row, col: next_col, score: next_score, dir: next_dir};
        let has_lower_score = next_score < score;
        if has_lower_score  {
            visited[next_row][next_col] = true;
            Some(next_path)
        } else { None }
    });
    next_new_items
}

fn get_path_with_lowering_score(entry_paths: &Vec<Path>, score_record: &Vec<Vec<[u32; 4]>>) -> Vec<Vec<bool>> {


    let num_rows = score_record.len();
    let num_cols = score_record[0].len();
    let mut backtracked: Vec<Vec<bool>> = (0..num_rows)
        .map(|_| (0..num_cols)
            .map(|_| false).collect()
        )
        .collect();

    for entry_path in entry_paths {
        let Path { row, col, score, dir } = *entry_path;
        let start_path = Path {row, col, score, dir };
        backtracked[row][col] = true;

        let mut curr_stack: VecDeque<Path> = VecDeque::from([start_path,]);
        while curr_stack.len() > 0 {
            while let Some(item) = curr_stack.pop_back() {
                if item.score == 0 { 
                    let Path { row, col, ..} = item;
                    backtracked[row][col] = true;
                    continue 
                }
                let new_steps: [Option<Path>; 4] = next_steps_lower_score(&item, score_record, &mut backtracked);
                for next_item in new_steps.into_iter().filter_map(|item| item) {
                    curr_stack.push_front(next_item)
                }
            }
        }
    }
    backtracked
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);

    let after = before.elapsed();
    println!("(Part  I) best path score: {score}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    let backtracked = get_path_with_lowering_score(&exit_paths, &score_record);
    let num_tiles = backtracked.into_iter()
        .flat_map(|rowdata| rowdata.into_iter())
        .filter(|&val| val)
        .count() as u32;
    let after = before.elapsed();
    println!("(Part II) num tiles: {num_tiles}");
    println!("(Part II) time elapsed: {after:.2?}");
}

#[test]
fn first_small_test() {
    let filename: &str = "test_input_1.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);
    assert_eq!(7036, score);

    let mut temp_map = score_map.clone();
    temp_map.iter_mut()
        .for_each(|rowdata| rowdata.iter_mut().for_each(|num| {if *num == u32::MAX { *num = 0 }}));
    // temp_map.iter().for_each(|row| println!("{row:?}"));
    temp_map.iter().for_each(|row| println!("{}", row.iter().map(|num| num.to_string()).map(|num_string| format!("{num_string:06}")).collect::<String>()));

    let backtracked = get_path_with_lowering_score(&exit_paths, &score_record);
    let num_tiles = backtracked.iter()
        .flat_map(|rowdata| rowdata.into_iter())
        .filter(|&&val| val)
        .count() as u32;

    backtracked.iter().for_each(|vec| println!("{}", vec.iter().map(|val| match val { true => "O", false => "." } ).map(|string| format!("{string}")).collect::<String>()));

    assert_eq!(45, num_tiles);
}

#[test]
fn second_small_test() {
    let filename: &str = "test_input_2.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);
    assert_eq!(11_048, score);

    let backtracked = get_path_with_lowering_score(&exit_paths, &score_record);
    let num_tiles = backtracked.iter()
        .flat_map(|rowdata| rowdata.into_iter())
        .filter(|&&val| val)
        .count() as u32;

    backtracked.iter().for_each(|vec| println!("{}", vec.iter().map(|val| match val { true => "O", false => "." } ).map(|string| format!("{string}")).collect::<String>()));

    assert_eq!(64, num_tiles);
}

#[test]
fn former_tests() {
    let filename: &str = "test_input_2.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);

    let mut temp_map = score_map.clone();
    temp_map.iter_mut()
        .for_each(|rowdata| rowdata.iter_mut().for_each(|num| {if *num == u32::MAX { *num = 0 }}));
    // temp_map.iter().for_each(|row| println!("{row:?}"));
    temp_map.iter().for_each(|row| println!("{}", row.iter().map(|num| num.to_string()).map(|num_string| format!("{num_string:06}")).collect::<String>()));

    assert_eq!(11_048, score);

    let filename: &str = "test_input_1.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);
    assert_eq!(7036, score);

    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let map: Vec<Vec<char>> = textdata.lines()
        .map(|string| string.chars().collect::<Vec<char>>())
        .collect();

    let (score, score_record, exit_paths): (u32, Vec<Vec<[u32; 4]>>, Vec<Path>) = best_path_score(&map);
    let score_map = make_scalar_score_map(&score_record);
    assert_eq!(91464, score);


}

