use std::fs;
use std::collections::{BTreeMap, BTreeSet};
use core::array::from_fn;
use std::time::Instant;
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;
use std::iter::once;

const NUM_PER_TEST: u64 = 100;

#[derive(Clone, Copy)]
struct GateString<'a> {
    wire1: &'a str,
    wire2: &'a str,
    op: &'a str,
    wire3: &'a str,
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, Debug)]
struct Gate {
    wire1: u16,
    wire2: u16,
    op: Operation,
    wire3: u16,
}

fn wire_from_line(string: &str) -> (&str, bool) {
    let mut split = string.split_whitespace();
    let first_part: &str = split.next().unwrap();
    let wire: &str = &first_part[0..(first_part.len()-1)];
    let on: bool = split.next().unwrap() == "1";
    (wire, on)
}

fn gate_from_line(string: &str) -> GateString {
    let mut split = string.split_whitespace();
    let [wire1, op, wire2, _, wire3]: [&str; 5] = from_fn(|_| split.next().unwrap());
    GateString { wire1, wire2, op, wire3 }
}

fn wire_names_with_char<'a>(chr: char, str_to_num: &BTreeMap<&'a str, u16>) -> Vec<&'a str> {
    let char_wires: Vec<&str> = str_to_num.keys()
        .filter(|&&string| string.contains(chr) && string.chars().last().unwrap().is_ascii_digit())
        .copied()
        .collect();
    char_wires
}

fn get_wire_connections_recursion_safe(wire3: u16, gates: &[Option<Gate>], depth: usize) -> Option<Vec<u16>> {
    if depth > gates.len() { return None }
    let index: usize = wire3 as usize;

    let gate = match gates[index] {
        Some(val) => val,
        None => return Some(Vec::new()),
    };
    let Gate { wire1, wire2, .. } = gate;
    let wire1_connections = get_wire_connections_recursion_safe(wire1, gates, depth+1)?;
    let wire2_connections = get_wire_connections_recursion_safe(wire2, gates, depth+1)?;

    let connections: Vec<u16> = once(wire1)
        .chain(once(wire2))
        .chain(once(wire3))
        .chain(wire1_connections)
        .chain(wire2_connections)
        .collect();
    Some(connections)
}

fn get_wire_connections(wire3: u16, gates: &[Option<Gate>]) -> Option<Vec<u16>> {
    let initial_recursion_depth = 0;
    let all_wires = get_wire_connections_recursion_safe(wire3, gates, initial_recursion_depth)?;
    Some(BTreeSet::from_iter(all_wires).into_iter().collect())
}


fn get_wire_value_recursion_safe(wire3: u16, gates: &[Option<Gate>], wires: &mut [Option<bool>], depth: usize) -> Option<bool> {
    if depth > wires.len() { return None }
    let index: usize = wire3 as usize;
    if let Some(val) = wires[index] { return Some(val) };

    let Gate { wire1, wire2, op, .. } = gates[index]?;
    let wire1_state = get_wire_value_recursion_safe(wire1, gates, wires, depth+1)?;
    let wire2_state = get_wire_value_recursion_safe(wire2, gates, wires, depth+1)?;
    let wire3_state = match op {
        Operation::And => wire1_state & wire2_state,
        Operation::Or => wire1_state | wire2_state,
        Operation::Xor => wire1_state ^ wire2_state,
    };
    wires[index] = Some(wire3_state);

    Some(wire3_state)
}

fn get_wire_value(wire3: u16, gates: &[Option<Gate>], wires: &mut [Option<bool>]) -> Option<bool> {
    let initial_recursion_depth = 0;
    get_wire_value_recursion_safe(wire3, gates, wires, initial_recursion_depth)
}

fn gate_to_gateshort(gate: &GateString, str_to_num: &BTreeMap<&str, u16>) -> Gate {
    let &GateString { wire1, wire2, op, wire3 } = gate;

    Gate { 
        wire1: *str_to_num.get(wire1).unwrap(),
        wire2: *str_to_num.get(wire2).unwrap(),
        op: match op {
            "AND" => Operation::And,
            "OR" => Operation::Or,
            "XOR" => Operation::Xor,
            other => panic!("Case ({other}) not covered!"),
        },
        wire3: *str_to_num.get(wire3).unwrap(),
    }
}

