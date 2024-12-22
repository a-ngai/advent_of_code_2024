use std::fs;
use std::collections::{BTreeSet, BTreeMap};
use std::time::Instant;

static STEPS: [Direction; 4] = [Direction::UP, Direction::RIGHT, Direction::DOWN, Direction::LEFT];

#[derive(Clone, Copy)]
enum Direction {
    UP,
    RIGHT,
    DOWN,
    LEFT,
}

impl Direction {
    fn step(self) -> [isize; 2] {
        let [row_step, col_step] = match self {
            Self::UP    => [-1,  0],
            Self::RIGHT => [ 0,  1],
            Self::DOWN  => [ 1,  0],
            Self::LEFT  => [ 0, -1],
        };
        [row_step, col_step]
    }

    fn fence_step(self) -> [isize; 3] {
        let [row_step, col_step, is_up_not_left] = match self {
            Self::UP    => [0, 0, 1],
            Self::RIGHT => [0, 1, 0],
            Self::DOWN  => [1, 0, 1],
            Self::LEFT  => [0, 0, 0],
        };
        [row_step, col_step, is_up_not_left]
    }
}

fn num_continuous_intervals(nums: &Vec<[usize; 2]>) -> usize {
    let mut num_iter = nums.iter();
    let [mut current_num, mut current_orientation] = nums[0];
    let mut num_straight_fences = 1;
    while let Some(&[next_num, next_orientation]) = num_iter.next() {
        if next_num - current_num > 1 || next_orientation != current_orientation { 
            num_straight_fences += 1;
        }
        current_num = next_num;
        current_orientation = next_orientation;
    }
    num_straight_fences
}

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Plot {
    row: usize,
    col: usize,
    chr: char,
}

struct AdjacentPlot {
    plot: Plot,
    adjacent: [bool; 4],  // up right down left; this is not great. Should somehow implement enums
                          // for this.
}

fn get_straight_fences(region_plots: &Vec<AdjacentPlot>) -> usize {
    let mut vert_fences: BTreeMap<usize, Vec<[usize; 2]>> = BTreeMap::new();
    let mut hort_fences: BTreeMap<usize, Vec<[usize; 2]>> = BTreeMap::new();
    for region in region_plots {
        let AdjacentPlot {plot, adjacent} = region;
        let Plot {row, col, chr} = plot;
        for (&is_adjacent, step) in adjacent.iter().zip(STEPS) {
            let [row_step, col_step, point_up_not_left] = step.fence_step();
            if is_adjacent { continue }
            let orientation = match step {
                Direction::UP => 1,
                Direction::LEFT => 1,
                Direction::RIGHT => 0,
                Direction::DOWN => 0,
            };

            let fence_row: usize = row+row_step as usize;
            let fence_col: usize = col+col_step as usize;
            let is_horizontal_fence = point_up_not_left != 0;
            if is_horizontal_fence {
                hort_fences.entry(fence_row).or_insert(Vec::new()).push([fence_col, orientation]);
            } else {
                vert_fences.entry(fence_col).or_insert(Vec::new()).push([fence_row, orientation]);
            }
        }
    }

    for values in hort_fences.values_mut() { values.sort(); }
    for values in vert_fences.values_mut() { values.sort(); }

    let hort_straight_fences: usize = hort_fences.values()
        .map(|cols| num_continuous_intervals(cols))
        .sum();
    let vert_straight_fences: usize = vert_fences.values()
        .map(|rows| num_continuous_intervals(rows))
        .sum();

    hort_straight_fences + vert_straight_fences

}

fn discount_fence_price(region_plots: &Vec<AdjacentPlot>) -> usize {
    let straight_fences: usize = get_straight_fences(region_plots);
    let area: usize = region_plots
        .iter()
        .count();
    straight_fences * area
}

fn adjacent_to_price(num_adjacent: usize) -> usize {
    4 - num_adjacent
}

fn fence_price(region_plots: &Vec<AdjacentPlot>) -> usize {
    let fences: usize = region_plots
        .iter()
        .map(|plot| adjacent_to_price(plot.adjacent.iter().filter(|&&val| val).count()))
        .sum();
    let area: usize = region_plots
        .iter()
        .count();
    fences * area
}

