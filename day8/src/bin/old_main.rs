use std::fs;
use std::collections::HashSet;
use std::time::Instant;


// to do!
//
// 1. change 2D to 1D arrays

#[derive(Clone)]
struct Node {
    name: char,
    loc: [u32; 2],
}

fn in_bounds(loc: &[u32; 2], size: &[u32; 2]) -> bool {
    let [row_size, col_size] = size;
    let [row, col] = loc;
    row < row_size && col < col_size
}

fn antinode_from_nodes(nodes: &[&Node; 2]) -> Option<[u32; 2]> {
    // draw a ray from node1 -> node 2. Make ONE anti-node in that direction.
    let [node1, node2] = nodes;
    let [row1, col1] = node1.loc;
    let [row2, col2] = node2.loc;
    
    let row3 = row1 as i32 + 2 * (row2 as i32 - row1 as i32);
    let col3 = col1 as i32 + 2 * (col2 as i32 - col1 as i32);

    let cannot_convert_unsigned = row3 < 0 || col3 < 0 ;
    if cannot_convert_unsigned { return None }

    // maybe "row3.try_into()?"
    Some([row3 as u32, col3 as u32])
}

fn multiple_antinodes_from_nodes(nodes: &[&Node; 2], size: &[u32; 2]) -> Vec<[u32; 2]> {
    // draw a ray from node1 -> node 2. Make ONE anti-node in that direction.
    let [node1, node2] = nodes;
    let [row1, col1] = node1.loc;
    let [row2, col2] = node2.loc;

    let mut antinodes: Vec<[u32; 2]> = Vec::new();
    for i in 0..(*size[..].iter().max().unwrap() as i32) {
        let row3 = row1 as i32 + i * (row2 as i32 - row1 as i32);
        let col3 = col1 as i32 + i * (col2 as i32 - col1 as i32);

        let cannot_convert_unsigned = row3 < 0 || col3 < 0 ;
        if cannot_convert_unsigned { continue }

        // maybe "row3.try_into()?"
        antinodes.push([row3 as u32, col3 as u32]);
    }
    antinodes
}

fn nodes_from_text(textdata: String) -> Vec<Node>{
    let nodes: Vec<Node> = textdata.lines().enumerate()
        .flat_map(|(row, string)| 
            string
            .chars()
            .enumerate()
            .map(move |(col, chr)| (chr, [row as u32, col as u32]) )
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

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    let size = size_from_textdata(&textdata);

    let vec_of_nodes: Vec<Node> = nodes_from_text(textdata);
    let vec_pairs_nodes: Vec<[&Node; 2]> = vec_of_nodes.iter()
        .flat_map(|node1| vec_of_nodes.iter().map(move |node2| [node1, node2]))
        .collect();
    println!("num vec pairs: {}", vec_pairs_nodes.len());

    let nonunique_antinodes: Vec<[u32; 2]> = vec_pairs_nodes.iter()
        .filter(|[node1, node2]| node1.name == node2.name)  // .inspect exists :)
        .filter(|[node1, node2]| node1.loc != node2.loc)
        .filter_map(|nodes| antinode_from_nodes(nodes)) // -> [usize; 2]
        .filter(|loc| in_bounds(loc, &size))
        .collect();
    println!("len of non-unique: {}", nonunique_antinodes.len());

    // let num = itertools::unique(non_unique_antinodes).len();
    let unique_antinodes: HashSet<[u32; 2]> = HashSet::from_iter(nonunique_antinodes.into_iter());
    let num_antinodes: usize = unique_antinodes.len();

    let after = before.elapsed();
    println!("Time elapsed (Part  I): {after:2?}");
    println!("(Part  I): num. of unique anti-nodes: {num_antinodes}");

    let nonunique_antinodes: Vec<[u32; 2]> = vec_pairs_nodes.iter()  // &[u32; 2]
        .filter(|[node1, node2]| node1.name == node2.name)  // .inspect exists :)
        .filter(|[node1, node2]| node1.loc != node2.loc)
        .flat_map(|nodes| multiple_antinodes_from_nodes(nodes, &size))
        .filter(|loc| in_bounds(loc, &size))
        .collect();
    println!("len of non-unique: {}", nonunique_antinodes.len());

    let unique_antinodes: HashSet<[u32; 2]> = HashSet::from_iter(nonunique_antinodes.into_iter());
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

    let nonunique_antinodes: Vec<[u32; 2]> = vec_pairs_nodes.iter()
        .filter(|[node1, node2]| node1.name == node2.name)  // .inspect exists :)
        .filter(|[node1, node2]| node1.loc != node2.loc)
        .filter_map(|nodes| antinode_from_nodes(nodes)) // -> [usize; 2]
        .filter(|loc| in_bounds(loc, &size))
        .collect();

    // let num = itertools::unique(non_unique_antinodes).len();

    let unique_antinodes: HashSet<[u32; 2]> = HashSet::from_iter(nonunique_antinodes.into_iter());
    let num_antinodes: usize = unique_antinodes.len();
    assert_eq!(14, num_antinodes);
}

