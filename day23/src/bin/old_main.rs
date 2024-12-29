use std::fs;
use std::collections::{BTreeSet, BTreeMap};
use ndarray::Array2;
use std::iter;
use std::time::Instant;

fn make_index_map<'a>(connections: &Vec<[&'a str; 2]>) -> (BTreeMap<&'a str, usize>, BTreeMap<usize, &'a str>) {
    let mut set: BTreeSet<&str> = BTreeSet::new();
    connections.iter().for_each(|[str1, str2]| {
        set.insert(str1);
        set.insert(str2);
    });

    let mut map: BTreeMap<&str, usize> = BTreeMap::new();
    set.iter().enumerate()
        .for_each(|(index, string)| { map.entry(string).or_insert(index); });

    let mut inv_map: BTreeMap<usize, &str> = BTreeMap::new();
    set.iter().enumerate()
        .for_each(|(index, string)| { inv_map.entry(index).or_insert(string); });

    (map, inv_map)
}

fn make_directed_matrix(connections: &Vec<[&str; 2]>, map: &BTreeMap<&str, usize>) -> Array2<u32> {
    let num_entries: usize = map.len();
    let mut adjacency_matrix: Array2<u32> = Array2::zeros((num_entries, num_entries));
    connections.iter().for_each(|[str1, str2]| {
        let loc1: usize = *map.get(str1).unwrap();
        let loc2: usize = *map.get(str2).unwrap();
        adjacency_matrix[[loc1, loc2]] += 1;
        // adjacency_matrix[[loc2, loc1]] += 1;
    });

    adjacency_matrix
}