fn collect_like_plots(remaining_locs: &mut BTreeSet<Plot>, starting_plot: &Plot, map_bounds: &[usize; 2], chr_map: &Vec<Vec<char>>) -> Vec<AdjacentPlot> {
    // breadth-first search
    let mut found_regions: Vec<AdjacentPlot> = Vec::new();
    let mut queue: Vec<Plot> = Vec::new();
    queue.push(starting_plot.clone());
    while let Some(plot) = queue.pop() {
        if !remaining_locs.remove(&plot) { continue }
        let adjacent: [Option<Plot>; 4] = get_same_adjacent_plots(&plot, map_bounds, &chr_map);
        let mut adjacent_bools: [bool; 4] = [false; 4];
        for (i, result) in adjacent.iter().enumerate() {
            adjacent_bools[i] = result.is_some();
        }

        let adjacent_plot: AdjacentPlot = AdjacentPlot {
            plot,
            adjacent: adjacent_bools
        };
        found_regions.push(adjacent_plot);

        for plot_result in adjacent {
            let next_plot = match plot_result {
                Some(val) => val,
                None => continue,
            };
            queue.push(next_plot);
        }
    }
    found_regions
}

fn get_same_adjacent_plots(plot: &Plot, map_bounds: &[usize; 2], 
    chr_map: &Vec<Vec<char>>) -> [Option<Plot>; 4] {
    let chr = plot.chr;
    let row = plot.row as isize;
    let col = plot.col as isize;

    let [max_row, max_col] = map_bounds;
    let max_row = *max_row as isize;
    let max_col = *max_col as isize;

    let mut next_plots: [Option<Plot>; 4] = [None, None, None, None];
    for (i, direction) in STEPS.iter().enumerate() {
        let [row_step, col_step] = direction.step();
        let (next_row, next_col) = (row+row_step, col+col_step);
        let out_bounds: bool = next_row < 0 || next_row >= max_row || next_col < 0 || next_col >= max_col;
        if out_bounds { continue }
        let (next_row, next_col) = (next_row as usize, next_col as usize);
        let next_chr: char = chr_map[next_row][next_col];
        if next_chr != chr { continue }

        next_plots[i] = Some(Plot {row:next_row, col:next_col, chr});
    };
    next_plots
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    // Put the entire data into a hashmap with locations and letters
    let chr_map: Vec<Vec<char>> = textdata
        .lines()
        .map(|string| string .chars().collect())
        .collect();
    let map_bounds: [usize; 2] = [chr_map.len(), chr_map[0].len()];

    let plots: Vec<Plot> = textdata
        .lines()
        .enumerate()
        .flat_map(|(row, string)| string
            .chars()
            .enumerate()
            .map(move | (col, chr)| Plot { row, col, chr } ) )
        .collect();

    // keep starting new searches while elements still in hashset
    let mut remaining_locs: BTreeSet<Plot> = BTreeSet::from_iter(plots);
    let mut found_regions: Vec<Vec<AdjacentPlot>> = Vec::new();
    while let Some(starting_plot) = remaining_locs.iter().next() {
        let starting_plot = starting_plot.clone();
        let region: Vec<AdjacentPlot> = collect_like_plots(&mut remaining_locs, &starting_plot, &map_bounds, &chr_map);
        found_regions.push(region);
    }

    let total_price: usize = found_regions
        .iter()
        .map(fence_price)
        .sum();

    let after = before.elapsed();
    println!("(Part  I) total price: {total_price}");
    println!("(Part  I) elapsed time: {after:.2?}");

    let discount_price: usize = found_regions
        .iter()
        .map(discount_fence_price)
        .sum();
    let after = before.elapsed();
    println!("(Part II) total price: {discount_price}");
    println!("(Part II) elapsed time: {after:.2?}");

}

#[test]
fn small_scale() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    // Put the entire data into a hashmap with locations and letters
    let chr_map: Vec<Vec<char>> = textdata
        .lines()
        .map(|string| string .chars().collect())
        .collect();
    let map_bounds: [usize; 2] = [chr_map.len(), chr_map[0].len()];

    let plots: Vec<Plot> = textdata
        .lines()
        .enumerate()
        .flat_map(|(row, string)| string
            .chars()
            .enumerate()
            .map(move | (col, chr)| Plot { row, col, chr } ) )
        .collect();

    // keep starting new searches while elements still in hashset
    let mut remaining_locs: BTreeSet<Plot> = BTreeSet::from_iter(plots);
    let mut found_regions: Vec<Vec<AdjacentPlot>> = Vec::new();
    while let Some(starting_plot) = remaining_locs.iter().next() {
        let starting_plot = starting_plot.clone();
        let region: Vec<AdjacentPlot> = collect_like_plots(&mut remaining_locs, &starting_plot, &map_bounds, &chr_map);
        found_regions.push(region);
    }

    let total_price: usize = found_regions
        .iter()
        .map(fence_price)
        .sum();
    assert_eq!(1930, total_price);
}

