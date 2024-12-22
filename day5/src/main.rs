use std::fs;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::time::Instant;

struct PairOrdering {
    item: HashMap<usize, HashMap<usize, Ordering>>,
}

fn data_from_text(textdata: String) -> (Vec<[usize; 2]>, Vec<Vec<usize>>) {
    let mut file_iter = textdata.lines();
    let ordering_pairs: Vec<[usize; 2]> = file_iter
        .by_ref()
        .take_while(|string| !string.is_empty())  // can also use string.contains('|')
        .map(|string| string
            .split('|')
            .map(|part| part.parse().unwrap())
            .collect::<Vec<usize>>()
            .try_into().unwrap()
        )
        .collect();

    let entries: Vec<Vec<usize>> = file_iter
        .map(|parts| parts
            .split(',')
            .map(|string| string
                .parse()
                .unwrap_or_else(|_| panic!("Cannot parse ({string})")))
            .collect()
        )
        .collect();

    (ordering_pairs, entries)
}

fn ordering_map(ordering_pairs: &Vec<[usize; 2]>) -> PairOrdering {
    let mut compare: HashMap<usize, HashMap<usize, Ordering>> = HashMap::new();

    for pair in ordering_pairs {
        let num1 = pair[0];
        let num2 = pair[1];
        let num1_entry = compare.entry(num1).or_insert(HashMap::new());
        num1_entry.entry(num2).or_insert(Ordering::Less);
        let num2_entry = compare.entry(num2).or_insert(HashMap::new());
        num2_entry.entry(num1).or_insert(Ordering::Greater);
    }

    PairOrdering {item: compare}
}

fn compare(num1: &usize, num2: &usize, ordering: &PairOrdering) -> Ordering {
    let num1_entry = ordering.item.get(&num1);
    let num2_entry = match num1_entry {
        Some(hashmap) => hashmap,
        None => return Ordering::Equal,
    };
    let order = match num2_entry.get(&num2) {
        Some(order) => *order,
        None => return Ordering::Equal,
    };

    order
}

fn middle_page(vec: &Vec<usize>) -> usize {
    let len = vec.len();
    vec[len/2]
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    let (ordering_pairs, entries) = data_from_text(textdata);
    let ordering = ordering_map(&ordering_pairs);

    let mut sorted_entries = entries.clone();
    sorted_entries.iter_mut()
        .for_each(|entry| 
            entry.sort_by(|num1, num2| compare(num1, num2, &ordering)));

    let sum_middle_pages: usize = entries.iter().zip(sorted_entries.iter())
        .filter(|(norm, sorted)| norm == sorted )
        .map(|(norm, _sorted)| norm)
        .map(middle_page)
        .sum();

    let after = before.elapsed();
    println!("(Part  I) sum of properly-ordered middle pages: {sum_middle_pages}");
    println!("    elapsed time: {after:.2?}");
    let before = Instant::now();

    let sum_false_middle_pages: usize = entries.iter().zip(sorted_entries.iter())
        .filter(|(norm, sorted)| norm != sorted )
        .map(|(_norm, sorted)| sorted)
        .map(middle_page)
        .sum();

    let after = before.elapsed();
    println!("(Part II) sum of falsely-ordered middle pages: {sum_false_middle_pages}");
    println!("    elapsed time: {after:.2?}");
}

#[test]
fn test_separate_textdata() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let (ordering_pairs, entries) = data_from_text(textdata);
    assert_eq!(21, ordering_pairs.len());
    assert_eq!(6, entries.len());
}

#[test]
fn test_order() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let (ordering_pairs, entries) = data_from_text(textdata);
    let ordering = ordering_map(&ordering_pairs);

    let mut sorted_entries = entries.clone();
    println!("first entry: {:?}", sorted_entries[0]);
    for entry in sorted_entries.iter_mut() {
        entry.sort_by(|num1, num2| compare(num1, num2, &ordering))
    }
    assert_eq!(vec![75, 47, 61, 53, 29], sorted_entries[0]);
    assert_eq!(vec![97, 61, 53, 29, 13], sorted_entries[1]);
    assert_eq!(vec![75, 29, 13],         sorted_entries[2]);
    assert_ne!(vec![75, 97, 47, 61, 53], sorted_entries[3]);
    assert_ne!(vec![61, 13, 29],         sorted_entries[4]);
    assert_ne!(vec![97, 13, 75, 29, 47], sorted_entries[5]);
}

#[test]
fn test_num() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let (ordering_pairs, entries) = data_from_text(textdata);
    let ordering = ordering_map(&ordering_pairs);

    let mut sorted_entries = entries.clone();
    for entry in sorted_entries.iter_mut() {
        entry.sort_by(|num1, num2| compare(num1, num2, &ordering))
    }

    let num_proper_order: usize = entries.iter().zip(sorted_entries.iter())
        .filter(|(norm, sorted)| norm == sorted )
        .count();
    assert_eq!(3, num_proper_order);

    let sum_middle_pages: usize = entries.iter().zip(sorted_entries.iter())
        .filter(|(norm, sorted)| norm == sorted )
        .map(|(norm, _sorted)| middle_page(norm))
        .sum();
    assert_eq!(143, sum_middle_pages);
}