fn make_subset_matrices(connections: Vec<[&str; 2]>, map: &BTreeMap<&str, usize>) -> [Array2<u32>; 4] {
    let and_t_not_t_connections: Vec<[&str; 2]> = connections.iter()
        .filter(|[str1, str2]| &str1[0..1] == "t" && &str2[0..1] != "t")
        .map(|&item| item)
        .collect();
    let and_t_not_t_directed: Array2<u32> = make_directed_matrix(&and_t_not_t_connections, &map);

    let not_t_and_t_connections: Vec<[&str; 2]> = connections.iter()
        .filter(|[str1, str2]| &str1[0..1] != "t" && &str2[0..1] == "t")
        .map(|&item| item)
        .collect();
    let not_t_and_t_directed: Array2<u32> = make_directed_matrix(&not_t_and_t_connections, &map);
     
    let and_t_and_t_connections: Vec<[&str; 2]> = connections.iter()
        .filter(|[str1, str2]| &str1[0..1] == "t" && &str2[0..1] == "t")
        .map(|&item| item)
        .collect();
    let and_t_and_t_directed: Array2<u32> = make_directed_matrix(&and_t_and_t_connections, &map);

    let not_t_not_t_connections: Vec<[&str; 2]> = connections.iter()
        .filter(|[str1, str2]| &str1[0..1] != "t" && &str2[0..1] != "t")
        .map(|&item| item)
        .collect();
    let not_t_not_t_directed: Array2<u32> = make_directed_matrix(&not_t_not_t_connections, &map);

    [
        and_t_and_t_directed, 
        and_t_not_t_directed, 
        not_t_and_t_directed, 
        not_t_not_t_directed, 
    ]
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let connections: Vec<[&str; 2]> = textdata.lines()
        .map(|string| string.split("-").collect::<Vec<&str>>().try_into().unwrap())
        .collect();

    let connections: Vec<[&str; 2]> = connections.into_iter()
        .flat_map(|[str1, str2]| [[str1, str2], [str2, str1]])
        .collect();

    let (map, inv_map) = make_index_map(&connections);

    let directed_matrix = make_directed_matrix(&connections, &map);

    let three_adjacency = directed_matrix
        .dot(&directed_matrix)
        .dot(&directed_matrix);
    println!("Trace of 3-adjacency matrix: {}", three_adjacency.diag().sum());

    let permutation_factor = 3 * 2 * 1;
    let num_three_cycles = three_adjacency
        .diag()
        .sum() 
        / permutation_factor ;

    // println!("Total number of 3-cycles: {num_three_cycles}");
    // the permutation factor depends on which nodes are allowed. We have three cases.

    let [
        and_t_and_t_directed, 
        and_t_not_t_directed, 
        not_t_and_t_directed, 
        not_t_not_t_directed, 
    ] = make_subset_matrices(connections, &map);

    // Case 1: only 1 t-node
    let t1_permutation_factor = 2;
    let t1_adjacency = and_t_not_t_directed
        .dot(&not_t_not_t_directed)
        .dot(&not_t_and_t_directed);
    // println!("Perm. number of 3-cycles with 1 t: {}", t1_adjacency.diag().sum());

    // Case 2: only 2 t-nodes
    let t2_permutation_factor = 2;
    let t2_adjacency = and_t_and_t_directed
        .dot(&and_t_not_t_directed)
        .dot(&not_t_and_t_directed);
    // println!("Perm. number of 3-cycles with 2 t: {}", t2_adjacency.diag().sum());

    // Case 3: only 3 t-nodes
    let t3_permutation_factor = 1;
    let t3_adjacency = and_t_and_t_directed
        .dot(&and_t_and_t_directed)
        .dot(&and_t_and_t_directed);
    // println!("Perm. number of 3-cycles with 3 t: {}", t3_adjacency.diag().sum());

    let unique_t_cycles: u32 = {
        t1_adjacency.diag().sum() / t1_permutation_factor
        + t2_adjacency.diag().sum() / t2_permutation_factor
        + t3_adjacency.diag().sum() / t3_permutation_factor
    };

    let after = before.elapsed();
    println!("(Part  I) Number of unique t-cycles: {unique_t_cycles}");
    println!("(Part  I) time elapsed: {after:.2?}");
    let before = Instant::now();

    // for Part 2, just prune the columns of the adjacency matrix with the minimum entries until
    // every remaining column is completely filled. This is the smallest completely connected
    // subgraph. And instead of mutating the adjacency matrix, just keep track of the rows we want
    // to compare.

    let mut keep_indices: Vec<usize> = (0..map.len()).collect();
    let complete_indices: Vec<usize> = loop {
        let adjacencies: Vec<u32> = keep_indices.iter()
            .map(|&index1| keep_indices
                .iter()
                .map(|&index2| directed_matrix[[index1, index2]])
                .sum()
            )
            .collect();
        let is_complete: bool = adjacencies.iter().all(|&val| val==adjacencies[0]);
        if is_complete { break keep_indices }
        let min: &u32 = adjacencies.iter().min().unwrap();
        let min_loc: usize = adjacencies.iter().position(|val| val==min).unwrap();
        keep_indices.swap_remove(min_loc);
    };
    // println!("{complete_indices:?}");
    // doesn't work. We'll go to the brute-force depth-first search with memoization

    let mut cliques: BTreeMap<Vec<usize>, bool> = BTreeMap::new();
    let mut queue: Vec<Vec<usize>> = (0..map.len())
        .flat_map(|num1| 
            (0..map.len()).map(move |num2| Vec::from([num1, num2]))
        )
        .filter(|vec| vec[0] != vec[1])
        .collect();

    while let Some(vec) = queue.pop() {
        let found_clique = is_clique(&vec, &directed_matrix);
        if !found_clique { continue }
        cliques.insert(vec.clone(), found_clique);
        ((*vec.iter().max().unwrap()+1)..map.len())
            .for_each(|index| {
            queue.push( vec.iter()
                .map(|&item| item)
                .chain(iter::once(index) )
                .collect()
            );
        });
    }

    let mut size: usize = 0;
    let mut max_clique: Vec<usize> = Vec::new();
    cliques.iter()
        .filter(|(_, &val)| val)
        .for_each(|(key, _)| if key.len() > size { 
            size = key.len();
            max_clique = key.clone();
        } );

    let mut clique_strings: Vec<&str> = max_clique.iter()
        .map(|num| *inv_map.get(num).unwrap() )
        .collect();
    clique_strings.sort();
    let password: String = clique_strings.join(",");

    let after = before.elapsed();
    println!("(Part II) password: {password}");
    println!("(Part II) time elapsed: {after:.2?}");
}

