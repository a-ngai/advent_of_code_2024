use std::fs;
use std::iter;
use std::time::Instant;

fn main() {
    let filename: &str = "input.txt";
    let textdata = fs::read_to_string(filename).unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let disk: Vec<usize> = textdata
        .lines().next().unwrap()
        .chars()
        .map(|chr| chr.to_digit(10).unwrap() as usize)
        .collect();
    let mut compacted: Vec<usize> = Vec::new();

    let mut reverse_block_iter = disk.iter()
        .step_by(2)
        .enumerate()
        .rev()
        .flat_map(|(id, &num)| iter::repeat_n(id, num));
    let mut forward_block_iter = disk.iter().step_by(2).enumerate();
    let forward_space_iter = disk.iter().skip(1).step_by(2);

    let mut reverse_id: usize = usize::MAX;
    for ((forward_id, num), skip) in forward_block_iter.by_ref().zip(forward_space_iter) {
        if reverse_id <= forward_id { 
            compacted.extend(reverse_block_iter.by_ref().take_while(|&reverse_id| reverse_id==forward_id)) ;
            break
        }
        compacted.extend(iter::repeat_n(forward_id, *num));
         
        for _ in 0..*skip { 
            reverse_id = reverse_block_iter.next().unwrap();
            if reverse_id <= forward_id { break }
            compacted.push(reverse_id) ;
        }
    }

    let checksum: usize  = compacted.iter().enumerate().map(|(id, &num)| id * num).sum();

    let after = before.elapsed();
    println!("(Part  I) checksum: {checksum}");
    println!("Time elapsed (Part  I): {after:2?}");
    let before = Instant::now();

    let mut compacted: Vec<usize> = iter::repeat_n(0, disk.iter().sum()).collect();
    let forward_block_iter = disk.iter().step_by(2).enumerate();
    let forward_space_iter = disk.iter().skip(1).step_by(2).chain([0,].iter());

    let mut block_locations: Vec<[usize; 3]> = Vec::new();
    let mut spaces: Vec<[usize; 2]> = Vec::new();
    let mut loc: usize = 0;
    for ((block_id, &num), &space) in forward_block_iter.zip(forward_space_iter) {
        block_locations.push([block_id, loc, num]);
        loc += num;
        spaces.push([loc, space]);
        loc += space;
    }

    for block_location in block_locations.into_iter().rev() {
        let [block_id, block_loc, block_len] = block_location;  // basically a pointer
        let look_for_space = spaces.iter()
            .take_while(|&[space_loc, _space_len]| block_loc > *space_loc)
            .position(|&[_space_loc, space_len]| block_len <= space_len);

        let ([replace_loc, replace_len], index) = match look_for_space {
            Some(val) => (&mut spaces[val], val),
            None => (&mut [block_loc, block_len], 0),
        };

        for compact_index in *replace_loc..(*replace_loc+block_len) { compacted[compact_index] = block_id }
        *replace_loc += block_len;
        *replace_len -= block_len;

        if *replace_len == 0usize {
            //println!("removed something!");
            spaces.remove(index);
            //println!("{}", spaces.len());
        }
    }

    let checksum: usize  = compacted.iter().enumerate().map(|(id, &num)| id * num).sum();
    let after = before.elapsed();

    println!("(Part II) checksum: {checksum}");
    println!("Time elapsed (Part II): {after:2?}");

}

#[test]
fn test_input() {
    let disk: Vec<usize> = vec![ 2,3,3,3,1,3,3,1,2,1,4,1,4,1,3,1,4,0,2 ];
    let mut compacted: Vec<usize> = Vec::new();

    let mut reverse_block_iter = disk.iter()
        .step_by(2)
        .enumerate()
        .rev()
        .flat_map(|(id, &num)| iter::repeat_n(id, num));

    let mut forward_block_iter = disk.iter()
        .step_by(2)
        .enumerate();
    let forward_space_iter = disk.iter()
        .skip(1)
        .step_by(2);

    let mut reverse_id: usize = usize::MAX;
    for ((forward_id, num), skip) in forward_block_iter.by_ref().zip(forward_space_iter) {
        if reverse_id <= forward_id { 
            compacted.extend(reverse_block_iter.by_ref().take_while(|&reverse_id| reverse_id==forward_id)) ;
            break
        }
        compacted.extend(iter::repeat_n(forward_id, *num));
         
        for _ in 0..*skip { 
            reverse_id = reverse_block_iter.next().unwrap();
            if reverse_id <= forward_id { break }
            compacted.push(reverse_id) ;
        }
    }

    assert_eq!(compacted, vec![0,0,9,9,8,1,1,1,8,8,8,2,7,7,7,3,3,3,6,4,4,6,5,5,5,5,6,6]);

    let checksum: usize  = compacted.iter().enumerate().map(|(id, &num)| id * num).sum();
    assert_eq!(checksum, 1928);
}

#[test]
fn test_input_part2() {
    let disk: Vec<usize> = vec![ 2,3,3,3,1,3,3,1,2,1,4,1,4,1,3,1,4,0,2 ];
    // let mut compacted: Vec<usize> = Vec::new();
    let mut compacted: Vec<usize> = iter::repeat_n(0, disk.iter().sum()).collect();

    let forward_block_iter = disk.iter()
        .step_by(2)
        .enumerate();
    let forward_space_iter = disk.iter()
        .skip(1)
        .step_by(2)
        .chain([0,].iter());

    let mut block_locations: Vec<[usize; 3]> = Vec::new();
    let mut spaces: Vec<[usize; 2]> = Vec::new();
    let mut loc: usize = 0;
    for ((block_id, &num), &space) in forward_block_iter.zip(forward_space_iter) {
        block_locations.push([block_id, loc, num]);
        loc += num;
        spaces.push([loc, space]);
        loc += space;
    }

    for block_location in block_locations.into_iter().rev() {
        let [block_id, block_loc, block_len] = block_location;
        let look_for_space = spaces.iter()
            .take_while(|&[space_loc, _space_len]| block_loc > *space_loc)
            .position(|&[_space_loc, space_len]| block_len <= space_len);

        let [replace_loc, replace_len] = match look_for_space {
            Some(val) => &mut spaces[val],
            None => {
                &mut [block_loc, block_len]
            }
        };

        for index in *replace_loc..(*replace_loc+block_len) {
            compacted[index] = block_id;
        }
        *replace_loc += block_len;
        *replace_len -= block_len;

    }

    let checksum: usize  = compacted.iter().enumerate().map(|(id, &num)| id * num).sum();
    assert_eq!(checksum, 2858);
}

