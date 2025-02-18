use std::fs;
use std::array::from_fn;
use std::time::Instant;


#[derive(Clone, Copy)]
struct Key {
    levels: [u8; 5],
}

#[derive(Clone, Copy)]
struct Lock {
    levels: [u8; 5],
}
fn textdata_to_key_locks(textdata: String) -> (Vec<Key>, Vec<Lock>) {
    let mut lines = textdata.lines();
    let mut keys: Vec<Key> = Vec::new();
    let mut locks: Vec<Lock> = Vec::new();
    loop {
        let block_data: [[char; 5]; 7] = from_fn(|_| {
            let mut char_iter = lines.next().unwrap().chars();
            from_fn(|_| char_iter.next().unwrap())
        });

        let is_lock = block_data[0][0] == '#';
        let terminal_char = match is_lock {
            true => '.',
            false => '#',
        };
        let levels: [u8; 5] = from_fn( |col| 
            (0..7).map(|row| block_data[row][col]).position(|chr| chr==terminal_char).unwrap() as u8
        );

        match is_lock {
            true => locks.push(Lock { levels }),
            false => keys.push(Key { levels }),
        }
        if lines.next().is_none() { break }
    }
    (keys, locks)
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot open file ({filename})"));

    let before = Instant::now();

    let (keys, locks): (Vec<Key>, Vec<Lock>) = textdata_to_key_locks(textdata);
    let mut key_map: [[Vec<usize>; 5]; 7] = from_fn(|_| from_fn(|_| Vec::new()));
    keys.iter().enumerate()
        .flat_map(|(key_id, key)| key.levels.into_iter()
            .enumerate()
            .map(move |(col, level)| (key_id, col, level)))
        .flat_map(|(key_id, col, level)| (level..7).map(move |row| (key_id, col, row)))
        .for_each(|(key_id, col, row)| key_map[row as usize][col].push(key_id));

    let key_map = key_map;

    let mut pass: usize = 0;
    for Lock { levels } in locks.iter() {
        let mut fit: Vec<bool> = (0..keys.len()).map(|_| true).collect();
        levels.iter()
            .enumerate()
            .map(|(col, &row_level)| (col, row_level as usize -1))
            .flat_map(|(col, row)| &key_map[row][col])
            .for_each(|num| {fit[*num] = false;});
        
        pass += fit.into_iter().filter(|&val| val).count();
    }

    let after = before.elapsed();

    let key_map_numbers: [[usize; 5]; 7] = key_map.clone().map(|row| row.map(|vec| vec.len()));
    key_map_numbers.iter()
        .for_each(|vec| {
            // println!("{}", vec.iter().map(|num| format!("{num:>4}")).collect::<String>());
            println!("{}", vec.iter().fold(String::new(), |string, next| string + &format!("{next:>4}")));
        });

    println!("(Part I) number of fits: {pass}");
    println!("(Part I) time elapsed: {after:.2?}");
}