fn is_clique(vec: &Vec<usize>, adjacency: &Array2<u32>) -> bool {
    vec.iter()
        .flat_map(|&num1| vec.iter().map(move |&num2| (num1, num2)))
        .filter(|(num1, num2)| num2 != num1)
        .all(|(num1, num2)| adjacency[[num1, num2]] == 1 )
}

#[test]
fn small_test() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let connections: Vec<[&str; 2]> = textdata.lines()
        .map(|string| string.split("-").collect::<Vec<&str>>().try_into().unwrap())
        .collect();
    let connections: Vec<[&str; 2]> = connections.into_iter()
        .flat_map(|[str1, str2]| [[str1, str2], [str2, str1]])
        .collect();

    let map = make_index_map(&connections);

    let directed_matrix = make_directed_matrix(&connections, &map);

    let three_adjacency = directed_matrix
        .dot(&directed_matrix)
        .dot(&directed_matrix);
    println!("Trace of 3-adjacency matrix: {}", three_adjacency.diag().sum());

    let permutation_factor = 3 * 2 * 1;
    let num_three_cycles = three_adjacency
        .diag()
        .sum() 
        / permutation_factor ;
    assert_eq!(12, num_three_cycles);

    let [
        and_t_and_t_directed, 
        and_t_not_t_directed, 
        not_t_and_t_directed, 
        not_t_not_t_directed, 
    ] = make_subset_matrices(connections, &map);

    // Case 1: only 1 t-node
    let t1_permutation_factor = 2;
    let t1_adjacency = and_t_not_t_directed
        .dot(&not_t_not_t_directed)
        .dot(&not_t_and_t_directed);
    println!("Perm. number of 3-cycles with 1 t: {}", t1_adjacency.diag().sum());

    // Case 2: only 2 t-nodes
    let t2_permutation_factor = 2;
    let t2_adjacency = and_t_and_t_directed
        .dot(&and_t_not_t_directed)
        .dot(&not_t_and_t_directed);
    println!("Perm. number of 3-cycles with 2 t: {}", t2_adjacency.diag().sum());

    // Case 3: only 3 t-nodes
    let t3_permutation_factor = 1;
    let t3_adjacency = and_t_and_t_directed
        .dot(&and_t_and_t_directed)
        .dot(&and_t_and_t_directed);
    println!("Perm. number of 3-cycles with 3 t: {}", t3_adjacency.diag().sum());

    let unique_t_cycles: u32 = {
        t1_adjacency.diag().sum() / t1_permutation_factor
        + t2_adjacency.diag().sum() / t2_permutation_factor
        + t3_adjacency.diag().sum() / t3_permutation_factor
    };
    assert_eq!(7, unique_t_cycles);

    println!("{directed_matrix:?}");

    let mut keep_indices: Vec<usize> = (0..map.len()).collect();
    let complete_indices: Vec<usize> = loop {
        let adjacencies: Vec<u32> = keep_indices.iter()
            .map(|&index1| keep_indices
                .iter()
                .map(|&index2| directed_matrix[[index1, index2]])
                .sum()
            )
            .collect();
        println!("adjacencies: {adjacencies:?}");
        let is_complete: bool = adjacencies.iter().all(|&val| val==adjacencies[0]);
        if is_complete { break keep_indices }
        let min: &u32 = adjacencies.iter().min().unwrap();
        let min_loc: usize = adjacencies.iter().position(|val| val==min).unwrap();
        keep_indices.swap_remove(min_loc);
        println!("running!");
    };

}