fn info_from_textdata(textdata: &str) -> (Vec<Option<bool>>, Vec<Option<Gate>>, BTreeMap<&str, u16>, BTreeMap<u16, &str>) {
    let mut lines = textdata.lines();

    let xy_wire_info: BTreeMap<&str, bool> = BTreeMap::from_iter(
        lines.by_ref()
            .take_while(|line| !line.is_empty())
            .map(wire_from_line)
        );

    let gates_info: BTreeMap<&str, GateString> = lines.by_ref()
        .take_while(|line| !line.is_empty())
        .map(gate_from_line)
        .map(|gate| (gate.wire3, gate))
        .collect();

    let wires_set: BTreeSet<&str> = BTreeSet::from_iter(
        xy_wire_info.keys()
            .chain(gates_info.keys())
            .copied()
        );

    let num_to_str: BTreeMap<u16, &str> = BTreeMap::from_iter(
        wires_set.iter().copied().enumerate().map(|(num, val)| (num as u16, val))
        );
    let str_to_num: BTreeMap<&str, u16> = BTreeMap::from_iter(
        num_to_str.iter().map(|(&string, &index)| (index, string))
        );

    let wires: Vec<Option<bool>> = wires_set
        .iter()
        .map(|string| xy_wire_info.get(string).copied())
        .collect();

    let gates: Vec<Option<Gate>> = wires_set
        .iter()
        .map(|string| gates_info.get(string).copied())
        .map(|gate| match gate { Some(val) => Some(gate_to_gateshort(&val, &str_to_num)), None => None})
        .collect();

    (wires, gates, str_to_num, num_to_str)
}

fn set_x_y_wires(loop_wires: &mut [Option<bool>], x_wires: &[u16], y_wires: &[u16], x_num: u64, y_num: u64) {
    let mut manip_x = x_num;
    let mut manip_y = y_num;
    for (&x_wire, &y_wire) in x_wires.iter().zip(y_wires.iter()) {
        loop_wires[x_wire as usize] = Some((manip_x & 1) == 1);
        loop_wires[y_wire as usize] = Some((manip_y & 1) == 1);
        manip_x >>= 1;
        manip_y >>= 1;
    }
}

fn measure_error(test_numbers: &[[u64; 2]], digits: &[usize], x_wires: &[u16], y_wires: &[u16], z_wires: &[u16], wires: &Vec<Option<bool>>, gates: &[Option<Gate>]) -> Option<usize> {
    let mut measure: Option<usize> = None;
    let mut loop_wires = wires.to_owned();
    
    for &[test_x, test_y] in test_numbers {
        loop_wires.iter_mut().zip(wires.iter())
            .for_each(|(old, new)| *old = *new);
        set_x_y_wires(&mut loop_wires, x_wires, y_wires, test_x, test_y);

        let z_wire_results: Vec<Option<bool>> = z_wires.iter()
            .map(|&num| get_wire_value( num, gates, &mut loop_wires))
            .collect();
        if z_wire_results.iter().any(|&result| result.is_none()) { break }

        let z_digits: Vec<bool> = z_wires.iter()
            .map(|&num| get_wire_value( num, gates, &mut loop_wires).unwrap()).collect();

        let mut add_digits: Vec<bool> = Vec::new();
        let mut shift_add = test_x + test_y;
        for _ in 0..z_wires.len() {
            add_digits.push((shift_add & 1) == 1);
            shift_add >>= 1;
        }

        let correct_measure: usize = add_digits.into_iter().zip(z_digits.into_iter())
            .enumerate()
            .filter(|(num, _)| digits.contains(num))
            .map(|(_index, (add, z))| add == z)
            .map(|val| match val {true => 0, false => 1})
            .sum();

        match measure {
            Some(num) => measure = Some(num + correct_measure),
            None => measure = Some(correct_measure),
        }

    }
    measure
}


