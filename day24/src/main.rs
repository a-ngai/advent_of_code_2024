use std::fs;
use std::collections::{BTreeMap, BTreeSet};
use core::array::from_fn;
use std::time::Instant;
use rand::{SeedableRng, Rng};
use rand_chacha::ChaCha8Rng;
use std::iter::once;

// Possible optimization:
// - Not all gate swaps lead to a change in the z-digit. We can drastically cut down the number of gate swaps
// we test if we keep track of this.

const FALSE_THRESHOLD: f32 = 1e-5;
const NUM_SWAPS: usize = 4;

fn tolerance_sample_num(fail_prob: f32, num_swaps: usize, num_gates: usize) -> usize {
    // Assumes that each random sample has a 50% chance of a false positive.
    // Linear approximation where fail_prob << 1.
    assert!((0. ..=1.).contains(&fail_prob), "fail_probe ({fail_prob}) must be between 0 and 1");
    let approx_sample_num_float: f32 = - (fail_prob / ((num_swaps * num_gates.pow(2)) as f32)).log2();
    let sample_num: usize = approx_sample_num_float.ceil() as usize;
    sample_num
}

#[derive(Clone, Debug)]
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
        .map(|string| gates_info.get(string))
        .map(|gate| gate.map(|val| gate_to_gateshort(val, &str_to_num)))
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

fn swap_wires(wires: &mut [Option<bool>], swaps: &[[usize; 2]]) {
    for &[num1, num2] in swaps {
        let wire1 = (*wires)[num1];
        let wire2 = (*wires)[num2];
        wires[num1] = wire2;
        wires[num2] = wire1;
    };
}

fn wires_to_binary(char_wires: &[u16], gates: &[Option<Gate>], wires: &mut [Option<bool>]) -> String {
    char_wires.iter()
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

    // Strategy to determine swapped wires
    // 1. Starting with the ones digit, test if the following combinations work:
    //      i. (x=0, y=0), (x=0, y=1), (x=1, y=0), (x=1, y=1)
    // 2. if they work, then record the wires that are connected to the digit as "correct"
    // 3. If they don't work, then try swapping each of the connected wires to a not-proven wire,
    //    until it's found.
    // 4. Move on to the next digit, keeping track of which wires are "correct" until the end.

    let num_gates: usize = default_gates.len();
    let num_per_test: usize = tolerance_sample_num(FALSE_THRESHOLD, NUM_SWAPS, num_gates);
    println!("    false-positive tolerance: {FALSE_THRESHOLD:.0e}  ->  num of random samples: {num_per_test}");
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let test_numbers: Vec<[u64; 2]> = (0..num_per_test)
        .map(|_| from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32))))
        .collect();


    let mut swapped_numbers: Vec<[u16; 2]> = Vec::new();
    let mut unproven_wires: BTreeSet<u16> = BTreeSet::from_iter(0u16..wires.len() as u16);
    let mut proven_gates = default_gates.clone();
    let mut proven_wires: Vec<Vec<Option<bool>>> = (0..test_numbers.len())
        .map(|_| default_wires.clone())
        .collect();
    proven_wires.iter_mut().zip(test_numbers.iter()).for_each(|(wires, &[test_x, test_y])| { 
        set_x_y_wires(wires, &x_wires, &y_wires, test_x, test_y); 
    });

    let test_add_digits: Vec<Vec<bool>> = test_numbers.into_iter()
        .map(|array| array.iter().sum())
        .map(|num| {
            let mut shift_add: u64 = num;
            let mut add_digits: Vec<bool> = Vec::new();
            for _ in 0..z_wires.len() {
                add_digits.push((shift_add & 1) == 1);
                shift_add >>= 1;
            }
            add_digits
        })
        .collect();

    let mut test_gates: Vec<Option<Gate>> = Vec::new();
    let mut test_wires: Vec<Vec<Option<bool>>> = Vec::new();
    for (digit, &z_wire)  in z_wires.iter().enumerate() {

        test_gates.clone_from(&proven_gates);
        test_wires.clone_from(&proven_wires);

        let connected_wires: Vec<u16> = get_wire_connections(z_wire, &test_gates).unwrap();

        let measure: usize = (0..num_per_test)
            .map(|index| get_wire_value(z_wire, &test_gates, &mut test_wires[index]).unwrap())
            .enumerate()
            .map(|(index, val)| test_add_digits[index][digit] != val)
            .filter(|&val| val)
            .count();

        if measure == 0 {
            proven_gates.clone_from(&test_gates);
            proven_wires.clone_from(&test_wires);
            connected_wires.iter().for_each(|num| {unproven_wires.remove(num);});
            continue
        }

        let candidate_swaps: Vec<[u16; 2]> = connected_wires.iter()
            .flat_map(|&num1| unproven_wires.iter().map(move |&num2| [num1, num2]) )
            .collect();

        for &[num1, num2] in candidate_swaps.iter() {

            let test_comb = [[num1 as usize, num2 as usize],];
            let test_gates = match make_gate_swap(&proven_gates, &test_comb) {
                Ok(val) => val,
                Err(_) => continue
            };
            test_wires.clone_from(&proven_wires);
            test_wires.iter_mut().for_each(|wires| {swap_wires(wires, &test_comb);});

            if get_wire_value(z_wire, &test_gates, &mut test_wires[0]).is_none() { continue }

        let measure: usize = (0..num_per_test)
            .map(|index| get_wire_value(z_wire, &test_gates, &mut test_wires[index]).unwrap())
            .enumerate()
            .map(|(index, val)| test_add_digits[index][digit] != val)
            .filter(|&val| val)
            .count();

            if measure == 0 { 
                swapped_numbers.push([num1, num2]);
                proven_gates.clone_from(&test_gates);
                proven_wires.clone_from(&test_wires);
                break
            }
        }
    }

    let mut names: Vec<&str> = swapped_numbers.iter()
        .flatten().map(|num| *num_to_str.get(num).unwrap())
        .collect();
    names.sort();

    let after = before.elapsed();
    println!("(Part II) swap names: {}", names.join(","));
    println!("(Part II) time elapsed: {after:.2?}");
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

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let num_gates: usize = default_gates.len();
    let num_per_test: usize = tolerance_sample_num(FALSE_THRESHOLD, NUM_SWAPS, num_gates);
    let test_numbers: Vec<[u64; 2]> = (0..num_per_test)
        .map(|_| from_fn(|_| rng.gen_range(0..2u64.pow(x_wires.len() as u32))))
        .collect();

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