#[test]
fn small_scale_straight_fences() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    // Put the entire data into a hashmap with locations and letters
    let chr_map: Vec<Vec<char>> = textdata
        .lines()
        .map(|string| string .chars().collect())
        .collect();
    let map_bounds: [usize; 2] = [chr_map.len(), chr_map[0].len()];

    let plots: Vec<Plot> = textdata
        .lines()
        .enumerate()
        .flat_map(|(row, string)| string
            .chars()
            .enumerate()
            .map(move | (col, chr)| Plot { row, col, chr } ) )
        .collect();

    // keep starting new searches while elements still in hashset
    let mut remaining_locs: BTreeSet<Plot> = BTreeSet::from_iter(plots);
    let mut found_regions: Vec<Vec<AdjacentPlot>> = Vec::new();
    while let Some(starting_plot) = remaining_locs.iter().next() {
        let starting_plot = starting_plot.clone();
        let region: Vec<AdjacentPlot> = collect_like_plots(&mut remaining_locs, &starting_plot, &map_bounds, &chr_map);
        found_regions.push(region);
    }

    let total_price: usize = found_regions
        .iter()
        .map(discount_fence_price)
        .sum();
    assert_eq!(1206, total_price);
}

#[test]
fn straight_fences_5x5() {
    let filename: &str = "test_input_5x5.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    // Put the entire data into a hashmap with locations and letters
    let chr_map: Vec<Vec<char>> = textdata
        .lines()
        .map(|string| string .chars().collect())
        .collect();
    let map_bounds: [usize; 2] = [chr_map.len(), chr_map[0].len()];

    let plots: Vec<Plot> = textdata
        .lines()
        .enumerate()
        .flat_map(|(row, string)| string
            .chars()
            .enumerate()
            .map(move | (col, chr)| Plot { row, col, chr } ) )
        .collect();

    // keep starting new searches while elements still in hashset
    let mut remaining_locs: BTreeSet<Plot> = BTreeSet::from_iter(plots);
    let mut found_regions: Vec<Vec<AdjacentPlot>> = Vec::new();
    while let Some(starting_plot) = remaining_locs.iter().next() {
        let starting_plot = starting_plot.clone();
        let region: Vec<AdjacentPlot> = collect_like_plots(&mut remaining_locs, &starting_plot, &map_bounds, &chr_map);
        found_regions.push(region);
    }

    let total_price: usize = found_regions
        .iter()
        .map(discount_fence_price)
        .sum();
    assert_eq!(236, total_price);
}

#[test]
fn straight_fences_6x6() {
    let filename: &str = "test_input_6x6.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    // Put the entire data into a hashmap with locations and letters
    let chr_map: Vec<Vec<char>> = textdata
        .lines()
        .map(|string| string .chars().collect())
        .collect();
    let map_bounds: [usize; 2] = [chr_map.len(), chr_map[0].len()];

    let plots: Vec<Plot> = textdata
        .lines()
        .enumerate()
        .flat_map(|(row, string)| string
            .chars()
            .enumerate()
            .map(move | (col, chr)| Plot { row, col, chr } ) )
        .collect();

    // keep starting new searches while elements still in hashset
    let mut remaining_locs: BTreeSet<Plot> = BTreeSet::from_iter(plots);
    let mut found_regions: Vec<Vec<AdjacentPlot>> = Vec::new();
    while let Some(starting_plot) = remaining_locs.iter().next() {
        let starting_plot = starting_plot.clone();
        let region: Vec<AdjacentPlot> = collect_like_plots(&mut remaining_locs, &starting_plot, &map_bounds, &chr_map);
        found_regions.push(region);
    }

    let total_price: usize = found_regions
        .iter()
        .map(discount_fence_price)
        .sum();
    assert_eq!(368, total_price);
}