fn make_gate_swap<'a>(gates: &[Option<Gate>], swaps: &[[usize; 2]]) -> Result<Vec<Option<Gate>>, &'a str>{
    let mut swap_gates: Vec<Option<Gate>> = gates.to_owned();
    for &[num1, num2] in swaps {
        let gate1 = match swap_gates[num1] {
            Some(gate) => Some(gate),
            None => return Err("Found None variant"),
        };
        let gate2 = match swap_gates[num2] {
            Some(gate) => Some(gate),
            None => return Err("Found None variant"),
        };
        swap_gates[num1] = gate2;
        swap_gates[num2] = gate1;
    }

    Ok(swap_gates)
}

fn make_wire_swap(wires: &[Option<bool>], swaps: &[[usize; 2]]) -> Vec<Option<bool>> {
    let mut swap_wires = wires.to_owned();
    for &[num1, num2] in swaps {
        let wire1 = swap_wires[num1];
        let wire2 = swap_wires[num2];
        swap_wires[num1] = wire2;
        swap_wires[num2] = wire1;
    }
    swap_wires
}

fn wires_to_binary(char_wires: &[u16], gates: &[Option<Gate>], wires: &mut [Option<bool>]) -> String {
    char_wires.into_iter()
        .map(|&index| get_wire_value(index, gates, wires))
        .map(|val| match val.unwrap() { true => '1', false => '0' })
        .rev()
        .collect()
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let item = info_from_textdata(&textdata);
    let default_wires: Vec<Option<bool>> = item.0;
    let default_gates: Vec<Option<Gate>> = item.1;
    let str_to_num: BTreeMap<&str, u16> = item.2;
    let num_to_str: BTreeMap<u16, &str> = item.3;

    let mut x_wire_names = wire_names_with_char('x', &str_to_num);
    let mut y_wire_names = wire_names_with_char('y', &str_to_num);
    let mut z_wire_names = wire_names_with_char('z', &str_to_num);
    x_wire_names.sort();
    y_wire_names.sort();
    z_wire_names.sort();

    let x_wires: Vec<u16> = x_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();
    let y_wires: Vec<u16> = y_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();
    let z_wires: Vec<u16> = z_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();

    let before_loop = Instant::now();
    let mut wires = default_wires.clone();
    let z_binary = wires_to_binary(&z_wires, &default_gates, &mut wires);

    let number = usize::from_str_radix(z_binary.as_str(), 2).unwrap();
    let after_loop = before_loop.elapsed();

    let after = before.elapsed();
    println!("(Part  I) decimal: {number}");
    println!("(Part  I) time elapsed: {after:.2?}");
    println!("(Part  I) time single prop.: {after_loop:.2?}");
    let before = Instant::now();

    // For part 2, brute force won't work. There are ~2^8 wires, and combined with 8 wrong wires
    // yields ~(2^8)^8 ~ 2^64 possible combinations with a few reducing symmetry factors.

    // If we only consider single output wires, they are attached to ~2^6 wires, and on average
    // they should only contain ~3 wrong wires => ~2^18 = 262144 possible combinations, which is
    // not great but doable.

    // We could use a further heuristic; every time we swap the correct wires, the result should
    // have fewer mistakes.

    // let mut rng = ChaCha8Rng::seed_from_u64(2);
    // let test_numbers: [[u64; 2]; NUM_PER_TEST as usize] = from_fn(|_| 
    //     from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32)))
    //     );
    // let inter_wires: Vec<u16> = num_to_str.iter()
    //     .filter(|(_, &string)| ! (
    //             ['x', 'y', 'z'].contains(&string.chars().next().unwrap()) 
    //             && string.chars().last().unwrap().is_ascii_digit()
    //             ))
    //     .map(|(&num, _)| num)
    //     .collect();

    // let num_inter_wires: u16 = inter_wires.len() as u16;
    // let combinations: Vec<[usize; 2]> = (0..num_inter_wires)
    //     .flat_map(|num1| ((num1+1)..num_inter_wires).map(move |num2| [num1, num2]))
    //     .map(|[num1, num2]| [inter_wires[num1 as usize], inter_wires[num2 as usize]])
    //     .map(|[num1, num2]| [num1 as usize, num2 as usize])
    //     .collect();

    // // filter out the non-possible combinations
    // let mut valid_combinations: Vec<[usize; 2]> = Vec::new();
    // for [num1, num2] in combinations {
    //     let swap_gates = match make_gate_swap(&default_gates, &[[num1, num2]]) {
    //         Ok(val) => val,
    //         Err(_) => continue
    //     };
    //     let swap_wires = make_wire_swap(&default_wires, &[[num1, num2]]);
    //     if measure_error(&test_numbers, &[0,], &x_wires, &y_wires, &z_wires, &swap_wires, &swap_gates).is_none() { continue }
    //     valid_combinations.push([num1, num2]);
    // }

    // let mut rng = ChaCha8Rng::seed_from_u64(2);
    // let test_numbers: [[u64; 2]; NUM_PER_TEST as usize] = from_fn(|_| 
    //     from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32)))
    //     );

    // // let mut test_numbers: Vec<[u64; 2]> = Vec::new();
    // // let mut shift = 1;
    // // (0..x_wires.len()).for_each(|_| {
    // //     test_numbers.push([0, shift]);
    // //     test_numbers.push([shift, 0]);
    // //     shift <<= 1;
    // // });

    let z_connections: Vec<Option<Vec<u16>>> = z_wires.iter()
        .map(|&num| get_wire_connections(num, &default_gates))
        .collect();
    let z_connections_string: String = z_connections.iter().enumerate().map(|(num, vec)| 
        format!("{num}: {}", vec.to_owned().unwrap().len())
        ).collect::<Vec<String>>()
        .join(", ");
    println!("{z_connections_string}");
    
    // Strategy to determine swapped wires
    // 1. Starting with the ones digit, test if the following combinations work:
    //      i. (x=0, y=0), (x=0, y=1), (x=1, y=0), (x=1, y=1)
    // 2. if they work, then record the wires that are connected to the digit as "correct"
    // 3. If they don't work, then try swapping each of the connected wires to a not-proven wire,
    //    until it's found.
    // 4. Move on to the next digit, keeping track of which wires are "correct" until the end.



    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let test_numbers: [[u64; 2]; NUM_PER_TEST as usize] = from_fn(|_| 
        from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32)))
        );


    let mut swapped_numbers: Vec<[u16; 2]> = Vec::new();
    let mut unproven_wires: BTreeSet<u16> = BTreeSet::from_iter(0u16..wires.len() as u16);
    let mut proven_wires: BTreeSet<u16> = BTreeSet::new();
    let mut swap_gates = default_gates.clone();
    let mut swap_wires = default_wires.clone();
    for digit in 0..x_wires.len() {

        // let test_numbers: Vec<[u64; 2]> = if digit == 0 {
        //     let shift = 2u64.pow(digit as u32);
        //     let comb = [0, shift];
        //     comb.into_iter()
        //         .flat_map(|num1| comb.into_iter().map(move |num2| [num1, num2]))
        //         .chain((digit+1..x_wires.len()).map(|power| [2u64.pow(power as u32), 0]))
        //         .chain((digit+1..x_wires.len()).map(|power| [0, 2u64.pow(power as u32)]))
        //         .chain((digit+1..x_wires.len()).map(|power| [2u64, 2u64.pow(power as u32)]))
        //         .collect()
        // } else {
        //     let low_shift = 2u64.pow(digit as u32);
        //     let shift = 2u64.pow(digit as u32);
        //     // let comb = [0, low_shift, shift, low_shift+shift];
        //     let comb = [0, shift];
        //     comb.into_iter()
        //         .flat_map(|num1| comb.into_iter().map(move |num2| [num1, num2]))
        //         .chain((digit+1..x_wires.len()).map(|power| [2u64.pow(power as u32), 0]))
        //         .chain((digit+1..x_wires.len()).map(|power| [0, 2u64.pow(power as u32)]))
        //         .chain((digit+1..x_wires.len()).map(|power| [2u64, 2u64.pow(power as u32)]))
        //         .collect()
        // };

        let connected_wires: Vec<u16> = get_wire_connections(z_wires[digit], &swap_gates).unwrap();

        let measure = measure_error(&test_numbers, &[digit,], &x_wires, &y_wires, &z_wires, &swap_wires, &swap_gates).unwrap();
        println!("measure for digit {digit}: {measure}");
        if measure == 0 {
            connected_wires.iter().for_each(|num| {unproven_wires.remove(num);});
            connected_wires.iter().for_each(|&num| {proven_wires.insert(num);});
            continue
        }
        // println!("safe: {proven_wires:?}");
        // println!("connected: {connected_wires:?}");

        let candidate_swaps: Vec<[u16; 2]> = connected_wires.iter()
            .flat_map(|&num1| unproven_wires.iter().map(move |&num2| [num1, num2]) )
            .collect();

        println!("here with {} possible combinations; {} proven", candidate_swaps.len(), wires.len()-unproven_wires.len());

        for &[num1, num2] in candidate_swaps.iter() {
            let test_comb = [[num1 as usize, num2 as usize],];
            let loop_gates = match make_gate_swap(&swap_gates, &test_comb) {
                Ok(val) => val,
                Err(_) => continue
            };
            let loop_wires = make_wire_swap(&swap_wires, &test_comb);
            let measure = measure_error(&test_numbers, &[digit,], &x_wires, &y_wires, &z_wires, &loop_wires, &loop_gates);

            if measure.is_none() { continue }
            // println!("{}, {test_comb:?}", measure.unwrap());
            if measure.unwrap() == 0 { 
                println!("found something: {test_comb:?}; {} {}", *num_to_str.get(&num1).unwrap(), *num_to_str.get(&num2).unwrap());
                swapped_numbers.push([num1, num2]);
                let loop_gates = match make_gate_swap(&swap_gates, &test_comb) {
                    Ok(val) => val,
                    Err(_) => continue
                };
                let loop_wires = make_wire_swap(&swap_wires, &test_comb);
                swap_gates = loop_gates;
                swap_wires = loop_wires;

                break
                
            }
            
        }


    }
    let mut names: Vec<&str> = swapped_numbers.iter().flat_map(|item| item).map(|num| *num_to_str.get(num).unwrap()).collect();
    names.sort();
    println!("{}", names.join(","));
    panic!("");


    // for digit in 0..x_wires.len() {
    // // for digit in 0..10 {
    //     println!("on loop {digit}");

    //     let mut candidate_combinations: Vec<[usize; 2]> = Vec::new();
    //     let mut errors: Vec<Option<usize>> = Vec::new();

    //     for &[num1, num2] in valid_combinations.iter() {
    //         let test_comb = [[num1, num2],];
    //         let swap_gates = match make_gate_swap(&default_gates, &test_comb) {
    //             Ok(val) => val,
    //             Err(_) => continue
    //         };
    //         let swap_wires = make_wire_swap(&default_wires, &test_comb);
    //         let measure = measure_error(&test_numbers, &[digit,], &x_wires, &y_wires, &z_wires, &swap_wires, &swap_gates);

    //         if measure.is_none() { continue }

    //         errors.push(measure);
    //         candidate_combinations.push([num1, num2]);
    //     }

    //     fs::write(
    //         format!("swap_errors_{digit}.txt"), 
    //         errors.iter().flatten().map(|&num| num.to_string()).collect::<Vec<String>>().join("\n")
    //         ).expect("Could not write into file");

    //     valid_combinations = candidate_combinations;

    // }
    //



    // let measure_digits = [7,];
    // let mut current_best_comb: Vec<[usize; 2]> = Vec::new();
    // for run in 1..5 {
    //     println!("current best combination: {current_best_comb:?}");

    //     let swap_gates = make_gate_swap(&default_gates, &current_best_comb).unwrap();
    //     let swap_wires = make_wire_swap(&default_wires, &current_best_comb);
    //     let ref_error_value: usize = measure_error(&test_numbers, &measure_digits, &x_wires, &y_wires, &z_wires, &swap_wires, &swap_gates).unwrap();
    //     println!("ref error: {ref_error_value}");

    //     let mut candidate_combinations: Vec<[usize; 2]> = Vec::new();
    //     let mut errors: Vec<Option<usize>> = Vec::new();
    //     let mut test_comb = current_best_comb.clone();
    //     test_comb.push([0, 0]);

    //     for &[num1, num2] in valid_combinations.iter() {
    //         test_comb[run-1] = [num1, num2];
    //         let flattened: Vec<usize> = current_best_comb.iter().flat_map(|&vec| vec).collect();
    //         if flattened.contains(&num1) || flattened.contains(&num2) { continue }

    //         let swap_gates = match make_gate_swap(&default_gates, &test_comb) {
    //             Ok(val) => val,
    //             Err(_) => continue
    //         };
    //         let swap_wires = make_wire_swap(&default_wires, &test_comb);
    //         let measure = measure_error(&test_numbers, &measure_digits, &x_wires, &y_wires, &z_wires, &swap_wires, &swap_gates);

    //         if measure.is_none() { continue }
    //         if measure.unwrap() > ref_error_value { continue }

    //         errors.push(measure);
    //         candidate_combinations.push([num1, num2]);
    //     }
    //     let add_comb = *errors.iter().zip(candidate_combinations.iter()).min_by(|(err1, _), (err2, _)| err1.cmp(err2)).unwrap().1;
    //     current_best_comb.push(add_comb);

    //     fs::write(
    //         format!("swap_errors_{run}.txt"), 
    //         errors.iter().flatten().map(|&num| num.to_string()).collect::<Vec<String>>().join("\n")
    //         ).expect("Could not write into file");

    //     valid_combinations = candidate_combinations;

    // }




    let after = before.elapsed();
    println!("(Part II) time elapsed: {after:.2?}");

    // perform Dijkstra esp. for low number of tests
}

