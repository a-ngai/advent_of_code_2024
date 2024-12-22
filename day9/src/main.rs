use std::fs;
use std::iter;
use std::time::Instant;

struct SpacePointer {
    loc: usize,
    len: usize,
}

struct DataPointer {
    id: usize,
    loc: usize,
    len: usize,
}

fn make_pointers_from_disk(disk: &Vec<usize>) -> (Vec<DataPointer>, Vec<SpacePointer>) {

    let forward_data_iter = disk.iter()
        .step_by(2)
        .enumerate();
    let forward_space_iter = disk.iter()
        .skip(1)
        .step_by(2)
        .chain([0,].iter());

    let mut data_locations: Vec<DataPointer> = Vec::new();
    let mut spaces: Vec<SpacePointer> = Vec::new();
    let mut loc: usize = 0;
    for ((data_id, &num), &space) in forward_data_iter.zip(forward_space_iter) {
        data_locations.push(DataPointer { id: data_id, len: num, loc });
        loc += num;
        spaces.push(SpacePointer{ loc, len:space });
        loc += space;
    }
    (data_locations, spaces)
}

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
    let (mut data_locations, mut spaces) = make_pointers_from_disk(&disk);

    let mut reverse_block_iter = disk.iter()
        .step_by(2)
        .enumerate()
        .rev()
        .flat_map(|(id, &num)| iter::repeat_n(id, num));

    let mut reverse_id: usize = usize::MAX;
    for (data_pointer, space_pointer) in data_locations.iter().zip(spaces.iter()) {
        let forward_id = data_pointer.id;
        let num = data_pointer.len;
        let skip = space_pointer.len;
        if reverse_id <= forward_id { 
            compacted.extend(reverse_block_iter.by_ref().take_while(|&reverse_id| reverse_id==forward_id)) ;
            break
        }
        compacted.extend(iter::repeat_n(forward_id, num));
         
        for _ in 0..skip { 
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

    for data_pointer in data_locations.iter_mut().rev() {
        let data_loc = &mut data_pointer.loc;
        let data_len = data_pointer.len;
        let look_for_space = spaces.iter()
            .take_while(|space| *data_loc > space.loc)
            .position(|space| data_len <= space.len);

        let (space_pointer, index) = match look_for_space {
            Some(index) => (&mut spaces[index], index),
            None => continue,
        };

        data_pointer.loc = space_pointer.loc;
        space_pointer.loc += data_len;
        space_pointer.len -= data_len;
        if space_pointer.len == 0usize { spaces.remove(index); }
    }

    let checksum: usize = data_locations
        .iter()
        .flat_map(|pointer| (0..(pointer.len)).map(|i| pointer.id * (pointer.loc + i )))
        .sum();

    let after = before.elapsed();
    println!("(Part II) checksum: {checksum}");
    println!("Time elapsed (Part II): {after:2?}");
}

#[test]
fn test_input() {
    let disk: Vec<usize> = vec![ 2,3,3,3,1,3,3,1,2,1,4,1,4,1,3,1,4,0,2 ];

    let (data_locations, spaces) = make_pointers_from_disk(&disk);

    let mut compacted: Vec<usize> = Vec::new();
    let mut reverse_block_iter = disk.iter()
        .step_by(2)
        .enumerate()
        .rev()
        .flat_map(|(id, &num)| iter::repeat_n(id, num));

    let mut reverse_id: usize = usize::MAX;
    for (data_pointer, space_pointer) in data_locations.iter().zip(spaces.iter()) {
        let forward_id = data_pointer.id;
        let num = data_pointer.len;
        let skip = space_pointer.len;
        if reverse_id <= forward_id { 
            compacted.extend(reverse_block_iter.by_ref().take_while(|&reverse_id| reverse_id==forward_id)) ;
            break
        }
        compacted.extend(iter::repeat_n(forward_id, num));
         
        for _ in 0..skip { 
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

    let (mut data_locations, mut spaces) = make_pointers_from_disk(&disk);

    for data_location in data_locations.iter_mut().rev() {
        let data_loc = data_location.loc;
        let data_len = data_location.len;
        let look_for_space = spaces.iter()
            .take_while(|space| data_loc > space.loc)
            .position(|space| data_len <= space.len);

        let (space_pointer, index) = match look_for_space {
            Some(index) => (&mut spaces[index], index),
            None => continue,
        };

        data_location.loc = space_pointer.loc;
        space_pointer.loc += data_len;
        space_pointer.len -= data_len;
        if space_pointer.len == 0usize { spaces.remove(index); }
    }

    let checksum: usize = data_locations
        .iter()
        .flat_map(|pointer| (0..(pointer.len)).map(|i| pointer.id * (pointer.loc + i )))
        .sum();
    assert_eq!(checksum, 2858);
}
