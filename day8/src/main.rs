use std::fs;
use std::collections::HashSet;
use std::time::Instant;

struct Node {
    name: char,
    loc: u32,
}

fn antinode_from_nodes(nodes: &[&Node; 2], size: &[u32; 2]) -> Option<u32> {
    // draw a ray from node1 -> node 2. Make ONE anti-node in that direction.
    let [num_rows, num_cols] = size;
    let [node1, node2] = nodes;
    let loc1 = node1.loc;
    let loc2 = node2.loc;

    let row1 = loc1 / num_cols;
    let col1 = loc1 % num_cols;
    let row2 = loc2 / num_cols;
    let col2 = loc2 % num_cols;
    
    let row3 = row1 as i32 + 2 * (row2 as i32 - row1 as i32);
    let col3 = col1 as i32 + 2 * (col2 as i32 - col1 as i32);

    let cannot_convert_unsigned = row3 < 0 || col3 < 0 ;
    if cannot_convert_unsigned { return None }
    let row3 = row3 as u32;
    let col3 = col3 as u32;

    let out_bounds = row3 >= *num_rows || col3 >= *num_cols;
    if out_bounds { return None }

    // maybe "row3.try_into()?"
    let new_loc = row3*num_cols + col3;
    Some(new_loc)
}

fn multiple_antinodes_from_nodes(nodes: &[&Node; 2], size: &[u32; 2]) -> Vec<u32> {
    // draw a ray from node1 -> node 2. Make ONE anti-node in that direction.
    let [num_rows, num_cols] = size;
    let [node1, node2] = nodes;
    let loc1 = node1.loc;
    let loc2 = node2.loc;

    let row1 = loc1 / num_cols;
    let col1 = loc1 % num_cols;
    let row2 = loc2 / num_cols;
    let col2 = loc2 % num_cols;

    let mut antinodes: Vec<u32> = Vec::new();
    let max_steps = *size[..].iter().max().unwrap() as i32;
    for i in 0..max_steps {  // might be excessive; can cut this down!
                             // this could also be just an iterator
        let row3 = row1 as i32 + i * (row2 as i32 - row1 as i32);
        let col3 = col1 as i32 + i * (col2 as i32 - col1 as i32);

        let cannot_convert_unsigned = row3 < 0 || col3 < 0 ;
        if cannot_convert_unsigned { continue }
        let row3 = row3 as u32;
        let col3 = col3 as u32;

        let out_bounds = row3 >= *num_rows || col3 >= *num_cols;
        if out_bounds { continue }

        let new_loc = row3*num_cols + col3;
        antinodes.push(new_loc);
    }

    antinodes
}

fn nodes_from_text(textdata: String) -> Vec<Node>{
    let [_num_rows, num_cols] = size_from_textdata(&textdata);
    let nodes: Vec<Node> = textdata.lines().enumerate()
        .flat_map(|(row, string)| 
            string
            .chars()
            .enumerate()
            .map(move |(col, chr)| (chr, (row as u32)*num_cols + (col as u32)) )
        )
        .filter(|&(chr, _loc) | chr != '.')
        .map(|(name, loc)| Node { name, loc })
        .collect();
    nodes
}

fn size_from_textdata(textdata: &String) -> [u32; 2] {
    let row_size = textdata.lines().count();
    let col_size = textdata.lines().next().unwrap().len();
    [row_size as u32, col_size as u32]
}

fn different_nodes_same_char(nodes: &[&Node; 2]) -> bool {
    let [node1, node2] = nodes;
    node1.name == node2.name && node1.loc != node2.loc
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    let size = size_from_textdata(&textdata);

    let vec_of_nodes: Vec<Node> = nodes_from_text(textdata);
    let unique_antinodes: HashSet<u32> = vec_of_nodes.iter()
        .flat_map(|node1| vec_of_nodes.iter().map(move |node2| [node1, node2]))
        .filter(different_nodes_same_char)
        .filter_map(|nodes| antinode_from_nodes(&nodes, &size))
        .collect();
    let num_antinodes: usize = unique_antinodes.len();

    let after = before.elapsed();
    println!("Time elapsed (Part  I): {after:2?}");
    println!("(Part  I): num. of unique anti-nodes: {num_antinodes}");
    let before = Instant::now();

    let unique_antinodes: HashSet<u32> = vec_of_nodes.iter()  // &[u32; 2]
        .flat_map(|node1| vec_of_nodes.iter().map(move |node2| [node1, node2]))
        .filter(different_nodes_same_char)
        .flat_map(|nodes| multiple_antinodes_from_nodes(&nodes, &size))
        .collect();
    let num_antinodes: usize = unique_antinodes.len();

    let after = before.elapsed();
    println!("Time elapsed (Part II): {after:2?}");
    println!("(Part II): num. of unique anti-nodes: {num_antinodes}");
}

#[test]
fn test_input() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let size = size_from_textdata(&textdata);

    let vec_of_nodes: Vec<Node> = nodes_from_text(textdata);
    assert_eq!(7, vec_of_nodes.len());
    let vec_pairs_nodes: Vec<[&Node; 2]> = vec_of_nodes.iter()
        .flat_map(|node1| vec_of_nodes.iter().map(move |node2| [node1, node2]))
        .collect();

    let nonunique_antinodes: Vec<u32> = vec_pairs_nodes.iter()
        .filter(|[node1, node2]| node1.name == node2.name)
        .filter(|[node1, node2]| node1.loc != node2.loc)
        .filter_map(|nodes| antinode_from_nodes(nodes, &size))
        .collect();

    // let num = itertools::unique(non_unique_antinodes).len();

    let unique_antinodes: HashSet<u32> = HashSet::from_iter(nonunique_antinodes.into_iter());
    let num_antinodes: usize = unique_antinodes.len();
    assert_eq!(14, num_antinodes);
}