#[test]
fn small_test() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let item = info_from_textdata(&textdata);
    let default_wires: Vec<Option<bool>> = item.0;
    let default_gates: Vec<Option<Gate>> = item.1;
    let str_to_num: BTreeMap<&str, u16> = item.2;
    let num_to_str: BTreeMap<u16, &str> = item.3;

    let mut x_wire_names = wire_names_with_char('x', &str_to_num);
    let mut y_wire_names = wire_names_with_char('y', &str_to_num);
    let mut z_wire_names = wire_names_with_char('z', &str_to_num);
    x_wire_names.sort();
    y_wire_names.sort();
    z_wire_names.sort();

    let x_wires: Vec<u16> = x_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();
    let y_wires: Vec<u16> = y_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();
    let z_wires: Vec<u16> = z_wire_names.iter()
        .map(|string| str_to_num.get(string).copied().unwrap()) .collect();

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let test_numbers: [[u64; 2]; NUM_PER_TEST as usize] = from_fn(|_| 
        from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32)))
        );

    let mut loop_wires = default_wires.clone();

    for [x_num, y_num] in test_numbers {

        set_x_y_wires(&mut loop_wires, &x_wires, &y_wires, x_num, y_num);

        let x_binary = wires_to_binary(&x_wires, &default_gates, &mut loop_wires);
        let x_number = u64::from_str_radix(x_binary.as_str(), 2).unwrap();

        let y_binary = wires_to_binary(&y_wires, &default_gates, &mut loop_wires);
        let y_number = u64::from_str_radix(y_binary.as_str(), 2).unwrap();

        assert_eq!(x_num, x_number);
        assert_eq!(y_num, y_number);
    }
}

